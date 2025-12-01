use crate::error::{AppError, AppResult};
use crate::services::recurring_task_service::RecurringTaskService;
use crate::services::tool_registry::ToolRegistry;
use chrono::Utc;
use serde::Deserialize;
use serde_json::{json, Value as JsonValue};
use std::sync::Arc;

/// Schema for creating recurring task tools
pub fn create_recurring_task_schema() -> JsonValue {
    json!({
        "type": "object",
        "properties": {
            "title": {
                "type": "string",
                "description": "The title of the recurring task"
            },
            "description": {
                "type": "string",
                "description": "Optional description of the recurring task"
            },
            "recurrence_rule": {
                "type": "string",
                "description": "RRULE string defining the recurrence pattern (e.g., 'FREQ=DAILY', 'FREQ=WEEKLY', 'FREQ=MONTHLY')"
            },
            "priority": {
                "type": "string",
                "enum": ["low", "medium", "high"],
                "description": "Task priority level"
            },
            "tags": {
                "type": "array",
                "items": {"type": "string"},
                "description": "Optional tags for categorization"
            },
            "estimated_minutes": {
                "type": "integer",
                "minimum": 1,
                "description": "Estimated duration in minutes"
            }
        },
        "required": ["title", "recurrence_rule"]
    })
}

/// Schema for listing recurring task tools
pub fn list_recurring_tasks_schema() -> JsonValue {
    json!({
        "type": "object",
        "properties": {
            "is_active": {
                "type": "boolean",
                "description": "Filter by active status"
            },
            "title_contains": {
                "type": "string",
                "description": "Filter by title containing this text"
            },
            "tags": {
                "type": "array",
                "items": {"type": "string"},
                "description": "Filter by tags"
            }
        },
        "required": []
    })
}

/// Schema for updating recurring task tools
pub fn update_recurring_task_schema() -> JsonValue {
    json!({
        "type": "object",
        "properties": {
            "template_id": {
                "type": "string",
                "description": "ID of the recurring task template to update"
            },
            "title": {
                "type": "string",
                "description": "New title (optional)"
            },
            "description": {
                "type": "string",
                "description": "New description (optional)"
            },
            "recurrence_rule": {
                "type": "string",
                "description": "New RRULE string (optional)"
            },
            "priority": {
                "type": "string",
                "enum": ["low", "medium", "high"],
                "description": "New priority (optional)"
            },
            "tags": {
                "type": "array",
                "items": {"type": "string"},
                "description": "New tags (optional)"
            },
            "estimated_minutes": {
                "type": "integer",
                "description": "New estimated duration (optional)"
            },
            "is_active": {
                "type": "boolean",
                "description": "Activate or deactivate the template"
            }
        },
        "required": ["template_id"]
    })
}

/// Schema for generating instances
pub fn generate_instances_schema() -> JsonValue {
    json!({
        "type": "object",
        "properties": {
            "template_id": {
                "type": "string",
                "description": "ID of the recurring task template"
            }
        },
        "required": ["template_id"]
    })
}

/// Handle create_recurring_task tool execution
pub async fn handle_create_recurring_task(
    args: &JsonValue,
    service: &Arc<RecurringTaskService>,
) -> Result<JsonValue, AppError> {
    #[derive(Deserialize)]
    struct CreateArgs {
        title: String,
        description: Option<String>,
        recurrence_rule: String,
        priority: Option<String>,
        tags: Option<Vec<String>>,
        estimated_minutes: Option<i64>,
    }

    let args: CreateArgs = serde_json::from_value(args.clone())
        .map_err(|e| AppError::validation(&format!("Invalid arguments: {}", e)))?;

    let create_input = crate::models::recurring_task::RecurringTaskTemplateCreate {
        title: args.title,
        description: args.description,
        recurrence_rule_string: args.recurrence_rule,
        priority: args.priority,
        tags: args.tags,
        estimated_minutes: args.estimated_minutes,
    };

    let template = service.create_template(create_input)?;

    Ok(json!({
        "success": true,
        "template": {
            "id": template.id,
            "title": template.title,
            "recurrence_rule": template.recurrence_rule,
            "created_at": template.created_at
        },
        "message": "Recurring task created successfully"
    }))
}

/// Handle list_recurring_tasks tool execution
pub async fn handle_list_recurring_tasks(
    args: &JsonValue,
    service: &Arc<RecurringTaskService>,
) -> Result<JsonValue, AppError> {
    #[derive(Deserialize, Default)]
    struct ListArgs {
        is_active: Option<bool>,
        title_contains: Option<String>,
        tags: Option<Vec<String>>,
    }

    let args: ListArgs = serde_json::from_value(args.clone()).unwrap_or_default();

    let filter = crate::models::recurring_task::RecurringTaskTemplateFilter {
        is_active: args.is_active,
        title_contains: args.title_contains,
        created_after: None,
        created_before: None,
        tags: args.tags,
    };

    let templates = service.list_templates(Some(filter))?;

    let templates_json: Vec<_> = templates
        .into_iter()
        .map(|template| {
            json!({
                "id": template.id,
                "title": template.title,
                "description": template.description,
                "recurrence_rule": template.recurrence_rule,
                "priority": template.priority,
                "tags": template.tags,
                "estimated_minutes": template.estimated_minutes,
                "is_active": template.is_active,
                "created_at": template.created_at,
                "updated_at": template.updated_at
            })
        })
        .collect();

    Ok(json!({
        "success": true,
        "templates": templates_json,
        "count": templates_json.len()
    }))
}

