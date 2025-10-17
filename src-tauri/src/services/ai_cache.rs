use base64::engine::general_purpose::STANDARD_NO_PAD;
use base64::Engine;
use sha2::{Digest, Sha256};

/// Supported AI cache operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiCacheOperation {
    ParseTask,
    Recommendations,
    Schedule,
}

impl AiCacheOperation {
    pub fn as_str(self) -> &'static str {
        match self {
            AiCacheOperation::ParseTask => "parse",
            AiCacheOperation::Recommendations => "recommend",
            AiCacheOperation::Schedule => "schedule",
        }
    }
}

/// Unique cache identity constructed from operation + semantic hash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiCacheKey {
    operation: AiCacheOperation,
    semantic_hash: String,
}

impl AiCacheKey {
    pub fn new(operation: AiCacheOperation, semantic_hash: impl Into<String>) -> Self {
        Self {
            operation,
            semantic_hash: semantic_hash.into(),
        }
    }

    pub fn operation(&self) -> AiCacheOperation {
        self.operation
    }

    pub fn semantic_hash(&self) -> &str {
        &self.semantic_hash
    }

    /// Stable key for persistence (base64url encoded SHA-256).
    pub fn cache_key(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.operation.as_str().as_bytes());
        hasher.update(b":");
        hasher.update(self.semantic_hash.as_bytes());
        let digest = hasher.finalize();
        STANDARD_NO_PAD.encode(digest)
    }
}

impl From<&AiCacheKey> for String {
    fn from(value: &AiCacheKey) -> Self {
        value.cache_key()
    }
}
