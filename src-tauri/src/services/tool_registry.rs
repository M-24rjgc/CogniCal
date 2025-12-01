use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::time::{timeout, Duration};
use tracing::{debug, error, info, warn};

/// Type alias for tool handler functions
/// Handlers are async functions that take JSON parameters and return JSON results
pub type ToolHandler = Arc<
    dyn Fn(JsonValue) -> Pin<Box<dyn Future<Output = AppResult<JsonValue>> + Send>> + Send + Sync,
>;

/// Definition of a tool that can be called by the AI
#[derive(Clone)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    /// JSON Schema for parameters (OpenAI function calling format)
    pub parameters: JsonValue,
    pub handler: ToolHandler,
}

/// A tool call request from the AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: JsonValue,
}

/// Result of executing a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_call_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Registry for managing available tools
pub struct ToolRegistry {
    tools: HashMap<String, ToolDefinition>,
    timeout_duration: Duration,
}

impl ToolRegistry {
    /// Create a new tool registry with default timeout (15 seconds)
    /// This is more reasonable for complex AI tool calls that may involve
    /// database operations, API calls, or file I/O
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            timeout_duration: Duration::from_secs(15),
        }
    }

    /// Create a new tool registry with aggressive timeout for quick operations
    pub fn with_fast_timeout() -> Self {
        Self {
            tools: HashMap::new(),
            timeout_duration: Duration::from_secs(3), // Fast operations like validation, simple queries
        }
    }

    /// Create a new tool registry with slow timeout for intensive operations
    pub fn with_slow_timeout() -> Self {
        Self {
            tools: HashMap::new(),
            timeout_duration: Duration::from_secs(30), // Complex operations, large data processing
        }
    }

    /// Create a new tool registry with custom timeout
    pub fn with_timeout(timeout_ms: u64) -> Self {
        Self {
            tools: HashMap::new(),
            timeout_duration: Duration::from_millis(timeout_ms),
        }
    }

    /// Register a new tool with the registry
    ///
    /// # Arguments
    /// * `name` - Unique name for the tool
    /// * `description` - Human-readable description of what the tool does
    /// * `parameters` - JSON Schema defining the tool's parameters
    /// * `handler` - Async function that executes the tool
    ///
    /// # Returns
    /// * `Ok(())` if registration succeeds
    /// * `Err(AppError)` if the tool name is already registered or schema is invalid
    pub fn register_tool(
        &mut self,
        name: String,
        description: String,
        parameters: JsonValue,
        handler: ToolHandler,
    ) -> AppResult<()> {
        // Check if tool already exists
        if self.tools.contains_key(&name) {
            return Err(AppError::validation(format!(
                "Tool '{}' is already registered",
                name
            )));
        }

        // Validate that parameters is a valid JSON Schema object
        if !parameters.is_object() {
            return Err(AppError::validation(
                "Tool parameters must be a JSON object (JSON Schema)",
            ));
        }

        let tool_def = ToolDefinition {
            name: name.clone(),
            description,
            parameters,
            handler,
        };

        self.tools.insert(name.clone(), tool_def);
        info!(target: "tool_registry", tool_name = %name, "Tool registered successfully");

        Ok(())
    }

    /// Get all tool schemas in OpenAI function calling format
    ///
    /// Returns a vector of tool definitions formatted for AI consumption
    pub fn get_tool_schemas(&self) -> Vec<JsonValue> {
        self.tools
            .values()
            .map(|tool| {
                serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": tool.name,
                        "description": tool.description,
                        "parameters": tool.parameters,
                    }
                })
            })
            .collect()
    }

    /// Check if a tool with the given name exists
    pub fn has_tool(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    /// Get the number of registered tools
    pub fn tool_count(&self) -> usize {
        self.tools.len()
    }

    /// Get a list of all registered tool names
    pub fn tool_names(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }

    /// Validate a tool call against its schema
    ///
    /// # Arguments
    /// * `tool_call` - The tool call to validate
    ///
    /// # Returns
    /// * `Ok(())` if validation succeeds
    /// * `Err(AppError)` if the tool doesn't exist or parameters are invalid
    pub fn validate_tool_call(&self, tool_call: &ToolCall) -> AppResult<()> {
        // Check if tool exists
        let tool = self.tools.get(&tool_call.name).ok_or_else(|| {
            AppError::validation(format!("Tool '{}' not found in registry", tool_call.name))
        })?;

        // Compile the JSON Schema
        let schema = match jsonschema::JSONSchema::compile(&tool.parameters) {
            Ok(schema) => schema,
            Err(e) => {
                error!(
                    target: "tool_registry",
                    tool_name = %tool_call.name,
                    error = %e,
                    "Failed to compile tool schema"
                );
                return Err(AppError::validation(format!(
                    "Invalid schema for tool '{}': {}",
                    tool_call.name, e
                )));
            }
        };

        // Validate the arguments against the schema
        if let Err(validation_errors) = schema.validate(&tool_call.arguments) {
            let error_messages: Vec<String> = validation_errors
                .map(|e| {
                    let path = e.instance_path.to_string();
                    let path_display = if path.is_empty() {
                        "root".to_string()
                    } else {
                        path
                    };
                    format!("  - {}: {}", path_display, e)
                })
                .collect();

            let detailed_message = format!(
                "Parameter validation failed for tool '{}':\n{}",
                tool_call.name,
                error_messages.join("\n")
            );

            warn!(
                target: "tool_registry",
                tool_name = %tool_call.name,
                tool_call_id = %tool_call.id,
                "Tool call validation failed"
            );

            return Err(AppError::validation_with_details(
                detailed_message,
                serde_json::json!({
                    "tool_name": tool_call.name,
                    "tool_call_id": tool_call.id,
                    "errors": error_messages,
                }),
            ));
        }

        debug!(
            target: "tool_registry",
            tool_name = %tool_call.name,
            tool_call_id = %tool_call.id,
            "Tool call validated successfully"
        );

        Ok(())
    }

    /// Execute a tool call with timeout protection
    ///
    /// # Arguments
    /// * `tool_call` - The tool call to execute
    ///
    /// # Returns
    /// * `ToolResult` containing either the result or an error message
    pub async fn execute_tool(&self, tool_call: ToolCall) -> ToolResult {
        let tool_call_id = tool_call.id.clone();
        let tool_name = tool_call.name.clone();
        let correlation_id = uuid::Uuid::new_v4().to_string();

        info!(
            target: "tool_registry",
            tool_name = %tool_name,
            tool_call_id = %tool_call_id,
            correlation_id = %correlation_id,
            "Executing tool call"
        );

        // Validate the tool call first
        if let Err(e) = self.validate_tool_call(&tool_call) {
            error!(
                target: "tool_registry",
                tool_name = %tool_name,
                tool_call_id = %tool_call_id,
                correlation_id = %correlation_id,
                error = %e,
                "Tool call validation failed"
            );

            // Format user-friendly error message for AI
            let user_friendly_error = format!(
                "无法执行工具 '{}': 参数验证失败。{}",
                tool_name,
                self.format_validation_error(&e)
            );

            return ToolResult {
                tool_call_id,
                result: None,
                error: Some(user_friendly_error),
            };
        }

        // Get the tool handler
        let tool = match self.tools.get(&tool_name) {
            Some(tool) => tool,
            None => {
                // This shouldn't happen after validation, but handle it anyway
                let error_msg = format!("工具 '{}' 未找到", tool_name);
                error!(
                    target: "tool_registry",
                    tool_name = %tool_name,
                    tool_call_id = %tool_call_id,
                    correlation_id = %correlation_id,
                    "Tool not found"
                );
                return ToolResult {
                    tool_call_id,
                    result: None,
                    error: Some(error_msg),
                };
            }
        };

        // Execute the tool with timeout protection
        let handler = tool.handler.clone();
        let arguments = tool_call.arguments.clone();

        match timeout(self.timeout_duration, handler(arguments)).await {
            Ok(Ok(result)) => {
                info!(
                    target: "tool_registry",
                    tool_name = %tool_name,
                    tool_call_id = %tool_call_id,
                    correlation_id = %correlation_id,
                    "Tool executed successfully"
                );
                ToolResult {
                    tool_call_id,
                    result: Some(result),
                    error: None,
                }
            }
            Ok(Err(e)) => {
                error!(
                    target: "tool_registry",
                    tool_name = %tool_name,
                    tool_call_id = %tool_call_id,
                    correlation_id = %correlation_id,
                    error = %e,
                    "Tool execution failed"
                );

                // Format user-friendly error message for AI
                let user_friendly_error = format!(
                    "工具 '{}' 执行失败: {}",
                    tool_name,
                    self.format_execution_error(&e)
                );

                ToolResult {
                    tool_call_id,
                    result: None,
                    error: Some(user_friendly_error),
                }
            }
            Err(_) => {
                error!(
                    target: "tool_registry",
                    tool_name = %tool_name,
                    tool_call_id = %tool_call_id,
                    correlation_id = %correlation_id,
                    timeout_ms = ?self.timeout_duration.as_millis(),
                    "Tool execution timed out"
                );

                let timeout_error = format!(
                    "工具 '{}' 执行超时（超过 {}ms）。请稍后重试或简化请求。",
                    tool_name,
                    self.timeout_duration.as_millis()
                );

                ToolResult {
                    tool_call_id,
                    result: None,
                    error: Some(timeout_error),
                }
            }
        }
    }

    /// Format validation error for user-friendly display
    fn format_validation_error(&self, error: &AppError) -> String {
        match error {
            AppError::Validation {
                message, details, ..
            } => {
                // Extract key information from validation message
                let base_message = if message.contains("required") {
                    "缺少必需的参数"
                } else if message.contains("type") {
                    "参数类型不正确"
                } else if message.contains("format") {
                    "参数格式不正确"
                } else if message.contains("enum") {
                    "参数值不在允许的范围内"
                } else {
                    message.as_str()
                };

                // Add details if available
                if let Some(details_json) = details {
                    if let Some(errors) = details_json.get("errors").and_then(|e| e.as_array()) {
                        let error_list: Vec<String> = errors
                            .iter()
                            .filter_map(|e| e.as_str())
                            .take(3) // Limit to first 3 errors
                            .map(|s| s.to_string())
                            .collect();

                        if !error_list.is_empty() {
                            return format!(
                                "{}。详细信息: {}",
                                base_message,
                                error_list.join("; ")
                            );
                        }
                    }
                }

                base_message.to_string()
            }
            _ => error.to_string(),
        }
    }

    /// Format execution error for user-friendly display
    fn format_execution_error(&self, error: &AppError) -> String {
        match error {
            AppError::NotFound => "请求的资源未找到。请检查 ID 是否正确。".to_string(),
            AppError::Conflict { message } => {
                format!("操作冲突: {}。请检查是否存在重复或冲突的数据。", message)
            }
            AppError::Database { message } => {
                // Don't expose internal database errors to AI
                warn!(target: "tool_registry", database_error = %message, "Database error in tool execution");
                "数据库操作失败。请稍后重试。".to_string()
            }
            AppError::Validation { message, .. } => {
                format!("数据验证失败: {}", message)
            }
            AppError::Io(io_error) => {
                warn!(target: "tool_registry", io_error = %io_error, "IO error in tool execution");
                "文件或网络操作失败。请检查权限或网络连接。".to_string()
            }
            AppError::ToolExecutionFailed { tool_name, reason } => {
                format!("工具 '{}' 执行失败: {}", tool_name, reason)
            }
            _ => {
                // Log the full error for debugging
                error!(target: "tool_registry", error = %error, "Unexpected error in tool execution");
                format!("执行过程中发生错误: {}。请稍后重试。", error)
            }
        }
    }

    /// Execute multiple tool calls with controlled concurrency
    ///
    /// # Arguments
    /// * `tool_calls` - Vector of tool calls to execute
    /// * `max_concurrent` - Maximum number of concurrent executions (default: 5)
    ///
    /// # Returns
    /// * Vector of `ToolResult` in the same order as input
    pub async fn execute_tools_with_concurrency(
        &self,
        tool_calls: Vec<ToolCall>,
        max_concurrent: usize,
    ) -> Vec<ToolResult> {
        use tokio::sync::Semaphore;

        // Use semaphore to limit concurrent executions
        let semaphore = Arc::new(Semaphore::new(max_concurrent));
        let mut results = Vec::with_capacity(tool_calls.len());

        // Track original order by adding indices
        let indexed_calls: Vec<(usize, ToolCall)> = tool_calls.into_iter().enumerate().collect();

        // Create tasks for all tool calls
        let mut tasks = Vec::new();
        for (index, tool_call) in indexed_calls {
            let semaphore = semaphore.clone();
            let registry = self.clone_for_execution();

            let task = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                let result = registry.execute_tool(tool_call).await;
                (index, result)
            });

            tasks.push(task);
        }

        // Wait for all tasks and collect results
        let mut indexed_results = Vec::new();
        for task in tasks {
            match task.await {
                Ok((index, result)) => {
                    indexed_results.push((index, result));
                }
                Err(e) => {
                    error!(target: "tool_registry", error = %e, "Failed to join tool execution task");
                    indexed_results.push((
                        0,
                        ToolResult {
                            tool_call_id: "unknown".to_string(),
                            result: None,
                            error: Some(format!("Task join error: {}", e)),
                        },
                    ));
                }
            }
        }

        // Sort by original index and extract results
        indexed_results.sort_by_key(|(index, _)| *index);
        for (_, result) in indexed_results {
            results.push(result);
        }

        results
    }

    /// Execute multiple tool calls in parallel with default concurrency limit
    ///
    /// # Arguments
    /// * `tool_calls` - Vector of tool calls to execute
    ///
    /// # Returns
    /// * Vector of `ToolResult` in the same order as input
    pub async fn execute_tools(&self, tool_calls: Vec<ToolCall>) -> Vec<ToolResult> {
        self.execute_tools_with_concurrency(tool_calls, 5).await
    }

    /// Execute multiple tool calls with custom timeout per tool
    ///
    /// # Arguments
    /// * `tool_calls` - Vector of tool calls to execute
    /// * `per_tool_timeout` - Timeout for each individual tool in milliseconds
    ///
    /// # Returns
    /// * Vector of `ToolResult` in the same order as input
    pub async fn execute_tools_with_timeout(
        &self,
        tool_calls: Vec<ToolCall>,
        per_tool_timeout: u64,
    ) -> Vec<ToolResult> {
        let custom_registry = ToolRegistry {
            tools: self.tools.clone(),
            timeout_duration: Duration::from_millis(per_tool_timeout),
        };
        custom_registry.execute_tools(tool_calls).await
    }

    /// Helper method to clone the registry for parallel execution
    /// This creates a shallow clone that shares the tool handlers
    fn clone_for_execution(&self) -> Self {
        Self {
            tools: self.tools.clone(),
            timeout_duration: self.timeout_duration,
        }
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
