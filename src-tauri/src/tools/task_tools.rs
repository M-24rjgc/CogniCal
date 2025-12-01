use crate::error::{AppError, AppResult};
use crate::models::task::{TaskCreateInput, TaskUpdateInput};
use crate::services::task_service::TaskService;
use serde::Deserialize;
use serde_json::{json, Value as JsonValue};
use std::sync::Arc;
use tracing::{debug, error};

/// Tool schemas for task management operations
/// These schemas follow the OpenAI function calling format

/// Get the schema for the create_task tool
pub fn create_task_schema() -> JsonValue {
    json!({
        "type": "object",
        "properties": {
            "title": {
                "type": "string",
                "description": "The title of the task (required, max 160 characters)"
            },
            "description": {
                "type": "string",
                "description": "Detailed description of the task (optional)"
            },
            "priority": {
                "type": "string",
                "enum": ["low", "medium", "high", "urgent"],
                "description": "Priority level of the task (default: medium)"
            },
            "status": {
                "type": "string",
                "enum": ["backlog", "todo", "in_progress", "blocked", "done", "archived"],
                "description": "Current status of the task (default: todo)"
            },
            "due_at": {
                "type": "string",
                "format": "date-time",
                "description": "Due date in RFC3339 format (e.g., 2024-12-31T23:59:59Z)"
            },
            "tags": {
                "type": "array",
                "items": {
                    "type": "string"
                },
                "description": "Tags to categorize the task (max 30 tags, each max 32 chars)"
            },
            "estimated_hours": {
                "type": "number",
                "description": "Estimated hours to complete the task"
            }
        },
        "required": ["title"]
    })
}

/// Get the schema for the update_task tool
pub fn update_task_schema() -> JsonValue {
    json!({
        "type": "object",
        "properties": {
            "task_id": {
                "type": "string",
                "description": "The ID of the task to update (required)"
            },
            "title": {
                "type": "string",
                "description": "New title for the task"
            },
            "description": {
                "type": "string",
                "description": "New description for the task"
            },
            "priority": {
                "type": "string",
                "enum": ["low", "medium", "high", "urgent"],
                "description": "New priority level"
            },
            "status": {
                "type": "string",
                "enum": ["backlog", "todo", "in_progress", "blocked", "done", "archived"],
                "description": "New status"
            },
            "due_at": {
                "type": "string",
                "format": "date-time",
                "description": "New due date in RFC3339 format"
            },
            "tags": {
                "type": "array",
                "items": {
                    "type": "string"
                },
                "description": "New tags for the task"
            }
        },
        "required": ["task_id"]
    })
}

/// Get the schema for the delete_task tool
pub fn delete_task_schema() -> JsonValue {
    json!({
        "type": "object",
        "properties": {
            "task_id": {
                "type": "string",
                "description": "The ID of the task to delete (required)"
            }
        },
        "required": ["task_id"]
    })
}

/// Get the schema for the list_tasks tool
pub fn list_tasks_schema() -> JsonValue {
    json!({
        "type": "object",
        "properties": {
            "status": {
                "type": "string",
                "enum": ["backlog", "todo", "in_progress", "blocked", "done", "archived"],
                "description": "Filter tasks by status"
            },
            "priority": {
                "type": "string",
                "enum": ["low", "medium", "high", "urgent"],
                "description": "Filter tasks by priority"
            },
            "tag": {
                "type": "string",
                "description": "Filter tasks by a specific tag"
            }
        }
    })
}

/// Get the schema for the search_tasks tool
pub fn search_tasks_schema() -> JsonValue {
    json!({
        "type": "object",
        "properties": {
            "query": {
                "type": "string",
                "description": "Search query to match against task titles and descriptions (required)"
            },
            "status": {
                "type": "string",
                "enum": ["backlog", "todo", "in_progress", "blocked", "done", "archived"],
                "description": "Filter results by status"
            },
            "priority": {
                "type": "string",
                "enum": ["low", "medium", "high", "urgent"],
                "description": "Filter results by priority"
            }
        },
        "required": ["query"]
    })
}

/// Parameters for creating a task
#[derive(Debug, Deserialize)]
struct CreateTaskParams {
    title: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    priority: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    due_at: Option<String>,
    #[serde(default)]
    tags: Option<Vec<String>>,
    #[serde(default)]
    estimated_hours: Option<f64>,
}

/// Parameters for updating a task
#[derive(Debug, Deserialize)]
struct UpdateTaskParams {
    task_id: String,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    priority: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    due_at: Option<String>,
    #[serde(default)]
    tags: Option<Vec<String>>,
}

/// Parameters for deleting a task
#[derive(Debug, Deserialize)]
struct DeleteTaskParams {
    task_id: String,
}

