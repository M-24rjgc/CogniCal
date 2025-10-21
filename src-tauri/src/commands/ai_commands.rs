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

// Agent chat structures
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentChatRequest {
    pub conversation_id: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentChatResponse {
    pub message: String,
    pub tool_calls: Vec<serde_json::Value>,
    pub memory_stored: bool,
    pub metadata: AgentChatMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentChatMetadata {
    pub tokens_used: std::collections::HashMap<String, u64>,
    pub latency_ms: u128,
    pub memory_entries_used: usize,
    pub tools_executed: Vec<String>,
}

pub(crate) async fn ai_agent_chat_impl(
    app_state: &AppState,
    request: AgentChatRequest,
) -> CommandResult<AgentChatResponse> {
    if request.message.trim().is_empty() {
        return Err(CommandError::new(
            "VALIDATION_ERROR",
            "消息内容不能为空",
            None,
        ));
    }

    if request.conversation_id.trim().is_empty() {
        return Err(CommandError::new(
            "VALIDATION_ERROR",
            "会话ID不能为空",
            None,
        ));
    }

    debug!(
        target: "app::command",
        conversation_id = %request.conversation_id,
        message_len = request.message.len(),
        "ai_agent_chat invoked"
    );

    let agent_service = app_state.agent();
    match agent_service
        .chat(&request.conversation_id, &request.message)
        .await
    {
        Ok(response) => {
            debug!(
                target: "app::command",
                conversation_id = %request.conversation_id,
                response_len = response.message.len(),
                tools_executed = response.metadata.tools_executed.len(),
                memory_stored = response.memory_stored,
                "ai_agent_chat completed"
            );

            // Convert tool calls to JSON values for serialization
            let tool_calls: Vec<serde_json::Value> = response
                .tool_calls
                .iter()
                .map(|tc| {
                    serde_json::json!({
                        "id": tc.id,
                        "name": tc.name,
                        "arguments": tc.arguments,
                    })
                })
                .collect();

            Ok(AgentChatResponse {
                message: response.message,
                tool_calls,
                memory_stored: response.memory_stored,
                metadata: AgentChatMetadata {
                    tokens_used: response.metadata.tokens_used,
                    latency_ms: response.metadata.latency_ms,
                    memory_entries_used: response.metadata.memory_entries_used,
                    tools_executed: response.metadata.tools_executed,
                },
            })
        }
        Err(error) => {
            warn!(
                target: "app::command",
                error = %error,
                conversation_id = %request.conversation_id,
                "ai_agent_chat failed"
            );
            Err(CommandError::from(error))
        }
    }
}

#[tauri::command]
pub async fn ai_agent_chat(
    state: State<'_, AppState>,
    conversation_id: String,
    message: String,
) -> CommandResult<AgentChatResponse> {
    ai_agent_chat_impl(
        state.inner(),
        AgentChatRequest {
            conversation_id,
            message,
        },
    )
    .await
}

pub mod testing {
    use super::*;

    // Re-export request/response types for testing
    pub use super::{
        AgentChatRequest, AgentChatResponse, MemoryClearRequest, MemoryClearResponse,
        MemoryExportRequest, MemoryExportResponse, MemorySearchRequest, MemorySearchResponse,
    };

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

    /// Internal helper exposed for integration testing of command logic.
    pub async fn ai_agent_chat(
        app_state: &AppState,
        request: AgentChatRequest,
    ) -> CommandResult<AgentChatResponse> {
        ai_agent_chat_impl(app_state, request).await
    }

    /// Internal helper exposed for integration testing of memory search logic.
    pub async fn memory_search(
        app_state: &AppState,
        request: MemorySearchRequest,
    ) -> CommandResult<MemorySearchResponse> {
        memory_search_impl(app_state, request).await
    }

    /// Internal helper exposed for integration testing of memory export logic.
    pub async fn memory_export(
        app_state: &AppState,
        request: MemoryExportRequest,
    ) -> CommandResult<MemoryExportResponse> {
        memory_export_impl(app_state, request).await
    }

    /// Internal helper exposed for integration testing of memory clear logic.
    pub async fn memory_clear(
        app_state: &AppState,
        request: MemoryClearRequest,
    ) -> CommandResult<MemoryClearResponse> {
        memory_clear_impl(app_state, request).await
    }
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatRequest {
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatResponse {
    pub message: String,
    pub timestamp: String,
}

pub(crate) async fn ai_chat_impl(
    app_state: &AppState,
    request: ChatRequest,
) -> CommandResult<ChatResponse> {
    if request.message.trim().is_empty() {
        return Err(CommandError::new(
            "VALIDATION_ERROR",
            "消息内容不能为空",
            None,
        ));
    }

    debug!(
        target: "app::command",
        message_len = request.message.len(),
        "ai_chat invoked"
    );

    let service = app_state.ai();
    match service.chat(request.message).await {
        Ok(response_text) => {
            let response = ChatResponse {
                message: response_text,
                timestamp: chrono::Utc::now().to_rfc3339(),
            };
            debug!(
                target: "app::command",
                response_len = response.message.len(),
                "ai_chat completed"
            );
            Ok(response)
        }
        Err(error) => {
            warn!(target: "app::command", error = %error, "ai_chat failed");
            Err(CommandError::from(error))
        }
    }
}

#[tauri::command]
pub async fn ai_chat(state: State<'_, AppState>, message: String) -> CommandResult<ChatResponse> {
    ai_chat_impl(state.inner(), ChatRequest { message }).await
}

// Memory management structures and commands

#[derive(Debug, Serialize, Deserialize)]
pub struct MemorySearchRequest {
    pub query: String,
    #[serde(default)]
    pub filters: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemorySearchResponse {
    pub entries: Vec<MemoryEntryDto>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryEntryDto {
    pub id: String,
    pub conversation_id: String,
    pub user_message: String,
    pub assistant_message: String,
    pub timestamp: String,
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryExportRequest {
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryExportResponse {
    pub success: bool,
    pub path: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryClearRequest {
    pub conversation_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryClearResponse {
    pub success: bool,
    pub message: String,
}



// Memory command implementations

pub(crate) async fn memory_search_impl(
    _app_state: &AppState,
    request: MemorySearchRequest,
) -> CommandResult<MemorySearchResponse> {
    if request.query.trim().is_empty() {
        return Err(CommandError::new(
            "VALIDATION_ERROR",
            "搜索查询不能为空",
            None,
        ));
    }

    debug!(
        target: "app::command",
        query = %request.query,
        "memory_search invoked"
    );

    // Memory service is currently unavailable - return empty results
    warn!(
        target: "app::command",
        query = %request.query,
        "Memory service unavailable, returning empty results"
    );
    
    Ok(MemorySearchResponse {
        entries: Vec::new(),
    })
}

pub(crate) async fn memory_export_impl(
    _app_state: &AppState,
    request: MemoryExportRequest,
) -> CommandResult<MemoryExportResponse> {
    if request.path.trim().is_empty() {
        return Err(CommandError::new(
            "VALIDATION_ERROR",
            "导出路径不能为空",
            None,
        ));
    }

    debug!(
        target: "app::command",
        path = %request.path,
        "memory_export invoked"
    );

    // Memory service is currently unavailable
    warn!(
        target: "app::command",
        path = %request.path,
        "Memory service unavailable, cannot export"
    );
    
    Err(CommandError::new(
        "MEMORY_UNAVAILABLE",
        "记忆服务当前不可用，无法导出数据",
        None,
    ))
}

pub(crate) async fn memory_clear_impl(
    _app_state: &AppState,
    request: MemoryClearRequest,
) -> CommandResult<MemoryClearResponse> {
    if request.conversation_id.trim().is_empty() {
        return Err(CommandError::new(
            "VALIDATION_ERROR",
            "会话ID不能为空",
            None,
        ));
    }

    debug!(
        target: "app::command",
        conversation_id = %request.conversation_id,
        "memory_clear invoked"
    );

    // Memory service is currently unavailable
    warn!(
        target: "app::command",
        conversation_id = %request.conversation_id,
        "Memory service unavailable, cannot clear memory"
    );
    
    Err(CommandError::new(
        "MEMORY_UNAVAILABLE",
        "记忆服务当前不可用，无法清除记忆数据",
        None,
    ))
}

// Tauri command wrappers for memory operations

#[tauri::command]
pub async fn memory_search(
    state: State<'_, AppState>,
    query: String,
    filters: Option<std::collections::HashMap<String, String>>,
) -> CommandResult<MemorySearchResponse> {
    memory_search_impl(state.inner(), MemorySearchRequest { query, filters }).await
}

#[tauri::command]
pub async fn memory_export(
    state: State<'_, AppState>,
    path: String,
) -> CommandResult<MemoryExportResponse> {
    memory_export_impl(state.inner(), MemoryExportRequest { path }).await
}

#[tauri::command]
pub async fn memory_clear(
    state: State<'_, AppState>,
    conversation_id: String,
) -> CommandResult<MemoryClearResponse> {
    memory_clear_impl(state.inner(), MemoryClearRequest { conversation_id }).await
}
