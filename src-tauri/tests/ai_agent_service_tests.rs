use cognical_app_lib::db::DbPool;
use cognical_app_lib::services::ai_agent_service::{
    AgentContext, AgentMetadata, AgentResponse, AiAgentService,
};
use cognical_app_lib::services::ai_service::AiService;

use cognical_app_lib::services::tool_registry::{ToolCall, ToolRegistry};
use serde_json::json;
use std::sync::Arc;
use tempfile::TempDir;

/// Helper function to create a test AI agent service
async fn create_test_agent_service() -> (AiAgentService, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let _kb_path = temp_dir.path().join("kb");

    let db_pool = DbPool::new(&db_path).expect("Failed to create db pool");

    // Create AI service
    let ai_service = Arc::new(AiService::new(db_pool.clone()).expect("Failed to create AI service"));



    // Create tool registry
    let tool_registry = Arc::new(ToolRegistry::new());

    // Create agent service
    let agent_service = AiAgentService::new(ai_service, tool_registry);

    (agent_service, temp_dir)
}

#[tokio::test]
async fn test_agent_service_creation() {
    let (agent_service, _temp_dir) = create_test_agent_service().await;
    // If we got here, the service was created successfully
    drop(agent_service);
}

#[tokio::test]
async fn test_agent_context_structure() {
    let context = AgentContext {
        conversation_id: "test_conv".to_string(),
        available_tools: vec![],
        system_prompt: "Test prompt".to_string(),
    };

    assert_eq!(context.conversation_id, "test_conv");

    assert_eq!(context.available_tools.len(), 0);
    assert_eq!(context.system_prompt, "Test prompt");
}

#[tokio::test]
async fn test_agent_response_structure() {
    let response = AgentResponse {
        message: "Test response".to_string(),
        tool_calls: vec![],
        memory_stored: false,
        metadata: AgentMetadata::default(),
    };

    assert_eq!(response.message, "Test response");
    assert_eq!(response.tool_calls.len(), 0);
    assert!(!response.memory_stored);
    assert_eq!(response.metadata.tools_executed.len(), 0);
}

#[tokio::test]
async fn test_agent_metadata_default() {
    let metadata = AgentMetadata::default();

    assert_eq!(metadata.tokens_used.len(), 0);
    assert_eq!(metadata.latency_ms, 0);
    assert_eq!(metadata.memory_entries_used, 0);
    assert_eq!(metadata.tools_executed.len(), 0);
}

#[tokio::test]
async fn test_agent_response_serialization() {
    let response = AgentResponse {
        message: "Hello".to_string(),
        tool_calls: vec![],
        memory_stored: true,
        metadata: AgentMetadata {
            tokens_used: std::collections::HashMap::new(),
            latency_ms: 100,
            memory_entries_used: 2,
            tools_executed: vec!["test_tool".to_string()],
            correlation_id: Some("test-123".to_string()),
            errors: None,
            memory_available: Some(true),
            performance: None,
        },
    };

    let serialized = serde_json::to_string(&response).expect("Failed to serialize");
    assert!(serialized.contains("Hello"));
    assert!(serialized.contains("\"memory_stored\":true"));
    assert!(serialized.contains("\"latency_ms\":100"));
}

#[tokio::test]
async fn test_agent_response_deserialization() {
    let json_str = r#"{
        "message": "Test message",
        "tool_calls": [],
        "memory_stored": false,
        "metadata": {
            "tokens_used": {},
            "latency_ms": 50,
            "memory_entries_used": 0,
            "tools_executed": []
        }
    }"#;

    let response: AgentResponse = serde_json::from_str(json_str).expect("Failed to deserialize");
    assert_eq!(response.message, "Test message");
    assert!(!response.memory_stored);
    assert_eq!(response.metadata.latency_ms, 50);
}

#[tokio::test]
async fn test_tool_call_structure() {
    let tool_call = ToolCall {
        id: "call_123".to_string(),
        name: "test_tool".to_string(),
        arguments: json!({"param": "value"}),
    };

    assert_eq!(tool_call.id, "call_123");
    assert_eq!(tool_call.name, "test_tool");
    assert_eq!(tool_call.arguments["param"], "value");
}

#[tokio::test]
async fn test_agent_service_with_registered_tools() {
    let (agent_service, _temp_dir) = create_test_agent_service().await;

    // Register a simple test tool
    let tool_registry = Arc::new({
        let mut registry = ToolRegistry::new();
        registry
            .register_tool(
                "test_tool".to_string(),
                "A test tool".to_string(),
                json!({
                    "type": "object",
                    "properties": {
                        "message": {
                            "type": "string",
                            "description": "A test message"
                        }
                    },
                    "required": ["message"]
                }),
                Arc::new(|args| {
                    Box::pin(async move {
                        Ok(json!({
                            "result": format!("Received: {}", args["message"])
                        }))
                    })
                }),
            )
            .expect("Failed to register tool");
        registry
    });

    // Verify tool is registered
    assert_eq!(tool_registry.tool_count(), 1);
    assert!(tool_registry.has_tool("test_tool"));

    drop(agent_service);
}

#[tokio::test]
async fn test_agent_metadata_with_tools() {
    let mut metadata = AgentMetadata::default();
    metadata.tools_executed.push("create_task".to_string());
    metadata.tools_executed.push("list_tasks".to_string());
    metadata.latency_ms = 250;
    metadata.memory_entries_used = 3;

    assert_eq!(metadata.tools_executed.len(), 2);
    assert_eq!(metadata.tools_executed[0], "create_task");
    assert_eq!(metadata.tools_executed[1], "list_tasks");
    assert_eq!(metadata.latency_ms, 250);
    assert_eq!(metadata.memory_entries_used, 3);
}

// Note: Full end-to-end tests with actual AI calls would require:
// 1. A valid DeepSeek API key
// 2. Network connectivity
// 3. Memory service running
// These tests focus on the structure and basic functionality
// without requiring external dependencies
