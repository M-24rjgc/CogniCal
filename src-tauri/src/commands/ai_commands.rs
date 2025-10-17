use serde_json::{self, Value as JsonValue};
use tauri::State;
use tracing::{debug, warn};

use crate::models::ai::{TaskParseRequest, TaskParseResponse};
use crate::models::ai_types::AiStatusDto;

use super::{AppState, CommandError, CommandResult};

pub(crate) async fn tasks_parse_ai_impl(
    app_state: &AppState,
    request: TaskParseRequest,
) -> CommandResult<TaskParseResponse> {
    if request.input.trim().is_empty() {
        return Err(CommandError::new(
            "VALIDATION_ERROR",
            "待解析内容不能为空",
            None,
        ));
    }

    let has_context = request.context.is_some();
    debug!(
        target: "app::command",
        has_context,
        "tasks_parse_ai invoked"
    );

    let service = app_state.ai();
    match service.parse_task(request).await {
        Ok(response) => {
            let correlation_id = response
                .ai
                .metadata
                .as_ref()
                .and_then(|meta| meta.get("provider"))
                .and_then(|provider| provider.get("extra"))
                .and_then(|extra| extra.get("correlationId"))
                .and_then(|value| value.as_str())
                .unwrap_or("-");

            debug!(
                target: "app::command",
                source = ?response.ai.source,
                missing = response.missing_fields.len(),
                correlation_id = %correlation_id,
                "tasks_parse_ai completed"
            );
            Ok(response)
        }
        Err(error) => {
            let correlation_id = error.ai_correlation_id().unwrap_or("-");
            warn!(
                target: "app::command",
                error = %error,
                correlation_id = %correlation_id,
                "tasks_parse_ai failed"
            );
            Err(CommandError::from(error))
        }
    }
}

pub(crate) async fn ai_generate_recommendations_impl(
    app_state: &AppState,
    payload: JsonValue,
) -> CommandResult<JsonValue> {
    debug!(target: "app::command", "ai_generate_recommendations invoked");

    let service = app_state.ai();
    match service.generate_recommendations(payload).await {
        Ok(dto) => {
            let count = dto.recommendations.len();
            let value = serde_json::to_value(dto).map_err(|err| {
                CommandError::new("UNKNOWN", format!("推荐结果序列化失败: {err}"), None)
            })?;
            debug!(
                target: "app::command",
                recommendation_count = count,
                "ai_generate_recommendations completed"
            );
            Ok(value)
        }
        Err(error) => {
            warn!(
                target: "app::command",
                error = %error,
                "ai_generate_recommendations failed"
            );
            Err(CommandError::from(error))
        }
    }
}

pub(crate) async fn ai_plan_schedule_impl(
    app_state: &AppState,
    payload: JsonValue,
) -> CommandResult<JsonValue> {
    debug!(target: "app::command", "ai_plan_schedule invoked");

    let service = app_state.ai();
    match service.plan_schedule(payload).await {
        Ok(dto) => {
            let count = dto.items.len();
            let value = serde_json::to_value(dto).map_err(|err| {
                CommandError::new("UNKNOWN", format!("排程结果序列化失败: {err}"), None)
            })?;
            debug!(
                target: "app::command",
                block_count = count,
                "ai_plan_schedule completed"
            );
            Ok(value)
        }
        Err(error) => {
            warn!(target: "app::command", error = %error, "ai_plan_schedule failed");
            Err(CommandError::from(error))
        }
    }
}

pub(crate) async fn ai_status_impl(app_state: &AppState) -> CommandResult<AiStatusDto> {
    debug!(target: "app::command", "ai_status invoked");

    let service = app_state.ai();
    match service.status().await {
        Ok(status) => Ok(status),
        Err(error) => {
            warn!(
                target: "app::command",
                error = %error,
                "ai_status failed"
            );
            Err(CommandError::from(error))
        }
    }
}

#[tauri::command]
pub async fn tasks_parse_ai(
    state: State<'_, AppState>,
    request: TaskParseRequest,
) -> CommandResult<TaskParseResponse> {
    tasks_parse_ai_impl(state.inner(), request).await
}

#[tauri::command]
pub async fn ai_generate_recommendations(
    state: State<'_, AppState>,
    payload: JsonValue,
) -> CommandResult<JsonValue> {
    ai_generate_recommendations_impl(state.inner(), payload).await
}

#[tauri::command]
pub async fn ai_plan_schedule(
    state: State<'_, AppState>,
    payload: JsonValue,
) -> CommandResult<JsonValue> {
    ai_plan_schedule_impl(state.inner(), payload).await
}

#[tauri::command]
pub async fn ai_status(state: State<'_, AppState>) -> CommandResult<AiStatusDto> {
    ai_status_impl(state.inner()).await
}

pub mod testing {
    use super::*;

    /// Internal helper exposed for integration testing of command logic.
    pub async fn tasks_parse_ai(
        app_state: &AppState,
        request: TaskParseRequest,
    ) -> CommandResult<TaskParseResponse> {
        tasks_parse_ai_impl(app_state, request).await
    }

    /// Internal helper exposed for integration testing of command logic.
    pub async fn ai_generate_recommendations(
        app_state: &AppState,
        payload: JsonValue,
    ) -> CommandResult<JsonValue> {
        ai_generate_recommendations_impl(app_state, payload).await
    }

    /// Internal helper exposed for integration testing of command logic.
    pub async fn ai_plan_schedule(
        app_state: &AppState,
        payload: JsonValue,
    ) -> CommandResult<JsonValue> {
        ai_plan_schedule_impl(app_state, payload).await
    }

    /// Internal helper exposed for integration testing of command logic.
    pub async fn ai_status(app_state: &AppState) -> CommandResult<AiStatusDto> {
        ai_status_impl(app_state).await
    }
}
