use cognical_app_lib::error::AiErrorCode;
use cognical_app_lib::models::ai::{TaskParseContext, TaskParseRequest};
use cognical_app_lib::services::ai_service::testing::{map_http_error, parse_task_via_http};
use cognical_app_lib::services::prompt_templates::{
    build_recommendations_payload, build_schedule_payload, build_task_parse_payload,
};
use httpmock::prelude::*;
use reqwest::StatusCode;
use serde_json::json;
use std::time::Duration as StdDuration;

#[test]
fn build_task_parse_payload_includes_context_and_expectations() {
    let request = TaskParseRequest {
        input: "Finish the quarterly planning report".to_string(),
        context: Some(TaskParseContext {
            timezone: Some("Asia/Shanghai".into()),
            locale: Some("zh-CN".into()),
            reference_date: Some("2025-10-15T10:00:00Z".into()),
            metadata: Some(json!({"draftTitle": "Planning"})),
            existing_task_id: None,
            user_preferences: None,
        }),
    };

    let payload = build_task_parse_payload(&request);
    let obj = payload.as_object().expect("payload should be an object");

    assert_eq!(
        obj.get("operation").and_then(|v| v.as_str()),
        Some("parseTask")
    );
    assert_eq!(
        obj.get("input").and_then(|v| v.as_str()),
        Some(request.input.as_str())
    );

    let context = obj
        .get("context")
        .and_then(|value| value.as_object())
        .expect("context should be serialized");
    assert_eq!(
        context.get("timezone").and_then(|v| v.as_str()),
        Some("Asia/Shanghai")
    );

    let expectations = obj
        .get("expectations")
        .and_then(|value| value.as_object())
        .expect("expectations should exist");
    assert_eq!(
        expectations.get("minConfidence").and_then(|v| v.as_f64()),
        Some(0.5)
    );
    assert_eq!(
        expectations
            .get("languages")
            .and_then(|v| v.as_array())
            .map(|list| list.len()),
        Some(2)
    );
}

#[test]
fn build_recommendations_payload_has_defaults() {
    let input = json!({
        "tasks": [
            {"title": "Ship release", "priority": "high"},
            {"title": "Review docs", "priority": "medium"}
        ]
    });

    let payload = build_recommendations_payload(&input);
    let obj = payload.as_object().expect("payload should be object");

    assert_eq!(
        obj.get("operation").and_then(|value| value.as_str()),
        Some("generateRecommendations")
    );
    assert_eq!(obj.get("context"), Some(&input));

    let expectations = obj
        .get("expectations")
        .and_then(|value| value.as_object())
        .expect("expectations should be present");
    assert_eq!(
        expectations
            .get("maxRecommendations")
            .and_then(|value| value.as_u64()),
        Some(5)
    );
    assert_eq!(
        expectations
            .get("includeFollowUp")
            .and_then(|value| value.as_bool()),
        Some(true)
    );
}

#[test]
fn build_schedule_payload_has_expected_shape() {
    let input = json!({
        "tasks": [
            {"id": "task-1", "title": "Write summary", "estimatedMinutes": 90}
        ],
        "availability": {"startAt": "2025-10-15T09:00:00Z"}
    });

    let payload = build_schedule_payload(&input);
    let obj = payload.as_object().expect("payload should be object");

    assert_eq!(
        obj.get("operation").and_then(|value| value.as_str()),
        Some("planSchedule")
    );
    assert_eq!(obj.get("context"), Some(&input));

    let expectations = obj
        .get("expectations")
        .and_then(|value| value.as_object())
        .expect("expectations should be present");
    assert_eq!(
        expectations
            .get("granularity")
            .and_then(|value| value.as_str()),
        Some("30m")
    );
    assert_eq!(
        expectations
            .get("maxItems")
            .and_then(|value| value.as_u64()),
        Some(12)
    );
}

#[test]
fn deepseek_http_error_mapping_exposes_retry_semantics() {
    let (error, retryable) = map_http_error(StatusCode::UNAUTHORIZED);
    assert!(!retryable);
    assert_eq!(error.to_string(), "DeepSeek API Key 无效或未授权");
    assert_eq!(error.ai_code(), Some(AiErrorCode::MissingApiKey));
    assert_eq!(error.ai_correlation_id(), Some("test-correlation-id"));

    let (error, retryable) = map_http_error(StatusCode::FORBIDDEN);
    assert!(!retryable);
    assert_eq!(error.ai_code(), Some(AiErrorCode::Forbidden));
    assert_eq!(error.to_string(), "DeepSeek API 权限不足");
    assert_eq!(error.ai_correlation_id(), Some("test-correlation-id"));

    let (error, retryable) = map_http_error(StatusCode::TOO_MANY_REQUESTS);
    assert!(retryable);
    assert_eq!(error.to_string(), "DeepSeek 请求过于频繁，请稍后重试");
    assert_eq!(error.ai_code(), Some(AiErrorCode::RateLimited));

    let (error, retryable) = map_http_error(StatusCode::from_u16(503).unwrap());
    assert!(retryable);
    assert!(error
        .to_string()
        .contains("DeepSeek 服务暂时不可用 (状态码 503)"));
    assert_eq!(error.ai_code(), Some(AiErrorCode::DeepseekUnavailable));

    let (error, retryable) = map_http_error(StatusCode::NOT_FOUND);
    assert!(!retryable);
    assert_eq!(error.ai_code(), Some(AiErrorCode::InvalidRequest));
    assert_eq!(error.to_string(), "DeepSeek 接口地址无效");
    assert_eq!(error.ai_correlation_id(), Some("test-correlation-id"));

    let (error, retryable) = map_http_error(StatusCode::BAD_REQUEST);
    assert!(!retryable);
    assert_eq!(error.to_string(), "DeepSeek 请求格式无效");
    assert_eq!(error.ai_code(), Some(AiErrorCode::InvalidRequest));
}

