use crate::error::{AppError, AppResult};
use crate::services::ai_service::AiService;

use crate::services::tool_registry::{ToolCall, ToolRegistry, ToolResult};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, error, info, warn};

/// Context for an AI agent interaction
#[derive(Debug, Clone, Serialize)]
pub struct AgentContext {
    /// Conversation ID for this interaction
    pub conversation_id: String,

    /// Available tool schemas for the AI
    pub available_tools: Vec<JsonValue>,
    /// System prompt with memory and tool instructions
    pub system_prompt: String,
    /// Recent conversation history reconstructed from memory (alternating user/assistant)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub history_messages: Vec<ChatMessage>,
}

/// Response from the AI agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    /// The final message to return to the user
    pub message: String,
    /// Tool calls that were executed (if any)
    #[serde(default)]
    pub tool_calls: Vec<ToolCall>,
    /// Whether the conversation was stored in memory
    pub memory_stored: bool,
    /// Metadata about the interaction
    pub metadata: AgentMetadata,
}

/// Metadata about an agent interaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetadata {
    /// Token usage breakdown
    pub tokens_used: HashMap<String, u64>,
    /// Total latency in milliseconds
    pub latency_ms: u128,
    /// Number of memory entries used in context
    pub memory_entries_used: usize,
    /// Names of tools that were executed
    pub tools_executed: Vec<String>,
    /// Correlation ID for tracking this interaction
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,
    /// Errors encountered during execution
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<ErrorDetail>>,
    /// Whether memory service was available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_available: Option<bool>,
    /// Performance breakdown by component
    #[serde(skip_serializing_if = "Option::is_none")]
    pub performance: Option<PerformanceMetrics>,
}

/// Performance metrics for different components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Time spent building context (ms)
    pub context_building_ms: u128,
    /// Time spent on memory retrieval (ms)
    pub memory_retrieval_ms: u128,
    /// Time spent on AI API calls (ms)
    pub ai_api_ms: u128,
    /// Time spent executing tools (ms)
    pub tool_execution_ms: u128,
    /// Time spent storing conversation (ms)
    pub memory_storage_ms: u128,
    /// Individual tool execution times
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_timings: Option<HashMap<String, u128>>,
}

/// Details about an error that occurred
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetail {
    /// Type of error (e.g., "tool_execution", "memory_storage", "context_building")
    pub error_type: String,
    /// Error message
    pub message: String,
    /// Timestamp when error occurred
    pub timestamp: String,
    /// Additional context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<HashMap<String, String>>,
}

impl Default for AgentMetadata {
    fn default() -> Self {
        Self {
            tokens_used: HashMap::new(),
            latency_ms: 0,
            memory_entries_used: 0,
            tools_executed: Vec::new(),
            correlation_id: None,
            errors: None,
            memory_available: None,
            performance: None,
        }
    }
}

/// AI Agent Service that orchestrates memory, tools, and AI
pub struct AiAgentService {
    /// AI service for making LLM calls
    ai_service: Arc<AiService>,

    /// Tool registry for executing tool calls
    tool_registry: Arc<ToolRegistry>,

    /// Memory service for conversation context
    memory_service: Option<Arc<crate::services::memory_service::MemoryService>>,
}

impl AiAgentService {
    /// Create a new AI agent service
    ///
    /// # Arguments
    /// * `ai_service` - Service for making AI API calls
    /// * `tool_registry` - Registry of available tools
    pub fn new(ai_service: Arc<AiService>, tool_registry: Arc<ToolRegistry>) -> Self {
        info!(target: "ai_agent_service", "Creating AI agent service");
        Self {
            ai_service,
            tool_registry,
            memory_service: None,
        }
    }

    /// Create a new AI agent service with memory
    ///
    /// # Arguments
    /// * `ai_service` - Service for making AI API calls
    /// * `tool_registry` - Registry of available tools
    /// * `memory_service` - Memory service for conversation context
    pub fn new_with_memory(
        ai_service: Arc<AiService>,
        tool_registry: Arc<ToolRegistry>,
        memory_service: Arc<crate::services::memory_service::MemoryService>,
    ) -> Self {
        info!(target: "ai_agent_service", "Creating AI agent service with memory");
        Self {
            ai_service,
            tool_registry,
            memory_service: Some(memory_service),
        }
    }

