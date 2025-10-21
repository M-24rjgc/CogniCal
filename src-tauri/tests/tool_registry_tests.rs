use cognical_app_lib::services::tool_registry::{ToolCall, ToolRegistry};
use serde_json::json;
use std::sync::Arc;

/// Helper function to create a simple test tool handler
fn create_echo_handler() -> Arc<
    dyn Fn(serde_json::Value) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<serde_json::Value, cognical_app_lib::error::AppError>> + Send>,
    > + Send
        + Sync,
> {
    Arc::new(|args| {
        Box::pin(async move {
            // Echo back the arguments
            Ok(json!({
                "echoed": args,
                "status": "success"
            }))
        })
    })
}

/// Helper function to create a failing tool handler
fn create_failing_handler() -> Arc<
    dyn Fn(serde_json::Value) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<serde_json::Value, cognical_app_lib::error::AppError>> + Send>,
    > + Send
        + Sync,
> {
    Arc::new(|_args| {
        Box::pin(async move {
            Err(cognical_app_lib::error::AppError::other(
                "Tool execution failed",
            ))
        })
    })
}

/// Helper function to create a slow tool handler (for timeout testing)
fn create_slow_handler() -> Arc<
    dyn Fn(serde_json::Value) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<serde_json::Value, cognical_app_lib::error::AppError>> + Send>,
    > + Send
        + Sync,
> {
    Arc::new(|_args| {
        Box::pin(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            Ok(json!({"status": "completed"}))
        })
    })
}


#[test]
fn test_tool_registry_creation() {
    let registry = ToolRegistry::new();
    assert_eq!(registry.tool_count(), 0);
}

#[test]
fn test_register_tool_success() {
    let mut registry = ToolRegistry::new();

    let schema = json!({
        "type": "object",
        "properties": {
            "message": {
                "type": "string",
                "description": "The message to echo"
            }
        },
        "required": ["message"]
    });

    let result = registry.register_tool(
        "echo_tool".to_string(),
        "Echoes back the input message".to_string(),
        schema,
        create_echo_handler(),
    );

    assert!(result.is_ok());
    assert_eq!(registry.tool_count(), 1);
    assert!(registry.has_tool("echo_tool"));
}

#[test]
fn test_register_duplicate_tool_fails() {
    let mut registry = ToolRegistry::new();

    let schema = json!({
        "type": "object",
        "properties": {
            "message": {"type": "string"}
        }
    });

    // Register first time - should succeed
    let result1 = registry.register_tool(
        "test_tool".to_string(),
        "Test tool".to_string(),
        schema.clone(),
        create_echo_handler(),
    );
    assert!(result1.is_ok());

    // Register second time - should fail
    let result2 = registry.register_tool(
        "test_tool".to_string(),
        "Test tool duplicate".to_string(),
        schema,
        create_echo_handler(),
    );
    assert!(result2.is_err());
}

#[test]
fn test_register_tool_with_invalid_schema_fails() {
    let mut registry = ToolRegistry::new();

    // Schema must be an object, not a string
    let invalid_schema = json!("not an object");

    let result = registry.register_tool(
        "invalid_tool".to_string(),
        "Tool with invalid schema".to_string(),
        invalid_schema,
        create_echo_handler(),
    );

    assert!(result.is_err());
    assert_eq!(registry.tool_count(), 0);
}


#[test]
fn test_get_tool_schemas() {
    let mut registry = ToolRegistry::new();

    let schema1 = json!({
        "type": "object",
        "properties": {
            "name": {"type": "string"}
        }
    });

    let schema2 = json!({
        "type": "object",
        "properties": {
            "count": {"type": "number"}
        }
    });

    registry
        .register_tool(
            "tool1".to_string(),
            "First tool".to_string(),
            schema1,
            create_echo_handler(),
        )
        .unwrap();

    registry
        .register_tool(
            "tool2".to_string(),
            "Second tool".to_string(),
            schema2,
            create_echo_handler(),
        )
        .unwrap();

    let schemas = registry.get_tool_schemas();
    assert_eq!(schemas.len(), 2);

    // Verify OpenAI function calling format
    for schema in schemas {
        assert_eq!(schema["type"], "function");
        assert!(schema["function"]["name"].is_string());
        assert!(schema["function"]["description"].is_string());
        assert!(schema["function"]["parameters"].is_object());
    }
}

#[test]
fn test_validate_tool_call_success() {
    let mut registry = ToolRegistry::new();

    let schema = json!({
        "type": "object",
        "properties": {
            "message": {
                "type": "string",
                "description": "The message"
            },
            "count": {
                "type": "number",
                "description": "A number"
            }
        },
        "required": ["message"]
    });

    registry
        .register_tool(
            "test_tool".to_string(),
            "Test tool".to_string(),
            schema,
            create_echo_handler(),
        )
        .unwrap();

    let tool_call = ToolCall {
        id: "call_123".to_string(),
        name: "test_tool".to_string(),
        arguments: json!({
            "message": "Hello",
            "count": 42
        }),
    };

    let result = registry.validate_tool_call(&tool_call);
    assert!(result.is_ok());
}


#[test]
fn test_validate_tool_call_missing_required_field() {
    let mut registry = ToolRegistry::new();

    let schema = json!({
        "type": "object",
        "properties": {
            "message": {"type": "string"}
        },
        "required": ["message"]
    });

    registry
        .register_tool(
            "test_tool".to_string(),
            "Test tool".to_string(),
            schema,
            create_echo_handler(),
        )
        .unwrap();

    let tool_call = ToolCall {
        id: "call_123".to_string(),
        name: "test_tool".to_string(),
        arguments: json!({}), // Missing required "message" field
    };

    let result = registry.validate_tool_call(&tool_call);
    assert!(result.is_err());
}

