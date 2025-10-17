use std::time::Instant;

use chrono::{DateTime, Duration, Utc};
use serde_json::{json, Value as JsonValue};
use tracing::debug;

use crate::models::ai::{
    default_efficiency_prediction, default_focus_mode_recommendation, default_reasoning_steps,
    ParsedTaskPayload, TaskAiSource, TaskParseAiResult, TaskParseContext, TaskParseRequest,
    TaskParseResponse,
};
use crate::models::ai_types::{AiProviderMetadata, RecommendationDto, SchedulePlanDto};

#[derive(Debug, Default, Clone)]
pub struct CotEngine;

impl CotEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn synthesize(&self, request: &TaskParseRequest) -> TaskParseResponse {
        let trimmed_input = request.input.trim();
        let now = Utc::now();
        let planned_start_at = request
            .context
            .as_ref()
            .and_then(|ctx| ctx.reference_date.clone())
            .unwrap_or_else(|| now.to_rfc3339());
        let suggested_start = (now + Duration::hours(1)).to_rfc3339();
        let due_at = (now + Duration::hours(4)).to_rfc3339();

        let normalized_title = self.derive_title(trimmed_input, request.context.as_ref());
        let priority = self.derive_priority(trimmed_input, request.context.as_ref());
        let estimated_minutes = self.estimate_duration(trimmed_input);
        let tags = self.derive_tags(trimmed_input, request.context.as_ref());

        let payload = ParsedTaskPayload {
            title: Some(normalized_title.clone()),
            description: Some(trimmed_input.to_string()),
            status: Some("todo".to_string()),
            priority: Some(priority),
            planned_start_at: Some(planned_start_at.clone()),
            start_at: Some(suggested_start.clone()),
            due_at: Some(due_at.clone()),
            completed_at: None,
            estimated_minutes: Some(estimated_minutes),
            estimated_hours: Some((estimated_minutes as f64 / 60.0).max(0.5)),
            tags: Some(tags.clone()),
            owner_id: None,
            is_recurring: Some(false),
            recurrence: None,
            task_type: self.derive_task_type(trimmed_input),
            external_links: Some(Vec::new()),
        };

        let complexity_score = self.estimate_complexity(trimmed_input, &tags);
        let efficiency_prediction = default_efficiency_prediction();
        let focus_mode = default_focus_mode_recommendation(Some(suggested_start.clone()));
        let cot_steps = default_reasoning_steps();

        let metadata = self.build_metadata(request.context.as_ref(), complexity_score, &tags);

        let ai = TaskParseAiResult {
            summary: Some(format!(
                "系统建议以“{normalized_title}”为任务标题，并补充关键字段。"
            )),
            next_action: Some("确认主要目标是否准确，再拆分子任务。".to_string()),
            confidence: Some(self.estimate_confidence(trimmed_input)),
            metadata,
            complexity_score: Some(complexity_score as f64),
            suggested_start_at: Some(suggested_start.clone()),
            focus_mode: Some(focus_mode),
            efficiency_prediction: Some(efficiency_prediction),
            cot_steps: Some(cot_steps),
            cot_summary: Some(
                "该任务需要专注执行，建议预留 3 个番茄钟完成准备与交付。".to_string(),
            ),
            source: TaskAiSource::Live,
            generated_at: now.to_rfc3339(),
        };

        debug!(target: "app::ai::cot", "generated synthetic parse result");

