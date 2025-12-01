use crate::error::{AppError, AppResult};
use crate::models::dependency::{
    DependencyCreateInput, DependencyFilter, DependencyType,
};
use crate::services::dependency_service::DependencyService;
use serde::Deserialize;
use serde_json::{json, Value as JsonValue};
use std::sync::Arc;
use tracing::debug;

/// Schema definitions for dependency management tools

/// Get schema for get_dependency_graph tool
pub fn get_dependency_graph_schema() -> JsonValue {
    json!({
        "type": "object",
        "properties": {
            "task_filter": {
                "type": "array", 
                "items": {"type": "string"},
                "description": "Optional filter to get dependencies for specific tasks only"
            },
            "goal_filter": {
                "type": "array",
                "items": {"type": "string"},
                "description": "Optional filter to get dependencies for tasks belonging to specific goals"
            }
        },
        "required": []
    })
}

/// Get schema for get_task_dependencies tool  
pub fn get_task_dependencies_schema() -> JsonValue {
    json!({
        "type": "object",
        "properties": {
            "task_ids": {
                "type": "array",
                "items": {"type": "string"},
                "description": "List of task IDs to get dependencies for (required)"
            },
            "include_dependents": {
                "type": "boolean", 
                "default": true,
                "description": "Whether to include tasks that depend on the specified tasks"
            }
        },
        "required": ["task_ids"]
    })
}

/// Get schema for add_task_dependency tool
pub fn add_task_dependency_schema() -> JsonValue {
    json!({
        "type": "object",
        "properties": {
            "predecessor_id": {
                "type": "string",
                "description": "ID of the predecessor task (required)"
            },
            "successor_id": {
                "type": "string", 
                "description": "ID of the successor task (required)"
            },
            "dependency_type": {
                "type": "string",
                "enum": ["finish_to_start", "start_to_start", "finish_to_finish", "start_to_finish"],
                "default": "finish_to_start",
                "description": "Type of dependency relationship (default: finish_to_start)"
            }
        },
        "required": ["predecessor_id", "successor_id"]
    })
}

/// Get schema for remove_task_dependency tool
pub fn remove_task_dependency_schema() -> JsonValue {
    json!({
        "type": "object",
        "properties": {
            "dependency_id": {
                "type": "string",
                "description": "ID of the dependency relationship to remove (required)"
            }
        },
        "required": ["dependency_id"]
    })
}

/// Get schema for get_ready_tasks tool
pub fn get_ready_tasks_schema() -> JsonValue {
    json!({
        "type": "object", 
        "properties": {
            "limit": {
                "type": "integer",
                "default": 10,
                "description": "Maximum number of ready tasks to return"
            },
            "status_filter": {
                "type": "array",
                "items": {"type": "string"},
                "description": "Filter by task status"
            }
        },
        "required": []
    })
}

/// Get schema for get_critical_path tool
pub fn get_critical_path_schema() -> JsonValue {
    json!({
        "type": "object",
        "properties": {
            "task_id": {
                "type": "string",
                "description": "Specific task ID to analyze critical path for (optional)"
            },
            "goal_id": {
                "type": "string", 
                "description": "Goal ID to analyze critical path for (optional)"
            },
            "include_analysis": {
                "type": "boolean",
                "default": true,
                "description": "Include detailed analysis and recommendations"
            }
        },
        "required": []
    })
}

/// Get schema for validate_dependency tool
pub fn validate_dependency_schema() -> JsonValue {
    json!({
        "type": "object",
        "properties": {
            "predecessor_id": {
                "type": "string",
                "description": "ID of the predecessor task (required)"
            },
            "successor_id": {
                "type": "string",
                "description": "ID of the successor task (required)"
            },
            "dependency_type": {
                "type": "string",
                "enum": ["finish_to_start", "start_to_start", "finish_to_finish", "start_to_finish"],
                "description": "Type of dependency relationship to validate"
            }
        },
        "required": ["predecessor_id", "successor_id"]
    })
}

/// Get schema for get_dependency_metrics tool
pub fn get_dependency_metrics_schema() -> JsonValue {
    json!({
        "type": "object",
        "properties": {
            "time_range_days": {
                "type": "integer",
                "default": 30,
                "description": "Time range in days for metrics calculation"
            },
            "task_id": {
                "type": "string",
                "description": "Specific task ID to get metrics for (optional)"
            }
        },
        "required": []
    })
}

/// Tool implementations

