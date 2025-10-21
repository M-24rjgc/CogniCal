# AI Agent Developer Guide

## Overview

This guide explains how to extend the AI Agent system with new tools, customize memory behavior, and integrate with the agent architecture. It's intended for developers who want to add new capabilities to the AI Agent.

## Architecture Overview

The AI Agent system consists of several key components:

```
┌─────────────────────────────────────────────────────────────┐
│                    AI Agent Service                          │
│  - Orchestrates chat flow                                    │
│  - Builds context from memory and tools                      │
│  - Handles multi-turn tool calling                           │
└─────────────┬───────────────────────────┬───────────────────┘
              │                           │
    ┌─────────▼──────────┐    ┌──────────▼────────────┐
    │  Memory Service    │    │   Tool Registry       │
    │  - MCP Client      │    │   - Tool Definitions  │
    │  - Semantic Search │    │   - Validation        │
    │  - Storage         │    │   - Execution         │
    └────────────────────┘    └───────────────────────┘
```

## Adding New Tools

### Step 1: Define Your Tool Schema

Tools use the OpenAI function calling format. Create a schema that describes your tool:

```rust
use serde_json::json;

pub fn get_weather_tool_schema() -> serde_json::Value {
    json!({
        "type": "function",
        "function": {
            "name": "get_weather",
            "description": "Get the current weather for a location",
            "parameters": {
                "type": "object",
                "properties": {
                    "location": {
                        "type": "string",
                        "description": "The city and state, e.g. San Francisco, CA"
                    },
                    "unit": {
                        "type": "string",
                        "enum": ["celsius", "fahrenheit"],
                        "description": "The temperature unit"
                    }
                },
                "required": ["location"]
            }
        }
    })
}
```

**Schema Guidelines:**
- Use clear, descriptive names
- Provide detailed descriptions for the AI to understand
- Mark required parameters explicitly
- Use enums for constrained values
- Include examples in descriptions when helpful

### Step 2: Implement the Tool Handler

Create a handler function that executes your tool:

```rust
use crate::error::AppResult;
use crate::state::AppState;
use serde_json::{json, Value};
use std::sync::Arc;

pub async fn get_weather_tool(
    app_state: Arc<AppState>,
    args: Value,
) -> AppResult<Value> {
    // 1. Extract and validate parameters
    let location = args["location"]
        .as_str()
        .ok_or_else(|| AppError::InvalidInput("location is required".into()))?;
    
    let unit = args["unit"]
        .as_str()
        .unwrap_or("fahrenheit");
    
    // 2. Perform the action
    let weather_data = fetch_weather(location, unit).await?;
    
    // 3. Format the result for AI consumption
    Ok(json!({
        "location": location,
        "temperature": weather_data.temp,
        "unit": unit,
        "condition": weather_data.condition,
        "message": format!(
            "The weather in {} is {} {}°{} with {} conditions",
            location,
            weather_data.condition,
            weather_data.temp,
            unit.chars().next().unwrap().to_uppercase(),
            weather_data.condition
        )
    }))
}

async fn fetch_weather(location: &str, unit: &str) -> AppResult<WeatherData> {
    // Your implementation here
    // Call external API, database, etc.
    todo!()
}
```

**Handler Best Practices:**
- Validate all parameters thoroughly
- Return descriptive error messages
- Format results in a human-readable way
- Include a `message` field for the AI to use directly
- Keep handlers async for I/O operations
- Use proper error types from `AppError`

### Step 3: Register the Tool

Register your tool in the tool registry during application initialization:

```rust
// In src-tauri/src/main.rs or initialization code

use crate::tools::weather_tools::{get_weather_tool, get_weather_tool_schema};

fn register_tools(tool_registry: &mut ToolRegistry, app_state: Arc<AppState>) {
    // Register the weather tool
    let app_state_clone = app_state.clone();
    tool_registry
        .register_tool(
            "get_weather".to_string(),
            "Get the current weather for a location".to_string(),
            get_weather_tool_schema(),
            Arc::new(move |args| {
                let state = app_state_clone.clone();
                Box::pin(async move {
                    get_weather_tool(state, args).await
                })
            }),
        )
        .expect("Failed to register get_weather tool");
}
```

**Registration Notes:**
- Register tools during app startup
- Use `Arc` for shared state
- Use `Box::pin` for async handlers
- Handle registration errors appropriately

### Step 4: Test Your Tool