        TaskParseResponse {
            payload,
            ai,
            missing_fields: vec!["ownerId".to_string()],
        }
    }

    fn derive_title(&self, input: &str, context: Option<&TaskParseContext>) -> String {
        let fallback = context
            .and_then(|ctx| ctx.metadata.as_ref())
            .and_then(|meta| meta.get("draftTitle"))
            .and_then(JsonValue::as_str)
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "AI 解析任务".to_string());

        input
            .lines()
            .next()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .map(|line| line.chars().take(160).collect::<String>())
            .filter(|title| !title.is_empty())
            .unwrap_or(fallback)
    }

    fn derive_priority(&self, input: &str, context: Option<&TaskParseContext>) -> String {
        if let Some(priority) = context
            .and_then(|ctx| ctx.metadata.as_ref())
            .and_then(|meta| meta.get("preferredPriority"))
            .and_then(JsonValue::as_str)
        {
            return priority.to_lowercase();
        }

        let lower = input.to_lowercase();
        if lower.contains("紧急") || lower.contains("urgent") {
            "urgent".to_string()
        } else if lower.contains("重要") || lower.contains("high") {
            "high".to_string()
        } else {
            "medium".to_string()
        }
    }

    fn derive_tags(&self, input: &str, context: Option<&TaskParseContext>) -> Vec<String> {
        let mut tags = Vec::new();

        if let Some(predefined) = context
            .and_then(|ctx| ctx.metadata.as_ref())
            .and_then(|meta| meta.get("preferredTags"))
            .and_then(JsonValue::as_array)
        {
            for tag in predefined {
                if let Some(value) = tag.as_str() {
                    tags.push(value.trim().to_string());
                }
            }
        }

        let lower = input.to_lowercase();
        if lower.contains("会议") || lower.contains("meeting") {
            tags.push("meeting".to_string());
        }
        if lower.contains("报告") || lower.contains("report") {
            tags.push("report".to_string());
        }
        if lower.contains("学习") || lower.contains("study") {
            tags.push("learning".to_string());
        }

        if tags.is_empty() {
            tags.push("ai".to_string());
        }

        tags.sort();
        tags.dedup();
        tags
    }

    fn derive_task_type(&self, input: &str) -> Option<String> {
        let lower = input.to_lowercase();
        if lower.contains("学习") || lower.contains("study") {
            Some("study".to_string())
        } else if lower.contains("家庭") || lower.contains("life") {
            Some("life".to_string())
        } else if lower.contains("计划") || lower.contains("project") {
            Some("work".to_string())
        } else {
            Some("work".to_string())
        }
    }

    fn estimate_duration(&self, input: &str) -> i64 {
        let word_count = input.split_whitespace().count() as i64;
        let base = 90;
        let increment = (word_count / 40) * 15;
        (base + increment).clamp(30, 6 * 60)
    }

    fn estimate_complexity(&self, input: &str, tags: &[String]) -> i32 {
        let length_score = (input.split_whitespace().count() / 25) as i32;
        let tag_score = tags.len() as i32;
        (3 + length_score + tag_score).clamp(1, 10)
    }

    fn estimate_confidence(&self, input: &str) -> f64 {
        let length = input.split_whitespace().count() as f64;
        (0.6 + (length.min(200.0) / 400.0)).clamp(0.55, 0.92)
    }

    fn build_metadata(
        &self,
        context: Option<&TaskParseContext>,
        complexity_score: i32,
        tags: &[String],
    ) -> Option<JsonValue> {
        let mut metadata = serde_json::json!({
            "complexity": complexity_score,
            "tagHints": tags,
        });

        if let Some(ctx) = context {
            if let Some(locale) = ctx.locale.as_ref() {
                metadata["locale"] = serde_json::Value::String(locale.clone());
            }
            if let Some(timezone) = ctx.timezone.as_ref() {
                metadata["timezone"] = serde_json::Value::String(timezone.clone());
            }
        }

        Some(metadata)
    }

    /// Generate deterministic heuristic recommendations when offline.
    pub fn generate_recommendations(&self, input: &JsonValue) -> RecommendationDto {
        let started_at = Instant::now();
        let tasks = Self::extract_tasks(input);
        let focus_tags = Self::extract_focus_tags(input);

        let mut recommendations = Vec::new();

        for (idx, task) in tasks.iter().enumerate().take(3) {
            let task_title = task
                .get("title")
                .and_then(JsonValue::as_str)
                .unwrap_or("当前任务");
            let priority = task
                .get("priority")
                .and_then(JsonValue::as_str)
                .unwrap_or("medium");
            let category = task
                .get("category")
                .and_then(JsonValue::as_str)
                .or_else(|| task.get("taskType").and_then(JsonValue::as_str))
                .unwrap_or("general");

            let detail =
                format!("围绕“{task_title}”安排一个明确的可交付成果，并提前准备相关资料。");
            let next_action = match priority {
                "urgent" | "high" => "立即预留时间块并标记关键阻碍。",
                "low" => "在低能量时段处理，保持连续性。",
                _ => "规划 1 小时专注时间并确认依赖项。",
            };
            let impact = match category {
                "focus" | "deep-work" => "focus",
                "collaboration" => "team",
                _ => "progress",
            };

            recommendations.push(json!({
                "id": format!("local-cot-rec-{}", idx + 1),
                "title": format!("优先推进：{task_title}"),
                "detail": detail,
                "priority": priority,
                "impact": impact,
                "nextAction": next_action,
            }));
        }

        if recommendations.is_empty() {
            recommendations.push(json!({
                "id": "local-cot-rec-1",
                "title": "建立今日执行清单",
                "detail": "回顾本周关键任务，选出最具影响力的 3 项安排在高能时段。",
                "priority": "medium",
                "impact": "progress",
                "nextAction": "创建番茄钟并邀请相关协作方。",
            }));
        }

        RecommendationDto {
            recommendations,
            telemetry: Some(Self::provider_metadata(
                started_at.elapsed().as_millis(),
                json!({
                    "mode": "offline",
                    "strategy": "heuristic-recommendations",
                    "taskCount": tasks.len(),
                    "focusTags": focus_tags,
                }),
            )),
        }
    }

    /// Generate a lightweight deterministic schedule when offline.
    pub fn plan_schedule(&self, input: &JsonValue) -> SchedulePlanDto {
        let started_at = Instant::now();
        let tasks = Self::extract_tasks(input);
        let mut anchor = Self::extract_schedule_anchor(input);

        let mut items = Vec::new();

        for (idx, task) in tasks.iter().enumerate() {
            let title = task
                .get("title")
                .and_then(JsonValue::as_str)
                .unwrap_or("专注任务");
            let task_id = task
                .get("id")
                .and_then(JsonValue::as_str)
                .map(|value| value.to_string());
            let duration_minutes = task
                .get("estimatedMinutes")
                .and_then(JsonValue::as_i64)
                .unwrap_or(60)
                .clamp(30, 180);

            let end = anchor + Duration::minutes(duration_minutes);

            items.push(json!({
                "taskId": task_id,
                "title": title,
                "startAt": anchor.to_rfc3339(),
                "endAt": end.to_rfc3339(),
                "confidence": (0.62 + (idx as f64 * 0.04)).min(0.85),
                "notes": format!("离线建议：在此时间段完成 {title}，并预留 10 分钟复盘。"),
            }));

            anchor = end + Duration::minutes(10);
        }

        if items.is_empty() {
            let fallback_end = anchor + Duration::minutes(60);
            items.push(json!({
                "taskId": JsonValue::Null,
                "title": "专注整理时段",
                "startAt": anchor.to_rfc3339(),
                "endAt": fallback_end.to_rfc3339(),
                "confidence": 0.6,
                "notes": "离线建议：梳理任务列表，确认优先级后再排期。",
            }));
        }

        SchedulePlanDto {
            items,
            telemetry: Some(Self::provider_metadata(
                started_at.elapsed().as_millis(),
                json!({
                    "mode": "offline",
                    "strategy": "fixed-interval",
                    "taskCount": tasks.len(),
                }),
            )),
        }
    }

    fn extract_tasks(input: &JsonValue) -> Vec<JsonValue> {
        if let Some(tasks) = input.get("tasks").and_then(JsonValue::as_array) {
            return tasks.iter().cloned().collect();
        }

        if let Some(tasks) = input
            .get("context")
            .and_then(JsonValue::as_object)
            .and_then(|ctx| ctx.get("tasks"))
            .and_then(JsonValue::as_array)
        {
            return tasks.iter().cloned().collect();
        }

        Vec::new()
    }

    fn extract_focus_tags(input: &JsonValue) -> Vec<String> {
        input
            .pointer("/context/focusTags")
            .and_then(JsonValue::as_array)
            .map(|tags| {
                tags.iter()
                    .filter_map(JsonValue::as_str)
                    .map(|tag| tag.to_string())
                    .collect()
            })
            .unwrap_or_default()
    }

    fn extract_schedule_anchor(input: &JsonValue) -> DateTime<Utc> {
        let from_input = input
            .pointer("/availability/startAt")
            .and_then(JsonValue::as_str)
            .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
            .map(|dt| dt.with_timezone(&Utc));

        if let Some(anchor) = from_input {
            return anchor;
        }

        let today = Utc::now().date_naive();
        today
            .and_hms_opt(9, 0, 0)
            .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
            .unwrap_or_else(Utc::now)
    }

    fn provider_metadata(latency_ms: u128, extra: JsonValue) -> AiProviderMetadata {
        AiProviderMetadata {
            provider_id: Some("local-cot".to_string()),
            model: Some("cot-engine".to_string()),
            latency_ms: Some(latency_ms),
            tokens_used: None,
            extra: Some(extra),
        }
    }
}
