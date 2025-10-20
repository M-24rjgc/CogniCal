use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration as StdDuration, Instant};

use chrono::{Duration, Utc};
use serde_json::{json, Value as JsonValue};
use tokio::time::sleep;
use tracing::{debug, warn};

use crate::db::repositories::ai_settings_repository::AiSettingsRepository;
use crate::db::DbPool;
use crate::error::{AiErrorCode, AppError, AppResult};
use crate::models::ai::{TaskParseRequest, TaskParseResponse};
use crate::models::ai_types::{
    AiProvider, AiProviderMetadata, AiResponseSource, AiStatusDto, ParsedTaskDto,
    RecommendationDto, SchedulePlanDto,
};
use crate::services::cache_service::CacheService;
use crate::services::prompt_templates::{
    build_recommendations_payload, build_schedule_payload, build_task_parse_payload,
    recommendations_system_prompt, schedule_planning_system_prompt, task_parsing_system_prompt,
};
use crate::utils::crypto::CryptoVault;
use crate::utils::redact::redact_sensitive_data;
use crate::utils::semantic::semantic_hash;
use reqwest::StatusCode;
use uuid::Uuid;

#[derive(Clone)]
pub struct AiService {
    db_pool: DbPool,
    provider: Arc<RwLock<Option<Arc<DeepSeekProvider>>>>,
    cache: CacheService,
    config: Arc<RwLock<AiServiceConfig>>,
}

const KEY_DEEPSEEK_API: &str = "deepseek_api_key";

#[derive(Debug, Clone)]
struct AiServiceConfig {
    api_key: Option<String>,
    api_base_url: String,
    model: String,
    http_timeout: StdDuration,
    cache_ttl: Duration,
}

impl AiService {
    pub fn new(db_pool: DbPool) -> AppResult<Self> {
        let config = AiServiceConfig::load(&db_pool)?;
        let cache = CacheService::new(db_pool.clone(), config.cache_ttl)?;
        let provider = config.build_provider()?;

        Ok(Self {
            db_pool,
            provider: Arc::new(RwLock::new(provider)),
            cache,
            config: Arc::new(RwLock::new(config)),
        })
    }

    pub async fn parse_task(&self, request: TaskParseRequest) -> AppResult<TaskParseResponse> {
        let trimmed_input = request.input.trim();
        if trimmed_input.is_empty() {
            return Err(AppError::validation("待解析内容不能为空"));
        }

        self.refresh_configuration()?;

        let provider = self.current_provider()?;

        let metadata = request
            .context
            .as_ref()
            .and_then(|ctx| ctx.metadata.as_ref());
        let semantic_key = semantic_hash(trimmed_input, metadata);

        if let Some(cached_response) = self.cache.get_parse(&semantic_key).await? {
            debug!(target: "app::ai", "cache hit for semantic hash");
            return self.handle_cache_hit(cached_response, &semantic_key);
        }

        let mut parsed = provider.parse_task(&request).await?;
        self.enrich_parse_metadata(&mut parsed, &semantic_key, false);

        parsed
            .reasoning
            .generated_at
            .get_or_insert_with(|| Utc::now().to_rfc3339());
        parsed
            .reasoning
            .source
            .get_or_insert(AiResponseSource::Online);

        let response: TaskParseResponse = parsed.clone().into();

        self.cache
            .put_parse(&semantic_key, trimmed_input, response.clone())
            .await?;

        Ok(response)
    }

    pub async fn generate_recommendations(
        &self,
        payload: JsonValue,
    ) -> AppResult<RecommendationDto> {
        debug!(target: "app::ai", "generating recommendations");

        self.refresh_configuration()?;

        let provider = self.current_provider()?;
        let dto = provider.generate_recommendations(&payload).await?;

        Ok(dto)
    }

    pub async fn plan_schedule(&self, payload: JsonValue) -> AppResult<SchedulePlanDto> {
        debug!(target: "app::ai", "planning schedule recommendations");

        self.refresh_configuration()?;

        let provider = self.current_provider()?;
        let dto = provider.plan_schedule(&payload).await?;

        Ok(dto)
    }