#[test]
fn test_validate_tool_call_wrong_type() {
    let mut registry = ToolRegistry::new();

    let schema = json!({
        "type": "object",
        "properties": {
            "count": {"type": "number"}
        },
        "required": ["count"]
    });

    registry
        .register_tool(
            "test_tool".to_string(),
            "Test tool".to_string(),
            schema,
            create_echo_handler(),
        )
        .unwrap();

    let tool_call = ToolCall {
        id: "call_123".to_string(),
        name: "test_tool".to_string(),
        arguments: json!({
            "count": "not a number" // Wrong type
        }),
    };

    let result = registry.validate_tool_call(&tool_call);
    assert!(result.is_err());
}

#[test]
fn test_validate_tool_call_nonexistent_tool() {
    let registry = ToolRegistry::new();

    let tool_call = ToolCall {
        id: "call_123".to_string(),
        name: "nonexistent_tool".to_string(),
        arguments: json!({}),
    };

    let result = registry.validate_tool_call(&tool_call);
    assert!(result.is_err());
}


#[tokio::test]
async fn test_execute_tool_success() {
    let mut registry = ToolRegistry::new();

    let schema = json!({
        "type": "object",
        "properties": {
            "message": {"type": "string"}
        },
        "required": ["message"]
    });

    registry
        .register_tool(
            "echo_tool".to_string(),
            "Echoes back the input".to_string(),
            schema,
            create_echo_handler(),
        )
        .unwrap();

    let tool_call = ToolCall {
        id: "call_123".to_string(),
        name: "echo_tool".to_string(),
        arguments: json!({
            "message": "Hello, World!"
        }),
    };

    let result = registry.execute_tool(tool_call).await;

    assert_eq!(result.tool_call_id, "call_123");
    assert!(result.error.is_none());
    assert!(result.result.is_some());

    let result_value = result.result.unwrap();
    assert_eq!(result_value["status"], "success");
    assert_eq!(result_value["echoed"]["message"], "Hello, World!");
}

#[tokio::test]
async fn test_execute_tool_validation_failure() {
    let mut registry = ToolRegistry::new();

    let schema = json!({
        "type": "object",
        "properties": {
            "count": {"type": "number"}
        },
        "required": ["count"]
    });

    registry
        .register_tool(
            "test_tool".to_string(),
            "Test tool".to_string(),
            schema,
            create_echo_handler(),
        )
        .unwrap();

    let tool_call = ToolCall {
        id: "call_456".to_string(),
        name: "test_tool".to_string(),
        arguments: json!({
            "count": "not a number"
        }),
    };

    let result = registry.execute_tool(tool_call).await;

    assert_eq!(result.tool_call_id, "call_456");
    assert!(result.result.is_none());
    assert!(result.error.is_some());
}


#[tokio::test]
async fn test_execute_tool_handler_failure() {
    let mut registry = ToolRegistry::new();

    let schema = json!({
        "type": "object",
        "properties": {
            "input": {"type": "string"}
        }
    });

    registry
        .register_tool(
            "failing_tool".to_string(),
            "A tool that always fails".to_string(),
            schema,
            create_failing_handler(),
        )
        .unwrap();

    let tool_call = ToolCall {
        id: "call_789".to_string(),
        name: "failing_tool".to_string(),
        arguments: json!({
            "input": "test"
        }),
    };

    let result = registry.execute_tool(tool_call).await;

    assert_eq!(result.tool_call_id, "call_789");
    assert!(result.result.is_none());
    assert!(result.error.is_some());
    // Error message is in Chinese: "工具 'failing_tool' 执行失败"
    assert!(result.error.unwrap().contains("执行失败"));
}

#[tokio::test]
async fn test_execute_tool_timeout() {
    let mut registry = ToolRegistry::with_timeout(100); // 100ms timeout

    let schema = json!({
        "type": "object",
        "properties": {}
    });

    registry
        .register_tool(
            "slow_tool".to_string(),
            "A slow tool".to_string(),
            schema,
            create_slow_handler(),
        )
        .unwrap();

    let tool_call = ToolCall {
        id: "call_timeout".to_string(),
        name: "slow_tool".to_string(),
        arguments: json!({}),
    };

    let result = registry.execute_tool(tool_call).await;

    assert_eq!(result.tool_call_id, "call_timeout");
    assert!(result.result.is_none());
    assert!(result.error.is_some());
    // Error message is in Chinese: "工具 'slow_tool' 执行超时"
    assert!(result.error.unwrap().contains("超时"));
}

#[tokio::test]
async fn test_execute_multiple_tools() {
    let mut registry = ToolRegistry::new();

    let schema = json!({
        "type": "object",
        "properties": {
            "value": {"type": "string"}
        }
    });

    registry
        .register_tool(
            "tool1".to_string(),
            "First tool".to_string(),
            schema.clone(),
            create_echo_handler(),
        )
        .unwrap();

    registry
        .register_tool(
            "tool2".to_string(),
            "Second tool".to_string(),
            schema,
            create_echo_handler(),
        )
        .unwrap();

    let tool_calls = vec![
        ToolCall {
            id: "call_1".to_string(),
            name: "tool1".to_string(),
            arguments: json!({"value": "first"}),
        },
        ToolCall {
            id: "call_2".to_string(),
            name: "tool2".to_string(),
            arguments: json!({"value": "second"}),
        },
    ];

    let results = registry.execute_tools(tool_calls).await;

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].tool_call_id, "call_1");
    assert_eq!(results[1].tool_call_id, "call_2");
    assert!(results[0].error.is_none());
    assert!(results[1].error.is_none());
}