/// Get the complete dependency graph for tasks
pub async fn get_dependency_graph_tool(
    dependency_service: Arc<DependencyService>,
    args: JsonValue,
) -> AppResult<JsonValue> {
    debug!("get_dependency_graph_tool invoked");

    let params: GetDependencyGraphParams = serde_json::from_value(args)
        .map_err(|e| AppError::validation(format!("Failed to parse parameters: {}", e)))?;

    let mut filter = None;
    
    if let Some(task_filter) = params.task_filter {
        filter = Some(DependencyFilter {
            task_ids: Some(task_filter),
            include_completed: None,
            max_depth: None,
        });
    }

    let graph = dependency_service.get_dependency_graph(filter).await?;

    let nodes_json = serde_json::to_value(&graph.nodes)?;
    let edges_json = serde_json::to_value(&graph.edges)?;

    // Create a summary for AI consumption
    let task_count = graph.nodes.len();
    let dependency_count = graph.edges.len();
    let ready_tasks = graph.nodes.values().filter(|node| node.is_ready).count();

    let summary = format!(
        "🔗 依赖关系图分析\n\n📊 总体统计:\n• 任务数量: {} 个\n• 依赖关系: {} 条\n• 可执行任务: {} 个\n• 关键路径: {} 个任务\n\n📋 任务状态:\n{}",
        task_count,
        dependency_count, 
        ready_tasks,
        graph.critical_path.len(),
        if graph.nodes.is_empty() {
            "暂无任务数据".to_string()
        } else {
            graph.nodes.values().take(5).map(|node| {
                let status_emoji = match node.status.as_str() {
                    "todo" => "📝",
                    "in_progress" => "🔄", 
                    "blocked" => "⛔",
                    "done" => "✅",
                    _ => "📋"
                };
                format!("{} 任务 {} (依赖: {}, 被依赖: {})", 
                    status_emoji, 
                    node.task_id,
                    node.dependencies.len(),
                    node.dependents.len()
                )
            }).collect::<Vec<_>>().join("\n")
        }
    );

    Ok(json!({
        "success": true,
        "graph": {
            "nodes": nodes_json,
            "edges": edges_json,
            "topological_order": graph.topological_order,
            "critical_path": graph.critical_path
        },
        "summary": summary,
        "metrics": {
            "total_tasks": task_count,
            "total_dependencies": dependency_count,
            "ready_tasks": ready_tasks,
            "critical_path_length": graph.critical_path.len()
        }
    }))
}

/// Get dependencies for specific tasks
pub async fn get_task_dependencies_tool(
    dependency_service: Arc<DependencyService>,
    args: JsonValue,
) -> AppResult<JsonValue> {
    debug!("get_task_dependencies_tool invoked");

    let params: GetTaskDependenciesParams = serde_json::from_value(args)
        .map_err(|e| AppError::validation(format!("Failed to parse parameters: {}", e)))?;

    let mut all_deps = Vec::new();
    
    for task_id in &params.task_ids {
        let deps = dependency_service.get_task_dependencies(task_id).await?;
        all_deps.extend(deps);
    }

    // Remove duplicates
    all_deps.sort_by(|a, b| a.id.cmp(&b.id));
    all_deps.dedup_by(|a, b| a.id == b.id);

    let deps_json = serde_json::to_value(&all_deps)?;

    let summary = format!(
        "🔍 任务依赖分析 ({} 个任务)\n\n{}",
        params.task_ids.len(),
        if all_deps.is_empty() {
            "所选任务暂无依赖关系".to_string()
        } else {
            all_deps.iter().map(|dep| {
                format!(
                    "🔗 任务 {} → 任务 {} ({})",
                    dep.predecessor_id,
                    dep.successor_id,
                    dep.dependency_type.to_string().replace('_', " ")
                )
            }).collect::<Vec<_>>().join("\n")
        }
    );

    Ok(json!({
        "success": true,
        "task_ids": params.task_ids,
        "dependencies": deps_json,
        "summary": summary,
        "count": all_deps.len()
    }))
}

