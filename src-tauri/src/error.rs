use std::fmt;

use rusqlite;
use serde_json::Value as JsonValue;
use thiserror::Error;
use tracing::{error, warn};

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiErrorCode {
    MissingApiKey,
    Forbidden,
    HttpTimeout,
    RateLimited,
    InvalidResponse,
    InvalidRequest,
    DeepseekUnavailable,
    Unknown,
}

impl AiErrorCode {
    pub fn as_str(self) -> &'static str {
        match self {
            AiErrorCode::MissingApiKey => "MISSING_API_KEY",
            AiErrorCode::Forbidden => "FORBIDDEN",
            AiErrorCode::HttpTimeout => "HTTP_TIMEOUT",
            AiErrorCode::RateLimited => "RATE_LIMITED",
            AiErrorCode::InvalidResponse => "INVALID_RESPONSE",
            AiErrorCode::InvalidRequest => "INVALID_REQUEST",
            AiErrorCode::DeepseekUnavailable => "DEEPSEEK_UNAVAILABLE",
            AiErrorCode::Unknown => "UNKNOWN_AI_ERROR",
        }
    }
}

impl fmt::Display for AiErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("数据库错误: {message}")]
    Database { message: String },

    #[error("记录未找到")]
    NotFound,

    #[error("记录冲突: {message}")]
    Conflict { message: String },

    #[error("验证失败: {message}")]
    Validation {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
        details: Option<JsonValue>,
    },

    #[error("{message}")]
    Ai {
        code: AiErrorCode,
        message: String,
        correlation_id: Option<String>,
        details: Option<JsonValue>,
    },

    #[error("序列化错误: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),
}

impl AppError {
    pub fn validation(message: impl Into<String>) -> Self {
        let message = message.into();
        warn!(target: "app::validation", %message, "validation error");
        AppError::Validation {
            message,
            source: None,
            details: None,
        }
    }

    pub fn validation_with_details(message: impl Into<String>, details: JsonValue) -> Self {
        let message = message.into();
        warn!(target: "app::validation", %message, details = %details, "validation error with details");
        AppError::Validation {
            message,
            source: None,
            details: Some(details),
        }
    }

    pub fn ai(code: AiErrorCode, message: impl Into<String>) -> Self {
        Self::ai_with_details(code, message, None, None)
    }

    pub fn ai_with_details(
        code: AiErrorCode,
        message: impl Into<String>,
        correlation_id: Option<&str>,
        details: Option<JsonValue>,
    ) -> Self {
        let message = message.into();
        let correlation = correlation_id.map(|value| value.to_string());
        match (&correlation, &details) {
            (Some(id), Some(payload)) => {
                warn!(
                    target: "app::ai::error",
                    code = %code,
                    correlation_id = %id,
                    details = %payload,
                    %message
                );
            }
            (Some(id), None) => {
                warn!(
                    target: "app::ai::error",
                    code = %code,
                    correlation_id = %id,
                    %message
                );
            }
            (None, Some(payload)) => {
                warn!(target: "app::ai::error", code = %code, details = %payload, %message);
            }
            (None, None) => {
                warn!(target: "app::ai::error", code = %code, %message);
            }
        }

        AppError::Ai {
            code,
            message,
            correlation_id: correlation,
            details,
        }
    }

    pub fn ai_code(&self) -> Option<AiErrorCode> {
        match self {
            AppError::Ai { code, .. } => Some(*code),
            _ => None,
        }
    }

    pub fn ai_correlation_id(&self) -> Option<&str> {
        match self {
            AppError::Ai { correlation_id, .. } => correlation_id.as_deref(),
            _ => None,
        }
    }

    pub fn ai_details(&self) -> Option<&JsonValue> {
        match self {
            AppError::Ai { details, .. } => details.as_ref(),
            _ => None,
        }
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        let message = message.into();
        warn!(target: "app::conflict", %message, "conflict error");
        AppError::Conflict { message }
    }

    pub fn not_found() -> Self {
        warn!(target: "app::database", "resource not found");
        AppError::NotFound
    }

    pub fn database(message: impl Into<String>) -> Self {
        let message = message.into();
        error!(target: "app::database", %message, "database error");
        AppError::Database { message }
    }

    pub fn other(message: impl Into<String>) -> Self {
        let message = message.into();
        error!(target: "app::other", %message, "other error");
        AppError::Other(message)
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(error: rusqlite::Error) -> Self {
        use rusqlite::Error::{QueryReturnedNoRows, SqliteFailure};
        use rusqlite::ErrorCode;

        match &error {
            QueryReturnedNoRows => AppError::not_found(),
            SqliteFailure(err, _) if err.code == ErrorCode::ConstraintViolation => {
                AppError::conflict("违反唯一性或约束限制")
            }
            _ => {
                error!(target: "app::database", error = ?error, "sqlite error");
                AppError::database(error.to_string())
            }
        }
    }
}
