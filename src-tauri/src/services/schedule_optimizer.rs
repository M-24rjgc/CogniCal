use std::cmp::Ordering;

use chrono::{offset::LocalResult, DateTime, Duration, FixedOffset, NaiveTime, TimeZone};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::services::schedule_utils;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SchedulableTask {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub due_at: Option<String>,
    #[serde(default)]
    pub earliest_start_at: Option<String>,
    #[serde(default)]
    pub estimated_minutes: Option<i64>,
    pub priority_weight: f32,
    #[serde(default)]
    pub is_parallelizable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TimeWindow {
    pub start_at: String,
    pub end_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExistingEvent {
    pub id: String,
    pub start_at: String,
    pub end_at: String,
    #[serde(default)]
    pub event_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct SchedulingPreferences {
    #[serde(default)]
    pub focus_start_minute: Option<u32>,
    #[serde(default)]
    pub focus_end_minute: Option<u32>,
    #[serde(default)]
    pub buffer_minutes_between_blocks: i64,
    #[serde(default)]
    pub prefer_compact_schedule: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleConstraints {
    #[serde(default)]
    pub planning_start_at: Option<String>,
    #[serde(default)]
    pub planning_end_at: Option<String>,
    #[serde(default)]
    pub available_windows: Vec<TimeWindow>,
    #[serde(default)]
    pub existing_events: Vec<ExistingEvent>,
    #[serde(default)]
    pub max_focus_minutes_per_day: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TimeBlockCandidate {
    pub id: String,
    pub task_id: String,
    pub start_at: String,
    pub end_at: String,
    pub flexibility: Option<String>,
    pub confidence: f32,
    #[serde(default)]
    pub conflict_flags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PlanRationaleStep {
    pub step: usize,
    pub thought: String,
    #[serde(default)]
    pub result: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleConflict {
    pub conflict_type: String,
    pub severity: ConflictSeverity,
    pub message: String,
    #[serde(default)]
    pub related_block_id: Option<String>,
    #[serde(default)]
    pub related_event_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ConflictSeverity {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PlanOption {
    pub id: String,
    pub label: String,
    pub rank: usize,
    pub score: f64,
    pub is_fallback: bool,
    pub blocks: Vec<TimeBlockCandidate>,
    pub rationale: Vec<PlanRationaleStep>,
    pub conflicts: Vec<ScheduleConflict>,
    pub risk_notes: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum PlanVariant {
    DeadlineFirst,
    PriorityFirst,
    FocusAligned,
}

pub struct ScheduleOptimizer {
    seed: u64,
}

impl ScheduleOptimizer {
    pub fn new(seed: Option<u64>) -> Self {
        let seed = seed.unwrap_or(42);
        Self { seed }
    }

    pub fn generate_plan_options(
        &self,
        tasks: Vec<SchedulableTask>,
        constraints: ScheduleConstraints,
        preferences: SchedulingPreferences,
    ) -> AppResult<Vec<PlanOption>> {
        if tasks.is_empty() {
            return Err(AppError::validation("没有可用于规划的任务"));
        }

        let parsed_windows = self.prepare_windows(&tasks, &constraints)?;
        let planning_start = parsed_windows
            .first()
            .map(|w| w.start)
            .ok_or_else(|| AppError::validation("未找到可用时间窗口"))?;

        let mut variants = vec![PlanVariant::DeadlineFirst, PlanVariant::PriorityFirst];
        if preferences.focus_start_minute.is_some() || preferences.focus_end_minute.is_some() {
            variants.push(PlanVariant::FocusAligned);
        }

        let mut options = Vec::new();
        for (idx, variant) in variants.iter().enumerate() {
            let plan_id = Uuid::new_v4().to_string();
            let (blocks, rationale, risk_notes, fallback) = self.build_blocks_for_variant(
                &tasks,
                variant,
                &parsed_windows,
                planning_start,
                &preferences,
            )?;

            let conflicts = detect_conflicts(
                &blocks,
                &constraints.existing_events,
                constraints.max_focus_minutes_per_day,
            )?;

            let score = self.score_option(&blocks, &tasks, &preferences, &conflicts)?;

            options.push(PlanOption {
                id: plan_id,
                label: variant.label(),
                rank: idx + 1,
                score,
                is_fallback: fallback,
                blocks,
                rationale,
                conflicts,
                risk_notes,
            });
        }

        options.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal));
        for (idx, option) in options.iter_mut().enumerate() {
            option.rank = idx + 1;
        }

        Ok(options)
    }

    fn build_blocks_for_variant(
        &self,
        tasks: &[SchedulableTask],
        variant: &PlanVariant,
        windows: &[ParsedWindow],
        planning_start: DateTime<FixedOffset>,
        preferences: &SchedulingPreferences,
    ) -> AppResult<(
        Vec<TimeBlockCandidate>,
        Vec<PlanRationaleStep>,
        Vec<String>,
        bool,
    )> {
        let ordered_tasks = self.order_tasks(tasks, variant)?;
        let mut rationale = Vec::new();
        rationale.push(PlanRationaleStep {
            step: 1,
            thought: format!("按 {:?} 策略排序 {} 个任务", variant, ordered_tasks.len()),
            result: None,
        });

        let mut blocks = Vec::new();
        let mut risk_notes = Vec::new();
        let mut fallback = false;
        let buffer_minutes = preferences.buffer_minutes_between_blocks.max(0);

        let mut cursor_window_idx = 0;
        let mut cursor_time = planning_start;

        for task in ordered_tasks {
            let task_start_constraint = if let Some(raw) = &task.earliest_start_at {
                Some(schedule_utils::parse_datetime(raw)?)
            } else {
                None
            };

            let due_at = if let Some(raw) = &task.due_at {
                Some(schedule_utils::parse_datetime(raw)?)
            } else {
                None
            };

            if let Some(start_constraint) = task_start_constraint {
                if cursor_time < start_constraint {
                    cursor_time = start_constraint;
                }
            }

            let mut remaining = task.estimated_minutes.unwrap_or(60).max(15);
            let mut first_block = true;

            while remaining > 0 {
                if cursor_window_idx >= windows.len() {
                    fallback = true;
                    // 无可用窗口可继续，提前结束
                    break;
                }

                let current_window = &windows[cursor_window_idx];
                if cursor_time >= current_window.end {
                    cursor_window_idx += 1;
                    if cursor_window_idx < windows.len() {
                        cursor_time = windows[cursor_window_idx].start;
                        continue;
                    } else {
                        fallback = true;
                        break;
                    }
                }

                let aligned_start =
                    schedule_utils::clamp_time_to_window(cursor_time, current_window.start);
                let available_minutes =
                    schedule_utils::duration_minutes(aligned_start, current_window.end)?;

                if available_minutes <= 0 {
                    cursor_window_idx += 1;
                    if cursor_window_idx < windows.len() {
                        cursor_time = windows[cursor_window_idx].start;
                        continue;
                    } else {
                        fallback = true;
                        break;
                    }
                }

                let block_minutes = available_minutes.min(remaining);
                let end_time = schedule_utils::add_minutes(aligned_start, block_minutes)?;

                let mut flags = Vec::new();
                if !first_block {
                    flags.push("split-task".to_string());
                }

                if let Some(due) = due_at {
                    if end_time > due {
                        flags.push("deadline-risk".to_string());
                        risk_notes.push(format!(
                            "任务 {} 规划结束时间超出截止时间 {}",
                            task.title,
                            due.to_rfc3339()
                        ));
                    } else if (due - end_time).num_minutes() < 30 {
                        risk_notes.push(format!(
                            "任务 {} 仅剩 {} 分钟缓冲",
                            task.title,
                            (due - end_time).num_minutes()
                        ));
                    }
                }

                if !preferences.prefer_compact_schedule && block_minutes > 120 {
                    flags.push("long-session".to_string());
                }

                let block_id = Uuid::new_v4().to_string();
                blocks.push(TimeBlockCandidate {
                    id: block_id,
                    task_id: task.id.clone(),
                    start_at: schedule_utils::format_datetime(aligned_start),
                    end_at: schedule_utils::format_datetime(end_time),
                    flexibility: Some(if task.is_parallelizable {
                        "flexible".to_string()
                    } else {
                        "fixed".to_string()
                    }),
                    confidence: self.estimate_confidence(block_minutes, &preferences, &flags),
                    conflict_flags: flags,
                });

                remaining -= block_minutes;
                cursor_time = schedule_utils::add_minutes(end_time, buffer_minutes)?;
                first_block = false;

                if remaining > 0 {
                    rationale.push(PlanRationaleStep {
                        step: rationale.len() + 1,
                        thought: format!("任务 {} 需要拆分，剩余 {} 分钟", task.title, remaining),
                        result: None,
                    });
                }
            }

            if remaining > 0 {
                risk_notes.push(format!(
                    "任务 {} 未能完全排程，剩余 {} 分钟",
                    task.title, remaining
                ));
                fallback = true;
            }
        }

        rationale.push(PlanRationaleStep {
            step: rationale.len() + 1,
            thought: "完成时间块生成".to_string(),
            result: Some(format!("共生成 {} 个时间块", blocks.len())),
        });

        Ok((blocks, rationale, risk_notes, fallback))
    }

    fn order_tasks(
        &self,
        tasks: &[SchedulableTask],
        variant: &PlanVariant,
    ) -> AppResult<Vec<SchedulableTask>> {
        let mut tasks = tasks.to_vec();
        match variant {
            PlanVariant::DeadlineFirst => {
                tasks.sort_by(|a, b| {
                    compare_datetime_opt(&a.due_at, &b.due_at).then_with(|| self.tie_breaker(a, b))
                });
            }
            PlanVariant::PriorityFirst => {
                tasks.sort_by(|a, b| {
                    b.priority_weight
                        .partial_cmp(&a.priority_weight)
                        .unwrap_or(Ordering::Equal)
                        .then_with(|| compare_datetime_opt(&a.due_at, &b.due_at))
                        .then_with(|| self.tie_breaker(a, b))
                });
            }
            PlanVariant::FocusAligned => {
                tasks.sort_by(|a, b| {
                    let earliest_a =
                        compare_datetime_opt(&a.earliest_start_at, &b.earliest_start_at);
                    if earliest_a == Ordering::Equal {
                        compare_datetime_opt(&a.due_at, &b.due_at)
                            .then_with(|| self.tie_breaker(a, b))
                    } else {
                        earliest_a
                    }
                });
            }
        }
        Ok(tasks)
    }

    fn prepare_windows(
        &self,
        tasks: &[SchedulableTask],
        constraints: &ScheduleConstraints,
    ) -> AppResult<Vec<ParsedWindow>> {
        let mut windows = Vec::new();
        for window in &constraints.available_windows {
            let start = schedule_utils::parse_datetime(&window.start_at)?;
            let end = schedule_utils::parse_datetime(&window.end_at)?;
            schedule_utils::ensure_window(start, end)?;
            windows.push(ParsedWindow { start, end });
        }

        if windows.is_empty() {
            let fallback_start = if let Some(raw) = constraints
                .planning_start_at
                .as_ref()
                .and_then(|raw| schedule_utils::parse_datetime(raw).ok())
            {
                raw
            } else if let Ok(Some(candidate)) = earliest_task_time(tasks) {
                candidate
            } else {
                current_fixed_offset(self.seed)
            };

            let fallback_end = constraints
                .planning_end_at
                .as_ref()
                .and_then(|raw| schedule_utils::parse_datetime(raw).ok())
                .unwrap_or_else(|| fallback_start + Duration::days(3));

            let mut day_start = fallback_start;
            while day_start < fallback_end {
                let window_start =
                    build_window_time(day_start, NaiveTime::from_hms_opt(9, 0, 0).unwrap());
                let window_end =
                    build_window_time(day_start, NaiveTime::from_hms_opt(18, 0, 0).unwrap());

                schedule_utils::ensure_window(window_start, window_end)?;
                windows.push(ParsedWindow {
                    start: window_start,
                    end: window_end,
                });

                day_start += Duration::days(1);
            }
        }

        windows.sort_by_key(|w| w.start);
        Ok(windows)
    }

    fn score_option(
        &self,
        blocks: &[TimeBlockCandidate],
        tasks: &[SchedulableTask],
        preferences: &SchedulingPreferences,
        conflicts: &[ScheduleConflict],
    ) -> AppResult<f64> {
        let mut lateness_penalty = 0.0;
        for block in blocks {
            if let Some(task) = tasks.iter().find(|t| t.id == block.task_id) {
                if let Some(due) = &task.due_at {
                    let due_time = schedule_utils::parse_datetime(due)?;
                    let block_end = schedule_utils::parse_datetime(&block.end_at)?;
                    if block_end > due_time {
                        lateness_penalty += (block_end - due_time).num_minutes().max(0) as f64;
                    }
                }
            }
        }

        let conflict_penalty: f64 = conflicts
            .iter()
            .map(|c| match c.severity {
                ConflictSeverity::Low => 10.0,
                ConflictSeverity::Medium => 30.0,
                ConflictSeverity::High => 60.0,
            })
            .sum();

        let focus_bonus = if let (Some(start), Some(end)) =
            (preferences.focus_start_minute, preferences.focus_end_minute)
        {
            let preferred_range = start..end;
            let mut aligned_minutes = 0.0;
            let mut total_minutes = 0.0;
            for block in blocks {
                let start_time = schedule_utils::parse_datetime(&block.start_at)?;
                let end_time = schedule_utils::parse_datetime(&block.end_at)?;
                let block_minutes = schedule_utils::duration_minutes(start_time, end_time)? as f64;
                total_minutes += block_minutes;

                let start_minute = schedule_utils::midnight_minutes_of(start_time) as u32;
                let end_minute = schedule_utils::midnight_minutes_of(end_time) as u32;
                let overlap = overlap_in_minutes(start_minute, end_minute, &preferred_range);
                aligned_minutes += overlap as f64;
            }

            if total_minutes > 0.0 {
                (aligned_minutes / total_minutes) * 80.0
            } else {
                0.0
            }
        } else {
            0.0
        };

        let mut base = 100.0 - lateness_penalty * 0.2 - conflict_penalty;
        base += focus_bonus;

        if preferences.prefer_compact_schedule {
            let compact_penalty = blocks
                .windows(2)
                .map(|window_pair| -> AppResult<f64> {
                    let prev_end = schedule_utils::parse_datetime(&window_pair[0].end_at)?;
                    let next_start = schedule_utils::parse_datetime(&window_pair[1].start_at)?;
                    Ok(schedule_utils::duration_minutes(prev_end, next_start)?.abs() as f64)
                })
                .collect::<AppResult<Vec<_>>>()?
                .into_iter()
                .sum::<f64>();
            base -= compact_penalty * 0.05;
        }

        Ok(base.max(0.0))
    }

    fn estimate_confidence(
        &self,
        block_minutes: i64,
        preferences: &SchedulingPreferences,
        flags: &[String],
    ) -> f32 {
        let mut confidence: f32 = 0.85;
        if block_minutes > 120 {
            confidence -= 0.1;
        }
        if preferences.buffer_minutes_between_blocks < 10 {
            confidence -= 0.05;
        }
        if flags.iter().any(|f| f == "deadline-risk") {
            confidence -= 0.2;
        }
        confidence.clamp(0.0, 1.0)
    }
}

pub fn detect_conflicts(
    blocks: &[TimeBlockCandidate],
    existing_events: &[ExistingEvent],
    max_daily_minutes: Option<i64>,
) -> AppResult<Vec<ScheduleConflict>> {
    let mut conflicts = Vec::new();

    for block in blocks {
        let block_start = schedule_utils::parse_datetime(&block.start_at)?;
        let block_end = schedule_utils::parse_datetime(&block.end_at)?;

        for event in existing_events {
            let event_start = schedule_utils::parse_datetime(&event.start_at)?;
            let event_end = schedule_utils::parse_datetime(&event.end_at)?;

            if schedule_utils::overlaps(block_start, block_end, event_start, event_end)? {
                conflicts.push(ScheduleConflict {
                    conflict_type: "calendar-overlap".to_string(),
                    severity: ConflictSeverity::High,
                    message: format!(
                        "时间块 [{} - {}] 与事件 {} 冲突",
                        block.start_at, block.end_at, event.id
                    ),
                    related_block_id: Some(block.id.clone()),
                    related_event_id: Some(event.id.clone()),
                });
            }
        }
    }

    let mut day_totals = std::collections::BTreeMap::new();
    for block in blocks {
        let start = schedule_utils::parse_datetime(&block.start_at)?;
        let end = schedule_utils::parse_datetime(&block.end_at)?;
        let minutes = schedule_utils::duration_minutes(start, end)?;
        let entry = day_totals.entry(start.date_naive()).or_insert(0);
        *entry += minutes;
    }

    if let Some(limit) = max_daily_minutes {
        for (day, minutes) in day_totals {
            if minutes > limit {
                conflicts.push(ScheduleConflict {
                    conflict_type: "daily-overload".to_string(),
                    severity: ConflictSeverity::Medium,
                    message: format!("{} 当日排程 {} 分钟，超过上限 {} 分钟", day, minutes, limit),
                    related_block_id: None,
                    related_event_id: None,
                });
            }
        }
    }

    conflicts.sort_by(|a, b| match (&a.severity, &b.severity) {
        (ConflictSeverity::High, ConflictSeverity::Medium)
        | (ConflictSeverity::High, ConflictSeverity::Low)
        | (ConflictSeverity::Medium, ConflictSeverity::Low) => Ordering::Less,
        (ConflictSeverity::Medium, ConflictSeverity::High)
        | (ConflictSeverity::Low, ConflictSeverity::High)
        | (ConflictSeverity::Low, ConflictSeverity::Medium) => Ordering::Greater,
        _ => Ordering::Equal,
    });

    Ok(conflicts)
}

fn compare_datetime_opt(a: &Option<String>, b: &Option<String>) -> Ordering {
    match (a, b) {
        (Some(a), Some(b)) => match (
            schedule_utils::parse_datetime(a),
            schedule_utils::parse_datetime(b),
        ) {
            (Ok(a_dt), Ok(b_dt)) => a_dt.cmp(&b_dt),
            _ => Ordering::Equal,
        },
        (Some(_), Option::None) => Ordering::Less,
        (Option::None, Some(_)) => Ordering::Greater,
        (Option::None, Option::None) => Ordering::Equal,
    }
}

fn earliest_task_time(tasks: &[SchedulableTask]) -> AppResult<Option<DateTime<FixedOffset>>> {
    let mut earliest: Option<DateTime<FixedOffset>> = None;
    for task in tasks {
        if let Some(start) = &task.earliest_start_at {
            let parsed = schedule_utils::parse_datetime(start)?;
            match earliest {
                Some(current) if parsed >= current => {}
                _ => earliest = Some(parsed),
            }
        }
        if let Some(due) = &task.due_at {
            let parsed = schedule_utils::parse_datetime(due)?;
            match earliest {
                Some(current) if parsed >= current => {}
                _ => earliest = Some(parsed - Duration::hours(2)),
            }
        }
    }
    Ok(earliest)
}

fn current_fixed_offset(seed: u64) -> DateTime<FixedOffset> {
    let now = chrono::Utc::now();
    let offset = FixedOffset::east_opt(0).expect("UTC offset should exist");
    let base = now.with_timezone(&offset);
    // 为了在测试中保持确定性，根据 seed 调整秒数但不影响分钟级调度
    let adjustment = (seed % 60) as i64;
    base + Duration::seconds(adjustment)
}

fn overlap_in_minutes(start: u32, end: u32, range: &std::ops::Range<u32>) -> u32 {
    if start >= range.end || end <= range.start {
        0
    } else {
        let effective_start = start.max(range.start);
        let effective_end = end.min(range.end);
        effective_end.saturating_sub(effective_start)
    }
}

#[derive(Debug, Clone, Copy)]
struct ParsedWindow {
    start: DateTime<FixedOffset>,
    end: DateTime<FixedOffset>,
}

impl PlanVariant {
    fn label(&self) -> String {
        match self {
            PlanVariant::DeadlineFirst => "截止时间优先".to_string(),
            PlanVariant::PriorityFirst => "优先级优先".to_string(),
            PlanVariant::FocusAligned => "专注时段优先".to_string(),
        }
    }
}

impl ScheduleOptimizer {
    fn tie_breaker(&self, a: &SchedulableTask, b: &SchedulableTask) -> Ordering {
        let a_hash = deterministic_hash(&a.id, self.seed);
        let b_hash = deterministic_hash(&b.id, self.seed);
        a_hash.cmp(&b_hash)
    }
}

fn deterministic_hash(value: &str, seed: u64) -> u64 {
    let mut hash: u64 = seed; // FNV-1a like simple hash
    for byte in value.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(1_099_511_628_211u64);
    }
    hash
}

fn build_window_time(
    day_start: DateTime<FixedOffset>,
    naive_time: NaiveTime,
) -> DateTime<FixedOffset> {
    let offset = day_start.offset().clone();
    let naive = day_start.date_naive().and_time(naive_time);
    match offset.from_local_datetime(&naive) {
        LocalResult::Single(dt) => dt,
        LocalResult::Ambiguous(first, _) => first,
        LocalResult::None => day_start,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::AppResult;
    use chrono::{Duration, NaiveDate, TimeZone};

    fn dt(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> DateTime<FixedOffset> {
        let offset = FixedOffset::east_opt(0).expect("offset");
        let naive = NaiveDate::from_ymd_opt(year, month, day)
            .expect("valid date")
            .and_hms_opt(hour, minute, 0)
            .expect("valid time");
        offset
            .from_local_datetime(&naive)
            .single()
            .expect("valid datetime")
    }

    fn iso(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> String {
        schedule_utils::format_datetime(dt(year, month, day, hour, minute))
    }

    #[test]
    fn generate_plan_options_detects_conflicts_and_sorts_by_score() -> AppResult<()> {
        let optimizer = ScheduleOptimizer::new(Some(7));
        let tasks = vec![
            SchedulableTask {
                id: "task-1".to_string(),
                title: "Spec Draft".to_string(),
                due_at: Some(iso(2025, 5, 1, 12, 0)),
                earliest_start_at: Some(iso(2025, 5, 1, 9, 0)),
                estimated_minutes: Some(150),
                priority_weight: 0.9,
                is_parallelizable: false,
            },
            SchedulableTask {
                id: "task-2".to_string(),
                title: "API Wiring".to_string(),
                due_at: Some(iso(2025, 5, 1, 16, 0)),
                earliest_start_at: None,
                estimated_minutes: Some(120),
                priority_weight: 0.7,
                is_parallelizable: true,
            },
            SchedulableTask {
                id: "task-3".to_string(),
                title: "Review".to_string(),
                due_at: Some(iso(2025, 5, 1, 18, 0)),
                earliest_start_at: Some(iso(2025, 5, 1, 13, 0)),
                estimated_minutes: Some(120),
                priority_weight: 0.5,
                is_parallelizable: false,
            },
        ];

        let constraints = ScheduleConstraints {
            available_windows: vec![TimeWindow {
                start_at: iso(2025, 5, 1, 9, 0),
                end_at: iso(2025, 5, 1, 13, 0),
            }],
            existing_events: vec![ExistingEvent {
                id: "event-1".to_string(),
                start_at: iso(2025, 5, 1, 10, 0),
                end_at: iso(2025, 5, 1, 11, 0),
                event_type: Some("meeting".to_string()),
            }],
            max_focus_minutes_per_day: Some(210),
            ..Default::default()
        };

        let preferences = SchedulingPreferences {
            focus_start_minute: Some(8 * 60 + 30),
            focus_end_minute: Some(12 * 60 + 30),
            buffer_minutes_between_blocks: 15,
            prefer_compact_schedule: true,
        };

        let options = optimizer.generate_plan_options(tasks, constraints, preferences)?;
        assert!(options.len() >= 2);
        for pair in options.windows(2) {
            assert!(pair[0].score + f64::EPSILON >= pair[1].score);
        }
        assert!(options.iter().any(|opt| opt.is_fallback));

        let conflict_option = options
            .iter()
            .find(|opt| !opt.conflicts.is_empty())
            .expect("expected at least one option with conflicts");
        let first = &conflict_option.conflicts[0];
        assert_eq!(first.conflict_type, "calendar-overlap");
        assert_eq!(first.severity, ConflictSeverity::High);

        Ok(())
    }

    #[test]
    fn detect_conflicts_prioritizes_high_severity_and_daily_limits() -> AppResult<()> {
        let start = dt(2025, 5, 2, 9, 0);
        let block = TimeBlockCandidate {
            id: "block-1".to_string(),
            task_id: "task-1".to_string(),
            start_at: schedule_utils::format_datetime(start),
            end_at: schedule_utils::format_datetime(start + Duration::minutes(120)),
            flexibility: None,
            confidence: 0.8,
            conflict_flags: Vec::new(),
        };

        let overlapping = ExistingEvent {
            id: "event".to_string(),
            start_at: schedule_utils::format_datetime(start + Duration::minutes(30)),
            end_at: schedule_utils::format_datetime(start + Duration::minutes(90)),
            event_type: None,
        };

        let conflicts = detect_conflicts(&[block.clone()], &[overlapping], Some(60))?;
        assert_eq!(conflicts.len(), 2);
        assert_eq!(conflicts[0].conflict_type, "calendar-overlap");
        assert_eq!(conflicts[0].severity, ConflictSeverity::High);
        assert!(conflicts
            .iter()
            .any(|conflict| conflict.conflict_type == "daily-overload"));

        Ok(())
    }
}