    /// Main chat method that orchestrates the full agent flow
    ///
    /// # Arguments
    /// * `conversation_id` - Unique identifier for this conversation
    /// * `message` - User's message
    ///
    /// # Returns
    /// * `AgentResponse` containing the AI's response and metadata
    pub async fn chat(&self, conversation_id: &str, message: &str) -> AppResult<AgentResponse> {
        let start_time = Instant::now();
        let correlation_id = uuid::Uuid::new_v4().to_string();

        // Initialize performance metrics
        let mut perf_metrics = PerformanceMetrics {
            context_building_ms: 0,
            memory_retrieval_ms: 0,
            ai_api_ms: 0,
            tool_execution_ms: 0,
            memory_storage_ms: 0,
            tool_timings: Some(HashMap::new()),
        };

        info!(
            target: "ai_agent_service",
            conversation_id = conversation_id,
            correlation_id = %correlation_id,
            message_len = message.len(),
            "Starting agent chat"
        );

        // Build context from memory and tools
        let context_start = Instant::now();
        let context = match self.build_context(conversation_id, message).await {
            Ok(ctx) => ctx,
            Err(e) => {
                error!(
                    target: "ai_agent_service",
                    error = %e,
                    correlation_id = %correlation_id,
                    "Failed to build context, continuing with minimal context"
                );
                // Fallback to minimal context
                AgentContext {
                    conversation_id: conversation_id.to_string(),
                    available_tools: self.tool_registry.get_tool_schemas(),
                    system_prompt: "You are a helpful AI assistant.".to_string(),
                    history_messages: Vec::new(),
                }
            }
        };
        perf_metrics.context_building_ms = context_start.elapsed().as_millis();

        // Check memory service availability
        let memory_available = self.memory_service.is_some();
        let mut error_details = Vec::new();

        if !memory_available {
            warn!(
                target: "ai_agent_service",
                correlation_id = %correlation_id,
                "Memory service unavailable, operating in stateless mode"
            );

            let mut context_map = std::collections::HashMap::new();
            context_map.insert("fallback_mode".to_string(), "true".to_string());
            context_map.insert(
                "feature_impact".to_string(),
                "conversation_history_disabled".to_string(),
            );

            error_details.push(ErrorDetail {
                error_type: "memory_unavailable".to_string(),
                message: "记忆服务当前不可用。AI 将在无状态模式下运行，无法记住之前的对话。功能将受到限制，但您仍然可以进行对话和使用工具。".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                context: Some(context_map),
            });
        }

        // Prepare messages for AI with tool schemas
        let system_prompt = &context.system_prompt;
        let tool_schemas = &context.available_tools;
        let history_messages = &context.history_messages;

        // First AI call with tools
        let ai_start = Instant::now();
        let ai_response = self
            .call_ai_with_tools(message, system_prompt, tool_schemas, history_messages)
            .await?;
        perf_metrics.ai_api_ms += ai_start.elapsed().as_millis();

        let mut final_message = ai_response.message.clone();
        let mut tool_calls_executed = Vec::new();
        let mut tools_used = Vec::new();

        // Check if AI wants to use tools
        if !ai_response.tool_calls.is_empty() {
            debug!(
                target: "ai_agent_service",
                tool_count = ai_response.tool_calls.len(),
                correlation_id = %correlation_id,
                "AI requested tool calls"
            );

            // Execute tool calls with error handling
            let tool_start = Instant::now();
            let tool_results = self
                .execute_tool_calls_with_retry(ai_response.tool_calls.clone(), &correlation_id)
                .await;
            perf_metrics.tool_execution_ms = tool_start.elapsed().as_millis();

            // Track which tools were used and collect errors
            for (tool_call, result) in ai_response.tool_calls.iter().zip(tool_results.iter()) {
                tools_used.push(tool_call.name.clone());
                if let Some(ref error) = result.error {
                    let mut context_map = HashMap::new();
                    context_map.insert("tool_name".to_string(), tool_call.name.clone());
                    context_map.insert("tool_call_id".to_string(), tool_call.id.clone());
                    context_map.insert("arguments".to_string(), tool_call.arguments.to_string());

                    error!(
                        target: "ai_agent_service",
                        tool_name = %tool_call.name,
                        tool_call_id = %tool_call.id,
                        correlation_id = %correlation_id,
                        error = %error,
                        "Tool execution failed"
                    );

                    error_details.push(ErrorDetail {
                        error_type: "tool_execution".to_string(),
                        message: format!("工具 '{}' 执行失败: {}", tool_call.name, error),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        context: Some(context_map),
                    });
                }
            }

            tool_calls_executed = ai_response.tool_calls.clone();

            // Send tool results back to AI for final response
            let ai_start2 = Instant::now();
            final_message = self
                .call_ai_with_tool_results(message, system_prompt, &tool_results)
                .await?;
            perf_metrics.ai_api_ms += ai_start2.elapsed().as_millis();
        }

        // Store conversation in memory (with error handling)
        let storage_start = Instant::now();
        let memory_stored = if memory_available {
            match self
                .store_conversation(
                    conversation_id,
                    message,
                    &final_message,
                    AgentMetadata {
                        tokens_used: HashMap::new(),
                        latency_ms: start_time.elapsed().as_millis(),
                        memory_entries_used: 0, // No memory context available
                        tools_executed: tools_used.clone(),
                        correlation_id: Some(correlation_id.clone()),
                        errors: if error_details.is_empty() {
                            None
                        } else {
                            Some(error_details.clone())
                        },
                        memory_available: Some(memory_available),
                        performance: Some(perf_metrics.clone()),
                    },
                )
                .await
            {
                Ok(_) => true,
                Err(e) => {
                    error_details.push(ErrorDetail {
                        error_type: "memory_storage".to_string(),
                        message: format!("Failed to store conversation: {}", e),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        context: None,
                    });
                    false
                }
            }
        } else {
            false
        };
        perf_metrics.memory_storage_ms = storage_start.elapsed().as_millis();

        let total_latency = start_time.elapsed().as_millis();

        info!(
            target: "ai_agent_service",
            conversation_id = conversation_id,
            correlation_id = %correlation_id,
            latency_ms = total_latency,
            context_building_ms = perf_metrics.context_building_ms,
            memory_retrieval_ms = perf_metrics.memory_retrieval_ms,
            ai_api_ms = perf_metrics.ai_api_ms,
            tool_execution_ms = perf_metrics.tool_execution_ms,
            memory_storage_ms = perf_metrics.memory_storage_ms,
            tools_used = tools_used.len(),
            errors = error_details.len(),
            memory_stored = memory_stored,
            memory_available = memory_available,
            "Agent chat completed"
        );

        Ok(AgentResponse {
            message: final_message,
            tool_calls: tool_calls_executed,
            memory_stored,
            metadata: AgentMetadata {
                tokens_used: HashMap::new(),
                latency_ms: total_latency,
                memory_entries_used: 0, // No memory context available
                tools_executed: tools_used,
                correlation_id: Some(correlation_id),
                errors: if error_details.is_empty() {
                    None
                } else {
                    Some(error_details)
                },
                memory_available: Some(memory_available),
                performance: Some(perf_metrics),
            },
        })
    }