#[tokio::test]
async fn deepseek_parse_task_surfaces_correlation_and_tokens() {
    let server = MockServer::start_async().await;

    let parsed_payload = json!({
        "payload": {"title": "整理周报"},
        "missingFields": [],
        "reasoning": {
            "summary": "整理本周工作亮点",
            "generatedAt": "2025-10-16T08:00:00Z",
            "source": "online"
        }
    });
    let content_string = serde_json::to_string(&parsed_payload).expect("valid JSON string");

    let _mock = server
        .mock_async(|when, then| {
            when.method(POST).path("/v1/chat/completions");
            then.status(200)
                .header("content-type", "application/json")
                .json_body(json!({
                    "choices": [{
                        "message": {"content": content_string}
                    }],
                    "usage": {
                        "prompt_tokens": 64,
                        "completion_tokens": 32,
                        "total_tokens": 96
                    }
                }));
        })
        .await;

    let request = TaskParseRequest {
        input: "整理本周周报".into(),
        context: None,
    };

    let dto = parse_task_via_http(&server.base_url(), StdDuration::from_secs(2), request)
        .await
        .expect("parse task succeeds");

    let provider = dto.reasoning.provider.expect("provider metadata");
    let extra = provider.extra.expect("provider extra metadata");
    let correlation = extra
        .get("correlationId")
        .and_then(|value| value.as_str())
        .expect("correlation id present");
    assert!(!correlation.is_empty());

    let tokens = provider.tokens_used.expect("token usage present");
    assert_eq!(tokens.get("prompt"), Some(&64));
    assert_eq!(tokens.get("completion"), Some(&32));
    assert_eq!(tokens.get("total"), Some(&96));
}

#[tokio::test]
async fn deepseek_parse_task_reports_invalid_json() {
    let server = MockServer::start_async().await;

    let _mock = server
        .mock_async(|when, then| {
            when.method(POST).path("/v1/chat/completions");
            then.status(200)
                .header("content-type", "application/json")
                .json_body(json!({
                    "choices": [{
                        "message": {"content": "not-json"}
                    }],
                    "usage": {}
                }));
        })
        .await;

    let request = TaskParseRequest {
        input: "need structured output".into(),
        context: None,
    };

    let error = parse_task_via_http(&server.base_url(), StdDuration::from_secs(2), request)
        .await
        .expect_err("should fail due to invalid JSON");

    assert_eq!(error.ai_code(), Some(AiErrorCode::InvalidResponse));
    assert!(error.ai_correlation_id().is_some());
    assert!(error
        .to_string()
        .contains("DeepSeek 响应内容非 JSON"));
}

#[tokio::test]
async fn deepseek_parse_task_maps_timeouts_to_http_timeout() {
    let server = MockServer::start_async().await;

    let parsed_payload = json!({
        "payload": {"title": "临时任务"},
        "missingFields": [],
        "reasoning": {
            "summary": "test",
            "generatedAt": "2025-10-16T08:00:00Z",
            "source": "online"
        }
    });
    let content_string = serde_json::to_string(&parsed_payload).expect("json");

    let _mock = server
        .mock_async(|when, then| {
            when.method(POST).path("/v1/chat/completions");
            then.status(200)
                .delay(StdDuration::from_millis(250))
                .header("content-type", "application/json")
                .json_body(json!({
                    "choices": [{
                        "message": {"content": content_string}
                    }],
                    "usage": {
                        "prompt_tokens": 1,
                        "completion_tokens": 1,
                        "total_tokens": 2
                    }
                }));
        })
        .await;

    let request = TaskParseRequest {
        input: "timeout".into(),
        context: None,
    };

    let error = parse_task_via_http(
        &server.base_url(),
        StdDuration::from_millis(100),
        request,
    )
    .await
    .expect_err("should timeout");

    assert_eq!(error.ai_code(), Some(AiErrorCode::HttpTimeout));
    assert!(error.ai_correlation_id().is_some());
}
