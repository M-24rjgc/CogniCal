use tauri::State;

use crate::commands::{AppState, CommandResult};
use crate::models::goal::{CreateGoalRequest, Goal, GoalTaskAssociation, GoalWithProgress, UpdateGoalRequest};

#[tauri::command]
pub async fn create_goal(
    state: State<'_, AppState>,
    request: CreateGoalRequest,
) -> CommandResult<Goal> {
    let service = state.goals();
    service.create_goal(request).map_err(Into::into)
}

#[tauri::command]
pub async fn get_goal(
    state: State<'_, AppState>,
    id: String,
) -> CommandResult<Goal> {
    let service = state.goals();
    service.get_goal(&id).map_err(Into::into)
}

#[tauri::command]
pub async fn list_goals(
    state: State<'_, AppState>,
    parent_goal_id: Option<String>,
) -> CommandResult<Vec<Goal>> {
    let service = state.goals();
    service.list_goals(parent_goal_id).map_err(Into::into)
}

#[tauri::command]
pub async fn update_goal(
    state: State<'_, AppState>,
    id: String,
    request: UpdateGoalRequest,
) -> CommandResult<Goal> {
    let service = state.goals();
    service.update_goal(&id, request).map_err(Into::into)
}

#[tauri::command]
pub async fn delete_goal(
    state: State<'_, AppState>,
    id: String,
) -> CommandResult<()> {
    let service = state.goals();
    service.delete_goal(&id).map_err(Into::into)
}

#[tauri::command]
pub async fn associate_task_with_goal(
    state: State<'_, AppState>,
    goal_id: String,
    task_id: String,
) -> CommandResult<GoalTaskAssociation> {
    let service = state.goals();
    service.associate_task(&goal_id, &task_id).map_err(Into::into)
}

#[tauri::command]
pub async fn dissociate_task_from_goal(
    state: State<'_, AppState>,
    goal_id: String,
    task_id: String,
) -> CommandResult<()> {
    let service = state.goals();
    service.dissociate_task(&goal_id, &task_id).map_err(Into::into)
}

#[tauri::command]
pub async fn get_goal_tasks(
    state: State<'_, AppState>,
    goal_id: String,
) -> CommandResult<Vec<String>> {
    let service = state.goals();
    service.get_goal_tasks(&goal_id).map_err(Into::into)
}

#[tauri::command]
pub async fn get_goal_with_progress(
    state: State<'_, AppState>,
    id: String,
) -> CommandResult<GoalWithProgress> {
    let service = state.goals();
    service.get_goal_with_progress(&id).map_err(Into::into)
}
