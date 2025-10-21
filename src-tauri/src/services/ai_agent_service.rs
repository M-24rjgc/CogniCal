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
}

impl AiAgentService {
    /// Create a new AI agent service
    ///
    /// # Arguments
    /// * `ai_service` - Service for making AI API calls
    /// * `tool_registry` - Registry of available tools
    pub fn new(
        ai_service: Arc<AiService>,
        tool_registry: Arc<ToolRegistry>,
    ) -> Self {
        info!(target: "ai_agent_service", "Creating AI agent service");
        Self {
            ai_service,
            tool_registry,
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
    pub async fn chat(
        &self,
        conversation_id: &str,
        message: &str,
    ) -> AppResult<AgentResponse> {
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
                }
            }
        };
        perf_metrics.context_building_ms = context_start.elapsed().as_millis();

        // Memory service is no longer available
        let memory_available = false;
        let mut error_details = Vec::new();
        
        if !memory_available {
            warn!(
                target: "ai_agent_service",
                correlation_id = %correlation_id,
                "Memory service unavailable, operating in stateless mode"
            );
            
            let mut context_map = std::collections::HashMap::new();
            context_map.insert("fallback_mode".to_string(), "true".to_string());
            context_map.insert("feature_impact".to_string(), "conversation_history_disabled".to_string());
            
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

        // First AI call with tools
        let ai_start = Instant::now();
        let ai_response = self
            .call_ai_with_tools(message, system_prompt, tool_schemas)
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
            let tool_results = self.execute_tool_calls_with_retry(
                ai_response.tool_calls.clone(),
                &correlation_id
            ).await;
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
            match self.store_conversation(
                conversation_id,
                message,
                &final_message,
                AgentMetadata {
                    tokens_used: HashMap::new(),
                    latency_ms: start_time.elapsed().as_millis(),
                    memory_entries_used: 0, // No memory context available
                    tools_executed: tools_used.clone(),
                    correlation_id: Some(correlation_id.clone()),
                    errors: if error_details.is_empty() { None } else { Some(error_details.clone()) },
                    memory_available: Some(memory_available),
                    performance: Some(perf_metrics.clone()),
                },
            )
            .await {
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
                errors: if error_details.is_empty() { None } else { Some(error_details) },
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
    async fn build_context(
        &self,
        conversation_id: &str,
        _message: &str,
    ) -> AppResult<AgentContext> {
        let start_time = std::time::Instant::now();
        
        debug!(
            target: "ai_agent_service",
            conversation_id = conversation_id,
            "Building agent context"
        );

        // Load tool schemas (memory service no longer available)
        let tool_schemas = self.tool_registry.get_tool_schemas();

        // No memory context available
        let _memory_context: Option<String> = None;

        // Build system prompt with tool instructions (no memory)
        let system_prompt = "You are a helpful AI assistant.".to_string();

        let elapsed = start_time.elapsed();
        debug!(
            target: "ai_agent_service",
            elapsed_ms = elapsed.as_millis(),
            "Context building completed"
        );

        Ok(AgentContext {
            conversation_id: conversation_id.to_string(),
            available_tools: tool_schemas,
            system_prompt,
        })
    }



    /// Call AI with tool schemas
    async fn call_ai_with_tools(
        &self,
        message: &str,
        system_prompt: &str,
        tool_schemas: &[JsonValue],
    ) -> AppResult<AiResponse> {
        use reqwest::Client;
        use serde_json::json;

        // Get API key from settings
        let api_key = self.ai_service.get_api_key()?;
        
        // Build messages array
        let messages = json!([
            {
                "role": "system",
                "content": system_prompt
            },
            {
                "role": "user",
                "content": message
            }
        ]);

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
                .send()
        )
        .await
        .map_err(|_| {
            error!(target: "ai_agent_service", "AI call timed out after 30 seconds");
            AppError::ai(
                crate::error::AiErrorCode::HttpTimeout,
                "AI 响应超时。请稍后重试。"
            )
        })?
        .map_err(|e| {
            error!(target: "ai_agent_service", error = %e, "HTTP request failed");
            AppError::ai(
                crate::error::AiErrorCode::DeepseekUnavailable,
                format!("无法连接到 AI 服务: {}", e)
            )
        })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await
                .unwrap_or_else(|e| format!("Failed to read error response: {}", e));
            error!(
                target: "ai_agent_service",
                status = %status,
                error = %error_text,
                "DeepSeek API error"
            );
            return Err(AppError::ai(
                crate::error::AiErrorCode::InvalidResponse,
                format!("AI API 错误: {}", error_text)
            ));
        }

        let response_json: JsonValue = response.json().await.map_err(|e| {
            error!(target: "ai_agent_service", error = %e, "Failed to parse AI response");
            AppError::ai(
                crate::error::AiErrorCode::InvalidResponse,
                "无法解析 AI 响应"
            )
        })?;

        // Extract message and tool calls
        let choice = &response_json["choices"][0];
        let message_obj = &choice["message"];
        
        let content = message_obj["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let mut tool_calls = Vec::new();
        
        // Parse tool calls if present
        if let Some(tool_calls_array) = message_obj["tool_calls"].as_array() {
            for tool_call in tool_calls_array {
                if let (Some(id), Some(function)) = (
                    tool_call["id"].as_str(),
                    tool_call["function"].as_object()
                ) {
                    if let (Some(name), Some(arguments_str)) = (
                        function.get("name").and_then(|v| v.as_str()),
                        function.get("arguments").and_then(|v| v.as_str())
                    ) {
                        // Parse arguments JSON string
                        let arguments: JsonValue = serde_json::from_str(arguments_str)
                            .unwrap_or_else(|_| json!({}));
                        
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
        _user_message: &str,
        _assistant_message: &str,
        metadata: AgentMetadata,
    ) -> AppResult<()> {
        debug!(
            target: "ai_agent_service",
            conversation_id = conversation_id,
            "Storing conversation in memory"
        );

        // Build metadata map
        let mut meta_map = HashMap::new();
        meta_map.insert("latency_ms".to_string(), metadata.latency_ms.to_string());
        meta_map.insert(
            "memory_entries_used".to_string(),
            metadata.memory_entries_used.to_string(),
        );
        if !metadata.tools_executed.is_empty() {
            meta_map.insert(
                "tools_used".to_string(),
                metadata.tools_executed.join(","),
            );
        }

        // Memory service no longer available - skip storage
        debug!(
            target: "ai_agent_service",
            conversation_id = conversation_id,
            "Memory service not available, skipping conversation storage"
        );
        Ok(())
    }
}

/// Internal structure for AI responses
#[derive(Debug)]
struct AiResponse {
    message: String,
    tool_calls: Vec<ToolCall>,
}
