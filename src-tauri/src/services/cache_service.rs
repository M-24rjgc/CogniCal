use std::sync::Arc;

use chrono::{Duration, Utc};
use rusqlite::{Connection, OptionalExtension};
use tauri::async_runtime;
use tracing::debug;

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::ai::TaskParseResponse;
use crate::services::ai_cache::{AiCacheKey, AiCacheOperation};

const CACHE_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS ai_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_ai_settings_updated_at
    ON ai_settings(updated_at);

CREATE TABLE IF NOT EXISTS ai_cache (
    cache_key TEXT PRIMARY KEY,
    operation TEXT NOT NULL CHECK(operation IN ('parse','recommend','schedule')),
    semantic_hash TEXT NOT NULL,
    raw_input TEXT NOT NULL,
    response_json TEXT NOT NULL,
    created_at TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    hit_count INTEGER NOT NULL DEFAULT 0,
    metadata_json TEXT
);
CREATE INDEX IF NOT EXISTS idx_ai_cache_operation
    ON ai_cache(operation);
CREATE INDEX IF NOT EXISTS idx_ai_cache_semantic_hash
    ON ai_cache(semantic_hash);
CREATE INDEX IF NOT EXISTS idx_ai_cache_expires_at
    ON ai_cache(expires_at);
"#;

#[derive(Debug, Clone)]
pub struct CacheService {
    db: Arc<DbPool>,
    ttl: Duration,
}

impl CacheService {
    pub fn new(db: DbPool, ttl: Duration) -> AppResult<Self> {
        let service = Self {
            db: Arc::new(db),
            ttl,
        };
        service.bootstrap()?;
        Ok(service)
    }

    pub async fn get_parse(&self, semantic_hash: &str) -> AppResult<Option<TaskParseResponse>> {
        let key = AiCacheKey::new(AiCacheOperation::ParseTask, semantic_hash.to_string());
        self.get_task_response(key).await
    }

    pub async fn put_parse(
        &self,
        semantic_hash: &str,
        raw_input: &str,
        response: TaskParseResponse,
    ) -> AppResult<()> {
        let key = AiCacheKey::new(AiCacheOperation::ParseTask, semantic_hash.to_string());
        self.put_task_response(key, raw_input, response).await
    }

    pub async fn purge_expired(&self) -> AppResult<()> {
        let db = Arc::clone(&self.db);
        async_runtime::spawn_blocking(move || {
            let conn = db.get_connection()?;
            ensure_schema(&conn)?;
            let now = Utc::now().to_rfc3339();
            let deleted = conn.execute("DELETE FROM ai_cache WHERE expires_at <= ?1", [&now])?;
            if deleted > 0 {
                debug!(target: "app::ai::cache", deleted, "purged expired cache entries");
            }
            Ok(())
        })
        .await
        .map_err(|err| AppError::other(format!("缓存清理失败: {err}")))?
    }

    fn bootstrap(&self) -> AppResult<()> {
        self.db.with_connection(|conn| {
            ensure_schema(conn)?;
            Ok(())
        })
    }

    async fn get_task_response(&self, key: AiCacheKey) -> AppResult<Option<TaskParseResponse>> {
        let cache_key: String = (&key).into();
        let db = Arc::clone(&self.db);

        async_runtime::spawn_blocking(move || {
            let conn = db.get_connection()?;
            ensure_schema(&conn)?;

            let now = Utc::now().to_rfc3339();
            let mut stmt = conn.prepare(
                "SELECT response_json FROM ai_cache WHERE cache_key = ?1 AND expires_at > ?2",
            )?;

            let result = stmt
                .query_row([&cache_key, &now], |row| row.get::<_, String>(0))
                .optional()?;

            if let Some(payload) = result {
                let response: TaskParseResponse =
                    serde_json::from_str(&payload).map_err(|err| {
                        rusqlite::Error::FromSqlConversionFailure(
                            0,
                            rusqlite::types::Type::Text,
                            Box::new(err),
                        )
                    })?;

                conn.execute(
                    "UPDATE ai_cache SET hit_count = hit_count + 1 WHERE cache_key = ?1",
                    [&cache_key],
                )?;

                debug!(
                    target: "app::ai::cache",
                    cache_key = %cache_key,
                    operation = key.operation().as_str(),
                    "cache hit"
                );

                Ok(Some(response))
            } else {
                Ok(None)
            }
        })
        .await
        .map_err(|err| AppError::other(format!("缓存查询失败: {err}")))?
    }

    async fn put_task_response(
        &self,
        key: AiCacheKey,
        raw_input: &str,
        mut response: TaskParseResponse,
    ) -> AppResult<()> {
        let cache_key: String = (&key).into();
        let operation = key.operation().as_str().to_string();
        let semantic_hash = key.semantic_hash().to_string();
        let input = raw_input.to_string();
        let db = Arc::clone(&self.db);
        let ttl = self.ttl;

        async_runtime::spawn_blocking(move || {
            let conn = db.get_connection()?;
            ensure_schema(&conn)?;

            let now = Utc::now();
            if response.ai.generated_at.is_empty() {
                response.ai.generated_at = now.to_rfc3339();
            }

            let expires_at = now + ttl;
            let response_json = serde_json::to_string(&response)?;
            let metadata_json = response
                .ai
                .metadata
                .clone()
                .and_then(|value| serde_json::to_string(&value).ok());

            conn.execute(
                r#"
                INSERT INTO ai_cache (
                    cache_key,
                    operation,
                    semantic_hash,
                    raw_input,
                    response_json,
                    created_at,
                    expires_at,
                    hit_count,
                    metadata_json
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0, ?8)
                ON CONFLICT(cache_key) DO UPDATE SET
                    operation = excluded.operation,
                    semantic_hash = excluded.semantic_hash,
                    raw_input = excluded.raw_input,
                    response_json = excluded.response_json,
                    created_at = excluded.created_at,
                    expires_at = excluded.expires_at,
                    metadata_json = excluded.metadata_json
                "#,
                (
                    &cache_key,
                    &operation,
                    &semantic_hash,
                    &input,
                    &response_json,
                    now.to_rfc3339(),
                    expires_at.to_rfc3339(),
                    metadata_json,
                ),
            )?;

            debug!(
                target: "app::ai::cache",
                cache_key = %cache_key,
                operation = %operation,
                "cached ai response"
            );

            Ok(())
        })
        .await
        .map_err(|err| AppError::other(format!("缓存写入失败: {err}")))?
    }
}

fn ensure_schema(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(CACHE_SCHEMA)?;
    Ok(())
}