Create comprehensive tests for your tool:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::create_test_app_state;

    #[tokio::test]
    async fn test_get_weather_tool() {
        let app_state = create_test_app_state().await;
        
        let args = json!({
            "location": "San Francisco, CA",
            "unit": "celsius"
        });
        
        let result = get_weather_tool(app_state, args)
            .await
            .expect("Tool should execute successfully");
        
        assert_eq!(result["location"], "San Francisco, CA");
        assert_eq!(result["unit"], "celsius");
        assert!(result["temperature"].is_number());
        assert!(result["message"].is_string());
    }
    
    #[tokio::test]
    async fn test_get_weather_missing_location() {
        let app_state = create_test_app_state().await;
        
        let args = json!({
            "unit": "celsius"
        });
        
        let result = get_weather_tool(app_state, args).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_get_weather_default_unit() {
        let app_state = create_test_app_state().await;
        
        let args = json!({
            "location": "New York, NY"
        });
        
        let result = get_weather_tool(app_state, args)
            .await
            .expect("Tool should use default unit");
        
        assert_eq!(result["unit"], "fahrenheit");
    }
}
```

**Testing Guidelines:**
- Test successful execution
- Test error cases (missing parameters, invalid values)
- Test default values
- Test edge cases specific to your tool
- Use integration tests for tools that interact with services

## Tool Design Patterns

### Pattern 1: Simple Query Tool

For tools that retrieve information without side effects:

```rust
pub async fn list_items_tool(
    app_state: Arc<AppState>,
    args: Value,
) -> AppResult<Value> {
    let filter = args["filter"].as_str();
    let limit = args["limit"].as_u64().unwrap_or(10);
    
    let items = app_state.service.list_items(filter, limit).await?;
    
    Ok(json!({
        "items": items,
        "count": items.len(),
        "message": format!("Found {} items", items.len())
    }))
}
```

### Pattern 2: Action Tool

For tools that perform actions with side effects:

```rust
pub async fn create_item_tool(
    app_state: Arc<AppState>,
    args: Value,
) -> AppResult<Value> {
    let name = args["name"]
        .as_str()
        .ok_or_else(|| AppError::InvalidInput("name is required".into()))?;
    
    let item = app_state.service.create_item(name).await?;
    
    Ok(json!({
        "id": item.id,
        "name": item.name,
        "created_at": item.created_at,
        "message": format!("Created item '{}' with ID {}", item.name, item.id)
    }))
}
```

### Pattern 3: Multi-Step Tool

For tools that require multiple operations:

```rust
pub async fn complex_operation_tool(
    app_state: Arc<AppState>,
    args: Value,
) -> AppResult<Value> {
    // Step 1: Validate prerequisites
    let item_id = args["item_id"].as_str()
        .ok_or_else(|| AppError::InvalidInput("item_id is required".into()))?;
    
    let item = app_state.service.get_item(item_id).await?;
    
    // Step 2: Perform operation
    let result = app_state.service.process_item(&item).await?;
    
    // Step 3: Update state
    app_state.service.update_item_status(item_id, "processed").await?;
    
    // Step 4: Return comprehensive result
    Ok(json!({
        "item_id": item_id,
        "result": result,
        "status": "processed",
        "message": format!("Successfully processed item {}", item_id)
    }))
}
```

### Pattern 4: Validation-Heavy Tool

For tools with complex validation requirements:

```rust
pub async fn schedule_event_tool(
    app_state: Arc<AppState>,
    args: Value,
) -> AppResult<Value> {
    // Extract parameters
    let title = args["title"].as_str()
        .ok_or_else(|| AppError::InvalidInput("title is required".into()))?;
    
    let date_str = args["date"].as_str()
        .ok_or_else(|| AppError::InvalidInput("date is required".into()))?;
    
    // Validate date format
    let date = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .map_err(|_| AppError::InvalidInput(
            "date must be in YYYY-MM-DD format".into()
        ))?;
    
    // Validate date is in the future
    if date < chrono::Local::now().date_naive() {
        return Err(AppError::InvalidInput(
            "date must be in the future".into()
        ));
    }
    
    // Check for conflicts
    let conflicts = app_state.calendar.check_conflicts(&date).await?;
    if !conflicts.is_empty() {
        return Ok(json!({
            "success": false,
            "conflicts": conflicts,
            "message": format!(
                "Cannot schedule event - {} conflicting events found",
                conflicts.len()
            )
        }));
    }
    
    // Create event
    let event = app_state.calendar.create_event(title, date).await?;
    
    Ok(json!({
        "success": true,
        "event_id": event.id,
        "message": format!("Scheduled '{}' for {}", title, date)
    }))
}
```

## Memory Service API

### Storing Conversations

```rust
use crate::services::memory_service::MemoryService;

