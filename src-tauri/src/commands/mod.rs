pub mod ai;
pub mod ai_commands;
pub mod analytics;
pub mod cache;
pub mod community;
pub mod dependency_commands;
pub mod feedback;
pub mod goal_commands;
pub mod planning;
pub mod recurring_commands;
pub mod settings;
pub mod task;
pub mod wellness;

use std::sync::Arc;

use serde::Serialize;
use serde_json::{Map as JsonMap, Value as JsonValue};
use tracing::{error, warn};

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::services::ai_agent_service::AiAgentService;
use crate::services::ai_service::AiService;
use crate::services::analytics_service::AnalyticsService;
use crate::services::community_service::CommunityService;
use crate::services::dependency_service::DependencyService;
use crate::services::feedback_service::FeedbackService;
use crate::services::goal_service::GoalService;
use crate::services::memory_service::MemoryService;
use crate::services::planning_service::PlanningService;
use crate::services::productivity_score_service::ProductivityScoreService;
use crate::services::settings_service::SettingsService;
use crate::services::task_service::TaskService;
use crate::services::tool_registry::ToolRegistry;
use crate::services::wellness_service::WellnessService;
use crate::services::workload_forecast_service::WorkloadForecastService;

#[derive(Clone)]
pub struct AppState {
    db_pool: DbPool,
    task_service: Arc<TaskService>,
    ai_service: Arc<AiService>,
    planning_service: Arc<PlanningService>,
    analytics_service: Arc<AnalyticsService>,
    productivity_score_service: Arc<ProductivityScoreService>,
    settings_service: Arc<SettingsService>,
    wellness_service: Arc<WellnessService>,
    workload_forecast_service: Arc<WorkloadForecastService>,
    feedback_service: Arc<FeedbackService>,
    pub community_service: CommunityService,
    dependency_service: Arc<DependencyService>,
    memory_service: Arc<MemoryService>,
    goal_service: Arc<GoalService>,
    recurring_task_service: Arc<crate::services::recurring_task_service::RecurringTaskService>,

    tool_registry: Arc<ToolRegistry>,
    agent_service: Arc<AiAgentService>,
}

impl AppState {
    pub fn new(db_pool: DbPool, memory_base_dir: std::path::PathBuf) -> AppResult<Self> {
        let task_service = Arc::new(TaskService::new(db_pool.clone()));
        let ai_service = Arc::new(AiService::new(db_pool.clone())?);
        let planning_service = Arc::new(PlanningService::new(
            db_pool.clone(),
            Arc::clone(&task_service),
            Arc::clone(&ai_service),
        ));
        let analytics_service = Arc::new(AnalyticsService::new(
            db_pool.clone(),
            Arc::clone(&task_service),
        )?);

        let productivity_score_service = Arc::new(ProductivityScoreService::new(db_pool.clone()));
        let settings_service = Arc::new(SettingsService::new(db_pool.clone())?);
        let wellness_service = Arc::new(WellnessService::new(
            db_pool.clone(),
            Arc::clone(&settings_service),
        ));
        let workload_forecast_service = Arc::new(WorkloadForecastService::new(
            db_pool.clone(),
            Arc::clone(&task_service),
        ));
        let feedback_service = Arc::new(FeedbackService::new(
            db_pool.clone(),
            Arc::clone(&settings_service),
        ));
        let community_service = CommunityService::new(db_pool.clone());

        // Initialize memory service with provided base directory
        let memory_dir = memory_base_dir.join("memory");
        let memory_service = Arc::new(MemoryService::new(memory_dir)?);

        // Initialize goal service
        let goal_service = Arc::new(GoalService::new(db_pool.clone()));

        // Initialize dependency service
        let dependency_service = Arc::new(DependencyService::new(db_pool.clone()));

        // Initialize recurring task service
        let recurring_task_service = Arc::new(
            crate::services::recurring_task_service::RecurringTaskService::new(db_pool.clone()),
        );

        // Initialize tool registry and register tools
        let mut tool_registry = ToolRegistry::new();

        // Register unified time management tools (replaces task_tools and calendar_tools)
        crate::tools::time_management_tools::register_time_management_tools(
            &mut tool_registry,
            Arc::clone(&task_service),
        )?;

        // Register dependency management tools
        crate::tools::dependency_tools::register_dependency_tools(
            &mut tool_registry,
            Arc::clone(&dependency_service),
        )?;

        // Register goal management tools
        crate::tools::goal_tools::register_goal_tools(
            &mut tool_registry,
            Arc::clone(&goal_service),
        )?;

        // Register recurring task management tools
        crate::tools::recurring_task_tools::register_recurring_task_tools(
            &mut tool_registry,
            Arc::clone(&recurring_task_service),
        )?;

        let tool_registry = Arc::new(tool_registry);

        // Initialize AI agent service with memory
        let agent_service = Arc::new(AiAgentService::new_with_memory(
            Arc::clone(&ai_service),
            Arc::clone(&tool_registry),
            Arc::clone(&memory_service),
        ));

        analytics_service.ensure_snapshot_job()?;
        wellness_service.ensure_nudge_job()?;
        workload_forecast_service.ensure_nightly_job()?;

        Ok(Self {
            db_pool,
            task_service,
            ai_service,
            planning_service,
            analytics_service,
            productivity_score_service,
            settings_service,
            wellness_service,
            workload_forecast_service,
            feedback_service,
            community_service,
            dependency_service,
            memory_service,
            goal_service,
            recurring_task_service,

            tool_registry,
            agent_service,
        })
    }

