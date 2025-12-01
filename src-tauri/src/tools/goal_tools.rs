use crate::error::{AppError, AppResult};
use crate::models::goal::{CreateGoalRequest, GoalStatus, UpdateGoalRequest};
use crate::services::goal_service::GoalService;
use serde::Deserialize;
use serde_json::{json, Value as JsonValue};
use std::sync::Arc;
use tracing::debug;

/// Get goals
pub fn get_goals_schema() -> JsonValue {
    json!({
        "type": "object",
        "properties": {
            "parent_goal_id": {
                "type": "string",
                "description": "Filter goals by parent goal ID"
            }
        },
        "required": []
    })
}

/// Get goal details schema
pub fn get_goal_details_schema() -> JsonValue {
    json!({
        "type": "object",
        "properties": {
            "goal_id": {
                "type": "string",
                "description": "Goal ID to get details for (required)"
            }
        },
        "required": ["goal_id"]
    })
}

/// Get goal tasks schema
pub fn get_goal_tasks_schema() -> JsonValue {
    json!({
        "type": "object",
        "properties": {
            "goal_id": {
                "type": "string",
                "description": "Goal ID to get tasks for (required)"
            }
        },
        "required": ["goal_id"]
    })
}

/// Get goal progress schema
pub fn get_goal_progress_schema() -> JsonValue {
    json!({
        "type": "object",
        "properties": {
            "goal_id": {
                "type": "string",
                "description": "Goal ID to get progress for (required)"
            }
        },
        "required": ["goal_id"]
    })
}

/// Create goal schema
pub fn create_goal_schema() -> JsonValue {
    json!({
        "type": "object",
        "properties": {
            "title": {
                "type": "string",
                "description": "Goal title (required)"
            },
            "description": {
                "type": "string",
                "description": "Goal description (optional)"
            },
            "parent_goal_id": {
                "type": "string",
                "description": "Parent goal ID for hierarchical goals (optional)"
            },
            "priority": {
                "type": "string",
                "default": "medium",
                "description": "Priority level: low, medium, high, urgent"
            }
        },
        "required": ["title"]
    })
}

/// Update goal schema
pub fn update_goal_schema() -> JsonValue {
    json!({
        "type": "object",
        "properties": {
            "goal_id": {
                "type": "string",
                "description": "Goal ID to update (required)"
            },
            "title": {
                "type": "string",
                "description": "New goal title (optional)"
            },
            "description": {
                "type": "string",
                "description": "New goal description (optional)"
            },
            "status": {
                "type": "string",
                "enum": ["not_started", "in_progress", "completed", "on_hold", "cancelled"],
                "description": "New goal status (optional)"
            },
            "priority": {
                "type": "string",
                "description": "New priority level (optional)"
            }
        },
        "required": ["goal_id"]
    })
}

/// Associate task with goal schema
pub fn associate_task_with_goal_schema() -> JsonValue {
    json!({
        "type": "object",
        "properties": {
            "goal_id": {
                "type": "string",
                "description": "Goal ID to associate task with (required)"
            },
            "task_id": {
                "type": "string",
                "description": "Task ID to associate with goal (required)"
            }
        },
        "required": ["goal_id", "task_id"]
    })
}

/// Tool implementations

/// Get goals with filtering and hierarchy support
pub async fn get_goals_tool(
    goal_service: Arc<GoalService>,
    args: JsonValue,
) -> AppResult<JsonValue> {
    debug!("get_goals_tool invoked");

    #[derive(Debug, Deserialize, Default)]
    struct GetGoalsParams {
        parent_goal_id: Option<String>,
    }

    let params: GetGoalsParams = serde_json::from_value(args).unwrap_or_default();

    let goals = goal_service.list_goals(params.parent_goal_id)?;
    let goals_json = serde_json::to_value(&goals)?;

    Ok(json!({
        "success": true,
        "goals": goals_json,
        "count": goals.len()
    }))
}

/// Get detailed information about a specific goal
pub async fn get_goal_details_tool(
    goal_service: Arc<GoalService>,
    args: JsonValue,
) -> AppResult<JsonValue> {
    debug!("get_goal_details_tool invoked");

    #[derive(Debug, Deserialize)]
    struct GetGoalDetailsParams {
        goal_id: String,
    }

    let params: GetGoalDetailsParams = serde_json::from_value(args)
        .map_err(|e| AppError::validation(format!("Failed to parse parameters: {}", e)))?;

    let goal = goal_service.get_goal(&params.goal_id)?;

    Ok(json!({
        "success": true,
        "goal": serde_json::to_value(&goal)?,
        "goal_id": params.goal_id
    }))
}

/// Get tasks associated with a specific goal
pub async fn get_goal_tasks_tool(
    goal_service: Arc<GoalService>,
    args: JsonValue,
) -> AppResult<JsonValue> {
    debug!("get_goal_tasks_tool invoked");

    #[derive(Debug, Deserialize)]
    struct GetGoalTasksParams {
        goal_id: String,
    }

    let params: GetGoalTasksParams = serde_json::from_value(args)
        .map_err(|e| AppError::validation(format!("Failed to parse parameters: {}", e)))?;

    let task_ids = goal_service.get_goal_tasks(&params.goal_id)?;

    Ok(json!({
        "success": true,
        "goal_id": params.goal_id,
        "task_ids": serde_json::to_value(&task_ids)?,
        "count": task_ids.len()
    }))
}

