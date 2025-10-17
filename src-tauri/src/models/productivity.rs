use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductivityScoreRecord {
    pub snapshot_date: String,
    pub composite_score: f64,
    #[serde(default)]
    pub dimension_scores: Value,
    #[serde(default)]
    pub weight_breakdown: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explanation: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductivityScoreUpsert {
    pub snapshot_date: String,
    pub composite_score: f64,
    pub dimension_scores: Value,
    pub weight_breakdown: Value,
    #[serde(default)]
    pub explanation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductivityScoreHistoryResponse {
    pub scores: Vec<ProductivityScoreRecord>,
    pub start_date: String,
    pub end_date: String,
    pub total_scores: usize,
}