    pub async fn status(&self) -> AppResult<AiStatusDto> {
        self.refresh_configuration()?;

        let has_api_key = {
            let guard = self.config.read().expect("config lock poisoned");
            guard.api_key.is_some()
        };

        let last_checked_at = Utc::now().to_rfc3339();
        if !has_api_key {
            return Ok(AiStatusDto {
                mode: AiResponseSource::Online,
                has_api_key: false,
                last_checked_at,
                latency_ms: None,
                provider: None,
                message: Some("DeepSeek API Key 未配置".to_string()),
            });
        }

        let provider = self.current_provider()?;

        match provider.ping().await {
            Ok(metadata) => {
                let latency_ms = metadata.latency_ms;
                Ok(AiStatusDto {
                    mode: AiResponseSource::Online,
                    has_api_key,
                    last_checked_at,
                    latency_ms,
                    provider: Some(metadata),
                    message: None,
                })
            }
            Err(error) => {
                warn!(
                    target: "app::ai",
                    error = %error,
                    "DeepSeek provider ping failed"
                );
                Err(error)
            }
        }
    }

    pub async fn chat(&self, message: String) -> AppResult<String> {
        debug!(target: "app::ai", message_len = message.len(), "chat invoked");

        self.refresh_configuration()?;
        let provider = self.current_provider()?;

        provider.chat(&message).await
    }

    fn refresh_configuration(&self) -> AppResult<()> {
        let config = AiServiceConfig::load(&self.db_pool)?;

        let mut provider_update: Option<Option<Arc<DeepSeekProvider>>> = None;

        {
            let mut current = self.config.write().expect("config lock poisoned");
            if current.differs_from(&config) {
                provider_update = Some(config.build_provider()?);
                *current = config;
            } else {
                *current = config;
            }
        }

        if let Some(update) = provider_update {
            let mut guard = self.provider.write().expect("provider lock poisoned");
            *guard = update;
        }

        Ok(())
    }

    fn current_provider(&self) -> AppResult<Arc<DeepSeekProvider>> {
        let guard = self.provider.read().expect("provider lock poisoned");
        guard
            .as_ref()
            .cloned()
            .ok_or_else(|| AppError::ai(AiErrorCode::MissingApiKey, "DeepSeek API Key 未配置"))
    }

    pub fn has_configured_provider(&self, _conn: &rusqlite::Connection) -> AppResult<bool> {
        self.refresh_configuration()?;
        let guard = self.provider.read().expect("provider lock poisoned");
        Ok(guard.is_some())
    }

    fn handle_cache_hit(
        &self,
        cached_response: TaskParseResponse,
        semantic_key: &str,
    ) -> AppResult<TaskParseResponse> {
        let mut dto = ParsedTaskDto::from(cached_response);
        self.enrich_parse_metadata(&mut dto, semantic_key, true);
        dto.reasoning.source = Some(AiResponseSource::Cache);
        Ok(TaskParseResponse::from(dto))
    }

    fn enrich_parse_metadata(
        &self,
        parsed: &mut ParsedTaskDto,
        semantic_key: &str,
        cache_hit: bool,
    ) {
        let mut metadata = parsed
            .reasoning
            .metadata
            .take()
            .unwrap_or_else(|| json!({}));

        if !metadata.is_object() {
            metadata = json!({ "value": metadata });
        }

        if let Some(map) = metadata.as_object_mut() {
            map.insert("semanticHash".to_string(), json!(semantic_key));
            map.insert("cacheHit".to_string(), json!(cache_hit));

            if cache_hit {
                map.insert("cacheHitAt".to_string(), json!(Utc::now().to_rfc3339()));
            } else {
                map.entry("cachedAt".to_string())
                    .or_insert_with(|| json!(Utc::now().to_rfc3339()));
            }
        }

        parsed.reasoning.metadata = Some(metadata);
    }
}

impl AiServiceConfig {
    fn from_env() -> Self {
        let api_key = std::env::var("COGNICAL_DEEPSEEK_API_KEY").ok();
        let api_base_url = std::env::var("COGNICAL_DEEPSEEK_BASE_URL")
            .ok()
            .unwrap_or_else(|| "https://api.deepseek.com".to_string());
        let model = std::env::var("COGNICAL_DEEPSEEK_MODEL")
            .ok()
            .unwrap_or_else(|| "deepseek-chat".to_string());

        Self {
            api_key,
            api_base_url,
            model,
            http_timeout: StdDuration::from_secs(30),
            cache_ttl: Duration::days(7),
        }
    }

