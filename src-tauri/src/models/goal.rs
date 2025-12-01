use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub parent_goal_id: Option<String>,
    pub status: GoalStatus,
    pub priority: String,
    pub target_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum GoalStatus {
    NotStarted,
    InProgress,
    Completed,
    OnHold,
    Cancelled,
}

impl GoalStatus {
    pub fn as_str(&self) -> &str {
        match self {
            GoalStatus::NotStarted => "not_started",
            GoalStatus::InProgress => "in_progress",
            GoalStatus::Completed => "completed",
            GoalStatus::OnHold => "on_hold",
            GoalStatus::Cancelled => "cancelled",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "not_started" => Ok(GoalStatus::NotStarted),
            "in_progress" => Ok(GoalStatus::InProgress),
            "completed" => Ok(GoalStatus::Completed),
            "on_hold" => Ok(GoalStatus::OnHold),
            "cancelled" => Ok(GoalStatus::Cancelled),
            _ => Err(format!("Invalid goal status: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalTaskAssociation {
    pub id: String,
    pub goal_id: String,
    pub task_id: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoalWithProgress {
    #[serde(flatten)]
    pub goal: Goal,
    pub progress_percentage: f32,
    pub total_tasks: i32,
    pub completed_tasks: i32,
    pub in_progress_tasks: i32,
    pub blocked_tasks: i32,
    pub child_goals: Vec<GoalWithProgress>,
    pub is_on_track: bool,
    pub days_until_target: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalMilestone {
    pub goal_id: String,
    pub milestone_name: String,
    pub target_date: DateTime<Utc>,
    pub is_achieved: bool,
    pub achieved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGoalRequest {
    pub title: String,
    pub description: Option<String>,
    pub parent_goal_id: Option<String>,
    pub priority: String,
    pub target_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateGoalRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<GoalStatus>,
    pub priority: Option<String>,
    pub target_date: Option<DateTime<Utc>>,
}