/// Parameters for listing tasks
#[derive(Debug, Deserialize)]
struct ListTasksParams {
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    priority: Option<String>,
    #[serde(default)]
    tag: Option<String>,
}

/// Parameters for searching tasks
#[derive(Debug, Deserialize)]
struct SearchTasksParams {
    query: String,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    priority: Option<String>,
}

/// Helper function to extract parameters from JSON
fn extract_params<T: for<'de> Deserialize<'de>>(args: &JsonValue) -> AppResult<T> {
    serde_json::from_value(args.clone())
        .map_err(|e| AppError::validation(format!("Failed to parse tool parameters: {}", e)))
}

/// Helper function to format a task record for AI consumption
fn format_task_for_ai(task: &crate::models::task::TaskRecord) -> JsonValue {
    json!({
        "id": task.id,
        "title": task.title,
        "description": task.description,
        "status": task.status,
        "priority": task.priority,
        "due_at": task.due_at,
        "tags": task.tags,
        "estimated_hours": task.estimated_hours,
        "created_at": task.created_at,
        "updated_at": task.updated_at,
    })
}

/// Helper function to format multiple tasks for AI consumption
fn format_tasks_summary(tasks: &[crate::models::task::TaskRecord]) -> String {
    if tasks.is_empty() {
        return "No tasks found.".to_string();
    }

    let mut summary = format!("Found {} task(s):\n\n", tasks.len());

    for (idx, task) in tasks.iter().enumerate() {
        summary.push_str(&format!(
            "{}. [{}] {} ({})\n",
            idx + 1,
            task.status.to_uppercase(),
            task.title,
            task.priority
        ));

        if let Some(desc) = &task.description {
            let short_desc = if desc.len() > 100 {
                // Safely truncate at character boundary
                let mut end = 100;
                while end > 0 && !desc.is_char_boundary(end) {
                    end -= 1;
                }
                format!("{}...", &desc[..end])
            } else {
                desc.clone()
            };
            summary.push_str(&format!("   Description: {}\n", short_desc));
        }

        if let Some(due) = &task.due_at {
            summary.push_str(&format!("   Due: {}\n", due));
        }

        if !task.tags.is_empty() {
            summary.push_str(&format!("   Tags: {}\n", task.tags.join(", ")));
        }

        summary.push_str(&format!("   ID: {}\n\n", task.id));
    }

    // Add statistics
    let status_counts = tasks
        .iter()
        .fold(std::collections::HashMap::new(), |mut acc, task| {
            *acc.entry(&task.status).or_insert(0) += 1;
            acc
        });

    summary.push_str("Summary by status:\n");
    for (status, count) in status_counts {
        summary.push_str(&format!("  - {}: {}\n", status, count));
    }

    summary
}

/// Create a new task
///
/// This tool allows the AI to create tasks with specified parameters.
/// Returns a confirmation message with the task ID and details.
pub async fn create_task_tool(
    task_service: Arc<TaskService>,
    args: JsonValue,
) -> AppResult<JsonValue> {
    debug!(target: "task_tools", "Creating task with args: {}", args);

    let params: CreateTaskParams = extract_params(&args)?;

    let input = TaskCreateInput {
        title: params.title,
        description: params.description,
        priority: params.priority,
        status: params.status,
        due_at: params.due_at,
        tags: params.tags,
        estimated_hours: params.estimated_hours,
        ..Default::default()
    };

    match task_service.create_task(input) {
        Ok(task) => {
            let message = format!(
                "✓ Task created successfully!\n\nTitle: {}\nStatus: {}\nPriority: {}\nID: {}",
                task.title, task.status, task.priority, task.id
            );

            Ok(json!({
                "success": true,
                "message": message,
                "task": format_task_for_ai(&task)
            }))
        }
        Err(e) => {
            error!(target: "task_tools", error = %e, "Failed to create task");
            Err(AppError::validation(format!(
                "Failed to create task: {}",
                e
            )))
        }
    }
}

