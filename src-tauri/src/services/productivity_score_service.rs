use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::db::repositories::analytics_repository::AnalyticsRepository;
use crate::db::repositories::productivity_repository::ProductivityRepository;
use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::analytics::AnalyticsSnapshotRecord;
use crate::models::productivity::{ProductivityScoreRecord, ProductivityScoreUpsert};
use crate::utils::cot::CoTSummarizer;

/// Productivity scoring engine that calculates composite and dimension scores
/// based on task completion, focus consistency, efficiency, and work-life balance.
pub struct ProductivityScoreService {
    db: DbPool,
    cot_summarizer: CoTSummarizer,
}

impl ProductivityScoreService {
    pub fn new(db: DbPool) -> Self {
        Self {
            db,
            cot_summarizer: CoTSummarizer::new(),
        }
    }

    /// Calculate productivity score for a specific date
    pub fn calculate_score_for_date(&self, date: &str) -> AppResult<ProductivityScoreRecord> {
        let conn = self.db.get_connection()?;

        // Parse date string
        let naive_date = NaiveDate::parse_from_str(date, "%Y-%m-%d")
            .map_err(|e| AppError::Other(format!("Invalid date format: {}", e)))?;

        // Get analytics data for the date
        let snapshot_row = AnalyticsRepository::find_by_date(&conn, &naive_date)?;

        if snapshot_row.is_none() {
            return Err(AppError::NotFound);
        }

        let snapshot_row = snapshot_row.unwrap();
        let snapshot = snapshot_row.into_record();

        // Calculate dimension scores
        let dimension_scores = self.calculate_dimension_scores(&snapshot)?;

        // Calculate weights based on data availability and user preferences
        let weight_breakdown = self.calculate_weights(&dimension_scores)?;

        // Calculate composite score
        let composite_score =
            self.calculate_composite_score(&dimension_scores, &weight_breakdown)?;

        // Generate explanation using CoT
        let explanation =
            self.generate_explanation(&dimension_scores, &weight_breakdown, composite_score)?;

        // Create the score record
        let score_upsert = ProductivityScoreUpsert {
            snapshot_date: date.to_string(),
            composite_score,
            dimension_scores: serde_json::to_value(dimension_scores)?,
            weight_breakdown: serde_json::to_value(weight_breakdown)?,
            explanation: Some(explanation),
        };

        // Persist the score
        ProductivityRepository::upsert_score(&conn, &score_upsert)?;

        // Return the created record
        let created = ProductivityRepository::find_by_date(&conn, &naive_date)?
            .ok_or_else(|| AppError::Other("Failed to retrieve created score".to_string()))?;

        Ok(created)
    }

    /// Get productivity score history for a date range
    pub fn get_score_history(
        &self,
        _start_date: &str,
        _end_date: &str,
    ) -> AppResult<Vec<ProductivityScoreRecord>> {
        // This would need to be implemented in the repository
        // For now, return empty vec
        Ok(vec![])
    }

    /// Get the latest productivity score
    pub fn get_latest_score(&self) -> AppResult<Option<ProductivityScoreRecord>> {
        let conn = self.db.get_connection()?;

        // Get today's date
        let today = Utc::now().date_naive();

        ProductivityRepository::find_by_date(&conn, &today)
    }

    /// Calculate scores for all dimensions
    fn calculate_dimension_scores(
        &self,
        snapshot: &AnalyticsSnapshotRecord,
    ) -> AppResult<DimensionScores> {
        let completion_rate = snapshot.completion_rate.clamp(0.0, 1.0);
        let on_time_ratio = snapshot.on_time_ratio.clamp(0.0, 1.0);
        let focus_consistency = snapshot.focus_consistency.clamp(0.0, 1.0);
        let rest_balance = snapshot.rest_balance.clamp(0.0, 1.0);
        let efficiency_rating = snapshot.efficiency_rating.clamp(0.0, 1.0);

        Ok(DimensionScores {
            completion_rate: completion_rate * 100.0,
            on_time_ratio: on_time_ratio * 100.0,
            focus_consistency: focus_consistency * 100.0,
            rest_balance: rest_balance * 100.0,
            efficiency_rating: efficiency_rating * 100.0,
        })
    }

