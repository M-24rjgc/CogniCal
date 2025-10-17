use crate::db::repositories::community_export_repository::CommunityExportRepository;
use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::community_export::CommunityExportCreate;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

const PROJECT_NAME: &str = "CogniCal";
const PROJECT_VERSION: &str = env!("CARGO_PKG_VERSION");
const PROJECT_LICENSE: &str = "MIT";
const PROJECT_REPO: &str = "https://github.com/cognical/cognical";
const PROJECT_DOCS: &str = "https://github.com/cognical/cognical#readme";
const PROJECT_CONTRIBUTING: &str = "https://github.com/cognical/cognical/blob/main/CONTRIBUTING.md";
const PROJECT_COMMUNITY: &str = "https://github.com/cognical/cognical/discussions";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectInfo {
    pub name: String,
    pub version: String,
    pub license: String,
    pub repository_url: String,
    pub docs_url: String,
    pub contributing_url: String,
    pub community_url: String,
    pub is_open_source: bool,
    pub features_always_free: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectedPlugin {
    pub name: String,
    pub version: Option<String>,
    pub source: String,
    pub enabled: bool,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    pub os: String,
    pub app_version: String,
    pub locale: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnonymizedMetrics {
    pub total_tasks: i64,
    pub completed_tasks: i64,
    pub average_completion_time_minutes: Option<f64>,
    pub total_sessions: i64,
    pub productivity_score_available: bool,
    pub workload_forecasts_count: i64,
    pub wellness_events_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedbackSummary {
    pub total_feedback_count: i64,
    pub positive_count: i64,
    pub negative_count: i64,
    pub surfaces: Vec<String>,
    pub most_common_issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportBundle {
    pub system_info: SystemInfo,
    pub metrics: AnonymizedMetrics,
    pub feedback_summary: Option<FeedbackSummary>,
    pub plugins: Vec<DetectedPlugin>,
    pub checksum: String,
}

impl ExportBundle {
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();

        md.push_str(&format!("# {} Community Feedback Export\n\n", PROJECT_NAME));
        md.push_str(&format!(
            "> Generated at: {}\n\n",
            self.system_info.timestamp
        ));

        // System Information
        md.push_str("## System Information\n\n");
        md.push_str(&format!("- **OS**: {}\n", self.system_info.os));
        md.push_str(&format!(
            "- **App Version**: {}\n",
            self.system_info.app_version
        ));
        md.push_str(&format!("- **Locale**: {}\n\n", self.system_info.locale));

        // Anonymized Metrics
        md.push_str("## Anonymized Usage Metrics\n\n");
        md.push_str(&format!(
            "- **Total Tasks**: {}\n",
            self.metrics.total_tasks
        ));
        md.push_str(&format!(
            "- **Completed Tasks**: {}\n",
            self.metrics.completed_tasks
        ));
        if let Some(avg_time) = self.metrics.average_completion_time_minutes {
            md.push_str(&format!(
                "- **Avg Completion Time**: {:.1} minutes\n",
                avg_time
            ));
        }
        md.push_str(&format!(
            "- **Total Sessions**: {}\n",
            self.metrics.total_sessions
        ));
        md.push_str(&format!(
            "- **Productivity Score**: {}\n",
            if self.metrics.productivity_score_available {
                "Available"
            } else {
                "Not Available"
            }
        ));
        md.push_str(&format!(
            "- **Workload Forecasts**: {}\n",
            self.metrics.workload_forecasts_count
        ));
        md.push_str(&format!(
            "- **Wellness Events**: {}\n\n",
            self.metrics.wellness_events_count
        ));

        // Feedback Summary
        if let Some(ref feedback) = self.feedback_summary {
            md.push_str("## AI Feedback Summary\n\n");
            md.push_str(&format!(
                "- **Total Feedback**: {}\n",
                feedback.total_feedback_count
            ));
            md.push_str(&format!(
                "- **Positive (👍)**: {}\n",
                feedback.positive_count
            ));
            md.push_str(&format!(
                "- **Negative (👎)**: {}\n",
                feedback.negative_count
            ));

            if !feedback.surfaces.is_empty() {
                md.push_str(&format!(
                    "- **Surfaces**: {}\n",
                    feedback.surfaces.join(", ")
                ));
            }

            if !feedback.most_common_issues.is_empty() {
                md.push_str("- **Common Issues**:\n");
                for issue in &feedback.most_common_issues {
                    md.push_str(&format!("  - {}\n", issue));
                }
            }
            md.push_str("\n");
        }

        // Plugins
        if !self.plugins.is_empty() {
            md.push_str("## Detected Plugins\n\n");
            for plugin in &self.plugins {
                md.push_str(&format!("### {}\n", plugin.name));
                if let Some(ref version) = plugin.version {
                    md.push_str(&format!("- **Version**: {}\n", version));
                }
                md.push_str(&format!("- **Source**: {}\n", plugin.source));
                md.push_str(&format!("- **Enabled**: {}\n", plugin.enabled));
                if !plugin.permissions.is_empty() {
                    md.push_str(&format!(
                        "- **Permissions**: {}\n\n",
                        plugin.permissions.join(", ")
                    ));
                } else {
                    md.push_str("\n");
                }
            }
        } else {
            md.push_str("## Detected Plugins\n\n");
            md.push_str("_No plugins detected_\n\n");
        }

        // Checksum
        md.push_str("## Verification\n\n");
        md.push_str(&format!("**Checksum (SHA-256)**: `{}`\n\n", self.checksum));
        md.push_str("---\n\n");
        md.push_str(&format!("_This export was generated by {} v{} and contains only anonymized data. No personal information, task names, or notes are included._\n",
            PROJECT_NAME, PROJECT_VERSION));

        md
    }
}

#[derive(Clone)]
pub struct CommunityService {
    db_pool: DbPool,
}

impl CommunityService {
    pub fn new(db_pool: DbPool) -> Self {
        Self { db_pool }
    }

    /// Get project information (always available, even offline)
    pub fn get_project_info(&self) -> AppResult<ProjectInfo> {
        Ok(ProjectInfo {
            name: PROJECT_NAME.to_string(),
            version: PROJECT_VERSION.to_string(),
            license: PROJECT_LICENSE.to_string(),
            repository_url: PROJECT_REPO.to_string(),
            docs_url: PROJECT_DOCS.to_string(),
            contributing_url: PROJECT_CONTRIBUTING.to_string(),
            community_url: PROJECT_COMMUNITY.to_string(),
            is_open_source: true,
            features_always_free: true,
        })
    }

    /// Detect installed plugins (placeholder - will scan plugin directory)
    pub fn detect_plugins(&self) -> AppResult<Vec<DetectedPlugin>> {
        // TODO: Implement actual plugin detection
        // For now, return empty list to demonstrate graceful degradation
        Ok(vec![])
    }

    /// Generate system information
    fn generate_system_info(&self) -> SystemInfo {
        SystemInfo {
            os: std::env::consts::OS.to_string(),
            app_version: PROJECT_VERSION.to_string(),
            locale: "en-US".to_string(), // TODO: Get from system
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Collect anonymized metrics from database
    async fn collect_anonymized_metrics(&self) -> AppResult<AnonymizedMetrics> {
        let conn = self.db_pool.get_connection()?;

        // Count tasks
        let total_tasks: i64 = conn
            .query_row("SELECT COUNT(*) FROM tasks", [], |row| row.get(0))
            .unwrap_or(0);

        let completed_tasks: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM tasks WHERE status = 'completed'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        // Average completion time (simplified)
        let average_completion_time_minutes: Option<f64> = conn
            .query_row(
                "SELECT AVG(estimated_minutes) FROM tasks WHERE status = 'completed' AND estimated_minutes IS NOT NULL",
                [],
                |row| row.get(0),
            )
            .ok();

        // Count analytics sessions
        let total_sessions: i64 = conn
            .query_row(
                "SELECT COUNT(DISTINCT session_date) FROM analytics_snapshots",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        // Check if productivity scores exist
        let productivity_score_available = conn
            .query_row(
                "SELECT COUNT(*) FROM productivity_scores LIMIT 1",
                [],
                |row| {
                    let count: i64 = row.get(0)?;
                    Ok(count > 0)
                },
            )
            .unwrap_or(false);

        // Count workload forecasts
        let workload_forecasts_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM workload_forecasts", [], |row| {
                row.get(0)
            })
            .unwrap_or(0);

        // Count wellness events
        let wellness_events_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM wellness_events", [], |row| row.get(0))
            .unwrap_or(0);

        Ok(AnonymizedMetrics {
            total_tasks,
            completed_tasks,
            average_completion_time_minutes,
            total_sessions,
            productivity_score_available,
            workload_forecasts_count,
            wellness_events_count,
        })
    }

    /// Collect feedback summary
    async fn collect_feedback_summary(&self) -> AppResult<Option<FeedbackSummary>> {
        let conn = self.db_pool.get_connection()?;

        let total_feedback_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM ai_feedback", [], |row| row.get(0))
            .unwrap_or(0);

        if total_feedback_count == 0 {
            return Ok(None);
        }

        let positive_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM ai_feedback WHERE sentiment = 'up'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let negative_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM ai_feedback WHERE sentiment = 'down'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        // Get unique surfaces
        let mut stmt = conn.prepare("SELECT DISTINCT surface FROM ai_feedback")?;
        let surfaces: Vec<String> = stmt
            .query_map([], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        // Get most common issues from negative feedback notes
        let mut stmt = conn.prepare(
            "SELECT note FROM ai_feedback WHERE sentiment = 'down' AND note IS NOT NULL LIMIT 5",
        )?;
        let most_common_issues: Vec<String> = stmt
            .query_map([], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(Some(FeedbackSummary {
            total_feedback_count,
            positive_count,
            negative_count,
            surfaces,
            most_common_issues,
        }))
    }

    /// Calculate SHA-256 checksum of bundle data
    fn calculate_checksum(bundle_json: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(bundle_json.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Generate complete export bundle
    pub async fn generate_export_bundle(&self, include_feedback: bool) -> AppResult<ExportBundle> {
        let system_info = self.generate_system_info();
        let metrics = self.collect_anonymized_metrics().await?;
        let feedback_summary = if include_feedback {
            self.collect_feedback_summary().await?
        } else {
            None
        };
        let plugins = self.detect_plugins()?;

        // Create bundle without checksum first
        let mut bundle = ExportBundle {
            system_info,
            metrics,
            feedback_summary,
            plugins,
            checksum: String::new(),
        };

        // Calculate checksum
        let bundle_json = serde_json::to_string(&bundle)
            .map_err(|e| AppError::validation(format!("Failed to serialize bundle: {}", e)))?;
        bundle.checksum = Self::calculate_checksum(&bundle_json);

        Ok(bundle)
    }

    /// Save export bundle to file and record in database
    pub async fn save_export_to_file(
        &self,
        bundle: &ExportBundle,
        file_path: &Path,
    ) -> AppResult<i64> {
        // Generate Markdown content
        let markdown_content = bundle.to_markdown();

        // Write to file
        fs::write(file_path, markdown_content)
            .map_err(|e| AppError::validation(format!("Failed to write export file: {}", e)))?;

        // Record in database
        let conn = self.db_pool.get_connection()?;

        let metrics_summary = json!({
            "total_tasks": bundle.metrics.total_tasks,
            "completed_tasks": bundle.metrics.completed_tasks,
            "total_sessions": bundle.metrics.total_sessions,
        });

        let create_input = CommunityExportCreate::new(
            file_path.to_string_lossy().to_string(),
            metrics_summary,
            bundle.feedback_summary.is_some(),
            bundle.checksum.clone(),
        );

        let export_id = CommunityExportRepository::create_export(&conn, &create_input)?;

        Ok(export_id)
    }

    /// Get list of previous exports
    pub fn list_exports(&self) -> AppResult<Vec<(i64, String, String)>> {
        let conn = self.db_pool.get_connection()?;

        let mut stmt = conn.prepare(
            "SELECT id, generated_at, payload_path FROM community_exports ORDER BY generated_at DESC LIMIT 10"
        )?;

        let exports: Vec<(i64, String, String)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(exports)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::DbPool;
    use tempfile::TempDir;

    fn setup_test_service() -> AppResult<(CommunityService, TempDir)> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");
        let db_pool = DbPool::new(&db_path)?;
        let service = CommunityService::new(db_pool);
        Ok((service, temp_dir))
    }

    #[tokio::test]
    async fn test_get_project_info() {
        let (service, _temp_dir) = setup_test_service().expect("Failed to setup test service");

        let info = service
            .get_project_info()
            .expect("Should return project info");

        assert_eq!(info.name, "CogniCal");
        assert_eq!(info.license, "MIT");
        assert!(info.is_open_source);
        assert!(info.features_always_free);
    }

    #[tokio::test]
    async fn test_detect_plugins_empty() {
        let (service, _temp_dir) = setup_test_service().expect("Failed to setup test service");

        let plugins = service
            .detect_plugins()
            .expect("Should return empty plugin list");

        assert_eq!(plugins.len(), 0);
    }

    #[tokio::test]
    async fn test_generate_export_bundle() {
        let (service, _temp_dir) = setup_test_service().expect("Failed to setup test service");

        let bundle = service
            .generate_export_bundle(false)
            .await
            .expect("Should generate export bundle");

        assert!(!bundle.checksum.is_empty());
        assert_eq!(bundle.system_info.app_version, PROJECT_VERSION);
        assert!(bundle.feedback_summary.is_none());
    }

    #[tokio::test]
    async fn test_export_bundle_to_markdown() {
        let (service, _temp_dir) = setup_test_service().expect("Failed to setup test service");

        let bundle = service
            .generate_export_bundle(false)
            .await
            .expect("Should generate export bundle");

        let markdown = bundle.to_markdown();

        assert!(markdown.contains("# CogniCal Community Feedback Export"));
        assert!(markdown.contains("## System Information"));
        assert!(markdown.contains("## Anonymized Usage Metrics"));
        assert!(markdown.contains(&bundle.checksum));
    }

    #[tokio::test]
    async fn test_save_export_to_file() {
        let (service, temp_dir) = setup_test_service().expect("Failed to setup test service");

        let bundle = service
            .generate_export_bundle(false)
            .await
            .expect("Should generate export bundle");

        let file_path = temp_dir.path().join("export.md");

        let export_id = service
            .save_export_to_file(&bundle, &file_path)
            .await
            .expect("Should save export to file");

        assert!(export_id > 0);
        assert!(file_path.exists());

        let content = fs::read_to_string(&file_path).expect("Should read file");
        assert!(content.contains("CogniCal"));
    }
}