    fn load(db_pool: &DbPool) -> AppResult<Self> {
        let mut config = Self::from_env();

        if config.api_key.is_none() {
            let vault = CryptoVault::from_database_path(db_pool.path())?;
            let stored = db_pool
                .with_connection(|conn| AiSettingsRepository::get(conn, KEY_DEEPSEEK_API))?;

            if let Some(row) = stored {
                match vault.decrypt(&row.value) {
                    Ok(bytes) => match String::from_utf8(bytes) {
                        Ok(value) => {
                            if !value.trim().is_empty() {
                                config.api_key = Some(value);
                            }
                        }
                        Err(err) => {
                            warn!(
                                target: "app::ai",
                                error = %err,
                                "failed to decode stored DeepSeek API key"
                            );
                        }
                    },
                    Err(err) => {
                        warn!(
                            target: "app::ai",
                            error = %err,
                            "failed to decrypt stored DeepSeek API key"
                        );
                    }
                }
            }
        }

        if let Some(value) = config.api_key.take() {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                config.api_key = None;
            } else {
                config.api_key = Some(trimmed.to_string());
            }
        }

        Ok(config)
    }

    fn differs_from(&self, other: &Self) -> bool {
        self.api_key != other.api_key
            || self.api_base_url != other.api_base_url
            || self.model != other.model
            || self.http_timeout != other.http_timeout
            || self.cache_ttl != other.cache_ttl
    }

    fn build_provider(&self) -> AppResult<Option<Arc<DeepSeekProvider>>> {
        match &self.api_key {
            Some(api_key) => {
                let provider = DeepSeekProvider::try_new(self, api_key.clone())?;
                Ok(Some(Arc::new(provider)))
            }
            None => Ok(None),
        }
    }
}

struct DeepSeekProvider {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    endpoint: String,
    model: String,
}

#[derive(Clone, Copy)]
enum DeepSeekOperation {
    ParseTask,
    Recommendations,
    Schedule,
}

impl DeepSeekOperation {
    fn as_str(self) -> &'static str {
        match self {
            DeepSeekOperation::ParseTask => "parseTask",
            DeepSeekOperation::Recommendations => "generateRecommendations",
            DeepSeekOperation::Schedule => "planSchedule",
        }
    }

    fn system_prompt(self) -> &'static str {
        match self {
            DeepSeekOperation::ParseTask => task_parsing_system_prompt(),
            DeepSeekOperation::Recommendations => recommendations_system_prompt(),
            DeepSeekOperation::Schedule => schedule_planning_system_prompt(),
        }
    }

    fn temperature(self) -> f32 {
        match self {
            DeepSeekOperation::ParseTask => 0.2,
            DeepSeekOperation::Recommendations => 0.4,
            DeepSeekOperation::Schedule => 0.3,
        }
    }
}

struct ChatInvocationResult {
    content: JsonValue,
    tokens_used: HashMap<String, u64>,
    latency_ms: u128,
    correlation_id: String,
}

impl DeepSeekProvider {
    fn try_new(config: &AiServiceConfig, api_key: String) -> AppResult<Self> {
        let client = reqwest::Client::builder()
            .timeout(config.http_timeout)
            .pool_max_idle_per_host(2)
            .pool_idle_timeout(Some(StdDuration::from_secs(90)))
            .build()
            .map_err(|err| AppError::other(format!("初始化 DeepSeek HTTP 客户端失败: {err}")))?;

        let base_url = config.api_base_url.trim_end_matches('/').to_string();
        let endpoint = format!("{}/v1/chat/completions", base_url);

        Ok(Self {
            client,
            api_key,
            base_url,
            endpoint,
            model: config.model.clone(),
        })
    }