/// Get progress information for a specific goal
pub async fn get_goal_progress_tool(
    goal_service: Arc<GoalService>,
    args: JsonValue,
) -> AppResult<JsonValue> {
    debug!("get_goal_progress_tool invoked");

    #[derive(Debug, Deserialize)]
    struct GetGoalProgressParams {
        goal_id: String,
    }

    let params: GetGoalProgressParams = serde_json::from_value(args)
        .map_err(|e| AppError::validation(format!("Failed to parse parameters: {}", e)))?;

    let goal_with_progress = goal_service.get_goal_with_progress(&params.goal_id)?;

    Ok(json!({
        "success": true,
        "goal_id": params.goal_id,
        "progress": serde_json::to_value(&goal_with_progress)?,
        "progress_percentage": goal_with_progress.progress_percentage,
        "total_tasks": goal_with_progress.total_tasks,
        "completed_tasks": goal_with_progress.completed_tasks
    }))
}

/// Create a new goal
pub async fn create_goal_tool(
    goal_service: Arc<GoalService>,
    args: JsonValue,
) -> AppResult<JsonValue> {
    debug!("create_goal_tool invoked");

    #[derive(Debug, Deserialize)]
    struct CreateGoalParams {
        title: String,
        description: Option<String>,
        parent_goal_id: Option<String>,
        priority: Option<String>,
    }

    let params: CreateGoalParams = serde_json::from_value(args)
        .map_err(|e| AppError::validation(format!("Failed to parse parameters: {}", e)))?;

    // 数据验证
    if params.title.trim().is_empty() {
        return Err(AppError::validation("目标标题不能为空"));
    }

    if params.title.len() > 200 {
        return Err(AppError::validation("目标标题长度不能超过200字符"));
    }

    // 验证优先级
    let priority = match params.priority.as_deref() {
        Some("low") | Some("medium") | Some("high") | Some("urgent") => params.priority.unwrap(),
        Some(_) => {
            return Err(AppError::validation(
                "无效的优先级值。有效值: low, medium, high, urgent",
            ));
        }
        None => "medium".to_string(),
    };

    let request = CreateGoalRequest {
        title: params.title.trim().to_string(),
        description: params.description,
        parent_goal_id: params.parent_goal_id,
        priority,
        target_date: None, // TODO: Add target_date support
    };

    let goal = goal_service.create_goal(request)?;

    let result = json!({
        "success": true,
        "goal_id": goal.id,
        "goal": serde_json::to_value(&goal)?,
        "message": format!("✅ 成功创建目标: {}", goal.title)
    });

    debug!(goal_id = %goal.id, "goal created successfully");
    Ok(result)
}

/// Update an existing goal
pub async fn update_goal_tool(
    goal_service: Arc<GoalService>,
    args: JsonValue,
) -> AppResult<JsonValue> {
    debug!("update_goal_tool invoked");

    #[derive(Debug, Deserialize)]
    struct UpdateGoalParams {
        goal_id: String,
        title: Option<String>,
        description: Option<String>,
        status: Option<String>,
        priority: Option<String>,
    }

    let params: UpdateGoalParams = serde_json::from_value(args)
        .map_err(|e| AppError::validation(format!("Failed to parse parameters: {}", e)))?;

    // Convert status string to GoalStatus if provided
    let status = params.status.and_then(|s| match s.as_str() {
        "not_started" => Some(GoalStatus::NotStarted),
        "in_progress" => Some(GoalStatus::InProgress),
        "completed" => Some(GoalStatus::Completed),
        "on_hold" => Some(GoalStatus::OnHold),
        "cancelled" => Some(GoalStatus::Cancelled),
        _ => None,
    });

    let request = UpdateGoalRequest {
        title: params.title,
        description: params.description,
        status,
        priority: params.priority,
        target_date: None,
    };

    let updated_goal = goal_service.update_goal(&params.goal_id, request)?;

    let result = json!({
        "success": true,
        "goal_id": params.goal_id,
        "goal": serde_json::to_value(&updated_goal)?,
        "message": format!("✅ 成功更新目标: {}", updated_goal.title)
    });

    debug!(goal_id = %params.goal_id, "goal updated successfully");
    Ok(result)
}

/// Associate a task with a goal
pub async fn associate_task_with_goal_tool(
    goal_service: Arc<GoalService>,
    args: JsonValue,
) -> AppResult<JsonValue> {
    debug!("associate_task_with_goal_tool invoked");

    #[derive(Debug, Deserialize)]
    struct AssociateTaskWithGoalParams {
        goal_id: String,
        task_id: String,
    }

    let params: AssociateTaskWithGoalParams = serde_json::from_value(args)
        .map_err(|e| AppError::validation(format!("Failed to parse parameters: {}", e)))?;

    let association = goal_service.associate_task(&params.goal_id, &params.task_id)?;

    let result = json!({
        "success": true,
        "association_id": association.id,
        "goal_id": params.goal_id,
        "task_id": params.task_id,
        "message": format!("🔗 成功关联任务 {} 到目标 {}", params.task_id, params.goal_id)
    });

    debug!(goal_id = %params.goal_id, task_id = %params.task_id, "task-goal association created");
    Ok(result)
}

