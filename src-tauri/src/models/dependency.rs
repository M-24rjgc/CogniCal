use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TaskDependency {
    pub id: String,
    pub predecessor_id: String,
    pub successor_id: String,
    pub dependency_type: DependencyType,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DependencyType {
    FinishToStart,  // A must finish before B can start (default)
    StartToStart,   // A must start before B can start
    FinishToFinish, // A must finish before B can finish
    StartToFinish,  // A must start before B can finish
}

impl Default for DependencyType {
    fn default() -> Self {
        DependencyType::FinishToStart
    }
}

impl std::fmt::Display for DependencyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DependencyType::FinishToStart => write!(f, "finish_to_start"),
            DependencyType::StartToStart => write!(f, "start_to_start"),
            DependencyType::FinishToFinish => write!(f, "finish_to_finish"),
            DependencyType::StartToFinish => write!(f, "start_to_finish"),
        }
    }
}

impl std::str::FromStr for DependencyType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "finish_to_start" => Ok(DependencyType::FinishToStart),
            "start_to_start" => Ok(DependencyType::StartToStart),
            "finish_to_finish" => Ok(DependencyType::FinishToFinish),
            "start_to_finish" => Ok(DependencyType::StartToFinish),
            _ => Err(format!("Invalid dependency type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencyCreateInput {
    pub predecessor_id: String,
    pub successor_id: String,
    pub dependency_type: Option<DependencyType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencyValidation {
    pub is_valid: bool,
    pub error_message: Option<String>,
    pub would_create_cycle: bool,
    pub cycle_path: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskNode {
    pub task_id: String,
    pub status: String,
    pub dependencies: Vec<String>,
    pub dependents: Vec<String>,
    pub is_ready: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencyEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub dependency_type: DependencyType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencyGraph {
    pub nodes: HashMap<String, TaskNode>,
    pub edges: Vec<DependencyEdge>,
    pub topological_order: Vec<String>,
    pub critical_path: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencyFilter {
    pub task_ids: Option<Vec<String>>,
    pub include_completed: Option<bool>,
    pub max_depth: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadyTask {
    pub id: String,
    pub title: String,
    pub status: String,
    pub priority: String,
    pub due_at: Option<String>,
}