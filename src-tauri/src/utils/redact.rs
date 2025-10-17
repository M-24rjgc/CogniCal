use crate::error::AppResult;
use serde_json::Value as JsonValue;

/// Redact sensitive data from JSON values
/// Removes or masks fields like: note, notes, description, title, personal info
pub fn redact_sensitive_data(data: &JsonValue) -> AppResult<JsonValue> {
    let redacted = redact_value(data);
    Ok(redacted)
}

fn redact_value(value: &JsonValue) -> JsonValue {
    match value {
        JsonValue::Object(map) => {
            let mut redacted_map = serde_json::Map::new();
            for (key, val) in map {
                let redacted_val = if is_sensitive_field(key) {
                    redact_string_value(val)
                } else {
                    redact_value(val)
                };
                redacted_map.insert(key.clone(), redacted_val);
            }
            JsonValue::Object(redacted_map)
        }
        JsonValue::Array(arr) => {
            let redacted_arr: Vec<JsonValue> = arr.iter().map(redact_value).collect();
            JsonValue::Array(redacted_arr)
        }
        _ => value.clone(),
    }
}

fn is_sensitive_field(field_name: &str) -> bool {
    let lower = field_name.to_lowercase();
    matches!(
        lower.as_str(),
        "note"
            | "notes"
            | "description"
            | "title"
            | "name"
            | "comment"
            | "comments"
            | "text"
            | "content"
    )
}

fn redact_string_value(value: &JsonValue) -> JsonValue {
    match value {
        JsonValue::String(s) if !s.is_empty() => JsonValue::String("[REDACTED]".to_string()),
        _ => value.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_redact_sensitive_fields() {
        let data = json!({
            "id": 123,
            "task_count": 5,
            "note": "Personal note here",
            "description": "Task description",
            "created_at": "2024-01-01T00:00:00Z"
        });

        let redacted = redact_sensitive_data(&data).unwrap();

        assert_eq!(redacted["id"], 123);
        assert_eq!(redacted["task_count"], 5);
        assert_eq!(redacted["note"], "[REDACTED]");
        assert_eq!(redacted["description"], "[REDACTED]");
        assert_eq!(redacted["created_at"], "2024-01-01T00:00:00Z");
    }

    #[test]
    fn test_redact_nested_objects() {
        let data = json!({
            "user": {
                "id": 1,
                "name": "John Doe"
            },
            "tasks": [
                {
                    "id": 1,
                    "title": "Buy groceries",
                    "duration": 30
                }
            ]
        });

        let redacted = redact_sensitive_data(&data).unwrap();

        assert_eq!(redacted["user"]["id"], 1);
        assert_eq!(redacted["user"]["name"], "[REDACTED]");
        assert_eq!(redacted["tasks"][0]["id"], 1);
        assert_eq!(redacted["tasks"][0]["title"], "[REDACTED]");
        assert_eq!(redacted["tasks"][0]["duration"], 30);
    }

    #[test]
    fn test_preserve_non_sensitive_data() {
        let data = json!({
            "count": 42,
            "status": "active",
            "metrics": {
                "score": 85.5,
                "rank": 10
            }
        });

        let redacted = redact_sensitive_data(&data).unwrap();

        // Should remain unchanged
        assert_eq!(redacted, data);
    }
}
