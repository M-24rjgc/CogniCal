use cognical_app_lib::db::DbPool;
use cognical_app_lib::models::ai::{TaskAiSource, TaskParseRequest};
use cognical_app_lib::services::ai_service::AiService;
use serde_json::Value;
use tempfile::tempdir;

fn clear_ai_env() {
    std::env::remove_var("COGNICAL_DEEPSEEK_API_KEY");
    std::env::remove_var("COGNICAL_DEEPSEEK_BASE_URL");
    std::env::remove_var("COGNICAL_DEEPSEEK_MODEL");
}

#[tokio::test]
async fn parse_task_uses_offline_engine_without_api_key_and_caches_results() {
    clear_ai_env();

    let dir = tempdir().expect("temp dir");
    let db_path = dir.path().join("ai-cache.sqlite");
    let pool = DbPool::new(db_path).expect("db pool");
    let service = AiService::new(pool).expect("ai service");

    let request = TaskParseRequest {
        input: "写一个 README 的总结".into(),
        context: None,
    };

    let first = service
        .parse_task(request.clone())
        .await
        .expect("first parse");

    assert_eq!(first.ai.source, TaskAiSource::Live);
    let metadata = first.ai.metadata.expect("metadata");
    assert_eq!(value_at(&metadata, &["provider", "mode"]).as_str(), Some("offline"));
    assert!(value_at(&metadata, &["semanticHash"]).as_str().is_some());

    let second = service
        .parse_task(request)
        .await
        .expect("second parse");

    assert_eq!(second.ai.source, TaskAiSource::Cache);
    let cache_hit = value_at(
        &second.ai.metadata.expect("cached metadata"),
        &["cacheHit"],
    );
    assert!(cache_hit.get("hitAt").and_then(Value::as_str).is_some());
    assert!(cache_hit.get("semanticHash").and_then(Value::as_str).is_some());
    
        let status = service.status().await.expect("status without api key");
        assert!(!status.has_api_key);
        assert_eq!(status.message.as_deref(), Some("DeepSeek API Key 未配置"));
}

fn value_at<'a>(value: &'a Value, path: &[&str]) -> &'a Value {
    path.iter()
        .fold(value, |current, key| current.get(key).unwrap_or(&Value::Null))
}
