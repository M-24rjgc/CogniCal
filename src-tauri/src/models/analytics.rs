use chrono::Duration;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum AnalyticsGrouping {
    Day,
    Week,
}

impl AnalyticsGrouping {
    pub fn as_str(&self) -> &'static str {
        match self {
            AnalyticsGrouping::Day => "day",
            AnalyticsGrouping::Week => "week",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AnalyticsRangeKey {
    #[serde(rename = "7d")]
    SevenDays,
    #[serde(rename = "30d")]
    ThirtyDays,
    #[serde(rename = "90d")]
    NinetyDays,
}

impl AnalyticsRangeKey {
    pub fn as_str(&self) -> &'static str {
        match self {
            AnalyticsRangeKey::SevenDays => "7d",
            AnalyticsRangeKey::ThirtyDays => "30d",
            AnalyticsRangeKey::NinetyDays => "90d",
        }
    }

    pub fn duration(&self) -> Duration {
        match self {
            AnalyticsRangeKey::SevenDays => Duration::days(7),
            AnalyticsRangeKey::ThirtyDays => Duration::days(30),
            AnalyticsRangeKey::NinetyDays => Duration::days(90),
        }
    }
}

impl Default for AnalyticsRangeKey {
    fn default() -> Self {
        AnalyticsRangeKey::SevenDays
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum AnalyticsExportFormat {
    Markdown,
    Json,
}

impl AnalyticsExportFormat {
    pub fn file_extension(&self) -> &'static str {
        match self {
            AnalyticsExportFormat::Markdown => "md",
            AnalyticsExportFormat::Json => "json",
        }
    }
}

impl Default for AnalyticsExportFormat {
    fn default() -> Self {
        AnalyticsExportFormat::Markdown
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsQueryParams {
    #[serde(default)]
    pub range: AnalyticsRangeKey,
    #[serde(default)]
    pub from: Option<String>,
    #[serde(default)]
    pub to: Option<String>,
    #[serde(default)]
    pub grouping: Option<AnalyticsGrouping>,
}

impl Default for AnalyticsQueryParams {
    fn default() -> Self {
        Self {
            range: AnalyticsRangeKey::SevenDays,
            from: None,
            to: None,
            grouping: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsExportParams {
    pub range: AnalyticsRangeKey,
    #[serde(default)]
    pub format: AnalyticsExportFormat,
    #[serde(default)]
    pub from: Option<String>,
    #[serde(default)]
    pub to: Option<String>,
}

impl Default for AnalyticsExportParams {
    fn default() -> Self {
        Self {
            range: AnalyticsRangeKey::SevenDays,
            format: AnalyticsExportFormat::Markdown,
            from: None,
            to: None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrendPoint {
    pub date: String,
    pub completion_rate: f64,
    pub productivity_score: f64,
    pub completed_tasks: i64,
    pub focus_minutes: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeAllocationEntry {
    pub label: String,
    pub minutes: i64,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeAllocationTypeEntry {
    #[serde(rename = "type")]
    pub kind: String,
    pub minutes: i64,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeAllocationPriorityEntry {
    pub priority: String,
    pub minutes: i64,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeAllocationBreakdown {
    #[serde(default)]
    pub by_type: Vec<TimeAllocationTypeEntry>,
    #[serde(default)]
    pub by_priority: Vec<TimeAllocationPriorityEntry>,
    #[serde(default)]
    pub by_status: Vec<TimeAllocationEntry>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EfficiencySuggestion {
    pub id: String,
    pub title: String,
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_task_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_plan_id: Option<String>,
    pub impact: String,
    pub confidence: f64,
    pub category: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InsightCard {
    pub id: String,
    pub headline: String,
    pub detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_href: Option<String>,
    pub severity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_ids: Option<Vec<String>>,
    pub generated_at: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsEfficiency {
    pub estimate_accuracy: f64,
    pub on_time_rate: f64,
    pub complexity_correlation: f64,
    #[serde(default)]
    pub suggestions: Vec<EfficiencySuggestion>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsSummary {
    pub total_completed: i64,
    pub completion_rate: f64,
    pub trend_delta: f64,
    pub workload_prediction: i64,
    pub focus_minutes: i64,
    pub overdue_tasks: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ZeroStateMeta {
    pub is_empty: bool,
    #[serde(default)]
    pub recommended_actions: Vec<String>,
    pub sample_data_available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sample_data_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub missing_configuration: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsMeta {
    pub generated_at: String,
    pub is_demo: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsOverview {
    pub range: AnalyticsRangeKey,
    pub summary: AnalyticsSummary,
    #[serde(default)]
    pub trend: Vec<TrendPoint>,
    pub time_allocation: TimeAllocationBreakdown,
    pub efficiency: AnalyticsEfficiency,
    #[serde(default)]
    pub insights: Vec<InsightCard>,
    pub zero_state: ZeroStateMeta,
    pub meta: AnalyticsMeta,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsHistoryPoint {
    pub date: String,
    pub productivity_score: f64,
    pub completion_rate: f64,
    pub focus_minutes: i64,
    pub completed_tasks: i64,
    pub overdue_tasks: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsHistoryResponse {
    pub range: AnalyticsRangeKey,
    pub grouping: AnalyticsGrouping,
    #[serde(default)]
    pub points: Vec<AnalyticsHistoryPoint>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsErrorSummary {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsOverviewResponse {
    pub overview: AnalyticsOverview,
    pub history: AnalyticsHistoryResponse,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<AnalyticsErrorSummary>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsExportResult {
    pub file_path: String,
    pub format: AnalyticsExportFormat,
    pub generated_at: String,
    pub is_demo: bool,
}

#[derive(Debug, Clone)]
pub struct AnalyticsSnapshotRecord {
    pub snapshot_date: String,
    pub total_tasks_completed: i64,
    pub completion_rate: f64,
    pub overdue_tasks: i64,
    pub total_focus_minutes: i64,
    pub productivity_score: f64,
    pub efficiency_rating: f64,
    pub time_spent_work: f64,
    pub time_spent_study: f64,
    pub time_spent_life: f64,
    pub time_spent_other: f64,
    pub on_time_ratio: f64,
    pub focus_consistency: f64,
    pub rest_balance: f64,
    pub capacity_risk: f64,
    pub created_at: String,
}