    async fn invoke_chat(
        &self,
        operation: DeepSeekOperation,
        payload: JsonValue,
    ) -> AppResult<ChatInvocationResult> {
        let correlation_id = Uuid::new_v4().to_string();
        let sanitized_payload = redact_sensitive_data(&payload)
            .unwrap_or_else(|_| JsonValue::String("<redacted>".to_string()));
        let sanitized_payload_str = serde_json::to_string(&sanitized_payload)
            .unwrap_or_else(|_| "\"<redacted>\"".to_string());

        let request_body = self.build_request_body(operation, &payload);
        let backoff_schedule = [
            StdDuration::from_secs(0),
            StdDuration::from_secs(1),
            StdDuration::from_secs(2),
            StdDuration::from_secs(4),
        ];

        let mut last_error: Option<AppError> = None;

        for (attempt, delay) in backoff_schedule.iter().enumerate() {
            if *delay > StdDuration::from_secs(0) {
                sleep(*delay).await;
            }

            debug!(
                target: "app::ai::deepseek",
                operation = operation.as_str(),
                attempt = attempt + 1,
                correlation_id = %correlation_id,
                payload = %sanitized_payload_str,
                "invoking DeepSeek"
            );

            let start = Instant::now();
            let response = self
                .client
                .post(&self.endpoint)
                .bearer_auth(&self.api_key)
                .json(&request_body)
                .send()
                .await;

            match response {
                Ok(resp) => {
                    let status = resp.status();
                    if status.is_success() {
                        let latency_ms = start.elapsed().as_millis();

                        let content_length = resp.content_length();
                        let content_type = resp
                            .headers()
                            .get("content-type")
                            .and_then(|v| v.to_str().ok())
                            .unwrap_or("unknown");

                        debug!(
                            target: "app::ai::deepseek",
                            correlation_id = %correlation_id,
                            latency_ms,
                            content_length = ?content_length,
                            content_type = %content_type,
                            "DeepSeek responded"
                        );

                        let body: JsonValue = resp.json().await.map_err(|err| {
                            AppError::ai_with_details(
                                AiErrorCode::InvalidResponse,
                                "解析 DeepSeek 响应失败",
                                Some(correlation_id.as_str()),
                                Some(json!({ "reason": err.to_string() })),
                            )
                        })?;

                        let content = body
                            .pointer("/choices/0/message/content")
                            .and_then(|value| value.as_str())
                            .ok_or_else(|| {
                                AppError::ai_with_details(
                                    AiErrorCode::InvalidResponse,
                                    "DeepSeek 响应缺少 message.content 字段",
                                    Some(correlation_id.as_str()),
                                    Some(json!({ "reason": "missing_message_content" })),
                                )
                            })?;
                        let content_value = Self::parse_content(content, &correlation_id)?;
                        let tokens_used = Self::extract_tokens(&body);

                        return Ok(ChatInvocationResult {
                            content: content_value,
                            tokens_used,
                            latency_ms,
                            correlation_id,
                        });
                    }

                    let (error, retryable) = Self::map_http_error(status, correlation_id.as_str());
                    warn!(
                        target: "app::ai::deepseek",
                        correlation_id = %correlation_id,
                        status = status.as_u16(),
                        retryable,
                        "DeepSeek 返回非成功状态"
                    );

                    if !retryable || attempt == backoff_schedule.len() - 1 {
                        return Err(error);
                    }

                    last_error = Some(error);
                    continue;
                }
                Err(err) => {
                    let (error, retryable) = Self::error_from_reqwest(err, correlation_id.as_str());
                    warn!(
                        target: "app::ai::deepseek",
                        correlation_id = %correlation_id,
                        retryable,
                        "DeepSeek 请求错误"
                    );

                    if !retryable || attempt == backoff_schedule.len() - 1 {
                        return Err(error);
                    }

                    last_error = Some(error);
                    continue;
                }
            }
        }

        if let Some(error) = last_error {
            Err(error)
        } else {
            Err(AppError::ai_with_details(
                AiErrorCode::DeepseekUnavailable,
                "DeepSeek 请求失败",
                Some(correlation_id.as_str()),
                None,
            ))
        }
    }

    fn build_request_body(&self, operation: DeepSeekOperation, payload: &JsonValue) -> JsonValue {
        let user_content = serde_json::to_string(payload).unwrap_or_else(|_| "{}".to_string());
        json!({
            "model": self.model,
            "temperature": operation.temperature(),
            "top_p": 0.9,
            "response_format": { "type": "json_object" },
            "messages": [
                { "role": "system", "content": operation.system_prompt() },
                { "role": "user", "content": user_content }
            ]
        })
    }

    fn parse_content(content: &str, correlation_id: &str) -> AppResult<JsonValue> {
        let trimmed = content.trim();
        let cleaned = if trimmed.starts_with("```") {
            let without_prefix = trimmed
                .trim_start_matches("```json")
                .trim_start_matches("```JSON")
                .trim_start_matches("```");
            let without_suffix = without_prefix.trim_end_matches("```").trim();
            without_suffix.to_string()
        } else {
            trimmed.to_string()
        };

        serde_json::from_str(&cleaned).map_err(|err| {
            AppError::ai_with_details(
                AiErrorCode::InvalidResponse,
                format!("DeepSeek 响应内容非 JSON: {err}"),
                Some(correlation_id),
                Some(json!({ "reason": "invalid_json" })),
            )
        })
    }

