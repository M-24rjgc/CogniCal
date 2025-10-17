use serde_json::{json, Value as JsonValue};

use crate::models::ai::TaskParseRequest;

/// System prompt guiding DeepSeek when parsing natural language tasks.
pub fn task_parsing_system_prompt() -> &'static str {
    r#"You are Cognical's task planning copilot. Your job is to read a human task description
and produce a structured JSON object strictly matching the provided schema. Always respond with
valid UTF-8 JSON. Do not wrap the response in markdown code blocks. The schema is:
{
  "payload": {
    "title": string|null,
    "description": string|null,
    "status": string|null,
    "priority": string|null,
    "plannedStartAt": string|null,
    "startAt": string|null,
    "dueAt": string|null,
    "completedAt": string|null,
    "estimatedMinutes": number|null,
    "estimatedHours": number|null,
    "tags": string[]|null,
    "ownerId": string|null,
    "isRecurring": boolean|null,
    "recurrence": object|null,
    "taskType": string|null,
    "externalLinks": string[]|null
  },
  "missingFields": string[],
  "reasoning": {
    "summary": string|null,
    "nextAction": string|null,
    "confidence": number|null,
    "cotSteps": object[]|null,
    "cotSummary": string|null,
    "complexityScore": number|null,
    "suggestedStartAt": string|null,
    "focusMode": object|null,
    "efficiencyPrediction": object|null,
    "metadata": object|null
  }
}
Use ISO-8601 timestamps in UTC. Provide clear, concise reasoning entries.

Example response:
{
    "payload": {
        "title": "Prepare quarterly OKR summary",
        "description": "Compile last quarter results and draft next OKR recommendations",
        "status": "in-progress",
        "priority": "high",
        "plannedStartAt": "2025-10-16T02:00:00Z",
        "startAt": null,
        "dueAt": "2025-10-18T12:00:00Z",
        "completedAt": null,
        "estimatedMinutes": 180,
        "estimatedHours": 3,
        "tags": ["okr", "planning"],
        "ownerId": null,
        "isRecurring": false,
        "recurrence": null,
        "taskType": "analysis",
        "externalLinks": ["https://example.com/okr-template"]
    },
    "missingFields": ["startAt"],
    "reasoning": {
        "summary": "Complete before the weekend to support quarterly review.",
        "nextAction": "Draft slides summarizing key achievements.",
        "confidence": 0.78,
        "cotSteps": null,
        "cotSummary": "Extracted fields from description and estimated duration.",
        "complexityScore": 0.6,
        "suggestedStartAt": "2025-10-16T02:00:00Z",
        "focusMode": null,
        "efficiencyPrediction": null,
        "metadata": {"source": "deepseek"}
    }
}
"
    "#
}

/// System prompt for recommendation generation.
pub fn recommendations_system_prompt() -> &'static str {
    r#"You are Cognical's productivity strategist. Based on the user context, return JSON with the schema:
{
  "recommendations": [{
     "id": string,
     "title": string,
     "detail": string,
     "priority": string,
     "impact": string,
     "nextAction": string
  }],
  "telemetry": object|null
}
Keep the list under five items and focus on actionable suggestions."
    "#
}

/// System prompt for schedule planning outputs.
pub fn schedule_planning_system_prompt() -> &'static str {
    r#"You are Cognical's planning assistant. Produce a structured JSON schedule following:
{
  "items": [{
     "taskId": string|null,
     "title": string,
     "startAt": string,
     "endAt": string,
     "confidence": number|null,
     "notes": string|null
  }],
  "telemetry": object|null
}
Ensure times are ISO-8601 UTC and sorted by startAt."
    "#
}

/// Build the user payload for task parsing requests.
pub fn build_task_parse_payload(request: &TaskParseRequest) -> JsonValue {
    let mut payload = serde_json::Map::new();
    payload.insert("operation".to_string(), json!("parseTask"));
    payload.insert("input".to_string(), json!(request.input));

    if let Some(context) = request.context.as_ref() {
        if let Ok(value) = serde_json::to_value(context) {
            payload.insert("context".to_string(), value);
        }
    }

    payload.insert(
        "expectations".to_string(),
        json!({
            "languages": ["zh-CN", "en"],
            "mustReturnAllFields": true,
            "timezoneFallback": "UTC",
            "minConfidence": 0.5
        }),
    );

    JsonValue::Object(payload)
}

/// Build the user payload for recommendation requests.
pub fn build_recommendations_payload(input: &JsonValue) -> JsonValue {
    json!({
        "operation": "generateRecommendations",
        "context": input,
        "expectations": {
            "maxRecommendations": 5,
            "includeFollowUp": true
        }
    })
}

/// Build the user payload for schedule planning requests.
pub fn build_schedule_payload(input: &JsonValue) -> JsonValue {
    json!({
        "operation": "planSchedule",
        "context": input,
        "expectations": {
            "maxItems": 12,
            "granularity": "30m",
            "timezoneFallback": "UTC"
        }
    })
}