    /// Build context for the AI from memory and tool schemas
    ///
    /// # Arguments
    /// * `conversation_id` - Conversation identifier
    /// * `message` - User's current message
    ///
    /// # Returns
    /// * `AgentContext` containing memory context, tool schemas, and system prompt
    async fn build_context(&self, conversation_id: &str, message: &str) -> AppResult<AgentContext> {
        let start_time = std::time::Instant::now();

        debug!(
            target: "ai_agent_service",
            conversation_id = conversation_id,
            "Building agent context"
        );

        // Load tool schemas
        let tool_schemas = self.tool_registry.get_tool_schemas();

        // Get memory context if available
        let memory_context = if let Some(ref memory_service) = self.memory_service {
            match memory_service.get_conversation_context(message, 2000).await {
                Ok(context) => {
                    if context.is_empty() {
                        None
                    } else {
                        Some(context)
                    }
                }
                Err(e) => {
                    warn!(
                        target: "ai_agent_service",
                        error = %e,
                        "Failed to retrieve memory context"
                    );
                    None
                }
            }
        } else {
            None
        };

        // Reconstruct recent conversation messages from this conversation_id
        let mut history_messages: Vec<ChatMessage> = Vec::new();
        if let Some(ref memory_service) = self.memory_service {
            if let Ok(mut docs) = memory_service
                .search_by_conversation_id(conversation_id)
                .await
            {
                // Sort by created_at ascending, then take the last few exchanges to control size
                docs.sort_by(|a, b| a.created_at.cmp(&b.created_at));

                // Limit to last 6 documents (≈ 6 exchanges)
                let take_n = 6_usize.min(docs.len());
                for doc in docs.iter().rev().take(take_n).rev() {
                    if let Some((user_msg, ai_msg)) =
                        Self::extract_messages_from_content(&doc.content)
                    {
                        history_messages.push(ChatMessage {
                            role: "user".to_string(),
                            content: user_msg,
                        });
                        history_messages.push(ChatMessage {
                            role: "assistant".to_string(),
                            content: ai_msg,
                        });
                    }
                }
            }
        }

        // Build system prompt with tool instructions and memory context
        let current_date = chrono::Local::now().format("%Y-%m-%d").to_string();
        let current_time = chrono::Local::now().format("%H:%M:%S").to_string();
        let current_datetime = chrono::Local::now().to_rfc3339();

        let mut system_prompt = format!(
            r#"You are CogniCal, an intelligent AI assistant specialized in unified time management and productivity.

## Current Information
- **Current Date**: {}
- **Current Time**: {}
- **Current DateTime**: {}

## Your Capabilities
You have access to powerful tools for unified time management that combines tasks and calendar events as one cohesive system. You should:

1. **Unified Time Management**: Tasks and calendar events are now unified - they're all "time-based items"
2. **Be Proactive**: When users ask about schedule, calendar, tasks, or time management, immediately use the appropriate tools
3. **Understand Context and User Intent**: 
   - Pay attention to conversation flow and user intent
   - If user initially asks to "create/安排/schedule" something and then provides details, they want to CREATE not SEARCH
   - "recent", "latest", "upcoming" means future dates from today
   - "past", "previous", "last week" means dates before today
   - "today" means the current date: {}

## CRITICAL: Tool Selection Rules
**For Creating New Items:**
- User says "创建/安排/schedule/建个/做个 + 时间 + 事情" → use `create_time_block`
- User provides complete info after you ask for details → CREATE, not search
- User says "快速安排 X 在 Y 时间" → use `quick_schedule`
- If user gives you title + time, they want to CREATE

**For Searching/Viewing Existing Items:**
- User says "查找/搜索/找/看看 + 已有/现有的 + 时间安排" → use `search_time_items`
- User says "查看/显示/列出 + 时间安排" → use `list_time_items`
- User says "检查/确认 + 是否已安排" → use `search_time_items`

**For Updating Items:**
- User says "修改/更新/调整/重安排 + 已有的 + 时间" → use `update_time_item`

## Tool Usage Rules
- ALWAYS call tools when users ask for data (don't ask "what date is it")
- Call tools FIRST, then format the results nicely for the user
- If multiple tools are needed, call them in parallel when possible
- Don't ask users for information you can infer from context
- **Remember**: All time-based management is now unified - tasks and calendar events are the same thing!
- **CRITICAL**: If user previously asked to create something and now provides the missing details, use CREATE tools!

## Response Style
- Be concise and actionable
- Show results in clear, formatted lists
- Use emojis appropriately for better UX (📅 for schedules, ⏰ for deadlines, 🕒 for time blocks)
- Remember user preferences from conversation history"#,
            current_date, current_time, current_datetime, current_date
        );

        if let Some(ref context) = memory_context {
            system_prompt.push_str("\n\n## Conversation History & Context\n");
            system_prompt.push_str(context);
            system_prompt.push_str("\n\nUse this history to provide personalized and context-aware responses. Remember user preferences and past interactions.");
        }

        if !history_messages.is_empty() {
            system_prompt
                .push_str("\n\n## Recent Conversation (historical messages provided separately)\n");
            system_prompt.push_str("Use the provided chat history messages to continue the dialogue naturally. Do not ask again for details the user already provided in earlier turns.\n");
        }

        let elapsed = start_time.elapsed();
        debug!(
            target: "ai_agent_service",
            elapsed_ms = elapsed.as_millis(),
            memory_available = memory_context.is_some(),
            "Context building completed"
        );

        Ok(AgentContext {
            conversation_id: conversation_id.to_string(),
            available_tools: tool_schemas,
            system_prompt,
            history_messages,
        })
    }

    /// Call AI with tool schemas
    async fn call_ai_with_tools(
        &self,
        message: &str,
        system_prompt: &str,
        tool_schemas: &[JsonValue],
        history_messages: &[ChatMessage],
    ) -> AppResult<AiResponse> {
        use reqwest::Client;
        use serde_json::json;

        // Get API key from settings
        let api_key = self.ai_service.get_api_key()?;

        // Build messages array with history
        let mut messages = vec![json!({"role": "system", "content": system_prompt})];
        for m in history_messages {
            messages.push(json!({"role": m.role, "content": m.content}));
        }
        messages.push(json!({"role": "user", "content": message}));

        // Build request body with tools
        let mut request_body = json!({
            "model": "deepseek-chat",
            "messages": messages,
            "temperature": 0.7,
        });

        // Add tools if available
        if !tool_schemas.is_empty() {
            request_body["tools"] = json!(tool_schemas);
            request_body["tool_choice"] = json!("auto");
        }

        debug!(
            target: "ai_agent_service",
            tool_count = tool_schemas.len(),
            "Calling DeepSeek API with tools"
        );

        // Call DeepSeek API
        let client = Client::new();
        let ai_timeout = tokio::time::Duration::from_secs(30);

        let response = tokio::time::timeout(
            ai_timeout,
            client
                .post("https://api.deepseek.com/v1/chat/completions")
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(&request_body)
                .send(),
        )
        .await
        .map_err(|_| {
            error!(target: "ai_agent_service", "AI call timed out after 30 seconds");
            AppError::ai(
                crate::error::AiErrorCode::HttpTimeout,
                "AI 响应超时。请稍后重试。",
            )
        })?
        .map_err(|e| {
            error!(target: "ai_agent_service", error = %e, "HTTP request failed");
            AppError::ai(
                crate::error::AiErrorCode::DeepseekUnavailable,
                format!("无法连接到 AI 服务: {}", e),
            )
        })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error response: {}", e));
            error!(
                target: "ai_agent_service",
                status = %status,
                error = %error_text,
                "DeepSeek API error"
            );
            return Err(AppError::ai(
                crate::error::AiErrorCode::InvalidResponse,
                format!("AI API 错误: {}", error_text),
            ));
        }

        let response_json: JsonValue = response.json().await.map_err(|e| {
            error!(target: "ai_agent_service", error = %e, "Failed to parse AI response");
            AppError::ai(
                crate::error::AiErrorCode::InvalidResponse,
                "无法解析 AI 响应",
            )
        })?;

        // Extract message and tool calls
        let choice = &response_json["choices"][0];
        let message_obj = &choice["message"];

        let content = message_obj["content"].as_str().unwrap_or("").to_string();

        let mut tool_calls = Vec::new();

        // Parse tool calls if present
        if let Some(tool_calls_array) = message_obj["tool_calls"].as_array() {
            for tool_call in tool_calls_array {
                if let (Some(id), Some(function)) =
                    (tool_call["id"].as_str(), tool_call["function"].as_object())
                {
                    if let (Some(name), Some(arguments_str)) = (
                        function.get("name").and_then(|v| v.as_str()),
                        function.get("arguments").and_then(|v| v.as_str()),
                    ) {
                        // Parse arguments JSON string
                        let arguments: JsonValue =
                            serde_json::from_str(arguments_str).unwrap_or_else(|_| json!({}));

                        tool_calls.push(ToolCall {
                            id: id.to_string(),
                            name: name.to_string(),
                            arguments,
                        });

                        debug!(
                            target: "ai_agent_service",
                            tool_id = %id,
                            tool_name = %name,
                            "Parsed tool call from AI response"
                        );
                    }
                }
            }
        }

        Ok(AiResponse {
            message: content,
            tool_calls,
        })
    }