/// Register all goal management tools
pub fn register_goal_tools(
    registry: &mut crate::services::tool_registry::ToolRegistry,
    goal_service: Arc<GoalService>,
) -> AppResult<()> {
    use crate::services::tool_registry::ToolHandler;
    use std::future::Future;
    use std::pin::Pin;

    // Register get_goals tool
    {
        let service = Arc::clone(&goal_service);
        let handler: ToolHandler = Arc::new(move |args: JsonValue| {
            let service = Arc::clone(&service);
            Box::pin(async move { get_goals_tool(service, args).await })
                as Pin<Box<dyn Future<Output = AppResult<JsonValue>> + Send>>
        });

        registry.register_tool(
            "get_goals".to_string(),
            "Get all goals with filtering and hierarchy support. Use when user wants to see goals, project objectives, or strategic plans.".to_string(),
            get_goals_schema(),
            handler,
        )?;
    }

    // Register get_goal_details tool
    {
        let service = Arc::clone(&goal_service);
        let handler: ToolHandler = Arc::new(move |args: JsonValue| {
            let service = Arc::clone(&service);
            Box::pin(async move { get_goal_details_tool(service, args).await })
                as Pin<Box<dyn Future<Output = AppResult<JsonValue>> + Send>>
        });

        registry.register_tool(
            "get_goal_details".to_string(),
            "Get detailed information about a specific goal. Use when user wants comprehensive information about a particular goal.".to_string(),
            get_goal_details_schema(),
            handler,
        )?;
    }

    // Register get_goal_tasks tool
    {
        let service = Arc::clone(&goal_service);
        let handler: ToolHandler = Arc::new(move |args: JsonValue| {
            let service = Arc::clone(&service);
            Box::pin(async move { get_goal_tasks_tool(service, args).await })
                as Pin<Box<dyn Future<Output = AppResult<JsonValue>> + Send>>
        });

        registry.register_tool(
            "get_goal_tasks".to_string(),
            "Get tasks associated with a specific goal. Use when user wants to see what tasks contribute to achieving a goal. Essential for linking tactical work to strategic objectives.".to_string(),
            get_goal_tasks_schema(),
            handler,
        )?;
    }

    // Register get_goal_progress tool
    {
        let service = Arc::clone(&goal_service);
        let handler: ToolHandler = Arc::new(move |args: JsonValue| {
            let service = Arc::clone(&service);
            Box::pin(async move { get_goal_progress_tool(service, args).await })
                as Pin<Box<dyn Future<Output = AppResult<JsonValue>> + Send>>
        });

        registry.register_tool(
            "get_goal_progress".to_string(),
            "Get detailed progress information for a goal including completion percentage and task statistics. Use for goal tracking and performance monitoring.".to_string(),
            get_goal_progress_schema(),
            handler,
        )?;
    }

    // Register create_goal tool
    {
        let service = Arc::clone(&goal_service);
        let handler: ToolHandler = Arc::new(move |args: JsonValue| {
            let service = Arc::clone(&service);
            Box::pin(async move { create_goal_tool(service, args).await })
                as Pin<Box<dyn Future<Output = AppResult<JsonValue>> + Send>>
        });

        registry.register_tool(
            "create_goal".to_string(),
            "Create a new goal or objective. Use when user wants to establish new goals, projects, or strategic targets. Supports hierarchical goal structure with parent goals.".to_string(),
            create_goal_schema(),
            handler,
        )?;
    }

    // Register update_goal tool
    {
        let service = Arc::clone(&goal_service);
        let handler: ToolHandler = Arc::new(move |args: JsonValue| {
            let service = Arc::clone(&service);
            Box::pin(async move { update_goal_tool(service, args).await })
                as Pin<Box<dyn Future<Output = AppResult<JsonValue>> + Send>>
        });

        registry.register_tool(
            "update_goal".to_string(),
            "Update an existing goal's properties. Use when user wants to modify goal details, change status, adjust priorities.".to_string(),
            update_goal_schema(),
            handler,
        )?;
    }

    // Register associate_task_with_goal tool
    {
        let service = Arc::clone(&goal_service);
        let handler: ToolHandler = Arc::new(move |args: JsonValue| {
            let service = Arc::clone(&service);
            Box::pin(async move { associate_task_with_goal_tool(service, args).await })
                as Pin<Box<dyn Future<Output = AppResult<JsonValue>> + Send>>
        });

        registry.register_tool(
            "associate_task_with_goal".to_string(),
            "Associate a task with a goal to link tactical work to strategic objectives. Use when user wants to tie specific tasks to broader goals or projects.".to_string(),
            associate_task_with_goal_schema(),
            handler,
        )?;
    }

    Ok(())
}