    pub fn tasks(&self) -> Arc<TaskService> {
        Arc::clone(&self.task_service)
    }

    pub fn ai(&self) -> Arc<AiService> {
        Arc::clone(&self.ai_service)
    }

    pub fn planning(&self) -> Arc<PlanningService> {
        Arc::clone(&self.planning_service)
    }

    pub fn analytics(&self) -> Arc<AnalyticsService> {
        Arc::clone(&self.analytics_service)
    }

    pub fn productivity_score_service(&self) -> Arc<ProductivityScoreService> {
        Arc::clone(&self.productivity_score_service)
    }

    pub fn settings(&self) -> Arc<SettingsService> {
        Arc::clone(&self.settings_service)
    }

    pub fn wellness(&self) -> Arc<WellnessService> {
        Arc::clone(&self.wellness_service)
    }

    pub fn workload_forecast(&self) -> Arc<WorkloadForecastService> {
        Arc::clone(&self.workload_forecast_service)
    }

    pub fn feedback(&self) -> Arc<FeedbackService> {
        Arc::clone(&self.feedback_service)
    }

    pub fn db(&self) -> DbPool {
        self.db_pool.clone()
    }

    pub fn planning_service(&self) -> Arc<PlanningService> {
        Arc::clone(&self.planning_service)
    }

    pub fn ai_service(&self) -> Arc<AiService> {
        Arc::clone(&self.ai_service)
    }

    pub fn tools(&self) -> Arc<ToolRegistry> {
        Arc::clone(&self.tool_registry)
    }

    pub fn agent(&self) -> Arc<AiAgentService> {
        Arc::clone(&self.agent_service)
    }

    pub fn memory(&self) -> Arc<MemoryService> {
        Arc::clone(&self.memory_service)
    }

    pub fn goals(&self) -> Arc<GoalService> {
        Arc::clone(&self.goal_service)
    }

    pub fn dependency_service(&self) -> Arc<DependencyService> {
        Arc::clone(&self.dependency_service)
    }

    /// Clear all cached data except settings
    pub fn clear_all_cache(&self) -> AppResult<CacheClearResult> {
        let mut result = CacheClearResult::default();

        self.db_pool.with_connection(|conn| {
            // Count before clearing
            result.tasks_cleared =
                conn.query_row("SELECT COUNT(*) FROM tasks", [], |row| row.get(0))?;
            result.planning_sessions_cleared = conn
                .query_row("SELECT COUNT(*) FROM planning_sessions", [], |row| {
                    row.get(0)
                })
                .unwrap_or(0);
            result.recommendations_cleared = conn
                .query_row("SELECT COUNT(*) FROM recommendations", [], |row| row.get(0))
                .unwrap_or(0);
            result.analytics_snapshots_cleared = conn
                .query_row(
                    "SELECT COUNT(*) FROM analytics_daily_snapshots",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(0);
            result.productivity_scores_cleared = conn
                .query_row("SELECT COUNT(*) FROM productivity_scores", [], |row| {
                    row.get(0)
                })
                .unwrap_or(0);
            result.wellness_nudges_cleared = conn
                .query_row("SELECT COUNT(*) FROM wellness_nudges", [], |row| row.get(0))
                .unwrap_or(0);
            result.workload_forecasts_cleared = conn
                .query_row("SELECT COUNT(*) FROM workload_forecasts", [], |row| {
                    row.get(0)
                })
                .unwrap_or(0);
            result.ai_feedback_cleared = conn
                .query_row("SELECT COUNT(*) FROM ai_feedback", [], |row| row.get(0))
                .unwrap_or(0);
            result.community_exports_cleared = conn
                .query_row("SELECT COUNT(*) FROM community_export_log", [], |row| {
                    row.get(0)
                })
                .unwrap_or(0);
            result.ai_cache_cleared = conn
                .query_row("SELECT COUNT(*) FROM ai_cache", [], |row| row.get(0))
                .unwrap_or(0);

            // Delete data (keep settings and ai_settings)
            conn.execute("DELETE FROM tasks", [])?;
            conn.execute("DELETE FROM planning_sessions", []).ok();
            conn.execute("DELETE FROM planning_options", []).ok();
            conn.execute("DELETE FROM recommendations", []).ok();
            conn.execute("DELETE FROM analytics_daily_snapshots", [])
                .ok();
            conn.execute("DELETE FROM productivity_scores", []).ok();
            conn.execute("DELETE FROM wellness_nudges", []).ok();
            conn.execute("DELETE FROM workload_forecasts", []).ok();
            conn.execute("DELETE FROM ai_feedback", []).ok();
            conn.execute("DELETE FROM community_export_log", []).ok();
            conn.execute("DELETE FROM ai_cache", []).ok();

            Ok(())
        })?;

        Ok(result)
    }

    // NOTE: additional AppState helpers remain above.
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheClearResult {
    pub tasks_cleared: i64,
    pub planning_sessions_cleared: i64,
    pub recommendations_cleared: i64,
    pub analytics_snapshots_cleared: i64,
    pub productivity_scores_cleared: i64,
    pub wellness_nudges_cleared: i64,
    pub workload_forecasts_cleared: i64,
    pub ai_feedback_cleared: i64,
    pub community_exports_cleared: i64,
    pub ai_cache_cleared: i64,
}

pub type CommandResult<T> = Result<T, CommandError>;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<JsonValue>,
}

impl CommandError {
    pub fn new(
        code: impl Into<String>,
        message: impl Into<String>,
        details: Option<JsonValue>,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details,
        }
    }
}