    /// Call AI with tool results to get final response
    async fn call_ai_with_tool_results(
        &self,
        original_message: &str,
        system_prompt: &str,
        tool_results: &[ToolResult],
    ) -> AppResult<String> {
        // Format tool results for AI
        let mut results_text = String::from("工具执行结果：\n\n");
        for result in tool_results {
            if let Some(ref result_data) = result.result {
                results_text.push_str(&format!("✓ 成功: {}\n", result_data));
            } else if let Some(ref error) = result.error {
                results_text.push_str(&format!("✗ 错误: {}\n", error));
            }
        }

        let full_prompt = format!(
            "{}\n\n原始用户消息: {}\n\n{}\n\n请根据工具执行结果，给用户一个友好的回复。",
            system_prompt, original_message, results_text
        );

        // Call AI with timeout protection (30 seconds)
        let ai_timeout = tokio::time::Duration::from_secs(30);
        tokio::time::timeout(ai_timeout, self.ai_service.chat(full_prompt))
            .await
            .map_err(|_| {
                error!(target: "ai_agent_service", "AI call with tool results timed out after 30 seconds");
                AppError::ai(
                    crate::error::AiErrorCode::HttpTimeout,
                    "AI 响应超时。请稍后重试。"
                )
            })?
    }

    /// Execute tool calls with retry logic for failed executions
    async fn execute_tool_calls_with_retry(
        &self,
        tool_calls: Vec<ToolCall>,
        correlation_id: &str,
    ) -> Vec<ToolResult> {
        info!(
            target: "ai_agent_service",
            tool_count = tool_calls.len(),
            correlation_id = %correlation_id,
            "Executing tool calls with retry"
        );

        // First attempt
        let mut results = self.tool_registry.execute_tools(tool_calls.clone()).await;

        // Identify failed tool calls
        let mut failed_indices = Vec::new();
        for (idx, result) in results.iter().enumerate() {
            if result.error.is_some() {
                failed_indices.push(idx);
            }
        }

        // Retry failed tool calls once
        if !failed_indices.is_empty() {
            warn!(
                target: "ai_agent_service",
                failed_count = failed_indices.len(),
                correlation_id = %correlation_id,
                "Retrying failed tool calls"
            );

            // Wait a bit before retrying
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

            // Collect failed tool calls for retry
            let retry_calls: Vec<ToolCall> = failed_indices
                .iter()
                .map(|&idx| tool_calls[idx].clone())
                .collect();

            // Execute retry
            let retry_results = self.tool_registry.execute_tools(retry_calls).await;

            // Update results with retry outcomes
            for (failed_idx_pos, &original_idx) in failed_indices.iter().enumerate() {
                let retry_result = &retry_results[failed_idx_pos];
                if retry_result.error.is_none() {
                    info!(
                        target: "ai_agent_service",
                        tool_call_id = %retry_result.tool_call_id,
                        correlation_id = %correlation_id,
                        "Tool retry succeeded"
                    );
                    results[original_idx] = retry_result.clone();
                } else {
                    error!(
                        target: "ai_agent_service",
                        tool_call_id = %retry_result.tool_call_id,
                        correlation_id = %correlation_id,
                        error = ?retry_result.error,
                        "Tool retry failed"
                    );
                }
            }
        }

        // Log final results
        let success_count = results.iter().filter(|r| r.error.is_none()).count();
        let failure_count = results.len() - success_count;

        info!(
            target: "ai_agent_service",
            total = results.len(),
            success = success_count,
            failures = failure_count,
            correlation_id = %correlation_id,
            "Tool execution completed"
        );

        results
    }

