use cognical_app_lib::commands::ai_commands::testing::{
    ai_generate_recommendations, ai_plan_schedule, ai_status, tasks_parse_ai,
};
use cognical_app_lib::commands::AppState;
use cognical_app_lib::db::DbPool;
use cognical_app_lib::models::ai::TaskParseRequest;
use cognical_app_lib::models::ai_types::AiResponseSource;
use serde_json::json;
use tempfile::TempDir;

fn init_state() -> (TempDir, AppState) {
    let dir = tempfile::tempdir().expect("temp dir");
    let db_path = dir.path().join("ai-tests.sqlite");
    let pool = DbPool::new(&db_path).expect("db pool");

    // Use temp directory as memory base directory for testing
    let memory_base_dir = dir.path().to_path_buf();

    let state = AppState::new(pool, memory_base_dir).expect("app state");
    (dir, state)
}

#[tokio::test]
async fn tasks_parse_ai_impl_validates_empty_input() {
    let (_dir, state) = init_state();

    let result = tasks_parse_ai(
        &state,
        TaskParseRequest {
            input: "    ".to_string(),
            context: None,
        },
    )
    .await;

    let error = result.expect_err("expected validation error");
    assert_eq!(error.code, "VALIDATION_ERROR");
    assert_eq!(error.message, "待解析内容不能为空");
}

#[tokio::test]
async fn tasks_parse_ai_impl_requires_api_key() {
    let (_dir, state) = init_state();

    let result = tasks_parse_ai(
        &state,
        TaskParseRequest {
            input: "整理季度 OKR 的执行计划".to_string(),
            context: None,
        },
    )
    .await;

    let error = result.expect_err("expected missing api key error");
    assert_eq!(error.code, "MISSING_API_KEY");
    assert_eq!(error.message, "DeepSeek API Key 未配置");
}

#[tokio::test]
async fn ai_generate_recommendations_requires_api_key() {
    let (_dir, state) = init_state();

    let request = json!({
        "tasks": [
            {"title": "Plan launch", "priority": "high"},
            {"title": "Write release notes", "priority": "medium"}
        ]
    });

    let result = ai_generate_recommendations(&state, request).await;

    let error = result.expect_err("expected missing api key error");
    assert_eq!(error.code, "MISSING_API_KEY");
    assert_eq!(error.message, "DeepSeek API Key 未配置");
}

#[tokio::test]
async fn ai_plan_schedule_impl_requires_api_key() {
    let (_dir, state) = init_state();

    let payload = json!({
        "tasks": [
            {"id": "task-1", "title": "Draft blog post", "estimatedMinutes": 75}
        ],
        "availability": {"startAt": "2025-10-15T08:00:00Z"}
    });

    let result = ai_plan_schedule(&state, payload).await;

    let error = result.expect_err("expected missing api key error");
    assert_eq!(error.code, "MISSING_API_KEY");
    assert_eq!(error.message, "DeepSeek API Key 未配置");
}

#[tokio::test]
async fn ai_status_impl_reports_missing_key() {
    let (_dir, state) = init_state();

    let status = ai_status(&state).await.expect("expected status response");

    assert_eq!(status.mode, AiResponseSource::Online);
    assert!(!status.has_api_key);
    assert!(status.latency_ms.is_none());
    assert_eq!(status.message.as_deref(), Some("DeepSeek API Key 未配置"));
}
