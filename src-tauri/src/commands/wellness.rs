use tauri::{async_runtime, State};

use crate::commands::{AppState, CommandError, CommandResult};
use crate::error::AppError;
use crate::models::wellness::{WellnessEventRecord, WellnessResponse};
use crate::services::wellness_service::WeeklySummary;

#[tauri::command]
pub async fn wellness_check_nudge(
    state: State<'_, AppState>,
) -> CommandResult<Option<WellnessEventRecord>> {
    let app_state = state.inner().clone();

    run_blocking(move || app_state.wellness().check_and_generate_nudge()).await
}

#[tauri::command]
pub async fn wellness_get_pending(
    state: State<'_, AppState>,
) -> CommandResult<Option<WellnessEventRecord>> {
    let app_state = state.inner().clone();

    run_blocking(move || app_state.wellness().get_pending_nudge()).await
}

#[tauri::command]
pub async fn wellness_respond(
    state: State<'_, AppState>,
    id: i64,
    response: String,
) -> CommandResult<WellnessEventRecord> {
    let app_state = state.inner().clone();

    run_blocking(move || {
        let wellness_response =
            WellnessResponse::try_from(response.as_str()).map_err(AppError::validation)?;
        app_state.wellness().respond_to_nudge(id, wellness_response)
    })
    .await
}

#[tauri::command]
pub async fn wellness_get_weekly_summary(
    state: State<'_, AppState>,
) -> CommandResult<WeeklySummary> {
    let app_state = state.inner().clone();

    run_blocking(move || app_state.wellness().get_weekly_summary()).await
}

async fn run_blocking<T: Send + 'static>(
    task: impl FnOnce() -> Result<T, AppError> + Send + 'static,
) -> CommandResult<T> {
    async_runtime::spawn_blocking(task)
        .await
        .map_err(|err| CommandError::new("UNKNOWN", format!("健康提醒任务执行失败: {err}"), None))?
        .map_err(CommandError::from)
}