/// Update an existing task
///
/// This tool allows the AI to update task fields.
/// Returns a confirmation message with the updated task details.
pub async fn update_task_tool(
    task_service: Arc<TaskService>,
    args: JsonValue,
) -> AppResult<JsonValue> {
    debug!(target: "task_tools", "Updating task with args: {}", args);

    let params: UpdateTaskParams = extract_params(&args)?;

    let update = TaskUpdateInput {
        title: params.title,
        description: params.description.map(Some),
        priority: params.priority,
        status: params.status,
        due_at: params.due_at.map(Some),
        tags: params.tags.map(Some),
        ..Default::default()
    };

    match task_service.update_task(&params.task_id, update) {
        Ok(task) => {
            let message = format!(
                "✓ Task updated successfully!\n\nTitle: {}\nStatus: {}\nPriority: {}\nID: {}",
                task.title, task.status, task.priority, task.id
            );

            Ok(json!({
                "success": true,
                "message": message,
                "task": format_task_for_ai(&task)
            }))
        }
        Err(e) => {
            error!(target: "task_tools", error = %e, task_id = %params.task_id, "Failed to update task");

            // Check if it's a not found error
            if matches!(e, AppError::NotFound) {
                Err(AppError::validation(format!(
                    "Task with ID '{}' not found. Please check the task ID and try again.",
                    params.task_id
                )))
            } else {
                Err(AppError::validation(format!(
                    "Failed to update task: {}",
                    e
                )))
            }
        }
    }
}

/// Delete a task
///
/// This tool allows the AI to delete tasks by ID.
/// Returns a confirmation message.
pub async fn delete_task_tool(
    task_service: Arc<TaskService>,
    args: JsonValue,
) -> AppResult<JsonValue> {
    debug!(target: "task_tools", "Deleting task with args: {}", args);

    let params: DeleteTaskParams = extract_params(&args)?;

    // First, get the task to show details in confirmation
    let task = task_service.get_task(&params.task_id).map_err(|e| {
        if matches!(e, AppError::NotFound) {
            AppError::validation(format!(
                "Task with ID '{}' not found. Please check the task ID and try again.",
                params.task_id
            ))
        } else {
            e
        }
    })?;

    match task_service.delete_task(&params.task_id) {
        Ok(_) => {
            let message = format!(
                "✓ Task deleted successfully!\n\nDeleted task: {}\nID: {}",
                task.title, task.id
            );

            Ok(json!({
                "success": true,
                "message": message,
                "deleted_task_id": task.id
            }))
        }
        Err(e) => {
            error!(target: "task_tools", error = %e, task_id = %params.task_id, "Failed to delete task");
            Err(AppError::validation(format!(
                "Failed to delete task: {}",
                e
            )))
        }
    }
}

/// List tasks with optional filters
///
/// This tool allows the AI to retrieve tasks with filters.
/// Returns a formatted list of tasks with summary statistics.
pub async fn list_tasks_tool(
    task_service: Arc<TaskService>,
    args: JsonValue,
) -> AppResult<JsonValue> {
    debug!(target: "task_tools", "Listing tasks with args: {}", args);

    let params: ListTasksParams = extract_params(&args)?;

    match task_service.list_tasks() {
        Ok(mut tasks) => {
            // Apply filters
            if let Some(status) = &params.status {
                tasks.retain(|t| t.status.eq_ignore_ascii_case(status));
            }

            if let Some(priority) = &params.priority {
                tasks.retain(|t| t.priority.eq_ignore_ascii_case(priority));
            }

            if let Some(tag) = &params.tag {
                tasks.retain(|t| t.tags.iter().any(|t_tag| t_tag.eq_ignore_ascii_case(tag)));
            }

            let summary = format_tasks_summary(&tasks);

            Ok(json!({
                "success": true,
                "message": summary,
                "count": tasks.len(),
                "tasks": tasks.iter().map(format_task_for_ai).collect::<Vec<_>>()
            }))
        }
        Err(e) => {
            error!(target: "task_tools", error = %e, "Failed to list tasks");
            Err(AppError::validation(format!("Failed to list tasks: {}", e)))
        }
    }
}

/// Search tasks by query string
///
/// This tool allows the AI to search tasks by matching against titles and descriptions.
/// Returns a formatted list of matching tasks.
pub async fn search_tasks_tool(
    task_service: Arc<TaskService>,
    args: JsonValue,
) -> AppResult<JsonValue> {
    debug!(target: "task_tools", "Searching tasks with args: {}", args);

    let params: SearchTasksParams = extract_params(&args)?;

    match task_service.list_tasks() {
        Ok(mut tasks) => {
            let query_lower = params.query.to_lowercase();

            // Search in title and description
            tasks.retain(|t| {
                let title_match = t.title.to_lowercase().contains(&query_lower);
                let desc_match = t
                    .description
                    .as_ref()
                    .map(|d| d.to_lowercase().contains(&query_lower))
                    .unwrap_or(false);
                title_match || desc_match
            });

            // Apply additional filters
            if let Some(status) = &params.status {
                tasks.retain(|t| t.status.eq_ignore_ascii_case(status));
            }

            if let Some(priority) = &params.priority {
                tasks.retain(|t| t.priority.eq_ignore_ascii_case(priority));
            }

            let summary = if tasks.is_empty() {
                format!("No tasks found matching query: '{}'", params.query)
            } else {
                format!(
                    "Search results for '{}':\n\n{}",
                    params.query,
                    format_tasks_summary(&tasks)
                )
            };

            Ok(json!({
                "success": true,
                "message": summary,
                "query": params.query,
                "count": tasks.len(),
                "tasks": tasks.iter().map(format_task_for_ai).collect::<Vec<_>>()
            }))
        }
        Err(e) => {
            error!(target: "task_tools", error = %e, "Failed to search tasks");
            Err(AppError::validation(format!(
                "Failed to search tasks: {}",
                e
            )))
        }
    }
}

