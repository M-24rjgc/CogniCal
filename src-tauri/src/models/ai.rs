use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct TaskParseContext {
    pub timezone: Option<String>,
    pub locale: Option<String>,
    pub reference_date: Option<String>,
    pub existing_task_id: Option<String>,
    pub metadata: Option<JsonValue>,
    pub user_preferences: Option<JsonValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskParseRequest {
    pub input: String,
    #[serde(default)]
    pub context: Option<TaskParseContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ParsedTaskPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub planned_start_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_minutes: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_hours: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_recurring: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recurrence: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_links: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskParseAiResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub complexity_score: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_start_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focus_mode: Option<TaskFocusModeRecommendation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub efficiency_prediction: Option<TaskEfficiencyPrediction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cot_steps: Option<Vec<TaskAiReasoningStep>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cot_summary: Option<String>,
    pub source: TaskAiSource,
    pub generated_at: String,
}

impl Default for TaskParseAiResult {
    fn default() -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            summary: None,
            next_action: None,
            confidence: None,
            metadata: None,
            complexity_score: None,
            suggested_start_at: None,
            focus_mode: None,
            efficiency_prediction: None,
            cot_steps: None,
            cot_summary: None,
            source: TaskAiSource::Live,
            generated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TaskFocusModeRecommendation {
    pub pomodoros: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommended_slots: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TaskEfficiencyPrediction {
    pub expected_hours: f64,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TaskAiReasoningStep {
    #[serde(default)]
    pub order: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default)]
    pub detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outcome: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TaskAiSource {
    Live,
    Cache,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskParseResponse {
    pub payload: ParsedTaskPayload,
    pub ai: TaskParseAiResult,
    pub missing_fields: Vec<String>,
}

impl TaskParseResponse {
    pub fn with_cache_source(mut self) -> Self {
        self.ai.source = TaskAiSource::Cache;
        self
    }
}

pub fn default_missing_fields() -> Vec<String> {
    vec!["ownerId".to_string()]
}

pub fn default_focus_mode_recommendation(slot: Option<String>) -> TaskFocusModeRecommendation {
    TaskFocusModeRecommendation {
        pomodoros: 3,
        recommended_slots: slot.map(|value| vec![value]),
    }
}

pub fn default_efficiency_prediction() -> TaskEfficiencyPrediction {
    TaskEfficiencyPrediction {
        expected_hours: 2.0,
        confidence: 0.6,
    }
}

pub fn default_reasoning_steps() -> Vec<TaskAiReasoningStep> {
    vec![
        TaskAiReasoningStep {
            order: 0,
            title: Some("提炼目标".to_string()),
            detail: "解析输入文本中的核心动词与期望结果。".to_string(),
            outcome: Some("得到任务主题与成功标准。".to_string()),
        },
        TaskAiReasoningStep {
            order: 1,
            title: Some("识别约束".to_string()),
            detail: "扫描文本中的日期、优先级与外部依赖。".to_string(),
            outcome: Some("确定计划开始与截止时间建议。".to_string()),
        },
        TaskAiReasoningStep {
            order: 2,
            title: Some("生成建议".to_string()),
            detail: "结合经验库预估复杂度与所需专注模式。".to_string(),
            outcome: Some("给出番茄钟数量与效率预测。".to_string()),
        },
    ]
}

pub fn default_ai_result() -> TaskParseAiResult {
    let now = Utc::now();
    let generated_at = now.to_rfc3339();
    let suggested_start = (now + Duration::hours(1)).to_rfc3339();

    TaskParseAiResult {
        summary: Some("系统建议根据输入自动补全关键字段。".to_string()),
        next_action: Some("确认主要目标是否准确，再拆分子任务。".to_string()),
        confidence: Some(0.75),
        metadata: None,
        complexity_score: Some(5.0),
        suggested_start_at: Some(suggested_start.clone()),
        focus_mode: Some(default_focus_mode_recommendation(Some(suggested_start))),
        efficiency_prediction: Some(default_efficiency_prediction()),
        cot_steps: Some(default_reasoning_steps()),
        cot_summary: Some("该任务需要专注执行，可按照建议的番茄钟节奏推进。".to_string()),
        source: TaskAiSource::Live,
        generated_at,
    }
}
