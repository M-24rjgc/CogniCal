use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};

use crate::error::AppResult;
use crate::models::ai::{
    TaskAiSource, TaskEfficiencyPrediction, TaskFocusModeRecommendation, TaskParseAiResult,
    TaskParseResponse,
};

/// Common input context shared by AI operations.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct AiOperationContext {
    pub locale: Option<String>,
    pub timezone: Option<String>,
    pub reference_date: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<JsonValue>,
}

/// Indicates where a provider response originated.
#[derive(Debug, Clone, Serialize, Deserialize, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AiResponseSource {
    Online,
    Offline,
    Cache,
}

impl Default for AiResponseSource {
    fn default() -> Self {
        AiResponseSource::Online
    }
}

/// Metadata describing the provider that produced a response.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct AiProviderMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u128>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens_used: Option<HashMap<String, u64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<JsonValue>,
}

/// Current connectivity status of the AI substrate.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct AiStatusDto {
    pub mode: AiResponseSource,
    pub has_api_key: bool,
    pub last_checked_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u128>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<AiProviderMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Result type for parsing a natural language task.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedTaskDto {
    pub payload: crate::models::ai::ParsedTaskPayload,
    pub missing_fields: Vec<String>,
    pub reasoning: ParsingReasoningDto,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct ParsingReasoningDto {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cot_steps: Option<Vec<crate::models::ai::TaskAiReasoningStep>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cot_summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub complexity_score: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_start_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focus_mode: Option<TaskFocusModeRecommendation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub efficiency_prediction: Option<TaskEfficiencyPrediction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<AiProviderMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generated_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<AiResponseSource>,
}

/// Placeholder DTO for future recommendation responses.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct RecommendationDto {
    pub recommendations: Vec<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub telemetry: Option<AiProviderMetadata>,
}

/// Placeholder DTO for future schedule planning responses.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct SchedulePlanDto {
    pub items: Vec<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub telemetry: Option<AiProviderMetadata>,
}

/// Shared provider contract to support online/offline execution.
#[async_trait::async_trait]
pub trait AiProvider: Send + Sync {
    async fn parse_task(
        &self,
        request: &crate::models::ai::TaskParseRequest,
    ) -> AppResult<ParsedTaskDto>;

    async fn generate_recommendations(&self, input: &JsonValue) -> AppResult<RecommendationDto>;

    async fn plan_schedule(&self, input: &JsonValue) -> AppResult<SchedulePlanDto>;

    async fn ping(&self) -> AppResult<AiProviderMetadata>;
}

impl From<ParsedTaskDto> for TaskParseResponse {
    fn from(dto: ParsedTaskDto) -> Self {
        let mut metadata_value = dto.reasoning.metadata.unwrap_or_else(|| json!({}));
        if !metadata_value.is_object() {
            metadata_value = json!({ "value": metadata_value });
        }

        if let Some(provider_meta) = dto.reasoning.provider.clone() {
            if let Ok(value) = serde_json::to_value(provider_meta) {
                if let Some(obj) = metadata_value.as_object_mut() {
                    obj.insert("provider".to_string(), value);
                }
            }
        }

        let metadata_option = if metadata_value
            .as_object()
            .map(|obj| obj.is_empty())
            .unwrap_or(false)
        {
            None
        } else {
            Some(metadata_value)
        };

        let source = dto.reasoning.source.unwrap_or(AiResponseSource::Online);
        let generated_at = dto
            .reasoning
            .generated_at
            .unwrap_or_else(|| Utc::now().to_rfc3339());

        let ai = TaskParseAiResult {
            summary: dto.reasoning.summary,
            next_action: dto.reasoning.next_action,
            confidence: dto.reasoning.confidence,
            metadata: metadata_option,
            complexity_score: dto.reasoning.complexity_score,
            suggested_start_at: dto.reasoning.suggested_start_at,
            focus_mode: dto.reasoning.focus_mode,
            efficiency_prediction: dto.reasoning.efficiency_prediction,
            cot_steps: dto.reasoning.cot_steps,
            cot_summary: dto.reasoning.cot_summary,
            source: match source {
                AiResponseSource::Online | AiResponseSource::Offline => TaskAiSource::Live,
                AiResponseSource::Cache => TaskAiSource::Cache,
            },
            generated_at,
        };

        TaskParseResponse {
            payload: dto.payload,
            missing_fields: dto.missing_fields,
            ai,
        }
    }
}

impl From<TaskParseResponse> for ParsedTaskDto {
    fn from(response: TaskParseResponse) -> Self {
        let TaskParseResponse {
            payload,
            missing_fields,
            ai,
        } = response;

        let mut provider = None;
        let metadata = ai.metadata.map(|mut meta| {
            if let Some(obj) = meta.as_object_mut() {
                if let Some(value) = obj.remove("provider") {
                    provider = serde_json::from_value(value).ok();
                }
            }
            meta
        });

        ParsedTaskDto {
            payload,
            missing_fields,
            reasoning: ParsingReasoningDto {
                summary: ai.summary,
                next_action: ai.next_action,
                confidence: ai.confidence,
                cot_steps: ai.cot_steps,
                cot_summary: ai.cot_summary,
                complexity_score: ai.complexity_score,
                suggested_start_at: ai.suggested_start_at,
                focus_mode: ai.focus_mode,
                efficiency_prediction: ai.efficiency_prediction,
                metadata,
                provider,
                generated_at: Some(ai.generated_at),
                source: Some(match ai.source {
                    TaskAiSource::Live => AiResponseSource::Online,
                    TaskAiSource::Cache => AiResponseSource::Cache,
                }),
            },
        }
    }
}