// Store a conversation turn
let memory_service = app_state.memory_service.clone();

memory_service.store_conversation(
    "conversation_123",           // Conversation ID
    "What's the weather today?",  // User message
    "It's sunny and 72°F",        // Assistant message
    Some(HashMap::from([
        ("tools_used".to_string(), "get_weather".to_string()),
        ("location".to_string(), "San Francisco".to_string()),
    ])),                          // Optional metadata
).await?;
```

### Retrieving Context

```rust
// Retrieve relevant context for a query
let context = memory_service.retrieve_context(
    "weather discussions",  // Query
    5,                      // Limit
).await?;

for entry in context.entries {
    println!("User: {}", entry.user_message);
    println!("AI: {}", entry.assistant_message);
    println!("Relevance: {:.2}", context.relevance_scores[0]);
}
```

### Searching Conversations

```rust
// Search with filters
let filters = HashMap::from([
    ("conversation_id".to_string(), "conversation_123".to_string()),
    ("date_from".to_string(), "2025-01-01".to_string()),
]);

let results = memory_service.search_conversations(
    "project deadlines",
    Some(filters),
).await?;
```

### Checking Availability

```rust
// Check if memory service is available
if memory_service.is_available() {
    // Use memory features
    let context = memory_service.retrieve_context(query, 5).await?;
} else {
    // Fallback to stateless mode
    log::warn!("Memory service unavailable, using stateless mode");
}
```

## Configuration Options

### Memory Configuration

```rust
use crate::services::memory_service::MemoryConfig;

let config = MemoryConfig {
    kb_path: PathBuf::from("/path/to/knowledge_base"),
    max_context_entries: 5,
    search_limit: 10,
    enable_graph: true,
};

let memory_service = MemoryService::new(db_pool, config)?;
```

**Configuration Options:**
- `kb_path`: Path to the knowledge base directory
- `max_context_entries`: Maximum number of memory entries to include in context
- `search_limit`: Maximum number of results to return from search
- `enable_graph`: Enable knowledge graph features (requires additional setup)

### Tool Registry Configuration

```rust
use crate::services::tool_registry::ToolRegistry;

let mut tool_registry = ToolRegistry::new();

// Configure timeout for tool execution
tool_registry.set_timeout(Duration::from_secs(5));

// Configure maximum concurrent tool executions
tool_registry.set_max_concurrent(10);
```

### AI Agent Configuration

```rust
use crate::services::ai_agent_service::AiAgentService;

let agent_service = AiAgentService::new(
    ai_service,
    memory_service,
    tool_registry,
);

// Configure system prompt
agent_service.set_system_prompt(
    "You are a helpful assistant specialized in productivity..."
);

// Configure token limits
agent_service.set_max_context_tokens(4000);
agent_service.set_max_response_tokens(1000);
```

## Advanced Topics

### Custom Memory Backends

You can implement custom memory backends by implementing the `MemoryBackend` trait:

```rust
use async_trait::async_trait;

#[async_trait]
pub trait MemoryBackend: Send + Sync {
    async fn store(
        &self,
        entry: MemoryEntry,
    ) -> AppResult<String>;
    
    async fn search(
        &self,
        query: &str,
        limit: usize,
    ) -> AppResult<Vec<MemoryEntry>>;
    
    async fn delete(
        &self,
        entry_id: &str,
    ) -> AppResult<()>;
}

// Example: Redis-based memory backend
pub struct RedisMemoryBackend {
    client: redis::Client,
}

#[async_trait]
impl MemoryBackend for RedisMemoryBackend {
    async fn store(&self, entry: MemoryEntry) -> AppResult<String> {
        // Implementation
        todo!()
    }
    
    async fn search(
        &self,
        query: &str,
        limit: usize,
    ) -> AppResult<Vec<MemoryEntry>> {
        // Implementation
        todo!()
    }
    
    async fn delete(&self, entry_id: &str) -> AppResult<()> {
        // Implementation
        todo!()
    }
}
```

### Tool Middleware

Add middleware to intercept tool execution:

```rust
pub type ToolMiddleware = Arc<
    dyn Fn(ToolCall) -> BoxFuture<'static, AppResult<ToolCall>> + Send + Sync
>;

impl ToolRegistry {
    pub fn add_middleware(&mut self, middleware: ToolMiddleware) {
        self.middleware.push(middleware);
    }
}

