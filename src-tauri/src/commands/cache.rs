use tauri::{async_runtime, State};

use crate::commands::{AppState, CacheClearResult, CommandError, CommandResult};
use crate::error::AppError;

#[tauri::command]
pub async fn cache_clear_all(state: State<'_, AppState>) -> CommandResult<CacheClearResult> {
    let app_state = state.inner().clone();
    run_blocking(move || app_state.clear_all_cache()).await
}

async fn run_blocking<T: Send + 'static>(
    task: impl FnOnce() -> Result<T, AppError> + Send + 'static,
) -> CommandResult<T> {
    async_runtime::spawn_blocking(task)
        .await
        .map_err(|err| CommandError::new("UNKNOWN", format!("缓存清除操作执行失败: {err}"), None))?
        .map_err(CommandError::from)
}
