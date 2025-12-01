use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::sync::Arc;

use chrono::{DateTime, FixedOffset, Utc};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::db::repositories::planning_repository::{
    PlanningOptionRow, PlanningRepository, PlanningSessionRow, PlanningTimeBlockRow,
};
use crate::db::repositories::task_repository::TaskRepository;
use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::planning::{
    PlanningOptionRecord, PlanningSessionRecord, PlanningTimeBlockRecord,
};
use crate::models::task::TaskRecord;
use crate::services::ai_service::AiService;
use crate::services::behavior_learning::{BehaviorLearningService, PreferenceSnapshot};
use crate::services::schedule_optimizer::{
    detect_conflicts, PlanOption, PlanRationaleStep, SchedulableTask, ScheduleConflict,
    ScheduleConstraints, ScheduleOptimizer, SchedulingPreferences, TimeBlockCandidate,
};
use crate::services::schedule_utils;
use crate::services::task_service::TaskService;

const DEFAULT_PREFERENCE_ID: &str = "default";

#[derive(Clone)]
pub struct PlanningService {
    db: DbPool,
    task_service: Arc<TaskService>,
    #[allow(dead_code)]
    ai_service: Arc<AiService>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratePlanInput {
    pub task_ids: Vec<String>,
    #[serde(default)]
    pub constraints: Option<ScheduleConstraints>,
    #[serde(default)]
    pub preference_id: Option<String>,
    #[serde(default)]
    pub seed: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeBlockOverride {
    pub block_id: String,
    #[serde(default)]
    pub start_at: Option<String>,
    #[serde(default)]
    pub end_at: Option<String>,
    #[serde(default)]
    pub flexibility: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyPlanInput {
    pub session_id: String,
    pub option_id: String,
    #[serde(default)]
    pub overrides: Vec<TimeBlockOverride>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolveConflictInput {
    pub session_id: String,
    pub option_id: String,
    #[serde(default)]
    pub adjustments: Vec<TimeBlockOverride>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanningSessionView {
    pub session: PlanningSessionRecord,
    pub options: Vec<PlanningOptionView>,
    #[serde(default)]
    pub conflicts: Vec<ScheduleConflict>,
    #[serde(default)]
    pub preference_snapshot: Option<PreferenceSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanningOptionView {
    pub option: PlanningOptionRecord,
    pub blocks: Vec<PlanningTimeBlockRecord>,
    #[serde(default)]
    pub conflicts: Vec<ScheduleConflict>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppliedPlan {
    pub session: PlanningSessionRecord,
    pub option: PlanningOptionView,
    #[serde(default)]
    pub conflicts: Vec<ScheduleConflict>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct OptionRiskMetadata {
    #[serde(default)]
    notes: Vec<String>,
    #[serde(default)]
    conflicts: Vec<ScheduleConflict>,
}

impl PlanningService {
    pub fn new(db: DbPool, task_service: Arc<TaskService>, ai_service: Arc<AiService>) -> Self {
        Self {
            db,
            task_service,
            ai_service,
        }
    }

    /// Get a reference to the task service
    pub fn get_task_service(&self) -> &Arc<TaskService> {
        &self.task_service
    }

    pub async fn generate_plan(&self, input: GeneratePlanInput) -> AppResult<PlanningSessionView> {
        if input.task_ids.is_empty() {
            return Err(AppError::validation("生成计划时至少需要一个任务"));
        }

        let conn = self.db.get_connection()?;
        let has_ai_key = self.ai_service.has_configured_provider(&conn)?;
        let seed = input.seed;

        let tasks = self.fetch_tasks(&input.task_ids)?;
        let tasks_by_id = tasks
            .iter()
            .map(|task| (task.id.clone(), task.clone()))
            .collect::<HashMap<_, _>>();

        let constraints = input.constraints.unwrap_or_default();
        if constraints.available_windows.is_empty() {
            debug!(target: "app::planning", "constraints without explicit windows, relying on optimizer fallback");
        }

        let preference_id = input
            .preference_id
            .as_deref()
            .unwrap_or(DEFAULT_PREFERENCE_ID)
            .to_string();

        // Load preferences and close connection before async call
        let preference_snapshot = {
            let behavior = BehaviorLearningService::new(&conn);
            behavior.load_preferences(&preference_id)?
        };
        let personalization_json = serde_json::to_value(&preference_snapshot)?;

        let scheduling_preferences = scheduling_preferences_from(&preference_snapshot);

        // Clone data needed for AI call (so we can drop conn)
        let tasks_for_ai = tasks.clone();
        let constraints_for_ai = constraints.clone();

        // Drop connection before async operations
        drop(conn);

        let options = if has_ai_key {
            let generated = self
                .generate_with_ai(
                    &tasks_for_ai,
                    &constraints_for_ai,
                    &scheduling_preferences,
                    &preference_snapshot,
                )
                .await?;
            info!(target: "app::planning", "Successfully generated plan options using DeepSeek AI");
            generated
        } else {
            warn!(target: "app::planning", "DeepSeek API Key 未配置，使用内置调度算法作为回退");
            self.generate_with_optimizer(
                &tasks_for_ai,
                &constraints_for_ai,
                &scheduling_preferences,
                seed,
            )?
        };

        // Reconnect for database operations
        let mut conn = self.db.get_connection()?;

        let generated_at = Utc::now().to_rfc3339();
        let session_id = Uuid::new_v4().to_string();
        let now = generated_at.clone();

        let session_record = PlanningSessionRecord {
            id: session_id.clone(),
            task_ids: input.task_ids,
            constraints: Some(serde_json::to_value(&constraints)?),
            generated_at: generated_at.clone(),
            status: "pending".to_string(),
            selected_option_id: None,
            personalization_snapshot: Some(personalization_json),
            created_at: now.clone(),
            updated_at: now.clone(),
        };

        let tx = conn.transaction()?;
        let tx_conn = tx.deref();

        let session_row = PlanningSessionRow::from_record(&session_record)?;
        PlanningRepository::insert_session(tx_conn, &session_row)?;

        for option in &options {
            let summary = build_option_summary(option, &tasks_by_id);
            let metadata = OptionRiskMetadata {
                notes: option.risk_notes.clone(),
                conflicts: option.conflicts.clone(),
            };

            let option_record = PlanningOptionRecord {
                id: option.id.clone(),
                session_id: session_record.id.clone(),
                rank: option.rank as i64,
                score: Some(option.score),
                summary: Some(summary),
                cot_steps: Some(serde_json::to_value(&option.rationale)?),
                risk_notes: Some(serde_json::to_value(&metadata)?),
                is_fallback: option.is_fallback,
                created_at: now.clone(),
            };

            let option_row = PlanningOptionRow::from_record(&option_record)?;
            PlanningRepository::insert_option(tx_conn, &option_row)?;

            for block in &option.blocks {
                let conflict_flags = if block.conflict_flags.is_empty() {
                    None
                } else {
                    Some(serde_json::to_value(&block.conflict_flags)?)
                };

                let block_record = PlanningTimeBlockRecord {
                    id: block.id.clone(),
                    option_id: option.id.clone(),
                    task_id: block.task_id.clone(),
                    start_at: block.start_at.clone(),
                    end_at: block.end_at.clone(),
                    flexibility: block.flexibility.clone(),
                    confidence: Some(block.confidence as f64),
                    conflict_flags,
                    applied_at: None,
                    actual_start_at: None,
                    actual_end_at: None,
                    status: "draft".to_string(),
                };

                let block_row = PlanningTimeBlockRow::from_record(&block_record)?;
                PlanningRepository::insert_time_block(tx_conn, &block_row)?;
            }
        }

        tx.commit()?;

        info!(target: "app::planning", session_id = %session_record.id, options = options.len(), "planning session generated");

        self.load_session_view(&session_record.id, &conn)
    }

    pub fn apply_option(&self, input: ApplyPlanInput) -> AppResult<AppliedPlan> {
        let mut conn = self.db.get_connection()?;
        let tx = conn.transaction()?;
        let tx_conn = tx.deref();

        let session_row = PlanningRepository::find_session_by_id(tx_conn, &input.session_id)?
            .ok_or_else(AppError::not_found)?;
        if session_row.status == "applied" {
            return Err(AppError::conflict("该规划会话已完成应用"));
        }

        let mut session_row_for_update = session_row.clone();
        let session_record = session_row.into_record()?;

        let mut option_row = PlanningRepository::find_option_by_id(tx_conn, &input.option_id)?
            .ok_or_else(AppError::not_found)?;

        if option_row.session_id != session_row_for_update.id {
            return Err(AppError::validation("目标方案不属于当前会话"));
        }

        let blocks_rows =
            PlanningRepository::list_time_blocks_for_option(tx_conn, &input.option_id)?;
        if blocks_rows.is_empty() {
            return Err(AppError::validation("没有可应用的时间块"));
        }

        let mut block_records = blocks_rows
            .into_iter()
            .map(|row| row.into_record())
            .collect::<AppResult<Vec<_>>>()?;

        apply_overrides(&mut block_records, &input.overrides)?;

        let constraints: ScheduleConstraints = session_record
            .constraints
            .clone()
            .map(|value| serde_json::from_value(value))
            .transpose()?
            .unwrap_or_default();

        let candidates = block_records
            .iter()
            .map(time_block_to_candidate)
            .collect::<AppResult<Vec<_>>>()?;

        let conflicts = detect_conflicts(
            &candidates,
            &constraints.existing_events,
            constraints.max_focus_minutes_per_day,
        )?;

        update_block_conflict_flags(&mut block_records, &conflicts)?;

        let mut metadata = parse_risk_metadata(&option_row);
        metadata.conflicts = conflicts.clone();
        option_row.risk_notes = Some(serde_json::to_string(&metadata)?);
        PlanningRepository::update_option(tx_conn, &option_row)?;

        let now = Utc::now().to_rfc3339();
        for block in block_records.iter_mut() {
            block.applied_at = Some(now.clone());
            block.status = "planned".to_string();
        }

        for block in &block_records {
            let row = PlanningTimeBlockRow::from_record(block)?;
            PlanningRepository::update_time_block(tx_conn, &row)?;
        }

        session_row_for_update.status = "applied".to_string();
        session_row_for_update.selected_option_id = Some(input.option_id.clone());
        session_row_for_update.updated_at = now.clone();
        PlanningRepository::update_session(tx_conn, &session_row_for_update)?;

        let earliest_map = earliest_start_by_task(&block_records)?;
        for (task_id, start_at) in earliest_map {
            if let Some(mut task_row) = TaskRepository::find_by_id(tx_conn, &task_id)? {
                if task_row.planned_start_at.as_ref() != Some(&start_at) {
                    task_row.planned_start_at = Some(start_at.clone());
                    task_row.updated_at = now.clone();
                    TaskRepository::update(tx_conn, &task_row)?;
                }
            } else {
                warn!(target: "app::planning", task_id = %task_id, "skipping task update because record not found");
            }
        }

        tx.commit()?;

        let view = self.load_session_view(&input.session_id, &conn)?;
        let option_view = view
            .options
            .iter()
            .find(|candidate| candidate.option.id == input.option_id)
            .cloned()
            .ok_or_else(AppError::not_found)?;

        Ok(AppliedPlan {
            session: view.session,
            conflicts: option_view.conflicts.clone(),
            option: option_view,
        })
    }

    pub fn resolve_conflicts(&self, input: ResolveConflictInput) -> AppResult<PlanningSessionView> {
        let mut conn = self.db.get_connection()?;
        let tx = conn.transaction()?;
        let tx_conn = tx.deref();

        let session_row = PlanningRepository::find_session_by_id(tx_conn, &input.session_id)?
            .ok_or_else(AppError::not_found)?;
        let mut session_row_for_update = session_row.clone();
        let session_record = session_row.into_record()?;

        let mut option_row = PlanningRepository::find_option_by_id(tx_conn, &input.option_id)?
            .ok_or_else(AppError::not_found)?;

        if option_row.session_id != session_row_for_update.id {
            return Err(AppError::validation("目标方案不属于当前会话"));
        }

        let blocks_rows =
            PlanningRepository::list_time_blocks_for_option(tx_conn, &input.option_id)?;
        if blocks_rows.is_empty() {
            return Err(AppError::validation("未找到可调整的时间块"));
        }

        let mut block_records = blocks_rows
            .into_iter()
            .map(|row| row.into_record())
            .collect::<AppResult<Vec<_>>>()?;

        apply_overrides(&mut block_records, &input.adjustments)?;

        let constraints: ScheduleConstraints = session_record
            .constraints
            .clone()
            .map(|value| serde_json::from_value(value))
            .transpose()?
            .unwrap_or_default();

        let candidates = block_records
            .iter()
            .map(time_block_to_candidate)
            .collect::<AppResult<Vec<_>>>()?;

        let conflicts = detect_conflicts(
            &candidates,
            &constraints.existing_events,
            constraints.max_focus_minutes_per_day,
        )?;

        update_block_conflict_flags(&mut block_records, &conflicts)?;

        let mut metadata = parse_risk_metadata(&option_row);
        metadata.conflicts = conflicts;
        option_row.risk_notes = Some(serde_json::to_string(&metadata)?);
        PlanningRepository::update_option(tx_conn, &option_row)?;

        for block in &block_records {
            let row = PlanningTimeBlockRow::from_record(block)?;
            PlanningRepository::update_time_block(tx_conn, &row)?;
        }

        session_row_for_update.updated_at = Utc::now().to_rfc3339();
        PlanningRepository::update_session(tx_conn, &session_row_for_update)?;

        tx.commit()?;

        self.load_session_view(&input.session_id, &conn)
    }

    fn fetch_tasks(&self, ids: &[String]) -> AppResult<Vec<TaskRecord>> {
        let mut results = Vec::new();
        for id in ids {
            let record = self.task_service.get_task(id)?;
            results.push(record);
        }
        Ok(results)
    }

    /// Generate plan options using DeepSeek AI service
    async fn generate_with_ai(
        &self,
        tasks: &[TaskRecord],
        constraints: &ScheduleConstraints,
        _preferences: &SchedulingPreferences,
        preference_snapshot: &PreferenceSnapshot,
    ) -> AppResult<Vec<PlanOption>> {
        // Build AI request payload
        let task_items: Vec<serde_json::Value> = tasks
            .iter()
            .map(|task| {
                json!({
                    "id": task.id,
                    "title": task.title,
                    "priority": task.priority,
                    "estimatedMinutes": task.estimated_minutes.or_else(|| task.estimated_hours.map(|h| (h * 60.0) as i64)),
                    "dueAt": task.due_at.as_ref(),
                    "startAt": task.start_at.as_ref().or(task.planned_start_at.as_ref()),
                })
            })
            .collect();

        let ai_payload = json!({
            "tasks": task_items,
            "constraints": {
                "planningStartAt": constraints.planning_start_at,
                "planningEndAt": constraints.planning_end_at,
                "availableWindows": constraints.available_windows,
                "existingEvents": constraints.existing_events,
                "maxFocusMinutesPerDay": constraints.max_focus_minutes_per_day,
            },
            "preferences": {
                "focusStartMinute": preference_snapshot.focus_start_minute,
                "focusEndMinute": preference_snapshot.focus_end_minute,
                "bufferMinutesBetweenBlocks": preference_snapshot.buffer_minutes_between_blocks,
                "preferCompactSchedule": preference_snapshot.prefer_compact_schedule,
                "avoidanceWindows": preference_snapshot.avoidance_windows,
            },
            "context": {
                "source": "planning_service",
                "timestamp": Utc::now().to_rfc3339(),
            }
        });

        // Call AI service
        let schedule_dto = self.ai_service.plan_schedule(ai_payload).await?;

        // Convert AI response to PlanOption format
        self.convert_ai_response_to_plan_options(schedule_dto, tasks)
    }

    fn generate_with_optimizer(
        &self,
        tasks: &[TaskRecord],
        constraints: &ScheduleConstraints,
        preferences: &SchedulingPreferences,
        seed: Option<u64>,
    ) -> AppResult<Vec<PlanOption>> {
        let optimizer = ScheduleOptimizer::new(seed);
        let schedulable_tasks = tasks
            .iter()
            .map(Self::map_schedulable_task)
            .collect::<Vec<_>>();

        optimizer.generate_plan_options(schedulable_tasks, constraints.clone(), preferences.clone())
    }

    /// Convert AI SchedulePlanDto to our PlanOption format
    fn convert_ai_response_to_plan_options(
        &self,
        dto: crate::models::ai_types::SchedulePlanDto,
        _tasks: &[TaskRecord],
    ) -> AppResult<Vec<PlanOption>> {
        let mut options = Vec::new();

        // For now, create a single option from the AI response
        // In the future, we could request multiple alternatives from AI
        let mut blocks = Vec::new();
        let mut rationale_steps = Vec::new();

        for (idx, item) in dto.items.iter().enumerate() {
            let task_id = item
                .get("taskId")
                .and_then(|v| v.as_str())
                .ok_or_else(|| AppError::validation("AI response missing taskId"))?
                .to_string();

            let start_at = item
                .get("startAt")
                .and_then(|v| v.as_str())
                .ok_or_else(|| AppError::validation("AI response missing startAt"))?
                .to_string();

            let end_at = item
                .get("endAt")
                .and_then(|v| v.as_str())
                .ok_or_else(|| AppError::validation("AI response missing endAt"))?
                .to_string();

            let confidence = item
                .get("confidence")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.75) as f32;

            let notes = item.get("notes").and_then(|v| v.as_str()).unwrap_or("");

            blocks.push(TimeBlockCandidate {
                id: Uuid::new_v4().to_string(),
                task_id: task_id.clone(),
                start_at,
                end_at,
                flexibility: Some("moderate".to_string()),
                confidence,
                conflict_flags: Vec::new(),
            });

            if !notes.is_empty() {
                rationale_steps.push(PlanRationaleStep {
                    step: idx + 1,
                    thought: format!("任务 {}: {}", task_id, notes),
                    result: None,
                });
            }
        }

        // Detect conflicts with constraints
        let conflicts = detect_conflicts(
            &blocks,
            &vec![], // No existing events for now
            None,
        )?;

        let option = PlanOption {
            id: Uuid::new_v4().to_string(),
            label: "DeepSeek AI 智能方案".to_string(),
            rank: 1,
            score: 90.0, // High score for AI-generated plan
            is_fallback: false,
            blocks,
            rationale: rationale_steps,
            conflicts: conflicts.clone(),
            risk_notes: if conflicts.is_empty() {
                vec!["AI 生成的智能规划方案，已优化任务时间分配".to_string()]
            } else {
                vec![format!(
                    "检测到 {} 个潜在冲突，可通过调整时间解决",
                    conflicts.len()
                )]
            },
        };

        options.push(option);
        Ok(options)
    }

    fn load_session_view(
        &self,
        session_id: &str,
        conn: &Connection,
    ) -> AppResult<PlanningSessionView> {
        let session_row = PlanningRepository::find_session_by_id(conn, session_id)?
            .ok_or_else(AppError::not_found)?;
        let session_record = session_row.clone().into_record()?;

        let preference_snapshot = session_record
            .personalization_snapshot
            .as_ref()
            .and_then(|value| serde_json::from_value::<PreferenceSnapshot>(value.clone()).ok());

        let option_rows = PlanningRepository::list_options_for_session(conn, session_id)?;
        let mut options = Vec::new();
        let mut aggregated_conflicts = Vec::new();

        for option_row in option_rows {
            let option_record = option_row.clone().into_record()?;
            let blocks_rows =
                PlanningRepository::list_time_blocks_for_option(conn, &option_row.id)?;
            let blocks = blocks_rows
                .into_iter()
                .map(|row| row.into_record())
                .collect::<AppResult<Vec<_>>>()?;

            let metadata = parse_risk_metadata(&option_row);
            merge_conflicts(&mut aggregated_conflicts, &metadata.conflicts);

            options.push(PlanningOptionView {
                option: option_record,
                blocks,
                conflicts: metadata.conflicts,
            });
        }

        let conflicts = dedupe_conflicts(aggregated_conflicts);

        Ok(PlanningSessionView {
            session: session_record,
            options,
            conflicts,
            preference_snapshot,
        })
    }
}

impl PlanningService {
    fn map_schedulable_task(task: &TaskRecord) -> SchedulableTask {
        SchedulableTask {
            id: task.id.clone(),
            title: task.title.clone(),
            due_at: task.due_at.clone(),
            earliest_start_at: task
                .start_at
                .as_ref()
                .or(task.planned_start_at.as_ref())
                .cloned(),
            estimated_minutes: task.estimated_minutes.or_else(|| {
                task.estimated_hours
                    .map(|hours| (hours * 60.0).round() as i64)
            }),
            priority_weight: priority_weight(&task.priority),
            is_parallelizable: task.tags.iter().any(|tag| {
                tag.eq_ignore_ascii_case("parallel") || tag.eq_ignore_ascii_case("parallelizable")
            }),
        }
    }
}

fn priority_weight(priority: &str) -> f32 {
    match priority.to_ascii_lowercase().as_str() {
        "urgent" => 1.2,
        "high" => 1.0,
        "medium" => 0.7,
        "low" => 0.4,
        _ => 0.6,
    }
}

fn scheduling_preferences_from(snapshot: &PreferenceSnapshot) -> SchedulingPreferences {
    SchedulingPreferences {
        focus_start_minute: snapshot.focus_start_minute,
        focus_end_minute: snapshot.focus_end_minute,
        buffer_minutes_between_blocks: snapshot.buffer_minutes_between_blocks,
        prefer_compact_schedule: snapshot.prefer_compact_schedule,
    }
}

fn build_option_summary(option: &PlanOption, tasks: &HashMap<String, TaskRecord>) -> String {
    let mut titles = Vec::new();
    for block in &option.blocks {
        if let Some(task) = tasks.get(&block.task_id) {
            if !titles.iter().any(|title: &String| title == &task.title) {
                titles.push(task.title.clone());
            }
        }
    }

    let preview = titles
        .iter()
        .take(3)
        .cloned()
        .collect::<Vec<_>>()
        .join("、");

    let suffix = if option.is_fallback {
        "（备选方案）"
    } else {
        ""
    };
    let block_count = option.blocks.len();
    let score = option.score;

    if preview.is_empty() {
        format!(
            "{}{} 含 {} 个时间块，综合评分 {:.1}",
            option.label, suffix, block_count, score
        )
    } else {
        format!(
            "{}{} 含 {} 个时间块，覆盖任务 {}，综合评分 {:.1}",
            option.label, suffix, block_count, preview, score
        )
    }
}

fn time_block_to_candidate(block: &PlanningTimeBlockRecord) -> AppResult<TimeBlockCandidate> {
    let flags = block
        .conflict_flags
        .as_ref()
        .and_then(|value| serde_json::from_value::<Vec<String>>(value.clone()).ok())
        .unwrap_or_default();

    Ok(TimeBlockCandidate {
        id: block.id.clone(),
        task_id: block.task_id.clone(),
        start_at: block.start_at.clone(),
        end_at: block.end_at.clone(),
        flexibility: block.flexibility.clone(),
        confidence: block.confidence.unwrap_or(0.75) as f32,
        conflict_flags: flags,
    })
}

fn apply_overrides(
    blocks: &mut [PlanningTimeBlockRecord],
    overrides: &[TimeBlockOverride],
) -> AppResult<()> {
    if overrides.is_empty() {
        return Ok(());
    }

    let index = blocks
        .iter()
        .enumerate()
        .map(|(idx, block)| (block.id.clone(), idx))
        .collect::<HashMap<_, _>>();

    for override_item in overrides {
        let position = index
            .get(&override_item.block_id)
            .copied()
            .ok_or_else(|| AppError::validation("尝试调整不存在的时间块"))?;
        let block = blocks
            .get_mut(position)
            .ok_or_else(|| AppError::validation("时间块索引越界"))?;

        if let Some(start) = &override_item.start_at {
            schedule_utils::parse_datetime(start)?;
            block.start_at = start.clone();
        }

        if let Some(end) = &override_item.end_at {
            schedule_utils::parse_datetime(end)?;
            block.end_at = end.clone();
        }

        let start_dt = schedule_utils::parse_datetime(&block.start_at)?;
        let end_dt = schedule_utils::parse_datetime(&block.end_at)?;
        schedule_utils::ensure_window(start_dt, end_dt)?;

        if let Some(flexibility) = &override_item.flexibility {
            block.flexibility = Some(flexibility.clone());
        }
    }

    Ok(())
}

fn update_block_conflict_flags(
    blocks: &mut [PlanningTimeBlockRecord],
    conflicts: &[ScheduleConflict],
) -> AppResult<()> {
    let mut flag_map = blocks
        .iter()
        .map(|block| {
            let flags = block
                .conflict_flags
                .as_ref()
                .and_then(|value| serde_json::from_value::<Vec<String>>(value.clone()).ok())
                .unwrap_or_default();
            (block.id.clone(), flags)
        })
        .collect::<HashMap<_, _>>();

    for conflict in conflicts {
        if let Some(block_id) = conflict.related_block_id.as_ref() {
            let entry = flag_map.entry(block_id.clone()).or_default();
            if !entry.iter().any(|flag| flag == &conflict.conflict_type) {
                entry.push(conflict.conflict_type.clone());
            }
        }
    }

    for block in blocks.iter_mut() {
        if let Some(flags) = flag_map.remove(&block.id) {
            block.conflict_flags = if flags.is_empty() {
                None
            } else {
                Some(json!(flags))
            };
        }
    }

    Ok(())
}

fn parse_risk_metadata(row: &PlanningOptionRow) -> OptionRiskMetadata {
    row.risk_notes
        .as_ref()
        .and_then(|raw| serde_json::from_str(raw).ok())
        .unwrap_or_default()
}

fn merge_conflicts(target: &mut Vec<ScheduleConflict>, incoming: &[ScheduleConflict]) {
    target.extend_from_slice(incoming);
}

fn dedupe_conflicts(conflicts: Vec<ScheduleConflict>) -> Vec<ScheduleConflict> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for conflict in conflicts {
        let key = conflict_key(&conflict);
        if seen.insert(key) {
            result.push(conflict);
        }
    }
    result
}

fn conflict_key(conflict: &ScheduleConflict) -> String {
    format!(
        "{}|{}|{}|{}",
        conflict.conflict_type,
        conflict.related_block_id.as_deref().unwrap_or("<none>"),
        conflict.related_event_id.as_deref().unwrap_or("<none>"),
        conflict.message
    )
}

fn earliest_start_by_task(
    blocks: &[PlanningTimeBlockRecord],
) -> AppResult<HashMap<String, String>> {
    let mut result: HashMap<String, (String, DateTime<FixedOffset>)> = HashMap::new();
    for block in blocks {
        let start = schedule_utils::parse_datetime(&block.start_at)?;
        match result.get_mut(&block.task_id) {
            Some(existing) => {
                if start < existing.1 {
                    *existing = (block.start_at.clone(), start);
                }
            }
            Option::None => {
                result.insert(block.task_id.clone(), (block.start_at.clone(), start));
            }
        }
    }

    Ok(result
        .into_iter()
        .map(|(task_id, (start, _))| (task_id, start))
        .collect())
}