/// Add a new dependency relationship between tasks
pub async fn add_task_dependency_tool(
    dependency_service: Arc<DependencyService>,
    args: JsonValue,
) -> AppResult<JsonValue> {
    debug!("add_task_dependency_tool invoked");

    let params: AddTaskDependencyParams = serde_json::from_value(args)
        .map_err(|e| AppError::validation(format!("Failed to parse parameters: {}", e)))?;

    // 数据验证
    if params.predecessor_id.trim().is_empty() || params.successor_id.trim().is_empty() {
        return Err(AppError::validation("前置任务和后续任务ID都不能为空"));
    }

    if params.predecessor_id == params.successor_id {
        return Err(AppError::validation("任务不能与自己建立依赖关系"));
    }

    // 验证并设置依赖类型
    let dependency_type = match params.dependency_type {
        Some(ref dt) => dt.as_str(),
        None => "finish_to_start",
    };

    let dependency_type_enum = match dependency_type {
        "finish_to_start" => DependencyType::FinishToStart,
        "start_to_start" => DependencyType::StartToStart,
        "finish_to_finish" => DependencyType::FinishToFinish,
        "start_to_finish" => DependencyType::StartToFinish,
        _ => return Err(AppError::validation(format!(
            "无效的依赖类型: {}. 有效值: finish_to_start, start_to_start, finish_to_finish, start_to_finish",
            dependency_type
        ))),
    };

    let input = DependencyCreateInput {
        predecessor_id: params.predecessor_id.trim().to_string(),
        successor_id: params.successor_id.trim().to_string(),
        dependency_type: Some(dependency_type_enum),
    };

    let dependency_id = dependency_service.add_dependency(input).await?;
    let dependency = dependency_service.get_dependency_by_id(&dependency_id)
        .await?
        .ok_or_else(|| AppError::not_found())?;

    let result = json!({
        "success": true,
        "dependency_id": dependency_id,
        "dependency": dependency,
        "message": format!("✅ 成功创建依赖关系: {} → {} ({})",
            dependency.predecessor_id,
            dependency.successor_id,
            dependency.dependency_type.to_string().replace('_', " ")
        )
    });

    debug!(dependency_id = %dependency_id, "task dependency added successfully");
    Ok(result)
}

/// Remove a dependency relationship
pub async fn remove_task_dependency_tool(
    dependency_service: Arc<DependencyService>,
    args: JsonValue,
) -> AppResult<JsonValue> {
    debug!("remove_task_dependency_tool invoked");

    let params: RemoveTaskDependencyParams = serde_json::from_value(args)
        .map_err(|e| AppError::validation(format!("Failed to parse parameters: {}", e)))?;

    dependency_service.remove_dependency(&params.dependency_id).await?;

    let result = json!({
        "success": true,
        "dependency_id": params.dependency_id,
        "message": format!("🗑️ 成功删除依赖关系: {}", params.dependency_id)
    });

    debug!(dependency_id = %params.dependency_id, "task dependency removed successfully");
    Ok(result)
}

/// Get tasks that are ready to execute (dependencies satisfied)
pub async fn get_ready_tasks_tool(
    dependency_service: Arc<DependencyService>,
    args: JsonValue,
) -> AppResult<JsonValue> {
    debug!("get_ready_tasks_tool invoked");

    let params: GetReadyTasksParams = serde_json::from_value(args)
        .map_err(|e| AppError::validation(format!("Failed to parse parameters: {}", e)))?;

    let ready_tasks = dependency_service.get_ready_tasks().await?;

    let limit = params.limit.unwrap_or(10);
    let filtered_tasks: Vec<_> = ready_tasks.into_iter()
        .take(limit)
        .collect();

    let tasks_json = serde_json::to_value(&filtered_tasks)?;

    let summary = format!(
        "🎯 可执行任务 (显示 {} 个)\n\n{}",
        filtered_tasks.len(),
        if filtered_tasks.is_empty() {
            "🎉 所有任务都已完成！或没有满足条件的任务。".to_string()
        } else {
            filtered_tasks.iter().map(|task| {
                let status_emoji = match task.status.as_str() {
                    "todo" => "📝",
                    "in_progress" => "🔄",
                    "blocked" => "⛔", 
                    "done" => "✅",
                    _ => "📋"
                };
                format!(
                    "{} 任务 {} - {}{}", 
                    status_emoji,
                    task.id,
                    task.title,
                    if task.due_at.is_some() {
                        " (有截止日期)".to_string()
                    } else {
                        String::new()
                    }
                )
            }).collect::<Vec<_>>().join("\n")
        }
    );

    Ok(json!({
        "success": true,
        "tasks": tasks_json,
        "summary": summary,
        "count": filtered_tasks.len()
    }))
}