/// Handle update_recurring_task tool execution
pub async fn handle_update_recurring_task(
    args: &JsonValue,
    service: &Arc<RecurringTaskService>,
) -> Result<JsonValue, AppError> {
    #[derive(Deserialize)]
    struct UpdateArgs {
        template_id: String,
        title: Option<String>,
        description: Option<String>,
        recurrence_rule: Option<String>,
        priority: Option<String>,
        tags: Option<Vec<String>>,
        estimated_minutes: Option<i64>,
        is_active: Option<bool>,
    }

    let args: UpdateArgs = serde_json::from_value(args.clone())
        .map_err(|e| AppError::validation(&format!("Invalid arguments: {}", e)))?;

    let recurrence_rule = if let Some(ref rule_string) = args.recurrence_rule {
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

    let update_input = crate::models::recurring_task::RecurringTaskTemplateUpdate {
        title: args.title,
        description: args.description.map(Some),
        recurrence_rule,
        priority: args.priority,
        tags: args.tags,
        estimated_minutes: args.estimated_minutes.map(Some),
        is_active: args.is_active,
    };

    let template = service.update_template(&args.template_id, update_input)?;

    Ok(json!({
        "success": true,
        "template": {
            "id": template.id,
            "title": template.title,
            "updated_at": template.updated_at
        },
        "message": "Recurring task updated successfully"
    }))
}

/// Handle generate_instances tool execution
pub async fn handle_generate_instances(
    args: &JsonValue,
    service: &Arc<RecurringTaskService>,
) -> Result<JsonValue, AppError> {
    #[derive(Deserialize)]
    struct GenerateArgs {
        template_id: String,
    }

    let args: GenerateArgs = serde_json::from_value(args.clone())
        .map_err(|e| AppError::validation(&format!("Invalid arguments: {}", e)))?;

    let instances = service.generate_instances_for_template(&args.template_id)?;

    let instances_json: Vec<_> = instances
        .into_iter()
        .map(|instance| {
            json!({
                "id": instance.id,
                "template_id": instance.template_id,
                "title": instance.title,
                "instance_date": instance.instance_date,
                "status": instance.status,
                "is_exception": instance.is_exception
            })
        })
        .collect();

    Ok(json!({
        "success": true,
        "instances": instances_json,
        "count": instances_json.len(),
        "generated_at": Utc::now()
    }))
}

/// Register all recurring task management tools with the tool registry
pub fn register_recurring_task_tools(
    registry: &mut ToolRegistry,
    recurring_task_service: Arc<RecurringTaskService>,
) -> AppResult<()> {
    use crate::services::tool_registry::ToolHandler;
    use std::future::Future;
    use std::pin::Pin;

    // Register create_recurring_task tool
    {
        let service = Arc::clone(&recurring_task_service);
        let handler: ToolHandler = Arc::new(move |args: JsonValue| {
            let service = Arc::clone(&service);
            Box::pin(async move {
                match handle_create_recurring_task(&args, &service).await {
                    Ok(result) => Ok(result),
                    Err(e) => Ok(json!({
                        "success": false,
                        "error": e.to_string()
                    })),
                }
            }) as Pin<Box<dyn Future<Output = AppResult<JsonValue>> + Send>>
        });

        registry.register_tool(
            "create_recurring_task".to_string(),
            "Create a new recurring task template".to_string(),
            create_recurring_task_schema(),
            handler,
        )?;
    }

    // Register list_recurring_tasks tool
    {
        let service = Arc::clone(&recurring_task_service);
        let handler: ToolHandler = Arc::new(move |args: JsonValue| {
            let service = Arc::clone(&service);
            Box::pin(async move {
                match handle_list_recurring_tasks(&args, &service).await {
                    Ok(result) => Ok(result),
                    Err(e) => Ok(json!({
                        "success": false,
                        "error": e.to_string()
                    })),
                }
            }) as Pin<Box<dyn Future<Output = AppResult<JsonValue>> + Send>>
        });

        registry.register_tool(
            "list_recurring_tasks".to_string(),
            "List existing recurring task templates".to_string(),
            list_recurring_tasks_schema(),
            handler,
        )?;
    }

    // Register update_recurring_task tool
    {
        let service = Arc::clone(&recurring_task_service);
        let handler: ToolHandler = Arc::new(move |args: JsonValue| {
            let service = Arc::clone(&service);
            Box::pin(async move {
                match handle_update_recurring_task(&args, &service).await {
                    Ok(result) => Ok(result),
                    Err(e) => Ok(json!({
                        "success": false,
                        "error": e.to_string()
                    })),
                }
            }) as Pin<Box<dyn Future<Output = AppResult<JsonValue>> + Send>>
        });

        registry.register_tool(
            "update_recurring_task".to_string(),
            "Update an existing recurring task template".to_string(),
            update_recurring_task_schema(),
            handler,
        )?;
    }

    // Register generate_instances tool
    {
        let service = Arc::clone(&recurring_task_service);
        let handler: ToolHandler = Arc::new(move |args: JsonValue| {
            let service = Arc::clone(&service);
            Box::pin(async move {
                match handle_generate_instances(&args, &service).await {
                    Ok(result) => Ok(result),
                    Err(e) => Ok(json!({
                        "success": false,
                        "error": e.to_string()
                    })),
                }
            }) as Pin<Box<dyn Future<Output = AppResult<JsonValue>> + Send>>
        });

        registry.register_tool(
            "generate_recurring_instances".to_string(),
            "Generate task instances for a recurring task template".to_string(),
            generate_instances_schema(),
            handler,
        )?;
    }

    tracing::debug!("Registered 4 recurring task management tools");
    Ok(())
}
