use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use tracing::{debug, info};

use crate::db::repositories::ai_feedback_repository::AiFeedbackRepository;
use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::ai_feedback::{
    AiFeedback, AiFeedbackCreate, AiFeedbackSentiment, AiFeedbackSurface,
};
use crate::services::settings_service::SettingsService;
use crate::utils::redact::redact_sensitive_data;

const MIN_FEEDBACK_FOR_DIGEST: usize = 5; // Minimum feedback entries to generate digest
const DIGEST_LOOKBACK_DAYS: i64 = 7; // Weekly digest looks back 7 days

/// Service for AI feedback capture and digest generation
pub struct FeedbackService {
    db: DbPool,
    settings_service: Arc<SettingsService>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedbackSubmission {
    pub surface: AiFeedbackSurface,
    pub session_id: Option<String>,
    pub sentiment: AiFeedbackSentiment,
    pub note: Option<String>,
    pub prompt_snapshot: String,
    pub context_snapshot: JsonValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeeklyDigest {
    pub period_start: String,
    pub period_end: String,
    pub total_feedback: usize,
    pub positive_count: usize,
    pub negative_count: usize,
    pub by_surface: Vec<SurfaceDigest>,
    pub insights: Vec<String>,
    pub adjustments_made: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SurfaceDigest {
    pub surface: String,
    pub positive: usize,
    pub negative: usize,
    pub satisfaction_rate: f64,
    pub sample_notes: Vec<String>,
}

impl FeedbackService {
    pub fn new(db: DbPool, settings_service: Arc<SettingsService>) -> Self {
        Self {
            db,
            settings_service,
        }
    }

    /// Check if user has opted out of AI feedback collection
    pub fn is_opted_out(&self) -> AppResult<bool> {
        let settings = self.settings_service.get()?;
        Ok(settings.ai_feedback_opt_out.unwrap_or(false))
    }

    /// Submit feedback with optional anonymization
    pub fn submit_feedback(&self, submission: &FeedbackSubmission) -> AppResult<i64> {
        // Check opt-out status
        if self.is_opted_out()? {
            return Err(AppError::validation(
                "AI feedback collection is disabled in settings",
            ));
        }

        let conn = self.db.get_connection()?;

        // Determine if anonymization is needed
        let should_anonymize =
            submission.note.is_some() || matches!(submission.sentiment, AiFeedbackSentiment::Down);

        let context_snapshot = if should_anonymize {
            redact_sensitive_data(&submission.context_snapshot)?
        } else {
            submission.context_snapshot.clone()
        };

        let create_input = AiFeedbackCreate {
            surface: submission.surface,
            session_id: submission.session_id.clone(),
            sentiment: submission.sentiment,
            note: submission.note.clone(),
            prompt_snapshot: submission.prompt_snapshot.clone(),
            context_snapshot,
            anonymized: should_anonymize,
        };

        let feedback_id = AiFeedbackRepository::create_feedback(&conn, &create_input)?;

        info!(
            "Submitted feedback: id={} surface={} sentiment={}",
            feedback_id,
            submission.surface.as_str(),
            submission.sentiment.as_str()
        );

        Ok(feedback_id)
    }

    /// Get recent feedback for a specific surface
    pub fn get_recent_feedback(
        &self,
        surface: AiFeedbackSurface,
        limit: Option<i64>,
    ) -> AppResult<Vec<AiFeedback>> {
        if self.is_opted_out()? {
            return Ok(Vec::new());
        }

        let conn = self.db.get_connection()?;
        AiFeedbackRepository::get_feedback_by_surface(&conn, surface, limit)
    }

    /// Get feedback for a specific session
    pub fn get_session_feedback(&self, session_id: &str) -> AppResult<Vec<AiFeedback>> {
        if self.is_opted_out()? {
            return Ok(Vec::new());
        }

        let conn = self.db.get_connection()?;
        AiFeedbackRepository::get_feedback_by_session(&conn, session_id)
    }

    /// Generate weekly digest if threshold is met
    pub fn generate_weekly_digest(&self) -> AppResult<Option<WeeklyDigest>> {
        if self.is_opted_out()? {
            return Ok(None);
        }

        let _conn = self.db.get_connection()?;

        let period_end = Utc::now();
        let period_start = period_end - Duration::days(DIGEST_LOOKBACK_DAYS);

        // Collect all feedback from the past week
        let all_feedback = self.get_feedback_since(&period_start)?;

        if all_feedback.len() < MIN_FEEDBACK_FOR_DIGEST {
            debug!(
                "Not enough feedback for digest: {} < {}",
                all_feedback.len(),
                MIN_FEEDBACK_FOR_DIGEST
            );
            return Ok(None);
        }

        let positive_count = all_feedback
            .iter()
            .filter(|f| matches!(f.sentiment, AiFeedbackSentiment::Up))
            .count();

        let negative_count = all_feedback
            .iter()
            .filter(|f| matches!(f.sentiment, AiFeedbackSentiment::Down))
            .count();

        // Group by surface
        let mut by_surface_map: std::collections::HashMap<String, Vec<&AiFeedback>> =
            std::collections::HashMap::new();
        for feedback in &all_feedback {
            by_surface_map
                .entry(feedback.surface.to_string())
                .or_insert_with(Vec::new)
                .push(feedback);
        }

        let mut by_surface = Vec::new();
        for (surface_name, feedbacks) in by_surface_map {
            let pos = feedbacks
                .iter()
                .filter(|f| matches!(f.sentiment, AiFeedbackSentiment::Up))
                .count();
            let neg = feedbacks
                .iter()
                .filter(|f| matches!(f.sentiment, AiFeedbackSentiment::Down))
                .count();

            let satisfaction_rate = if (pos + neg) > 0 {
                (pos as f64) / (pos + neg) as f64
            } else {
                0.0
            };

            // Collect sample notes (non-anonymized only)
            let sample_notes: Vec<String> = feedbacks
                .iter()
                .filter(|f| !f.anonymized && f.note.is_some())
                .filter_map(|f| f.note.clone())
                .take(3)
                .collect();

            by_surface.push(SurfaceDigest {
                surface: surface_name,
                positive: pos,
                negative: neg,
                satisfaction_rate,
                sample_notes,
            });
        }

        // Sort by negative count descending (prioritize problem areas)
        by_surface.sort_by(|a, b| b.negative.cmp(&a.negative));

        // Generate insights based on feedback patterns
        let insights = self.generate_insights(&all_feedback, &by_surface);

        // Generate adjustment recommendations
        let adjustments_made = self.generate_adjustments(&by_surface);

        let digest = WeeklyDigest {
            period_start: period_start.to_rfc3339(),
            period_end: period_end.to_rfc3339(),
            total_feedback: all_feedback.len(),
            positive_count,
            negative_count,
            by_surface,
            insights,
            adjustments_made,
        };

        info!(
            "Generated weekly digest: total={} positive={} negative={}",
            digest.total_feedback, digest.positive_count, digest.negative_count
        );

        Ok(Some(digest))
    }

    /// Purge all feedback data (for opt-out users)
    pub fn purge_all_feedback(&self) -> AppResult<i64> {
        let conn = self.db.get_connection()?;

        // Delete all feedback (using a very old date to delete everything)
        let future_date = Utc::now() + Duration::days(365 * 10); // Far future date
        let deleted_count =
            AiFeedbackRepository::delete_feedback_before(&conn, &future_date.to_rfc3339())?;

        info!("Purged all feedback: {} records deleted", deleted_count);

        Ok(deleted_count)
    }

    /// Get feedback statistics (admin/debug use)
    pub fn get_feedback_stats(&self, surface: Option<AiFeedbackSurface>) -> AppResult<JsonValue> {
        if self.is_opted_out()? {
            return Ok(serde_json::json!({
                "error": "Feedback collection is disabled"
            }));
        }

        let conn = self.db.get_connection()?;
        AiFeedbackRepository::get_feedback_stats(&conn, surface)
    }

    // --- Private helper methods ---

    /// Get all feedback since a specific date
    fn get_feedback_since(&self, since: &DateTime<Utc>) -> AppResult<Vec<AiFeedback>> {
        let conn = self.db.get_connection()?;

        // Get feedback for all surfaces
        let mut all_feedback = Vec::new();
        for surface in [
            AiFeedbackSurface::Score,
            AiFeedbackSurface::Recommendation,
            AiFeedbackSurface::Forecast,
        ] {
            let feedback = AiFeedbackRepository::get_feedback_by_surface(&conn, surface, None)?;
            all_feedback.extend(feedback);
        }

        // Filter by date
        all_feedback.retain(|f| {
            if let Ok(created) = DateTime::parse_from_rfc3339(&f.created_at) {
                created.with_timezone(&Utc) >= *since
            } else {
                false
            }
        });

        Ok(all_feedback)
    }

    /// Generate insights from feedback patterns
    fn generate_insights(
        &self,
        all_feedback: &[AiFeedback],
        by_surface: &[SurfaceDigest],
    ) -> Vec<String> {
        let mut insights = Vec::new();

        // Overall satisfaction
        let total_count = all_feedback.len();
        let positive_count = all_feedback
            .iter()
            .filter(|f| matches!(f.sentiment, AiFeedbackSentiment::Up))
            .count();

        if total_count > 0 {
            let satisfaction_rate = (positive_count as f64) / (total_count as f64);
            if satisfaction_rate >= 0.8 {
                insights.push(
                    "Overall satisfaction is high (≥80%). Keep up the good work!".to_string(),
                );
            } else if satisfaction_rate < 0.5 {
                insights.push("Overall satisfaction is below 50%. Review negative feedback for improvement areas.".to_string());
            }
        }

        // Surface-specific insights
        for surface in by_surface {
            if surface.satisfaction_rate < 0.5 && (surface.positive + surface.negative) >= 3 {
                insights.push(format!(
                    "{} feature needs attention: {}% satisfaction rate",
                    surface.surface,
                    (surface.satisfaction_rate * 100.0) as i32
                ));
            }
        }

        // Trend detection (if we have more than 10 feedback items)
        if all_feedback.len() > 10 {
            let recent_half = &all_feedback[..all_feedback.len() / 2];
            let older_half = &all_feedback[all_feedback.len() / 2..];

            let recent_positive_rate = recent_half
                .iter()
                .filter(|f| matches!(f.sentiment, AiFeedbackSentiment::Up))
                .count() as f64
                / recent_half.len() as f64;

            let older_positive_rate = older_half
                .iter()
                .filter(|f| matches!(f.sentiment, AiFeedbackSentiment::Up))
                .count() as f64
                / older_half.len() as f64;

            if recent_positive_rate > older_positive_rate + 0.2 {
                insights.push("Positive trend: Recent satisfaction is improving!".to_string());
            } else if recent_positive_rate < older_positive_rate - 0.2 {
                insights.push("Negative trend: Recent satisfaction is declining.".to_string());
            }
        }

        if insights.is_empty() {
            insights.push("Not enough data for detailed insights yet.".to_string());
        }

        insights
    }

    /// Generate adjustment recommendations based on feedback
    fn generate_adjustments(&self, by_surface: &[SurfaceDigest]) -> Vec<String> {
        let mut adjustments = Vec::new();

        for surface in by_surface {
            if surface.negative > surface.positive && surface.negative >= 3 {
                match surface.surface.as_str() {
                    "score" => {
                        adjustments.push(
                            "Consider reviewing productivity score calculation weights."
                                .to_string(),
                        );
                        adjustments
                            .push("Check if time tracking data quality is sufficient.".to_string());
                    }
                    "recommendation" => {
                        adjustments.push("Review recommendation algorithm parameters.".to_string());
                        adjustments
                            .push("Ensure task scheduling considers user preferences.".to_string());
                    }
                    "forecast" => {
                        adjustments.push(
                            "Improve workload forecast accuracy with more historical data."
                                .to_string(),
                        );
                        adjustments.push("Adjust forecast confidence thresholds.".to_string());
                    }
                    _ => {}
                }
            }
        }

        if adjustments.is_empty() {
            adjustments
                .push("All systems performing well. Continue monitoring feedback.".to_string());
        }

        adjustments
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::DbPool;
    use tempfile::TempDir;

    fn setup_test_service() -> AppResult<(FeedbackService, Arc<SettingsService>, TempDir)> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");
        let db = DbPool::new(&db_path)?;
        let settings_service = Arc::new(SettingsService::new(db.clone())?);
        let feedback_service = FeedbackService::new(db.clone(), settings_service.clone());
        Ok((feedback_service, settings_service, temp_dir))
    }

    #[test]
    fn test_submit_feedback() -> AppResult<()> {
        let (service, _, _temp_dir) = setup_test_service()?;

        let submission = FeedbackSubmission {
            surface: AiFeedbackSurface::Score,
            session_id: Some("test-session".to_string()),
            sentiment: AiFeedbackSentiment::Up,
            note: Some("Great feature!".to_string()),
            prompt_snapshot: "Calculate productivity score".to_string(),
            context_snapshot: serde_json::json!({"task_count": 5}),
        };

        let feedback_id = service.submit_feedback(&submission)?;
        assert!(feedback_id > 0);

        Ok(())
    }

    #[test]
    fn test_opt_out() -> AppResult<()> {
        let (service, settings_service, _temp_dir) = setup_test_service()?;

        // Enable opt-out
        let input = crate::services::settings_service::SettingsUpdateInput {
            ai_feedback_opt_out: Some(true),
            ..Default::default()
        };
        settings_service.update(input)?;

        // Try to submit feedback - should fail
        let submission = FeedbackSubmission {
            surface: AiFeedbackSurface::Score,
            session_id: None,
            sentiment: AiFeedbackSentiment::Up,
            note: None,
            prompt_snapshot: "test".to_string(),
            context_snapshot: serde_json::json!({}),
        };

        let result = service.submit_feedback(&submission);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_purge_feedback() -> AppResult<()> {
        let (service, _, _temp_dir) = setup_test_service()?;

        // Submit some feedback
        let submission = FeedbackSubmission {
            surface: AiFeedbackSurface::Recommendation,
            session_id: None,
            sentiment: AiFeedbackSentiment::Down,
            note: Some("Not helpful".to_string()),
            prompt_snapshot: "Generate plan".to_string(),
            context_snapshot: serde_json::json!({}),
        };

        service.submit_feedback(&submission)?;

        // Purge all
        let deleted = service.purge_all_feedback()?;
        assert!(deleted > 0);

        // Verify empty
        let feedback = service.get_recent_feedback(AiFeedbackSurface::Recommendation, None)?;
        assert_eq!(feedback.len(), 0);

        Ok(())
    }
}