/// Get critical path analysis for tasks
pub async fn get_critical_path_tool(
    dependency_service: Arc<DependencyService>,
    args: JsonValue,
) -> AppResult<JsonValue> {
    debug!("get_critical_path_tool invoked");

    let params: GetCriticalPathParams = serde_json::from_value(args)
        .map_err(|e| AppError::validation(format!("Failed to parse parameters: {}", e)))?;

    let (critical_path, analysis) = if let Some(ref task_id) = params.task_id {
        // Analyze critical path for specific task
        let path = dependency_service.calculate_critical_path(task_id).await?;
        let path_info = format!("任务 {} 的关键路径 ({} 个任务): {}", 
            task_id, 
            path.len(),
            path.join(" → ")
        );
        (path, Some(path_info))
    } else {
        // Get overall critical path
        let graph = dependency_service.get_dependency_graph(None).await?;
        (graph.critical_path, None)
    };

    let critical_path_json = serde_json::to_value(&critical_path)?;

    let summary = if let Some(ref analysis_text) = analysis {
        format!(
            "🎯 关键路径分析\n\n{}\n\n📊 路径统计:\n• 路径长度: {} 个任务\n• 关键任务: {}\n\n💡 建议:\n• 重点关注关键路径上的任务\n• 任何延迟都会影响整体进度\n• 优先分配资源给关键任务",
            analysis_text,
            critical_path.len(),
            critical_path.join(", ")
        )
    } else {
        format!(
            "🎯 整体关键路径分析\n\n📊 总体统计:\n• 关键路径长度: {} 个任务\n• 关键任务: {}\n\n💡 策略建议:\n• 优先完成关键路径上的任务\n• 监控关键任务的进度和风险\n• 及时处理关键路径上的阻塞问题",
            critical_path.len(),
            critical_path.join(", ")
        )
    };

    Ok(json!({
        "success": true,
        "critical_path": critical_path_json,
        "analysis": analysis,
        "summary": summary,
        "metrics": {
            "path_length": critical_path.len(),
            "task_count": critical_path.len(),
            "is_analyzed": params.include_analysis,
        }
    }))
}

/// Validate if a dependency relationship would be valid
pub async fn validate_dependency_tool(
    dependency_service: Arc<DependencyService>,
    args: JsonValue,
) -> AppResult<JsonValue> {
    debug!("validate_dependency_tool invoked");

    let params: ValidateDependencyParams = serde_json::from_value(args)
        .map_err(|e| AppError::validation(format!("Failed to parse parameters: {}", e)))?;

    let validation = dependency_service
        .validate_dependency(&params.predecessor_id, &params.successor_id)
        .await?;

    let validation_json = serde_json::to_value(&validation)?;

    let summary = if validation.is_valid {
        if validation.would_create_cycle {
            format!(
                "❌ 依赖关系验证失败\n\n🔄 循环依赖检测:\n• 创建此依赖关系会形成循环\n• 路径: {}\n\n💡 建议:\n• 重新设计任务依赖结构\n• 避免创建循环依赖",
                validation.cycle_path.map(|p| p.join(" → ")).unwrap_or_default()
            )
        } else {
            format!(
                "✅ 依赖关系验证通过\n\n📋 关系详情:\n• 前置任务: {}\n• 后续任务: {}\n• 依赖类型: {}\n\n🎯 可以安全创建此依赖关系",
                params.predecessor_id,
                params.successor_id,
                params.dependency_type.as_ref().map(|t| t.replace('_', " ")).unwrap_or("finish_to_start".to_string())
            )
        }
    } else {
        format!(
            "❌ 依赖关系验证失败\n\n⚠️ 错误信息:\n{}\n\n💡 建议:\n• 检查任务ID是否正确\n• 确保前置任务存在且已完成\n• 验证依赖类型设置",
            validation.error_message.unwrap_or("未知错误".to_string())
        )
    };

    Ok(json!({
        "success": true,
        "validation": validation_json,
        "summary": summary,
        "is_valid": validation.is_valid,
        "would_create_cycle": validation.would_create_cycle
    }))
}