    fn extract_tokens(body: &JsonValue) -> HashMap<String, u64> {
        let mut tokens = HashMap::new();

        if let Some(usage) = body.get("usage") {
            if let Some(value) = usage.get("prompt_tokens").and_then(|v| v.as_u64()) {
                tokens.insert("prompt".to_string(), value);
            }
            if let Some(value) = usage.get("completion_tokens").and_then(|v| v.as_u64()) {
                tokens.insert("completion".to_string(), value);
            }
            if let Some(value) = usage.get("total_tokens").and_then(|v| v.as_u64()) {
                tokens.insert("total".to_string(), value);
            }
        }

        tokens
    }

    fn build_provider_metadata(
        &self,
        tokens_used: HashMap<String, u64>,
        latency_ms: u128,
        correlation_id: Option<&str>,
    ) -> AiProviderMetadata {
        AiProviderMetadata {
            provider_id: Some("deepseek".to_string()),
            model: Some(self.model.clone()),
            latency_ms: Some(latency_ms),
            tokens_used: if tokens_used.is_empty() {
                None
            } else {
                Some(tokens_used)
            },
            extra: correlation_id.map(|id| json!({ "correlationId": id })),
        }
    }

    fn merge_metadata(
        existing: Option<AiProviderMetadata>,
        fallback: AiProviderMetadata,
    ) -> Option<AiProviderMetadata> {
        match existing {
            Some(mut meta) => {
                if meta.provider_id.is_none() {
                    meta.provider_id = fallback.provider_id;
                }
                if meta.model.is_none() {
                    meta.model = fallback.model;
                }
                if meta.latency_ms.is_none() {
                    meta.latency_ms = fallback.latency_ms;
                }

                match (meta.tokens_used.as_mut(), fallback.tokens_used) {
                    (Some(meta_tokens), Some(mut fallback_tokens)) => {
                        for (key, value) in fallback_tokens.drain() {
                            meta_tokens.entry(key).or_insert(value);
                        }
                    }
                    (None, tokens) => {
                        meta.tokens_used = tokens;
                    }
                    _ => {}
                }

                if meta.extra.is_none() {
                    meta.extra = fallback.extra;
                }

                Some(meta)
            }
            None => Some(fallback),
        }
    }

    fn map_http_error(status: StatusCode, correlation_id: &str) -> (AppError, bool) {
        match status {
            StatusCode::UNAUTHORIZED => (
                AppError::ai_with_details(
                    AiErrorCode::MissingApiKey,
                    "DeepSeek API Key 无效或未授权",
                    Some(correlation_id),
                    None,
                ),
                false,
            ),
            StatusCode::FORBIDDEN => (
                AppError::ai_with_details(
                    AiErrorCode::Forbidden,
                    "DeepSeek API 权限不足",
                    Some(correlation_id),
                    None,
                ),
                false,
            ),
            StatusCode::TOO_MANY_REQUESTS => (
                AppError::ai_with_details(
                    AiErrorCode::RateLimited,
                    "DeepSeek 请求过于频繁，请稍后重试",
                    Some(correlation_id),
                    None,
                ),
                true,
            ),
            status if status.is_server_error() => (
                AppError::ai_with_details(
                    AiErrorCode::DeepseekUnavailable,
                    format!("DeepSeek 服务暂时不可用 (状态码 {})", status.as_u16()),
                    Some(correlation_id),
                    None,
                ),
                true,
            ),
            StatusCode::BAD_REQUEST => (
                AppError::ai_with_details(
                    AiErrorCode::InvalidRequest,
                    "DeepSeek 请求格式无效",
                    Some(correlation_id),
                    None,
                ),
                false,
            ),
            StatusCode::NOT_FOUND => (
                AppError::ai_with_details(
                    AiErrorCode::InvalidRequest,
                    "DeepSeek 接口地址无效",
                    Some(correlation_id),
                    None,
                ),
                false,
            ),
            status => (
                AppError::ai_with_details(
                    AiErrorCode::Unknown,
                    format!("DeepSeek 返回错误状态码 {}", status.as_u16()),
                    Some(correlation_id),
                    None,
                ),
                false,
            ),
        }
    }

