use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deepseek_api_key: Option<String>,
    pub workday_start_minute: i16,
    pub workday_end_minute: i16,
    pub theme: String,
    pub updated_at: String,
    /// Privacy setting: Opt out of AI feedback collection
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_feedback_opt_out: Option<bool>,
}