// Example: Logging middleware
let logging_middleware: ToolMiddleware = Arc::new(|tool_call| {
    Box::pin(async move {
        log::info!("Executing tool: {}", tool_call.name);
        Ok(tool_call)
    })
});

tool_registry.add_middleware(logging_middleware);

// Example: Rate limiting middleware
let rate_limit_middleware: ToolMiddleware = Arc::new(|tool_call| {
    Box::pin(async move {
        if !check_rate_limit(&tool_call.name).await {
            return Err(AppError::RateLimitExceeded);
        }
        Ok(tool_call)
    })
});

tool_registry.add_middleware(rate_limit_middleware);
```

### Streaming Responses

Implement streaming for real-time updates:

```rust
use tokio::sync::mpsc;

pub async fn chat_with_streaming(
    agent_service: Arc<AiAgentService>,
    conversation_id: String,
    message: String,
) -> AppResult<mpsc::Receiver<StreamChunk>> {
    let (tx, rx) = mpsc::channel(100);
    
    tokio::spawn(async move {
        // Stream text chunks
        let mut stream = agent_service
            .chat_stream(&conversation_id, &message)
            .await?;
        
        while let Some(chunk) = stream.next().await {
            tx.send(StreamChunk::Text(chunk)).await.ok();
        }
        
        // Stream tool execution updates
        if let Some(tool_calls) = stream.tool_calls() {
            for tool_call in tool_calls {
                tx.send(StreamChunk::ToolStart(tool_call.name.clone())).await.ok();
                
                let result = agent_service.execute_tool(tool_call).await?;
                
                tx.send(StreamChunk::ToolComplete(result)).await.ok();
            }
        }
        
        Ok::<(), AppError>(())
    });
    
    Ok(rx)
}
```

## Error Handling

### Custom Error Types

Define specific error types for your tools:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WeatherToolError {
    #[error("Invalid location: {0}")]
    InvalidLocation(String),
    
    #[error("Weather service unavailable")]
    ServiceUnavailable,
    
    #[error("API rate limit exceeded")]
    RateLimitExceeded,
}

// Convert to AppError
impl From<WeatherToolError> for AppError {
    fn from(err: WeatherToolError) -> Self {
        AppError::ToolExecutionFailed {
            tool_name: "get_weather".to_string(),
            reason: err.to_string(),
        }
    }
}
```

### Graceful Degradation

Handle failures gracefully:

```rust
pub async fn resilient_tool_execution(
    tool_registry: &ToolRegistry,
    tool_call: ToolCall,
) -> AppResult<ToolResult> {
    // Try primary execution
    match tool_registry.execute_tool(tool_call.clone()).await {
        Ok(result) => Ok(result),
        Err(e) => {
            log::warn!("Tool execution failed: {}, attempting fallback", e);
            
            // Try fallback method
            match execute_fallback(&tool_call).await {
                Ok(result) => Ok(result),
                Err(fallback_err) => {
                    // Return user-friendly error
                    Ok(ToolResult {
                        tool_call_id: tool_call.id,
                        result: json!({
                            "success": false,
                            "message": "The operation could not be completed at this time"
                        }),
                        error: Some(format!("Primary: {}, Fallback: {}", e, fallback_err)),
                    })
                }
            }
        }
    }
}
```

## Performance Optimization

### Caching Tool Schemas

```rust
use std::sync::RwLock;

pub struct CachedToolRegistry {
    registry: ToolRegistry,
    schema_cache: RwLock<Option<Vec<Value>>>,
}

impl CachedToolRegistry {
    pub fn get_tool_schemas(&self) -> Vec<Value> {
        // Check cache first
        if let Some(cached) = self.schema_cache.read().unwrap().as_ref() {
            return cached.clone();
        }
        
        // Generate and cache
        let schemas = self.registry.get_tool_schemas();
        *self.schema_cache.write().unwrap() = Some(schemas.clone());
        schemas
    }
    
    pub fn invalidate_cache(&self) {
        *self.schema_cache.write().unwrap() = None;
    }
}
```

### Parallel Tool Execution

```rust
use futures::future::join_all;

pub async fn execute_tools_parallel(
    tool_registry: &ToolRegistry,
    tool_calls: Vec<ToolCall>,
) -> Vec<AppResult<ToolResult>> {
    let futures = tool_calls
        .into_iter()
        .map(|call| tool_registry.execute_tool(call));
    
    join_all(futures).await
}
```

### Memory Context Optimization

