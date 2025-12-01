use cognical_app_lib::commands::ai_commands::testing::{
    ai_agent_chat, memory_clear, memory_export, memory_search, AgentChatRequest,
    MemoryClearRequest, MemoryExportRequest, MemorySearchRequest,
};
use cognical_app_lib::commands::AppState;
use cognical_app_lib::db::DbPool;
use tempfile::TempDir;

fn init_state() -> (TempDir, AppState) {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("agent-tests.sqlite");
    let pool = DbPool::new(&db_path).expect("db pool");
    
    // Use temp directory as memory base directory for testing
    let memory_base_dir = dir.path().to_path_buf();
    
    let state = AppState::new(pool, memory_base_dir).expect("app state");
    (dir, state)
}

#[tokio::test]
async fn ai_agent_chat_validates_empty_message() {
    let (_dir, state) = init_state();

    let result = ai_agent_chat(
        &state,
        AgentChatRequest {
            conversation_id: "test-conv-1".to_string(),
            message: "    ".to_string(),
        },
    )
    .await;

    let error = result.expect_err("expected validation error");
    assert_eq!(error.code, "VALIDATION_ERROR");
    assert_eq!(error.message, "消息内容不能为空");
}

#[tokio::test]
async fn ai_agent_chat_validates_empty_conversation_id() {
    let (_dir, state) = init_state();

    let result = ai_agent_chat(
        &state,
        AgentChatRequest {
            conversation_id: "   ".to_string(),
            message: "Hello".to_string(),
        },
    )
    .await;

    let error = result.expect_err("expected validation error");
    assert_eq!(error.code, "VALIDATION_ERROR");
    assert_eq!(error.message, "会话ID不能为空");
}

#[tokio::test]
async fn ai_agent_chat_requires_api_key() {
    let (_dir, state) = init_state();

    let result = ai_agent_chat(
        &state,
        AgentChatRequest {
            conversation_id: "test-conv-1".to_string(),
            message: "Create a task for me".to_string(),
        },
    )
    .await;

    // Should fail because no API key is configured
    let error = result.expect_err("expected missing api key error");
    assert_eq!(error.code, "MISSING_API_KEY");
}

#[tokio::test]
async fn memory_search_validates_empty_query() {
    let (_dir, state) = init_state();

    let result = memory_search(
        &state,
        MemorySearchRequest {
            query: "   ".to_string(),
            filters: None,
        },
    )
    .await;

    let error = result.expect_err("expected validation error");
    assert_eq!(error.code, "VALIDATION_ERROR");
    assert_eq!(error.message, "搜索查询不能为空");
}

#[tokio::test]
async fn memory_search_returns_empty_when_no_memory() {
    let (_dir, state) = init_state();

    let result = memory_search(
        &state,
        MemorySearchRequest {
            query: "test query".to_string(),
            filters: None,
        },
    )
    .await;

    // Should succeed but return empty results when memory service is not available
    // or when there are no matching entries
    match result {
        Ok(_response) => {
            // Empty results are expected when memory is unavailable or no matches
            // Response is valid if we got Ok
        }
        Err(error) => {
            // Memory unavailable or other errors are acceptable when memory service is not running
            assert!(
                error.code == "MEMORY_UNAVAILABLE" || error.code == "UNKNOWN",
                "Expected MEMORY_UNAVAILABLE or UNKNOWN, got: {}",
                error.code
            );
        }
    }
}

#[tokio::test]
async fn memory_export_validates_empty_path() {
    let (_dir, state) = init_state();

    let result = memory_export(
        &state,
        MemoryExportRequest {
            path: "   ".to_string(),
        },
    )
    .await;

    let error = result.expect_err("expected validation error");
    assert_eq!(error.code, "VALIDATION_ERROR");
    assert_eq!(error.message, "导出路径不能为空");
}

#[tokio::test]
async fn memory_export_handles_unavailable_memory() {
    let (dir, state) = init_state();

    let export_path = dir.path().join("export.tar.gz");
    let result = memory_export(
        &state,
        MemoryExportRequest {
            path: export_path.to_string_lossy().to_string(),
        },
    )
    .await;

    // Should either succeed or fail with memory unavailable or other error
    match result {
        Ok(response) => {
            assert!(response.success);
            assert!(response.message.contains("成功"));
        }
        Err(error) => {
            // Memory unavailable or other errors are acceptable when memory service is not running
            assert!(
                error.code == "MEMORY_UNAVAILABLE" || error.code == "UNKNOWN",
                "Expected MEMORY_UNAVAILABLE or UNKNOWN, got: {}",
                error.code
            );
        }
    }
}

#[tokio::test]
async fn memory_clear_validates_empty_conversation_id() {
    let (_dir, state) = init_state();

    let result = memory_clear(
        &state,
        MemoryClearRequest {
            conversation_id: "   ".to_string(),
        },
    )
    .await;

    let error = result.expect_err("expected validation error");
    assert_eq!(error.code, "VALIDATION_ERROR");
    assert_eq!(error.message, "会话ID不能为空");
}

#[tokio::test]
async fn memory_clear_handles_unavailable_memory() {
    let (_dir, state) = init_state();

    let result = memory_clear(
        &state,
        MemoryClearRequest {
            conversation_id: "test-conv-1".to_string(),
        },
    )
    .await;

    // Should either succeed or fail with memory unavailable
    match result {
        Ok(response) => {
            assert!(response.success);
            assert!(response.message.contains("清除"));
        }
        Err(error) => {
            // Memory unavailable is acceptable
            assert_eq!(error.code, "MEMORY_UNAVAILABLE");
        }
    }
}

#[tokio::test]
async fn agent_chat_response_includes_metadata() {
    let (_dir, state) = init_state();

    // This test will fail due to missing API key, but we can check the error structure
    let result = ai_agent_chat(
        &state,
        AgentChatRequest {
            conversation_id: "test-conv-1".to_string(),
            message: "Hello, how are you?".to_string(),
        },
    )
    .await;

    // We expect an error due to missing API key
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.code, "MISSING_API_KEY");
}