    /// Store conversation in memory
    async fn store_conversation(
        &self,
        conversation_id: &str,
        user_message: &str,
        assistant_message: &str,
        _metadata: AgentMetadata,
    ) -> AppResult<()> {
        debug!(
            target: "ai_agent_service",
            conversation_id = conversation_id,
            "Storing conversation in memory"
        );

        if let Some(ref memory_service) = self.memory_service {
            // Extract topics from the conversation
            let topics = self.extract_conversation_topics(user_message, assistant_message);

            match memory_service
                .store_conversation(conversation_id, user_message, assistant_message, topics)
                .await
            {
                Ok(doc_id) => {
                    debug!(
                        target: "ai_agent_service",
                        conversation_id = conversation_id,
                        doc_id = %doc_id,
                        "Conversation stored successfully"
                    );
                    Ok(())
                }
                Err(e) => {
                    warn!(
                        target: "ai_agent_service",
                        error = %e,
                        conversation_id = conversation_id,
                        "Failed to store conversation"
                    );
                    Err(e)
                }
            }
        } else {
            debug!(
                target: "ai_agent_service",
                conversation_id = conversation_id,
                "Memory service not available, skipping conversation storage"
            );
            Ok(())
        }
    }

    /// Extract topics from conversation content
    fn extract_conversation_topics(
        &self,
        user_message: &str,
        assistant_message: &str,
    ) -> Vec<String> {
        let mut topics = Vec::new();
        let content = format!("{} {}", user_message, assistant_message).to_lowercase();

        // Simple keyword-based topic extraction
        let topic_keywords = [
            (
                "task management",
                vec!["task", "todo", "schedule", "deadline", "完成", "任务"],
            ),
            (
                "recurring tasks",
                vec![
                    "recurring",
                    "repeat",
                    "daily",
                    "weekly",
                    "monthly",
                    "重复",
                    "定期",
                ],
            ),
            (
                "project planning",
                vec![
                    "project",
                    "plan",
                    "milestone",
                    "goal",
                    "项目",
                    "计划",
                    "目标",
                ],
            ),
            (
                "dependencies",
                vec![
                    "dependency",
                    "depends",
                    "prerequisite",
                    "blocker",
                    "依赖",
                    "前置",
                ],
            ),
            (
                "ai assistance",
                vec!["ai", "assistant", "help", "suggest", "助手", "建议"],
            ),
            (
                "productivity",
                vec!["productivity", "efficiency", "workflow", "效率", "工作流"],
            ),
            (
                "calendar",
                vec![
                    "calendar",
                    "event",
                    "meeting",
                    "appointment",
                    "日历",
                    "会议",
                    "约会",
                ],
            ),
        ];

        for (topic, keywords) in &topic_keywords {
            if keywords.iter().any(|keyword| content.contains(keyword)) {
                topics.push(topic.to_string());
            }
        }

        if topics.is_empty() {
            topics.push("general".to_string());
        }

        topics
    }
}

