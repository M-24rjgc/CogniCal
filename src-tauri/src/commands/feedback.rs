use serde_json::Value as JsonValue;
use tauri::{async_runtime, State};

use crate::commands::{AppState, CommandError, CommandResult};
use crate::models::ai_feedback::AiFeedbackSurface;
use crate::services::feedback_service::{FeedbackSubmission, WeeklyDigest};

/// Submit AI feedback
#[tauri::command]
pub async fn feedback_submit(
    submission: FeedbackSubmission,
    state: State<'_, AppState>,
) -> CommandResult<i64> {
    let app_state = state.inner().clone();
    async_runtime::spawn_blocking(move || app_state.feedback().submit_feedback(&submission))
        .await
        .map_err(|e| CommandError::new("INTERNAL", e.to_string(), None))?
        .map_err(CommandError::from)
}

/// Get recent feedback for a surface
#[tauri::command]
pub async fn feedback_get_recent(
    surface: String,
    limit: Option<i64>,
    state: State<'_, AppState>,
) -> CommandResult<Vec<crate::models::ai_feedback::AiFeedback>> {
    let surface_enum: AiFeedbackSurface = surface
        .parse()
        .map_err(|e: String| CommandError::new("INVALID_INPUT", e, None))?;

    let app_state = state.inner().clone();
    async_runtime::spawn_blocking(move || {
        app_state
            .feedback()
            .get_recent_feedback(surface_enum, limit)
    })
    .await
    .map_err(|e| CommandError::new("INTERNAL", e.to_string(), None))?
    .map_err(CommandError::from)
}

/// Get feedback for a session
#[tauri::command]
pub async fn feedback_get_session(
    session_id: String,
    state: State<'_, AppState>,
) -> CommandResult<Vec<crate::models::ai_feedback::AiFeedback>> {
    let app_state = state.inner().clone();
    async_runtime::spawn_blocking(move || app_state.feedback().get_session_feedback(&session_id))
        .await
        .map_err(|e| CommandError::new("INTERNAL", e.to_string(), None))?
        .map_err(CommandError::from)
}

/// Generate weekly digest
#[tauri::command]
pub async fn feedback_get_weekly_digest(
    state: State<'_, AppState>,
) -> CommandResult<Option<WeeklyDigest>> {
    let app_state = state.inner().clone();
    async_runtime::spawn_blocking(move || app_state.feedback().generate_weekly_digest())
        .await
        .map_err(|e| CommandError::new("INTERNAL", e.to_string(), None))?
        .map_err(CommandError::from)
}

/// Check if user has opted out
#[tauri::command]
pub async fn feedback_check_opt_out(state: State<'_, AppState>) -> CommandResult<bool> {
    let app_state = state.inner().clone();
    async_runtime::spawn_blocking(move || app_state.feedback().is_opted_out())
        .await
        .map_err(|e| CommandError::new("INTERNAL", e.to_string(), None))?
        .map_err(CommandError::from)
}

/// Purge all feedback data
#[tauri::command]
pub async fn feedback_purge_all(state: State<'_, AppState>) -> CommandResult<i64> {
    let app_state = state.inner().clone();
    async_runtime::spawn_blocking(move || app_state.feedback().purge_all_feedback())
        .await
        .map_err(|e| CommandError::new("INTERNAL", e.to_string(), None))?
        .map_err(CommandError::from)
}

/// Get feedback statistics
#[tauri::command]
pub async fn feedback_get_stats(
    surface: Option<String>,
    state: State<'_, AppState>,
) -> CommandResult<JsonValue> {
    let surface_enum = if let Some(s) = surface {
        Some(
            s.parse::<AiFeedbackSurface>()
                .map_err(|e: String| CommandError::new("INVALID_INPUT", e, None))?,
        )
    } else {
        None
    };

    let app_state = state.inner().clone();
    async_runtime::spawn_blocking(move || app_state.feedback().get_feedback_stats(surface_enum))
        .await
        .map_err(|e| CommandError::new("INTERNAL", e.to_string(), None))?
        .map_err(CommandError::from)
}