    /// Calculate weights for each dimension based on data availability
    fn calculate_weights(&self, dimension_scores: &DimensionScores) -> AppResult<DimensionWeights> {
        let total_tasks = dimension_scores.completion_rate > 0.0;
        let has_focus_data = dimension_scores.focus_consistency > 0.0;
        let has_efficiency_data = dimension_scores.efficiency_rating > 0.0;
        let has_balance_data = dimension_scores.rest_balance > 0.0;

        let mut weights = DimensionWeights::default();
        let mut total_weight = 0.0;

        if total_tasks {
            weights.completion_rate = 0.3;
            total_weight += 0.3;
        }

        if has_focus_data {
            weights.focus_consistency = 0.2;
            total_weight += 0.2;
        }

        if has_efficiency_data {
            weights.efficiency_rating = 0.15;
            total_weight += 0.15;
        }

        if has_balance_data {
            weights.rest_balance = 0.15;
            total_weight += 0.15;
        }

        // Always include on-time ratio if we have completion data
        if total_tasks {
            weights.on_time_ratio = 0.2;
            total_weight += 0.2;
        }

        // Normalize weights if we don't have all data
        if total_weight > 0.0 && total_weight < 1.0 {
            let scale = 1.0 / total_weight;
            weights.completion_rate *= scale;
            weights.on_time_ratio *= scale;
            weights.focus_consistency *= scale;
            weights.efficiency_rating *= scale;
            weights.rest_balance *= scale;
        }

        Ok(weights)
    }

    /// Calculate composite score from dimensions and weights
    fn calculate_composite_score(
        &self,
        dimensions: &DimensionScores,
        weights: &DimensionWeights,
    ) -> AppResult<f64> {
        let composite = dimensions.completion_rate * weights.completion_rate
            + dimensions.on_time_ratio * weights.on_time_ratio
            + dimensions.focus_consistency * weights.focus_consistency
            + dimensions.efficiency_rating * weights.efficiency_rating
            + dimensions.rest_balance * weights.rest_balance;

        Ok(composite.clamp(0.0, 100.0))
    }

    /// Generate explanation using Chain-of-Thought reasoning
    fn generate_explanation(
        &self,
        dimensions: &DimensionScores,
        weights: &DimensionWeights,
        composite_score: f64,
    ) -> AppResult<String> {
        let context = serde_json::json!({
            "dimensions": {
                "completionRate": dimensions.completion_rate,
                "onTimeRatio": dimensions.on_time_ratio,
                "focusConsistency": dimensions.focus_consistency,
                "efficiencyRating": dimensions.efficiency_rating,
                "restBalance": dimensions.rest_balance
            },
            "weights": weights,
            "compositeScore": composite_score
        });

        let prompt = format!(
            "Analyze this productivity data and provide a concise explanation:\n{}\n\nFocus on: 1) Key strengths, 2) Areas for improvement, 3) Overall assessment",
            serde_json::to_string_pretty(&context)?
        );

        // Use CoT summarizer to generate explanation
        let explanation = match self.cot_summarizer.summarize_sync(&prompt) {
            Ok(summary) => summary,
            Err(err) => {
                error!(
                    target: "app::productivity",
                    "CoT summarization failed, using fallback: {}",
                    err
                );
                self.generate_fallback_explanation(dimensions, composite_score)
            }
        };

        Ok(explanation)
    }

    /// Generate fallback explanation when CoT fails
    fn generate_fallback_explanation(
        &self,
        dimensions: &DimensionScores,
        composite_score: f64,
    ) -> String {
        let mut insights = Vec::new();

        if dimensions.completion_rate >= 80.0 {
            insights.push("Strong task completion rate");
        } else if dimensions.completion_rate < 50.0 {
            insights.push("Consider focusing on completing more tasks");
        }

        if dimensions.focus_consistency >= 75.0 {
            insights.push("Good focus consistency");
        } else if dimensions.focus_consistency < 50.0 {
            insights.push("Work on maintaining focus during work sessions");
        }

        if dimensions.rest_balance >= 70.0 {
            insights.push("Healthy work-life balance");
        } else if dimensions.rest_balance < 40.0 {
            insights.push("Consider taking more breaks to maintain balance");
        }

        if composite_score >= 80.0 {
            insights.push("Excellent overall productivity");
        } else if composite_score >= 60.0 {
            insights.push("Good productivity with room for improvement");
        } else {
            insights.push("Focus on key areas to boost productivity");
        }

        insights.join(". ")
    }

    /// Batch calculate scores for multiple dates (for historical data)
    pub fn batch_calculate_scores(
        &self,
        dates: Vec<String>,
    ) -> AppResult<Vec<ProductivityScoreRecord>> {
        let mut results = Vec::new();

        for date in dates {
            match self.calculate_score_for_date(&date) {
                Ok(score) => results.push(score),
                Err(e) => {
                    error!(target: "app::productivity", "Failed to calculate score for {}: {}", date, e);
                    // Continue with other dates
                }
            }
        }

        Ok(results)
    }