impl From<AppError> for CommandError {
    fn from(error: AppError) -> Self {
        match error {
            AppError::Validation {
                message, details, ..
            } => CommandError::new("VALIDATION_ERROR", message, details),
            AppError::NotFound => CommandError::new("NOT_FOUND", "请求的资源不存在", None),
            AppError::Conflict { message } => CommandError::new("CONFLICT", message, None),
            AppError::Ai {
                code,
                message,
                correlation_id,
                details,
            } => {
                let mut merged = JsonMap::new();
                if let Some(existing) = details {
                    match existing {
                        JsonValue::Object(map) => {
                            for (key, value) in map {
                                merged.insert(key, value);
                            }
                        }
                        value => {
                            merged.insert("info".to_string(), value);
                        }
                    }
                }
                if let Some(id) = correlation_id {
                    merged.insert("correlationId".to_string(), JsonValue::String(id));
                }
                let detail_value = if merged.is_empty() {
                    None
                } else {
                    Some(JsonValue::Object(merged))
                };
                CommandError::new(code.as_str(), message, detail_value)
            }
            AppError::MemoryUnavailable(message) => {
                warn!(target: "app::command", %message, "memory unavailable in command");
                CommandError::new(
                    "MEMORY_UNAVAILABLE",
                    format!("内存功能暂时不可用: {}", message),
                    None,
                )
            }
            AppError::ToolExecutionFailed { tool_name, reason } => {
                error!(target: "app::command", %tool_name, %reason, "tool execution failed in command");
                CommandError::new(
                    "TOOL_EXECUTION_FAILED",
                    format!("工具执行失败: {}", reason),
                    Some(serde_json::json!({ "toolName": tool_name })),
                )
            }
            AppError::InvalidToolCall {
                tool_name,
                validation_error,
            } => {
                warn!(target: "app::command", %tool_name, %validation_error, "invalid tool call in command");
                CommandError::new(
                    "INVALID_TOOL_CALL",
                    format!("无效的工具调用: {}", validation_error),
                    Some(serde_json::json!({ "toolName": tool_name })),
                )
            }
            AppError::ContextTooLarge { tokens, limit } => {
                warn!(target: "app::command", tokens, limit, "context too large in command");
                CommandError::new(
                    "CONTEXT_TOO_LARGE",
                    format!("上下文过大 ({} tokens，限制 {} tokens)", tokens, limit),
                    Some(serde_json::json!({ "tokens": tokens, "limit": limit })),
                )
            }
            AppError::Database { message } => {
                error!(target: "app::command", %message, "database error in command");
                CommandError::new("UNKNOWN", message, None)
            }
            AppError::Serialization(error) => {
                error!(target: "app::command", error = %error, "serialization error in command");
                CommandError::new("UNKNOWN", "序列化失败", None)
            }
            AppError::Io(error) => {
                error!(target: "app::command", error = %error, "io error in command");
                CommandError::new("UNKNOWN", "文件系统读写失败", None)
            }
            AppError::Other(message) => {
                error!(target: "app::command", %message, "unexpected error in command");
                CommandError::new("UNKNOWN", message, None)
            }
        }
    }
}
