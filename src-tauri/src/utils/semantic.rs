use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine as _};
use serde_json::Value as JsonValue;
use sha2::{Digest, Sha256};

/// Generate a deterministic semantic hash for AI parse requests.
///
/// The hash uses a lower-cased, trimmed version of the input combined
/// with optional metadata context to reduce redundant cache misses
/// caused by whitespace or casing variations.
pub fn semantic_hash(input: &str, metadata: Option<&JsonValue>) -> String {
    let mut hasher = Sha256::new();
    let normalized = input.trim().to_lowercase();
    hasher.update(normalized.as_bytes());

    if let Some(metadata) = metadata {
        if let Ok(serialized) = serde_json::to_vec(metadata) {
            hasher.update(&serialized);
        }
    }

    let digest = hasher.finalize();
    STANDARD_NO_PAD.encode(digest)
}
