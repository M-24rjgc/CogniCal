use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PlanningSessionRecord {
    pub id: String,
    pub task_ids: Vec<String>,
    #[serde(default)]
    pub constraints: Option<JsonValue>,
    pub generated_at: String,
    pub status: String,
    #[serde(default)]
    pub selected_option_id: Option<String>,
    #[serde(default)]
    pub personalization_snapshot: Option<JsonValue>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PlanningOptionRecord {
    pub id: String,
    pub session_id: String,
    pub rank: i64,
    #[serde(default)]
    pub score: Option<f64>,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub cot_steps: Option<JsonValue>,
    #[serde(default)]
    pub risk_notes: Option<JsonValue>,
    pub is_fallback: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PlanningTimeBlockRecord {
    pub id: String,
    pub option_id: String,
    pub task_id: String,
    pub start_at: String,
    pub end_at: String,
    #[serde(default)]
    pub flexibility: Option<String>,
    #[serde(default)]
    pub confidence: Option<f64>,
    #[serde(default)]
    pub conflict_flags: Option<JsonValue>,
    #[serde(default)]
    pub applied_at: Option<String>,
    #[serde(default)]
    pub actual_start_at: Option<String>,
    #[serde(default)]
    pub actual_end_at: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SchedulePreferencesRecord {
    pub id: String,
    #[serde(default = "empty_object")]
    pub data: JsonValue,
    pub updated_at: String,
}

fn empty_object() -> JsonValue {
    JsonValue::Object(Default::default())
}