    fn error_from_reqwest(err: reqwest::Error, correlation_id: &str) -> (AppError, bool) {
        if err.is_timeout() {
            (
                AppError::ai_with_details(
                    AiErrorCode::HttpTimeout,
                    "DeepSeek 请求超时",
                    Some(correlation_id),
                    None,
                ),
                true,
            )
        } else if err.is_connect() {
            (
                AppError::ai_with_details(
                    AiErrorCode::DeepseekUnavailable,
                    "DeepSeek 网络连接失败",
                    Some(correlation_id),
                    None,
                ),
                true,
            )
        } else if let Some(status) = err.status() {
            Self::map_http_error(status, correlation_id)
        } else {
            (
                AppError::ai_with_details(
                    AiErrorCode::Unknown,
                    format!("DeepSeek 请求失败: {err}"),
                    Some(correlation_id),
                    None,
                ),
                false,
            )
        }
    }

    async fn chat(&self, message: &str) -> AppResult<String> {
        let correlation_id = Uuid::new_v4().to_string();
        
        let request_body = json!({
            "model": self.model,
            "messages": [
                {
                    "role": "system",
                    "content": "你是一个专业的任务管理和时间规划助手。你可以帮助用户提高工作效率、制定计划、解答问题。请用简洁、友好的方式回答用户的问题。"
                },
                {
                    "role": "user",
                    "content": message
                }
            ],
            "temperature": 0.7,
            "max_tokens": 2000
        });

        debug!(
            target: "app::ai::deepseek",
            correlation_id = %correlation_id,
            message_len = message.len(),
            "invoking DeepSeek chat"
        );

        let start = Instant::now();
        let response = self
            .client
            .post(&self.endpoint)
            .bearer_auth(&self.api_key)
            .json(&request_body)
            .send()
            .await;

        match response {
            Ok(resp) => {
                let status = resp.status();
                let latency_ms = start.elapsed().as_millis();

                if !status.is_success() {
                    let (error, _) = Self::map_http_error(status, correlation_id.as_str());
                    warn!(
                        target: "app::ai::deepseek",
                        correlation_id = %correlation_id,
                        status = status.as_u16(),
                        latency_ms,
                        "DeepSeek chat returned non-success status"
                    );
                    return Err(error);
                }

                let body: JsonValue = resp.json().await.map_err(|err| {
                    AppError::ai(
                        AiErrorCode::InvalidResponse,
                        format!("解析 DeepSeek 响应失败: {err}"),
                    )
                })?;

                let content = body["choices"][0]["message"]["content"]
                    .as_str()
                    .ok_or_else(|| {
                        AppError::ai(
                            AiErrorCode::InvalidResponse,
                            "DeepSeek 响应中缺少消息内容",
                        )
                    })?
                    .to_string();

                debug!(
                    target: "app::ai::deepseek",
                    correlation_id = %correlation_id,
                    latency_ms,
                    response_len = content.len(),
                    "DeepSeek chat completed"
                );

                Ok(content)
            }
            Err(err) => {
                let (error, _) = Self::error_from_reqwest(err, correlation_id.as_str());
                warn!(
                    target: "app::ai::deepseek",
                    correlation_id = %correlation_id,
                    "DeepSeek chat request failed"
                );
                Err(error)
            }
        }
    }
}

pub mod testing {
    use super::*;
    use std::time::Duration as StdDurationOverride;

    /// Expose DeepSeek error mapping for integration tests without widening the public API surface.
    pub fn map_http_error(status: StatusCode) -> (AppError, bool) {
        DeepSeekProvider::map_http_error(status, "test-correlation-id")
    }

    pub async fn parse_task_via_http(
        base_url: &str,
        timeout: StdDurationOverride,
        request: TaskParseRequest,
    ) -> AppResult<ParsedTaskDto> {
        let config = AiServiceConfig {
            api_key: Some("test-key".to_string()),
            api_base_url: base_url.trim_end_matches('/').to_string(),
            model: "deepseek-chat".to_string(),
            http_timeout: timeout,
            cache_ttl: Duration::minutes(5),
        };
        let provider = DeepSeekProvider::try_new(&config, "test-key".to_string())?;
        provider.parse_task(&request).await
    }
}

