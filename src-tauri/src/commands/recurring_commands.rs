use chrono::{DateTime, Utc};
use serde::Deserialize;
use tauri::{async_runtime, State};

use crate::commands::{AppState, CommandError, CommandResult};
use crate::error::AppError;
use crate::models::recurring_task::{
    RecurringTaskTemplate, RecurringTaskTemplateCreate, RecurringTaskTemplateFilter,
    RecurringTaskTemplateUpdate, TaskInstance,
};

/// Filter parameters for recurring task templates
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RecurringTaskTemplateFilterInput {
    pub is_active: Option<bool>,
    pub title_contains: Option<String>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub tags: Option<Vec<String>>,
}

/// Create recurring task template input
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CreateRecurringTaskInput {
    pub title: String,
    pub description: Option<String>,
    pub recurrence_rule_string: String,
    pub priority: Option<String>,
    pub tags: Option<Vec<String>>,
    pub estimated_minutes: Option<i64>,
}

impl Default for CreateRecurringTaskInput {
    fn default() -> Self {
        Self {
            title: String::new(),
            description: None,
            recurrence_rule_string: String::new(),
            priority: None,
            tags: None,
            estimated_minutes: None,
        }
    }
}

/// Update recurring task template input
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct UpdateRecurringTaskInput {
    pub title: Option<String>,
    pub description: Option<String>,
    pub recurrence_rule_string: Option<String>,
    pub priority: Option<String>,
    pub tags: Option<Vec<String>>,
    pub estimated_minutes: Option<i64>,
    pub is_active: Option<bool>,
}

/// List recurring task templates with filtering
#[tauri::command]
pub async fn recurring_template_list(
    state: State<'_, AppState>,
    filters: Option<RecurringTaskTemplateFilterInput>,
) -> CommandResult<Vec<RecurringTaskTemplate>> {
    let state = state.inner().clone();
    let filters = filters.unwrap_or_default();

    run_blocking(move || {
        let service = &state.recurring_task_service;

        // Convert input filter to service filter
        let filter = RecurringTaskTemplateFilter {
            is_active: filters.is_active,
            title_contains: filters.title_contains,
            created_after: filters.created_after,
            created_before: filters.created_before,
            tags: filters.tags,
        };

        service.list_templates(Some(filter))
    })
    .await
}

/// Create a recurring task template
#[tauri::command]
pub async fn recurring_template_create(
    state: State<'_, AppState>,
    input: CreateRecurringTaskInput,
) -> CommandResult<RecurringTaskTemplate> {
    let state = state.inner().clone();

    run_blocking(move || {
        let service = &state.recurring_task_service;

        // Validate input before calling service
        if input.title.trim().is_empty() {
            return Err(AppError::validation("Title cannot be empty"));
        }

        // Parse recurrence rule
        if let Err(e) =
            crate::services::rrule_parser::RRuleParser::parse(&input.recurrence_rule_string)
        {
            return Err(AppError::validation(&format!(
                "Invalid recurrence rule: {}",
                e
            )));
        }

        let create_input = RecurringTaskTemplateCreate {
            title: input.title.trim().to_string(),
            description: input.description,
            recurrence_rule_string: input.recurrence_rule_string,
            priority: input.priority,
            tags: input.tags,
            estimated_minutes: input.estimated_minutes,
        };

        service.create_template(create_input)
    })
    .await
}

/// Update a recurring task template
#[tauri::command]
pub async fn recurring_template_update(
    state: State<'_, AppState>,
    id: String,
    input: UpdateRecurringTaskInput,
) -> CommandResult<RecurringTaskTemplate> {
    let state = state.inner().clone();

    run_blocking(move || {
        let service = &state.recurring_task_service;

        // Validate title if provided
        if let Some(ref title) = input.title {
            if title.trim().is_empty() {
                return Err(AppError::validation("Title cannot be empty"));
            }
        }

        // Parse recurrence rule if provided
        let recurrence_rule = if let Some(ref rule_string) = input.recurrence_rule_string {
            match crate::services::rrule_parser::RRuleParser::parse(rule_string) {
                Ok(rule) => Some(rule),
                Err(e) => {
                    return Err(AppError::validation(&format!(
                        "Invalid recurrence rule: {}",
                        e
                    )))
                }
            }
        } else {
            None
        };

        let update_input = RecurringTaskTemplateUpdate {
            title: input.title.map(|s| s.trim().to_string()),
            description: input.description.map(Some),
            recurrence_rule,
            priority: input.priority,
            tags: input.tags,
            estimated_minutes: input.estimated_minutes.map(Some),
            is_active: input.is_active,
        };

        service.update_template(&id, update_input)
    })
    .await
}

/// Get a recurring task template by ID
#[tauri::command]
pub async fn recurring_template_get(
    state: State<'_, AppState>,
    id: String,
) -> CommandResult<RecurringTaskTemplate> {
    let state = state.inner().clone();

    run_blocking(move || {
        let service = &state.recurring_task_service;
        service.get_template(&id)
    })
    .await
}

/// Delete a recurring task template
#[tauri::command]
pub async fn recurring_template_delete(
    state: State<'_, AppState>,
    id: String,
) -> CommandResult<()> {
    let state = state.inner().clone();

    run_blocking(move || {
        let service = &state.recurring_task_service;
        service.delete_template(&id)
    })
    .await
}

/// Generate task instances for a template
#[tauri::command]
pub async fn recurring_template_generate_instances(
    state: State<'_, AppState>,
    id: String,
) -> CommandResult<Vec<TaskInstance>> {
    let state = state.inner().clone();

    run_blocking(move || {
        let service = &state.recurring_task_service;
        service.generate_instances_for_template(&id)
    })
    .await
}

/// Get template instances
#[tauri::command]
pub async fn recurring_template_instances(
    state: State<'_, AppState>,
    id: String,
    _start_date: Option<String>,
    _end_date: Option<String>,
) -> CommandResult<Vec<TaskInstance>> {
    let state = state.inner().clone();

    run_blocking(move || {
        let service = &state.recurring_task_service;
        service.generate_instances_for_template(&id)
    })
    .await
}

/// Convert recurring instance to regular task
#[tauri::command]
pub async fn recurring_task_to_regular(
    state: State<'_, AppState>,
    _instance_id: String,
) -> CommandResult<()> {
    let state = state.inner().clone();

    run_blocking(move || {
        let _service = &state.recurring_task_service;

        // This would need to be implemented in the service
        // For now, just return Ok as a placeholder
        // _service.convert_instance_to_regular_task(&instance_id)?;

        Ok(())
    })
    .await
}

async fn run_blocking<T: Send + 'static>(
    task: impl FnOnce() -> Result<T, AppError> + Send + 'static,
) -> CommandResult<T> {
    async_runtime::spawn_blocking(task)
        .await
        .map_err(|err| CommandError::new("UNKNOWN", format!("任务执行失败: {err}"), None))?
        .map_err(CommandError::from)
}