/// Register all task management tools with the tool registry
///
/// # Arguments
/// * `registry` - The tool registry to register tools with
/// * `task_service` - The task service to use for tool execution
pub fn register_task_tools(
    registry: &mut crate::services::tool_registry::ToolRegistry,
    task_service: Arc<TaskService>,
) -> AppResult<()> {
    use crate::services::tool_registry::ToolHandler;
    use std::future::Future;
    use std::pin::Pin;

    // Register create_task tool
    {
        let service = Arc::clone(&task_service);
        let handler: ToolHandler = Arc::new(move |args: JsonValue| {
            let service = Arc::clone(&service);
            Box::pin(async move { create_task_tool(service, args).await })
                as Pin<Box<dyn Future<Output = AppResult<JsonValue>> + Send>>
        });

        registry.register_tool(
            "create_task".to_string(),
            "Create a new task with the specified details".to_string(),
            json!({
                "type": "object",
                "properties": create_task_schema()["properties"],
                "required": ["title"]
            }),
            handler,
        )?;
    }

    // Register update_task tool
    {
        let service = Arc::clone(&task_service);
        let handler: ToolHandler = Arc::new(move |args: JsonValue| {
            let service = Arc::clone(&service);
            Box::pin(async move { update_task_tool(service, args).await })
                as Pin<Box<dyn Future<Output = AppResult<JsonValue>> + Send>>
        });

        registry.register_tool(
            "update_task".to_string(),
            "Update an existing task's fields".to_string(),
            json!({
                "type": "object",
                "properties": update_task_schema()["properties"],
                "required": ["id"]
            }),
            handler,
        )?;
    }

    // Register delete_task tool
    {
        let service = Arc::clone(&task_service);
        let handler: ToolHandler = Arc::new(move |args: JsonValue| {
            let service = Arc::clone(&service);
            Box::pin(async move { delete_task_tool(service, args).await })
                as Pin<Box<dyn Future<Output = AppResult<JsonValue>> + Send>>
        });

        registry.register_tool(
            "delete_task".to_string(),
            "Delete a task by ID".to_string(),
            json!({
                "type": "object",
                "properties": delete_task_schema()["properties"],
                "required": ["id"]
            }),
            handler,
        )?;
    }

    // Register list_tasks tool
    {
        let service = Arc::clone(&task_service);
        let handler: ToolHandler = Arc::new(move |args: JsonValue| {
            let service = Arc::clone(&service);
            Box::pin(async move { list_tasks_tool(service, args).await })
                as Pin<Box<dyn Future<Output = AppResult<JsonValue>> + Send>>
        });

        registry.register_tool(
            "list_tasks".to_string(),
            "List/view/show all tasks with optional filters. Use this when user asks to 'show tasks', 'list tasks', 'what tasks do I have', 'view my tasks', or similar. Can filter by: status (pending/completed), priority (low/medium/high), tags, or date_range. If no filters specified, returns all tasks. For date ranges, calculate from current date automatically.".to_string(),
            json!({
                "type": "object",
                "properties": list_tasks_schema()["properties"],
                "required": []
            }),
            handler,
        )?;
    }

    // Register search_tasks tool
    {
        let service = Arc::clone(&task_service);
        let handler: ToolHandler = Arc::new(move |args: JsonValue| {
            let service = Arc::clone(&service);
            Box::pin(async move { search_tasks_tool(service, args).await })
                as Pin<Box<dyn Future<Output = AppResult<JsonValue>> + Send>>
        });

        registry.register_tool(
            "search_tasks".to_string(),
            "Search/find tasks by keyword matching against titles and descriptions. Use when user asks to 'find task about X', 'search for tasks containing Y', or needs to locate specific tasks by content. Provide the search query as the 'query' parameter.".to_string(),
            json!({
                "type": "object",
                "properties": search_tasks_schema()["properties"],
                "required": ["query"]
            }),
            handler,
        )?;
    }

    debug!(target: "task_tools", "Registered 5 task management tools");
    Ok(())
}