/// Internal structure for AI responses
#[derive(Debug)]
struct AiResponse {
    message: String,
    tool_calls: Vec<ToolCall>,
}

/// Lightweight chat message used to pass prior conversation to the LLM
#[derive(Debug, Clone, Serialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl AiAgentService {
    /// Extract user/assistant messages from our stored markdown content
    /// Expected format (created by MemoryService::create_document_content):
    /// --- YAML frontmatter ---
    /// # Conversation Summary: ...
    /// ## User Message
    /// <user>
    ///
    /// ## AI Response
    /// <assistant>
    ///
    /// ## Topics
    fn extract_messages_from_content(content: &str) -> Option<(String, String)> {
        let user_tag = "## User Message";
        let ai_tag = "## AI Response";
        let topics_tag = "## Topics";

        let user_pos = content.find(user_tag)?;
        let ai_pos = content.find(ai_tag)?;

        // user message between end of user_tag line and start of ai_tag
        let user_section_start = content[user_pos..].find('\n').map(|o| user_pos + o + 1)?;
        let user_section = &content[user_section_start..ai_pos];

        // assistant between end of ai_tag line and next section (topics or end)
        let ai_section_start = content[ai_pos..].find('\n').map(|o| ai_pos + o + 1)?;
        let ai_end = content[ai_section_start..]
            .find(topics_tag)
            .map(|o| ai_section_start + o)
            .unwrap_or_else(|| content.len());
        let ai_section = &content[ai_section_start..ai_end];

        let user_trimmed = user_section.trim().to_string();
        let ai_trimmed = ai_section.trim().to_string();
        if user_trimmed.is_empty() || ai_trimmed.is_empty() {
            None
        } else {
            Some((user_trimmed, ai_trimmed))
        }
    }
}