```rust
pub async fn build_optimized_context(
    memory_service: &MemoryService,
    query: &str,
    max_tokens: usize,
) -> AppResult<MemoryContext> {
    // Retrieve more results than needed
    let mut context = memory_service
        .retrieve_context(query, 20)
        .await?;
    
    // Sort by relevance
    let mut entries_with_scores: Vec<_> = context.entries
        .into_iter()
        .zip(context.relevance_scores.iter())
        .collect();
    
    entries_with_scores.sort_by(|a, b| 
        b.1.partial_cmp(a.1).unwrap()
    );
    
    // Take entries until token limit
    let mut total_tokens = 0;
    let mut selected_entries = Vec::new();
    
    for (entry, score) in entries_with_scores {
        let entry_tokens = estimate_tokens(&entry);
        if total_tokens + entry_tokens > max_tokens {
            break;
        }
        total_tokens += entry_tokens;
        selected_entries.push(entry);
    }
    
    Ok(MemoryContext {
        entries: selected_entries,
        relevance_scores: vec![],
        total_tokens,
    })
}
```

## Testing Utilities

### Mock Memory Service

```rust
pub struct MockMemoryService {
    stored_conversations: Arc<RwLock<Vec<MemoryEntry>>>,
}

impl MockMemoryService {
    pub fn new() -> Self {
        Self {
            stored_conversations: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    pub async fn store_conversation(
        &self,
        conversation_id: &str,
        user_message: &str,
        assistant_message: &str,
        metadata: Option<HashMap<String, String>>,
    ) -> AppResult<String> {
        let entry = MemoryEntry {
            id: uuid::Uuid::new_v4().to_string(),
            conversation_id: conversation_id.to_string(),
            user_message: user_message.to_string(),
            assistant_message: assistant_message.to_string(),
            timestamp: chrono::Utc::now(),
            metadata: metadata.unwrap_or_default(),
        };
        
        self.stored_conversations.write().unwrap().push(entry.clone());
        Ok(entry.id)
    }
}
```

### Test Helpers

```rust
pub mod test_utils {
    use super::*;
    
    pub async fn create_test_app_state() -> Arc<AppState> {
        // Create test database
        let db_pool = create_test_db_pool().await;
        
        // Create test services
        let memory_service = Arc::new(MockMemoryService::new());
        let tool_registry = Arc::new(ToolRegistry::new());
        
        Arc::new(AppState {
            db: db_pool,
            memory_service,
            tool_registry,
            // ... other fields
        })
    }
    
    pub fn create_test_tool_call(name: &str, args: Value) -> ToolCall {
        ToolCall {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            arguments: args,
        }
    }
}
```

## Debugging

### Enable Debug Logging

```rust
// In main.rs
env_logger::Builder::from_env(
    env_logger::Env::default().default_filter_or("debug")
).init();

// In your code
log::debug!("Tool call: {:?}", tool_call);
log::debug!("Memory context: {} entries", context.entries.len());
```

### Tool Execution Tracing

```rust
use tracing::{info_span, instrument};

#[instrument(skip(app_state))]
pub async fn create_task_tool(
    app_state: Arc<AppState>,
    args: Value,
) -> AppResult<Value> {
    let span = info_span!("create_task_tool", task_title = ?args["title"]);
    let _enter = span.enter();
    
    // Your implementation
    todo!()
}
```

## Best Practices Summary

1. **Tool Design**:
   - Keep tools focused on a single responsibility
   - Provide clear, descriptive schemas
   - Return human-readable messages
   - Handle errors gracefully

2. **Memory Usage**:
   - Limit context size to avoid token limits
   - Use semantic search effectively
   - Store relevant metadata
   - Implement data retention policies

3. **Performance**:
   - Cache tool schemas
   - Execute independent tools in parallel
   - Optimize memory queries
   - Use streaming for long responses

4. **Testing**:
   - Write comprehensive unit tests
   - Test error scenarios
   - Use integration tests for complex flows
   - Mock external dependencies

5. **Error Handling**:
   - Provide specific error messages
   - Implement fallback mechanisms
   - Log errors with context
   - Never crash the application

## Resources

- **API Documentation**: See inline Rust docs with `cargo doc --open`
- **Example Tools**: Check `src-tauri/src/tools/` for reference implementations
- **Test Examples**: See `src-tauri/tests/` for testing patterns
- **Architecture Diagram**: See `design.md` for system overview

---

**Version**: 1.0  
**Last Updated**: January 2025  
**Questions?**: Open an issue or check the inline documentation
