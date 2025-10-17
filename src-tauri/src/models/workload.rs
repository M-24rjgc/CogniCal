use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum WorkloadHorizon {
    #[serde(rename = "7d")]
    SevenDays,
    #[serde(rename = "14d")]
    FourteenDays,
    #[serde(rename = "30d")]
    ThirtyDays,
}

impl WorkloadHorizon {
    pub fn as_str(&self) -> &'static str {
        match self {
            WorkloadHorizon::SevenDays => "7d",
            WorkloadHorizon::FourteenDays => "14d",
            WorkloadHorizon::ThirtyDays => "30d",
        }
    }
}

impl fmt::Display for WorkloadHorizon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl TryFrom<&str> for WorkloadHorizon {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "7d" => Ok(WorkloadHorizon::SevenDays),
            "14d" => Ok(WorkloadHorizon::FourteenDays),
            "30d" => Ok(WorkloadHorizon::ThirtyDays),
            other => Err(format!("unsupported workload horizon: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WorkloadRiskLevel {
    Ok,
    Warning,
    Critical,
}

impl WorkloadRiskLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            WorkloadRiskLevel::Ok => "ok",
            WorkloadRiskLevel::Warning => "warning",
            WorkloadRiskLevel::Critical => "critical",
        }
    }
}

impl fmt::Display for WorkloadRiskLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl TryFrom<&str> for WorkloadRiskLevel {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "ok" => Ok(WorkloadRiskLevel::Ok),
            "warning" => Ok(WorkloadRiskLevel::Warning),
            "critical" => Ok(WorkloadRiskLevel::Critical),
            other => Err(format!("unsupported risk level: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContributingTaskSummary {
    pub task_id: String,
    pub title: String,
    pub estimated_hours: f64,
    pub due_at: Option<String>,
    pub priority: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkloadForecastRecord {
    pub horizon: WorkloadHorizon,
    pub generated_at: String,
    pub risk_level: WorkloadRiskLevel,
    pub total_hours: f64,
    pub capacity_threshold: f64,
    pub contributing_tasks: Vec<ContributingTaskSummary>,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkloadForecastResponse {
    pub horizon: String,
    pub generated_at: String,
    pub risk_level: String,
    pub total_hours: f64,
    pub capacity_threshold: f64,
    pub contributing_tasks: Vec<ContributingTaskSummary>,
    pub confidence: f64,
    pub recommendations: Vec<String>,
}