#[async_trait::async_trait]
impl AiProvider for DeepSeekProvider {
    async fn parse_task(&self, request: &TaskParseRequest) -> AppResult<ParsedTaskDto> {
        let payload = build_task_parse_payload(request);
        let result = self
            .invoke_chat(DeepSeekOperation::ParseTask, payload)
            .await?;

        let ChatInvocationResult {
            content,
            tokens_used,
            latency_ms,
            correlation_id,
        } = result;

        let mut dto: ParsedTaskDto = serde_json::from_value(content.clone()).map_err(|err| {
            tracing::error!(
                target: "app::ai",
                correlation_id = %correlation_id,
                error = %err,
                response = ?content,
                "Failed to parse DeepSeek task response"
            );
            AppError::ai_with_details(
                AiErrorCode::InvalidResponse,
                format!("解析 DeepSeek 任务解析响应失败: {err}"),
                Some(correlation_id.as_str()),
                None,
            )
        })?;

        let metadata =
            self.build_provider_metadata(tokens_used, latency_ms, Some(correlation_id.as_str()));
        let existing = dto.reasoning.provider.take();
        dto.reasoning.provider = Self::merge_metadata(existing, metadata);
        dto.reasoning.source = Some(AiResponseSource::Online);
        dto.reasoning
            .generated_at
            .get_or_insert_with(|| Utc::now().to_rfc3339());

        Ok(dto)
    }

    async fn generate_recommendations(&self, input: &JsonValue) -> AppResult<RecommendationDto> {
        let payload = build_recommendations_payload(input);
        let result = self
            .invoke_chat(DeepSeekOperation::Recommendations, payload)
            .await?;

        let ChatInvocationResult {
            content,
            tokens_used,
            latency_ms,
            correlation_id,
        } = result;

        let mut dto: RecommendationDto = serde_json::from_value(content).map_err(|err| {
            AppError::ai_with_details(
                AiErrorCode::InvalidResponse,
                format!("解析 DeepSeek 推荐响应失败: {err}"),
                Some(correlation_id.as_str()),
                None,
            )
        })?;

        let metadata =
            self.build_provider_metadata(tokens_used, latency_ms, Some(correlation_id.as_str()));
        let existing = dto.telemetry.take();
        dto.telemetry = Self::merge_metadata(existing, metadata);

        Ok(dto)
    }

    async fn plan_schedule(&self, input: &JsonValue) -> AppResult<SchedulePlanDto> {
        let payload = build_schedule_payload(input);
        let result = self
            .invoke_chat(DeepSeekOperation::Schedule, payload)
            .await?;

        let ChatInvocationResult {
            content,
            tokens_used,
            latency_ms,
            correlation_id,
        } = result;

        let mut dto: SchedulePlanDto = serde_json::from_value(content).map_err(|err| {
            AppError::ai_with_details(
                AiErrorCode::InvalidResponse,
                format!("解析 DeepSeek 排程响应失败: {err}"),
                Some(correlation_id.as_str()),
                None,
            )
        })?;

        let metadata =
            self.build_provider_metadata(tokens_used, latency_ms, Some(correlation_id.as_str()));
        let existing = dto.telemetry.take();
        dto.telemetry = Self::merge_metadata(existing, metadata);

        Ok(dto)
    }

    async fn ping(&self) -> AppResult<AiProviderMetadata> {
        let url = format!("{}/v1/models", self.base_url);
        let start = Instant::now();
        let correlation_id = Uuid::new_v4().to_string();
        let response = self
            .client
            .get(&url)
            .bearer_auth(&self.api_key)
            .send()
            .await;

        match response {
            Ok(resp) => {
                let status = resp.status();
                if status.is_success() {
                    let latency_ms = start.elapsed().as_millis();
                    Ok(self.build_provider_metadata(
                        HashMap::new(),
                        latency_ms,
                        Some(correlation_id.as_str()),
                    ))
                } else {
                    let (error, _) = Self::map_http_error(status, correlation_id.as_str());
                    warn!(
                        target: "app::ai::deepseek",
                        correlation_id = %correlation_id,
                        status = status.as_u16(),
                        "DeepSeek ping returned non-success status"
                    );
                    Err(error)
                }
            }
            Err(err) => {
                let (error, _) = Self::error_from_reqwest(err, correlation_id.as_str());
                warn!(
                    target: "app::ai::deepseek",
                    correlation_id = %correlation_id,
                    "DeepSeek ping request failed"
                );
                Err(error)
            }
        }
    }
}