/// Get dependency-related metrics and statistics
pub async fn get_dependency_metrics_tool(
    dependency_service: Arc<DependencyService>,
    args: JsonValue,
) -> AppResult<JsonValue> {
    debug!("get_dependency_metrics_tool invoked");

    let params: GetDependencyMetricsParams = serde_json::from_value(args)
        .map_err(|e| AppError::validation(format!("Failed to parse parameters: {}", e)))?;

    let graph = dependency_service.get_dependency_graph(None).await?;

    // Calculate metrics
    let total_tasks = graph.nodes.len();
    let total_dependencies = graph.edges.len();
    let ready_tasks = graph.nodes.values().filter(|node| node.is_ready).count();
    
    // Dependency density
    let max_possible_dependencies = if total_tasks > 1 {
        total_tasks * (total_tasks - 1)
    } else { 0 };
    let dependency_density = if max_possible_dependencies > 0 {
        (total_dependencies as f64 / max_possible_dependencies as f64) * 100.0
    } else { 0.0 };

    // Average dependencies per task
    let avg_dependencies_per_task = if total_tasks > 0 {
        total_dependencies as f64 / total_tasks as f64
    } else { 0.0 };

    // Blocked tasks analysis
    let blocked_tasks = graph.nodes.values()
        .filter(|node| !node.is_ready && node.status != "done")
        .count();

    let metrics_json = json!({
        "total_tasks": total_tasks,
        "total_dependencies": total_dependencies,
        "ready_tasks": ready_tasks,
        "blocked_tasks": blocked_tasks,
        "dependency_density_percent": dependency_density.round(),
        "average_dependencies_per_task": avg_dependencies_per_task.round(),
        "critical_path_length": graph.critical_path.len()
    });

    let summary = format!(
        "📊 依赖关系指标分析 ({} 天范围)\n\n📈 核心指标:\n• 任务总数: {} 个\n• 依赖关系: {} 条\n• 可执行任务: {} 个\n• 阻塞任务: {} 个\n• 关键路径: {} 个任务\n\n🎯 关系分析:\n• 依赖密度: {:.1}%\n• 平均每个任务: {:.1} 个依赖\n\n💡 优化建议:{}",
        params.time_range_days.unwrap_or(30),
        total_tasks,
        total_dependencies,
        ready_tasks,
        blocked_tasks,
        graph.critical_path.len(),
        dependency_density,
        avg_dependencies_per_task,
        if dependency_density > 50.0 {
            "\n• 依赖关系较复杂，建议简化流程"
        } else if dependency_density < 10.0 {
            "\n• 依赖关系较少，可能缺乏必要的协调"
        } else {
            "\n• 依赖关系密度适中"
        }
    );

    Ok(json!({
        "success": true,
        "metrics": metrics_json,
        "summary": summary,
        "analysis_period_days": params.time_range_days.unwrap_or(30)
    }))
}

