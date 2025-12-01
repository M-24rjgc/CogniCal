use tauri::State;

use crate::commands::{AppState, CommandResult};
use crate::models::dependency::{DependencyCreateInput, DependencyFilter, DependencyValidation};

#[tauri::command]
pub async fn get_task_dependencies(
    state: State<'_, AppState>,
    filter: Option<DependencyFilter>,
) -> CommandResult<Vec<crate::models::dependency::TaskDependency>> {
    let service = state.dependency_service();

    // If a task_id filter is provided, get dependencies for that specific task
    // Otherwise, return empty list (could be extended to return all dependencies)
    if let Some(ref f) = filter {
        if let Some(ref task_ids) = f.task_ids {
            if !task_ids.is_empty() {
                let mut all_deps = Vec::new();
                for task_id in task_ids {
                    let deps = service.get_task_dependencies(task_id).await?;
                    all_deps.extend(deps);
                }
                // Remove duplicates
                all_deps.sort_by(|a, b| a.id.cmp(&b.id));
                all_deps.dedup_by(|a, b| a.id == b.id);
                return Ok(all_deps);
            }
        }
    }

    // If no specific filter, return all dependencies
    let all_deps = service.get_all_dependencies().await?;
    Ok(all_deps)
}

#[tauri::command]
pub async fn get_dependency_graph(
    state: State<'_, AppState>,
    filter: Option<DependencyFilter>,
) -> CommandResult<crate::models::dependency::DependencyGraph> {
    let service = state.dependency_service();
    service
        .get_dependency_graph(filter)
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub async fn get_ready_tasks(
    state: State<'_, AppState>,
) -> CommandResult<Vec<crate::models::dependency::ReadyTask>> {
    let service = state.dependency_service();
    service.get_ready_tasks().await.map_err(Into::into)
}

#[tauri::command]
pub async fn add_dependency(
    state: State<'_, AppState>,
    input: DependencyCreateInput,
) -> CommandResult<crate::models::dependency::TaskDependency> {
    let service = state.dependency_service();
    let dependency_id = service.add_dependency(input).await?;

    // Fetch and return the created dependency
    let dependency = service
        .get_dependency_by_id(&dependency_id)
        .await?
        .ok_or_else(|| crate::error::AppError::not_found())?;

    Ok(dependency)
}

#[tauri::command]
pub async fn remove_dependency(
    state: State<'_, AppState>,
    dependency_id: String,
) -> CommandResult<()> {
    let service = state.dependency_service();
    service
        .remove_dependency(&dependency_id)
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub async fn update_dependency_type(
    state: State<'_, AppState>,
    dependency_id: String,
    dependency_type: crate::models::dependency::DependencyType,
) -> CommandResult<()> {
    let service = state.dependency_service();
    service
        .update_dependency_type(&dependency_id, dependency_type)
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub async fn validate_dependency(
    state: State<'_, AppState>,
    predecessor_id: String,
    successor_id: String,
) -> CommandResult<DependencyValidation> {
    let service = state.dependency_service();
    service
        .validate_dependency(&predecessor_id, &successor_id)
        .await
        .map_err(Into::into)
}