    /// Get insufficient data warning
    pub fn has_insufficient_data(&self, snapshot: &AnalyticsSnapshotRecord) -> bool {
        snapshot.total_tasks_completed < 3
            || snapshot.total_focus_minutes < 60
            || snapshot.completion_rate < 0.1
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DimensionScores {
    pub completion_rate: f64,
    pub on_time_ratio: f64,
    pub focus_consistency: f64,
    pub rest_balance: f64,
    pub efficiency_rating: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DimensionWeights {
    pub completion_rate: f64,
    pub on_time_ratio: f64,
    pub focus_consistency: f64,
    pub rest_balance: f64,
    pub efficiency_rating: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::analytics::AnalyticsSnapshotRecord;
    use tempfile::tempdir;

    #[test]
    fn test_dimension_scores_calculation() {
        let (service, _dir) = create_test_service();
        let snapshot = AnalyticsSnapshotRecord {
            snapshot_date: "2025-10-13".to_string(),
            total_tasks_completed: 8,
            completion_rate: 0.8,
            overdue_tasks: 1,
            total_focus_minutes: 240,
            productivity_score: 75.0,
            efficiency_rating: 0.85,
            time_spent_work: 360.0,
            time_spent_study: 60.0,
            time_spent_life: 120.0,
            time_spent_other: 0.0,
            on_time_ratio: 0.75,
            focus_consistency: 0.7,
            rest_balance: 0.6,
            capacity_risk: 0.2,
            created_at: Utc::now().to_rfc3339(),
        };

        let dimensions = service.calculate_dimension_scores(&snapshot).unwrap();

        assert_eq!(dimensions.completion_rate, 80.0);
        assert_eq!(dimensions.on_time_ratio, 75.0);
        assert_eq!(dimensions.focus_consistency, 70.0);
        assert_eq!(dimensions.rest_balance, 60.0);
        assert_eq!(dimensions.efficiency_rating, 85.0);
    }

    #[test]
    fn test_composite_score_calculation() {
        let (service, _dir) = create_test_service();
        let dimensions = DimensionScores {
            completion_rate: 80.0,
            on_time_ratio: 75.0,
            focus_consistency: 70.0,
            rest_balance: 60.0,
            efficiency_rating: 85.0,
        };

        let weights = DimensionWeights {
            completion_rate: 0.3,
            on_time_ratio: 0.2,
            focus_consistency: 0.2,
            rest_balance: 0.15,
            efficiency_rating: 0.15,
        };

        let composite = service
            .calculate_composite_score(&dimensions, &weights)
            .unwrap();

        // Expected: 80*0.3 + 75*0.2 + 70*0.2 + 60*0.15 + 85*0.15 = 24 + 15 + 14 + 9 + 12.75 = 74.75
        assert!((composite - 74.75).abs() < 0.01);
    }

    #[test]
    fn test_insufficient_data_detection() {
        let (service, _dir) = create_test_service();

        let insufficient_snapshot = AnalyticsSnapshotRecord {
            snapshot_date: "2025-10-13".to_string(),
            total_tasks_completed: 1, // Too few tasks
            completion_rate: 0.5,
            overdue_tasks: 0,
            total_focus_minutes: 30, // Too little focus time
            productivity_score: 50.0,
            efficiency_rating: 0.6,
            time_spent_work: 60.0,
            time_spent_study: 0.0,
            time_spent_life: 30.0,
            time_spent_other: 0.0,
            on_time_ratio: 0.5,
            focus_consistency: 0.4,
            rest_balance: 0.3,
            capacity_risk: 0.1,
            created_at: Utc::now().to_rfc3339(),
        };

        assert!(service.has_insufficient_data(&insufficient_snapshot));

        let sufficient_snapshot = AnalyticsSnapshotRecord {
            snapshot_date: "2025-10-13".to_string(),
            total_tasks_completed: 5,
            completion_rate: 0.8,
            overdue_tasks: 1,
            total_focus_minutes: 180,
            productivity_score: 75.0,
            efficiency_rating: 0.85,
            time_spent_work: 300.0,
            time_spent_study: 60.0,
            time_spent_life: 90.0,
            time_spent_other: 0.0,
            on_time_ratio: 0.75,
            focus_consistency: 0.7,
            rest_balance: 0.6,
            capacity_risk: 0.2,
            created_at: Utc::now().to_rfc3339(),
        };

        assert!(!service.has_insufficient_data(&sufficient_snapshot));
    }

    fn create_test_service() -> (ProductivityScoreService, tempfile::TempDir) {
        let dir = tempdir().expect("create temp dir");
        let db_path = dir.path().join("productivity.sqlite");
        let pool = DbPool::new(db_path).expect("create db pool");
        (ProductivityScoreService::new(pool), dir)
    }
}