/// Parameter structs
#[derive(Debug, Deserialize)]
struct GetDependencyGraphParams {
    task_filter: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct GetTaskDependenciesParams {
    task_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct AddTaskDependencyParams {
    predecessor_id: String,
    successor_id: String,
    dependency_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RemoveTaskDependencyParams {
    dependency_id: String,
}

#[derive(Debug, Deserialize)]
struct GetReadyTasksParams {
    limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct GetCriticalPathParams {
    task_id: Option<String>,
    #[serde(default = "default_true")]
    include_analysis: bool,
}

#[derive(Debug, Deserialize)]
struct ValidateDependencyParams {
    predecessor_id: String,
    successor_id: String,
    dependency_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GetDependencyMetricsParams {
    time_range_days: Option<i64>,
}

fn default_true() -> bool {
    true
}

/// Register all dependency management tools
pub fn register_dependency_tools(
    registry: &mut crate::services::tool_registry::ToolRegistry,
    dependency_service: Arc<DependencyService>,
) -> AppResult<()> {
    use crate::services::tool_registry::ToolHandler;
    use std::future::Future;
    use std::pin::Pin;

    // Register get_dependency_graph tool
    {
        let service = Arc::clone(&dependency_service);
        let handler: ToolHandler = Arc::new(move |args: JsonValue| {
            let service = Arc::clone(&service);
            Box::pin(async move { get_dependency_graph_tool(service, args).await })
                as Pin<Box<dyn Future<Output = AppResult<JsonValue>> + Send>>
        });

        registry.register_tool(
            "get_dependency_graph".to_string(),
            "Get the complete dependency graph for all tasks. Use this to understand task relationships, dependencies, and overall project structure. Provides topology order and critical path analysis.".to_string(),
            get_dependency_graph_schema(),
            handler,
        )?;
    }

    // Register get_task_dependencies tool
    {
        let service = Arc::clone(&dependency_service);
        let handler: ToolHandler = Arc::new(move |args: JsonValue| {
            let service = Arc::clone(&service);
            Box::pin(async move { get_task_dependencies_tool(service, args).await })
                as Pin<Box<dyn Future<Output = AppResult<JsonValue>> + Send>>
        });

        registry.register_tool(
            "get_task_dependencies".to_string(),
            "Get dependencies for specific tasks. Use when user asks about what a task depends on or what depends on it. Requires list of task IDs.".to_string(),
            get_task_dependencies_schema(),
            handler,
        )?;
    }

    // Register add_task_dependency tool
    {
        let service = Arc::clone(&dependency_service);
        let handler: ToolHandler = Arc::new(move |args: JsonValue| {
            let service = Arc::clone(&service);
            Box::pin(async move { add_task_dependency_tool(service, args).await })
                as Pin<Box<dyn Future<Output = AppResult<JsonValue>> + Send>>
        });

        registry.register_tool(
            "add_task_dependency".to_string(),
            "Add a new dependency relationship between tasks. Use when user wants to establish task dependencies like 'A must finish before B starts'. Validates for cycles and conflicts.".to_string(),
            add_task_dependency_schema(),
            handler,
        )?;
    }

    // Register remove_task_dependency tool
    {
        let service = Arc::clone(&dependency_service);
        let handler: ToolHandler = Arc::new(move |args: JsonValue| {
            let service = Arc::clone(&service);
            Box::pin(async move { remove_task_dependency_tool(service, args).await })
                as Pin<Box<dyn Future<Output = AppResult<JsonValue>> + Send>>
        });

        registry.register_tool(
            "remove_task_dependency".to_string(),
            "Remove an existing dependency relationship between tasks. Use when user wants to break task dependencies.".to_string(),
            remove_task_dependency_schema(),
            handler,
        )?;
    }

    // Register get_ready_tasks tool
    {
        let service = Arc::clone(&dependency_service);
        let handler: ToolHandler = Arc::new(move |args: JsonValue| {
            let service = Arc::clone(&service);
            Box::pin(async move { get_ready_tasks_tool(service, args).await })
                as Pin<Box<dyn Future<Output = AppResult<JsonValue>> + Send>>
        });

        registry.register_tool(
            "get_ready_tasks".to_string(),
            "Get tasks that are ready to execute (all dependencies satisfied). Use when user asks 'what can I work on next?' or 'what tasks are available?'. Essential for workflow planning.".to_string(),
            get_ready_tasks_schema(),
            handler,
        )?;
    }

    // Register get_critical_path tool
    {
        let service = Arc::clone(&dependency_service);
        let handler: ToolHandler = Arc::new(move |args: JsonValue| {
            let service = Arc::clone(&service);
            Box::pin(async move { get_critical_path_tool(service, args).await })
                as Pin<Box<dyn Future<Output = AppResult<JsonValue>> + Send>>
        });

        registry.register_tool(
            "get_critical_path".to_string(),
            "Get critical path analysis for tasks. Use when user wants to understand which tasks are most important for project timeline. Critical path tasks determine overall project duration.".to_string(),
            get_critical_path_schema(),
            handler,
        )?;
    }

    // Register validate_dependency tool
    {
        let service = Arc::clone(&dependency_service);
        let handler: ToolHandler = Arc::new(move |args: JsonValue| {
            let service = Arc::clone(&service);
            Box::pin(async move { validate_dependency_tool(service, args).await })
                as Pin<Box<dyn Future<Output = AppResult<JsonValue>> + Send>>
        });

        registry.register_tool(
            "validate_dependency".to_string(),
            "Validate if a dependency relationship would be valid before creating it. Checks for cycles, missing tasks, and logical conflicts. Use when planning new dependencies.".to_string(),
            validate_dependency_schema(),
            handler,
        )?;
    }

    // Register get_dependency_metrics tool
    {
        let service = Arc::clone(&dependency_service);
        let handler: ToolHandler = Arc::new(move |args: JsonValue| {
            let service = Arc::clone(&service);
            Box::pin(async move { get_dependency_metrics_tool(service, args).await })
                as Pin<Box<dyn Future<Output = AppResult<JsonValue>> + Send>>
        });

        registry.register_tool(
            "get_dependency_metrics".to_string(),
            "Get dependency-related metrics and statistics. Provides analysis of dependency density, task relationships, and workflow efficiency. Use for project health assessment.".to_string(),
            get_dependency_metrics_schema(),
            handler,
        )?;
    }

    Ok(())
}