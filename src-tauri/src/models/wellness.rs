use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WellnessTriggerReason {
    FocusStreak,
    WorkStreak,
}

impl WellnessTriggerReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            WellnessTriggerReason::FocusStreak => "focus_streak",
            WellnessTriggerReason::WorkStreak => "work_streak",
        }
    }
}

impl fmt::Display for WellnessTriggerReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl TryFrom<&str> for WellnessTriggerReason {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "focus_streak" => Ok(WellnessTriggerReason::FocusStreak),
            "work_streak" => Ok(WellnessTriggerReason::WorkStreak),
            other => Err(format!("unsupported wellness trigger: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WellnessResponse {
    Completed,
    Snoozed,
    Ignored,
}

impl WellnessResponse {
    pub fn as_str(&self) -> &'static str {
        match self {
            WellnessResponse::Completed => "completed",
            WellnessResponse::Snoozed => "snoozed",
            WellnessResponse::Ignored => "ignored",
        }
    }
}

impl fmt::Display for WellnessResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl TryFrom<&str> for WellnessResponse {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "completed" => Ok(WellnessResponse::Completed),
            "snoozed" => Ok(WellnessResponse::Snoozed),
            "ignored" => Ok(WellnessResponse::Ignored),
            other => Err(format!("unsupported wellness response: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WellnessEventRecord {
    pub id: i64,
    pub window_start: String,
    pub trigger_reason: WellnessTriggerReason,
    pub recommended_break_minutes: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_micro_task: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<WellnessResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_at: Option<String>,
    pub deferral_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WellnessEventInsert {
    pub window_start: String,
    pub trigger_reason: WellnessTriggerReason,
    pub recommended_break_minutes: i64,
    #[serde(default)]
    pub suggested_micro_task: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WellnessEventResponseUpdate {
    pub response: WellnessResponse,
    pub response_at: String,
    #[serde(default)]
    pub deferral_count: i64,
}
