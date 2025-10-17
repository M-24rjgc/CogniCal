use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::models::ai::{
    TaskAiReasoningStep, TaskAiSource, TaskEfficiencyPrediction, TaskFocusModeRecommendation,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TaskRecord {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub planned_start_at: Option<String>,
    pub start_at: Option<String>,
    pub due_at: Option<String>,
    pub completed_at: Option<String>,
    pub estimated_minutes: Option<i64>,
    pub estimated_hours: Option<f64>,
    pub tags: Vec<String>,
    pub owner_id: Option<String>,
    pub task_type: Option<String>,
    pub is_recurring: bool,
    pub recurrence: Option<TaskRecurrence>,
    pub ai: Option<TaskAiInsights>,
    pub external_links: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TaskRecurrence {
    pub rule: String,
    pub until: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TaskAiInsights {
    pub summary: Option<String>,
    pub next_action: Option<String>,
    pub confidence: Option<f64>,
    pub metadata: Option<JsonValue>,
    pub complexity_score: Option<f64>,
    pub suggested_start_at: Option<String>,
    pub focus_mode: Option<TaskFocusModeRecommendation>,
    pub efficiency_prediction: Option<TaskEfficiencyPrediction>,
    pub cot_steps: Option<Vec<TaskAiReasoningStep>>,
    pub cot_summary: Option<String>,
    pub source: Option<TaskAiSource>,
    pub generated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TaskCreateInput {
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub priority: Option<String>,
    #[serde(default)]
    pub planned_start_at: Option<String>,
    #[serde(default)]
    pub start_at: Option<String>,
    #[serde(default)]
    pub due_at: Option<String>,
    #[serde(default)]
    pub completed_at: Option<String>,
    #[serde(default)]
    pub estimated_minutes: Option<i64>,
    #[serde(default)]
    pub estimated_hours: Option<f64>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub owner_id: Option<String>,
    #[serde(default)]
    pub is_recurring: Option<bool>,
    #[serde(default)]
    pub recurrence: Option<TaskRecurrence>,
    #[serde(default)]
    pub task_type: Option<String>,
    #[serde(default)]
    pub ai: Option<TaskAiInsights>,
    #[serde(default)]
    pub external_links: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TaskUpdateInput {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<Option<String>>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub priority: Option<String>,
    #[serde(default)]
    pub planned_start_at: Option<Option<String>>,
    #[serde(default)]
    pub start_at: Option<Option<String>>,
    #[serde(default)]
    pub due_at: Option<Option<String>>,
    #[serde(default)]
    pub completed_at: Option<Option<String>>,
    #[serde(default)]
    pub estimated_minutes: Option<Option<i64>>,
    #[serde(default)]
    pub estimated_hours: Option<Option<f64>>,
    #[serde(default)]
    pub tags: Option<Option<Vec<String>>>,
    #[serde(default)]
    pub owner_id: Option<Option<String>>,
    #[serde(default)]
    pub is_recurring: Option<bool>,
    #[serde(default)]
    pub recurrence: Option<Option<TaskRecurrence>>,
    #[serde(default)]
    pub task_type: Option<Option<String>>,
    #[serde(default)]
    pub ai: Option<Option<TaskAiInsights>>,
    #[serde(default)]
    pub external_links: Option<Option<Vec<String>>>,
}
