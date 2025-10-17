use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration as StdDuration;

use chrono::{DateTime, Duration, NaiveDate, TimeZone, Utc};
use tracing::{debug, error};

use crate::db::repositories::analytics_repository::{AnalyticsRepository, AnalyticsSnapshotRow};
use crate::db::repositories::planning_repository::PlanningTimeBlockRow;
use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::analytics::{
    AnalyticsEfficiency, AnalyticsExportFormat, AnalyticsExportParams, AnalyticsExportResult,
    AnalyticsGrouping, AnalyticsHistoryPoint, AnalyticsHistoryResponse, AnalyticsMeta,
    AnalyticsOverview, AnalyticsOverviewResponse, AnalyticsQueryParams, AnalyticsRangeKey,
    AnalyticsSnapshotRecord, AnalyticsSummary, EfficiencySuggestion, InsightCard,
    TimeAllocationBreakdown, TimeAllocationEntry, TimeAllocationPriorityEntry,
    TimeAllocationTypeEntry, TrendPoint, ZeroStateMeta,
};
use crate::models::planning::PlanningTimeBlockRecord;
use crate::models::task::TaskRecord;
use crate::services::task_service::TaskService;

const CACHE_TTL_SECONDS: i64 = 60;
const MIN_ESTIMATED_MINUTES: i64 = 15;
const REPORT_PREFIX: &str = "analytics-report";
const SNAPSHOT_JOB_HOUR: u32 = 1;
const SNAPSHOT_JOB_MINUTE: u32 = 15;
const SNAPSHOT_MIN_SLEEP_SECS: u64 = 60;
const SNAPSHOT_FALLBACK_SLEEP_SECS: u64 = 3600;
const SNAPSHOT_RETENTION_DAYS: i64 = 120;
const SNAPSHOT_LOOKBACK_DAYS: i64 = 7;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    range: AnalyticsRangeKey,
    start_ts: i64,
    end_ts: i64,
    grouping: AnalyticsGrouping,
}

#[derive(Clone)]
struct CachedOverview {
    response: AnalyticsOverviewResponse,
    cached_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
struct ResolvedQuery {
    params: AnalyticsQueryParams,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    grouping: AnalyticsGrouping,
    cache_key: CacheKey,
}

#[derive(Default, Clone)]
struct DailyStats {
    completed: i64,
    due: i64,
    focus_minutes: i64,
    overdue: i64,
}

pub struct AnalyticsService {
    db: DbPool,
    task_service: Arc<TaskService>,
    cache: RwLock<HashMap<CacheKey, CachedOverview>>,
    cache_ttl: Duration,
    reports_dir: PathBuf,
    snapshot_job_started: AtomicBool,
}

impl AnalyticsService {
    pub fn new(db: DbPool, task_service: Arc<TaskService>) -> AppResult<Self> {
        let reports_dir = default_reports_dir(db.path());
        std::fs::create_dir_all(&reports_dir)?;
        Ok(Self {
            db,
            task_service,
            cache: RwLock::new(HashMap::new()),
            cache_ttl: Duration::seconds(CACHE_TTL_SECONDS),
            reports_dir,
            snapshot_job_started: AtomicBool::new(false),
        })
    }

    pub fn ensure_snapshot_job(self: &Arc<Self>) -> AppResult<()> {
        if self
            .snapshot_job_started
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            let service = Arc::clone(self);
            if let Err(err) = service.capture_snapshot_for_previous_day() {
                error!(
                    target: "app::analytics",
                    error = %err,
                    "initial analytics snapshot capture failed"
                );
            }

            let runner = Arc::clone(self);
            if let Err(err) = thread::Builder::new()
                .name("analytics-snapshot-job".to_string())
                .spawn(move || {
                    runner.run_snapshot_loop();
                })
            {
                self.snapshot_job_started.store(false, Ordering::SeqCst);
                error!(
                    target: "app::analytics",
                    error = %err,
                    "failed to start analytics snapshot thread"
                );
                return Err(AppError::other(format!("无法启动分析快照任务: {err}")));
            }
        }

        Ok(())
    }

    pub fn fetch_overview(
        &self,
        params: AnalyticsQueryParams,
    ) -> AppResult<AnalyticsOverviewResponse> {
        let resolved = self.resolve_query(params)?;
        if let Some(cached) = self.try_get_cache(&resolved.cache_key) {
            debug!(target: "app::analytics", range = %resolved.params.range.as_str(), "analytics cache hit");
            return Ok(cached);
        }

        let response = self.compute_overview(&resolved)?;
        self.insert_cache(resolved.cache_key, response.clone());
        Ok(response)
    }

    pub fn fetch_history(
        &self,
        params: AnalyticsQueryParams,
    ) -> AppResult<AnalyticsHistoryResponse> {
        let mut params = params;
        if params.grouping.is_none() {
            params.grouping = Some(default_grouping(params.range));
        }
        let overview = self.fetch_overview(params)?;
        Ok(overview.history)
    }

    pub fn export_report(&self, params: AnalyticsExportParams) -> AppResult<AnalyticsExportResult> {
        let query_params = AnalyticsQueryParams {
            range: params.range,
            from: params.from.clone(),
            to: params.to.clone(),
            grouping: None,
        };
        let overview = self.fetch_overview(query_params)?;
        self.generate_report_file(overview, params.format)
    }

    fn resolve_query(&self, params: AnalyticsQueryParams) -> AppResult<ResolvedQuery> {
        let now = Utc::now();
        let grouping = params
            .grouping
            .unwrap_or_else(|| default_grouping(params.range));

        let end = match params.to.as_deref() {
            Some(value) => parse_query_datetime(value)?,
            None => now,
        };

        let start = match params.from.as_deref() {
            Some(value) => parse_query_datetime(value)?,
            None => end - params.range.duration(),
        };

        if start > end {
            return Err(AppError::validation("时间范围不合法"));
        }

        let cache_key = CacheKey {
            range: params.range,
            start_ts: start.timestamp(),
            end_ts: end.timestamp(),
            grouping,
        };

        Ok(ResolvedQuery {
            params,
            start,
            end,
            grouping,
            cache_key,
        })
    }

    fn compute_overview(&self, resolved: &ResolvedQuery) -> AppResult<AnalyticsOverviewResponse> {
        let tasks = self.task_service.list_tasks()?;
        let blocks = self.load_time_blocks(resolved.start, resolved.end)?;
        let daily_stats = build_daily_stats(&tasks, &blocks, resolved.start, resolved.end);
        let history_points = build_history_points(&daily_stats, resolved.grouping);

        let total_completed: i64 = daily_stats.iter().map(|(_, stats)| stats.completed).sum();
        let total_due: i64 = daily_stats.iter().map(|(_, stats)| stats.due).sum();
        let total_focus_minutes: i64 = daily_stats
            .iter()
            .map(|(_, stats)| stats.focus_minutes)
            .sum();
        let latest_overdue = daily_stats
            .last()
            .map(|(_, stats)| stats.overdue)
            .unwrap_or(0);

        let completion_rate = if total_due > 0 {
            clamp_ratio(total_completed as f64 / total_due as f64)
        } else if total_completed > 0 {
            1.0
        } else {
            0.0
        };

        let trend_delta = if history_points.len() >= 2 {
            let first = history_points.first().unwrap().completion_rate;
            let last = history_points.last().unwrap().completion_rate;
            ((last - first) * 100.0).round() / 10.0
        } else {
            0.0
        };

        let workload_prediction = predict_workload(&tasks);

        let (time_allocation, estimated_total) = build_time_allocation(&tasks);
        let (efficiency, suggestions) =
            build_efficiency_metrics(&tasks, &blocks, total_focus_minutes, estimated_total);

        let insights = build_insights(
            total_completed,
            completion_rate,
            total_focus_minutes,
            resolved.start,
            resolved.end,
        );

        let zero_state = ZeroStateMeta {
            is_empty: tasks.is_empty(),
            recommended_actions: if tasks.is_empty() {
                vec![
                    "创建你的第一项任务".to_string(),
                    "生成一份规划方案".to_string(),
                ]
            } else {
                Vec::new()
            },
            sample_data_available: false,
            sample_data_label: None,
            missing_configuration: None,
        };

        let trend: Vec<TrendPoint> = history_points
            .iter()
            .map(|point| TrendPoint {
                date: point.date.clone(),
                completion_rate: clamp_ratio(point.completion_rate),
                productivity_score: (point.productivity_score * 10.0).round() / 10.0,
                completed_tasks: point.completed_tasks,
                focus_minutes: point.focus_minutes,
            })
            .collect();

        let overview = AnalyticsOverview {
            range: resolved.params.range,
            summary: AnalyticsSummary {
                total_completed,
                completion_rate: (completion_rate * 1000.0).round() / 1000.0,
                trend_delta,
                workload_prediction,
                focus_minutes: total_focus_minutes,
                overdue_tasks: latest_overdue,
            },
            trend,
            time_allocation,
            efficiency: AnalyticsEfficiency {
                estimate_accuracy: (efficiency.estimate_accuracy * 1000.0).round() / 1000.0,
                on_time_rate: (efficiency.on_time_rate * 1000.0).round() / 1000.0,
                complexity_correlation: (efficiency.complexity_correlation * 1000.0).round()
                    / 1000.0,
                suggestions,
            },
            insights,
            zero_state,
            meta: AnalyticsMeta {
                generated_at: Utc::now().to_rfc3339(),
                is_demo: false,
            },
        };

        let history = AnalyticsHistoryResponse {
            range: resolved.params.range,
            grouping: resolved.grouping,
            points: history_points,
        };

        Ok(AnalyticsOverviewResponse {
            overview,
            history,
            error: None,
        })
    }

    fn load_time_blocks(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> AppResult<Vec<PlanningTimeBlockRecord>> {
        self.db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                r#"
                SELECT
                    id,
                    option_id,
                    task_id,
                    start_at,
                    end_at,
                    flexibility,
                    confidence,
                    conflict_flags,
                    applied_at,
                    actual_start_at,
                    actual_end_at,
                    status
                FROM planning_time_blocks
                WHERE COALESCE(actual_end_at, end_at) >= :start
                  AND COALESCE(actual_start_at, start_at) <= :end
            "#,
            )?;

            let rows = stmt
                .query_map(
                    rusqlite::named_params! { ":start": start.to_rfc3339(), ":end": end.to_rfc3339() },
                    |row| PlanningTimeBlockRow::try_from(row),
                )?
                .collect::<Result<Vec<_>, _>>()?;

            rows.into_iter()
                .map(|row| row.into_record())
                .collect()
        })
    }

    fn try_get_cache(&self, key: &CacheKey) -> Option<AnalyticsOverviewResponse> {
        let now = Utc::now();
        self.cache
            .read()
            .ok()
            .and_then(|guard| guard.get(key).cloned())
            .and_then(|entry| {
                if now - entry.cached_at <= self.cache_ttl {
                    Some(entry.response)
                } else {
                    None
                }
            })
    }

    fn insert_cache(&self, key: CacheKey, response: AnalyticsOverviewResponse) {
        if let Ok(mut guard) = self.cache.write() {
            guard.insert(
                key,
                CachedOverview {
                    response,
                    cached_at: Utc::now(),
                },
            );
        }
    }

    fn generate_report_file(
        &self,
        overview: AnalyticsOverviewResponse,
        format: AnalyticsExportFormat,
    ) -> AppResult<AnalyticsExportResult> {
        let timestamp = Utc::now().format("%Y%m%dT%H%M%SZ");
        let extension = format.file_extension();
        let filename = format!("{REPORT_PREFIX}-{}.{}", timestamp, extension);
        let path = self.reports_dir.join(filename);

        match format {
            AnalyticsExportFormat::Markdown => {
                let content = render_markdown_report(&overview);
                std::fs::write(&path, content)?;
            }
            AnalyticsExportFormat::Json => {
                let json = serde_json::to_string_pretty(&overview)?;
                std::fs::write(&path, json)?;
            }
        }

        Ok(AnalyticsExportResult {
            file_path: path.to_string_lossy().to_string(),
            format,
            generated_at: Utc::now().to_rfc3339(),
            is_demo: false,
        })
    }

    fn run_snapshot_loop(self: Arc<Self>) {
        loop {
            let now = Utc::now();
            let next_run = Self::next_snapshot_run(now);
            let sleep_duration = duration_until(next_run, now);
            thread::sleep(sleep_duration);

            if let Err(err) = self.capture_snapshot_for_previous_day() {
                error!(
                    target: "app::analytics",
                    error = %err,
                    "scheduled analytics snapshot capture failed"
                );
            }
        }
    }

    fn capture_snapshot_for_previous_day(&self) -> AppResult<()> {
        let today = Utc::now().date_naive();
        let target = today.pred_opt().unwrap_or(today);
        self.capture_snapshot_for_date(target)
    }

    fn capture_snapshot_for_date(&self, date: NaiveDate) -> AppResult<()> {
        let record = self.build_snapshot_record(date)?;
        let retention_cutoff = Self::retention_cutoff(date);
        self.persist_snapshot(&record, retention_cutoff)
    }

    fn build_snapshot_record(&self, date: NaiveDate) -> AppResult<AnalyticsSnapshotRecord> {
        let day_start = Utc.from_utc_datetime(&date.and_hms_opt(0, 0, 0).unwrap());
        let day_end = Utc.from_utc_datetime(&date.and_hms_opt(23, 59, 59).unwrap());

        let tasks = self.task_service.list_tasks()?;
        let day_blocks = self.load_time_blocks(day_start, day_end)?;

        let lookback_start = if SNAPSHOT_LOOKBACK_DAYS > 1 {
            day_start - Duration::days(SNAPSHOT_LOOKBACK_DAYS - 1)
        } else {
            day_start
        };

        let lookback_blocks = if SNAPSHOT_LOOKBACK_DAYS > 1 {
            self.load_time_blocks(lookback_start, day_end)?
        } else {
            day_blocks.clone()
        };

        let window_stats = build_daily_stats(&tasks, &lookback_blocks, lookback_start, day_end);
        let day_stats = window_stats
            .iter()
            .find(|(day, _)| *day == date)
            .map(|(_, stats)| stats.clone())
            .unwrap_or_default();

        let relevant_tasks: Vec<TaskRecord> = tasks
            .iter()
            .filter(|task| task_relevant_for_range(task, day_start, day_end, &day_blocks))
            .cloned()
            .collect();

        let estimated_total_minutes: i64 = relevant_tasks.iter().map(task_estimated_minutes).sum();

        let (efficiency, _) = build_efficiency_metrics(
            relevant_tasks.as_slice(),
            day_blocks.as_slice(),
            day_stats.focus_minutes,
            estimated_total_minutes,
        );

        let completion_rate =
            (completion_ratio(day_stats.completed, day_stats.due) * 10000.0).round() / 10000.0;
        let productivity_score =
            (productivity(day_stats.completed, day_stats.focus_minutes) * 100.0).round() / 100.0;
        let efficiency_rating = (mean(&[
            efficiency.estimate_accuracy,
            efficiency.on_time_rate,
            efficiency.complexity_correlation,
        ]) * 1000.0)
            .round()
            / 1000.0;

        let (time_spent_work, time_spent_study, time_spent_life, time_spent_other) =
            time_spent_breakdown(&relevant_tasks);

        let focus_samples: Vec<f64> = window_stats
            .iter()
            .map(|(_, stats)| stats.focus_minutes as f64)
            .collect();
        let focus_consistency = round_ratio(compute_focus_consistency(&focus_samples));
        let rest_balance = round_ratio(compute_rest_balance(day_stats.focus_minutes));
        let capacity_risk = round_ratio(compute_capacity_risk(
            tasks.as_slice(),
            day_stats.overdue,
            estimated_total_minutes,
        ));

        Ok(AnalyticsSnapshotRecord {
            snapshot_date: date.to_string(),
            total_tasks_completed: day_stats.completed,
            completion_rate,
            overdue_tasks: day_stats.overdue,
            total_focus_minutes: day_stats.focus_minutes,
            productivity_score,
            efficiency_rating,
            time_spent_work,
            time_spent_study,
            time_spent_life,
            time_spent_other,
            on_time_ratio: efficiency.on_time_rate,
            focus_consistency,
            rest_balance,
            capacity_risk,
            created_at: Utc::now().to_rfc3339(),
        })
    }

    fn persist_snapshot(
        &self,
        record: &AnalyticsSnapshotRecord,
        retention_cutoff: Option<NaiveDate>,
    ) -> AppResult<()> {
        let row = AnalyticsSnapshotRow::from_record(record);
        self.db.with_connection(|conn| {
            AnalyticsRepository::upsert_snapshot(conn, &row)?;
            if let Some(cutoff) = retention_cutoff {
                let _ = AnalyticsRepository::delete_before(conn, &cutoff)?;
            }
            Ok(())
        })
    }

    fn next_snapshot_run(now: DateTime<Utc>) -> DateTime<Utc> {
        let today_target = now
            .date_naive()
            .and_hms_opt(SNAPSHOT_JOB_HOUR, SNAPSHOT_JOB_MINUTE, 0)
            .unwrap();
        let candidate = Utc.from_utc_datetime(&today_target);
        if candidate > now {
            candidate
        } else {
            let next_date = now
                .date_naive()
                .succ_opt()
                .unwrap_or_else(|| now.date_naive());
            let next_target = next_date
                .and_hms_opt(SNAPSHOT_JOB_HOUR, SNAPSHOT_JOB_MINUTE, 0)
                .unwrap();
            Utc.from_utc_datetime(&next_target)
        }
    }

    fn retention_cutoff(date: NaiveDate) -> Option<NaiveDate> {
        if SNAPSHOT_RETENTION_DAYS <= 0 {
            return None;
        }
        date.checked_sub_signed(Duration::days(SNAPSHOT_RETENTION_DAYS))
    }
}

fn default_reports_dir(db_path: &Path) -> PathBuf {
    db_path
        .parent()
        .map(|dir| dir.join("reports"))
        .unwrap_or_else(|| std::env::temp_dir().join("cognical"))
}

fn duration_until(target: DateTime<Utc>, now: DateTime<Utc>) -> StdDuration {
    match (target - now).to_std() {
        Ok(duration) if duration >= StdDuration::from_secs(SNAPSHOT_MIN_SLEEP_SECS) => duration,
        Ok(_) => StdDuration::from_secs(SNAPSHOT_MIN_SLEEP_SECS),
        Err(_) => StdDuration::from_secs(SNAPSHOT_FALLBACK_SLEEP_SECS),
    }
}

fn default_grouping(range: AnalyticsRangeKey) -> AnalyticsGrouping {
    match range {
        AnalyticsRangeKey::SevenDays => AnalyticsGrouping::Day,
        AnalyticsRangeKey::ThirtyDays => AnalyticsGrouping::Day,
        AnalyticsRangeKey::NinetyDays => AnalyticsGrouping::Week,
    }
}

fn parse_query_datetime(value: &str) -> AppResult<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|_| AppError::validation("时间范围格式非法"))
}

fn parse_record_datetime(value: &Option<String>) -> Option<DateTime<Utc>> {
    value
        .as_deref()
        .and_then(|raw| DateTime::parse_from_rfc3339(raw).ok())
        .map(|dt| dt.with_timezone(&Utc))
}

fn time_spent_breakdown(tasks: &[TaskRecord]) -> (f64, f64, f64, f64) {
    let mut work_minutes = 0i64;
    let mut study_minutes = 0i64;
    let mut life_minutes = 0i64;
    let mut other_minutes = 0i64;

    for task in tasks {
        let minutes = task_estimated_minutes(task);
        match task.task_type.as_deref() {
            Some(t) if t.eq_ignore_ascii_case("work") => work_minutes += minutes,
            Some(t) if t.eq_ignore_ascii_case("study") => study_minutes += minutes,
            Some(t) if t.eq_ignore_ascii_case("life") => life_minutes += minutes,
            _ => other_minutes += minutes,
        }
    }

    (
        minutes_to_hours(work_minutes),
        minutes_to_hours(study_minutes),
        minutes_to_hours(life_minutes),
        minutes_to_hours(other_minutes),
    )
}

fn minutes_to_hours(minutes: i64) -> f64 {
    if minutes <= 0 {
        0.0
    } else {
        ((minutes as f64 / 60.0) * 100.0).round() / 100.0
    }
}

fn task_relevant_for_range(
    task: &TaskRecord,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    blocks: &[PlanningTimeBlockRecord],
) -> bool {
    let due_in_range = parse_record_datetime(&task.due_at)
        .map(|due| due >= start && due <= end)
        .unwrap_or(false);
    let completed_in_range = parse_record_datetime(&task.completed_at)
        .map(|completed| completed >= start && completed <= end)
        .unwrap_or(false);
    let scheduled_in_range = blocks
        .iter()
        .any(|block| block.task_id == task.id && block_overlaps_range(block, start, end));

    due_in_range || completed_in_range || scheduled_in_range
}

fn block_overlaps_range(
    block: &PlanningTimeBlockRecord,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> bool {
    match (parse_block_start(block), parse_block_end(block)) {
        (Some(block_start), Some(block_end)) => {
            if block_end <= block_start {
                return false;
            }

            let overlap_start = if block_start > start {
                block_start
            } else {
                start
            };
            let overlap_end = if block_end < end { block_end } else { end };
            overlap_end > overlap_start
        }
        _ => false,
    }
}

fn clamp_ratio(value: f64) -> f64 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

fn task_estimated_minutes(task: &TaskRecord) -> i64 {
    if let Some(minutes) = task.estimated_minutes {
        return minutes.max(MIN_ESTIMATED_MINUTES);
    }

    if let Some(hours) = task.estimated_hours {
        if hours.is_finite() && hours > 0.0 {
            let minutes = (hours * 60.0).round() as i64;
            return minutes.max(MIN_ESTIMATED_MINUTES);
        }
    }

    MIN_ESTIMATED_MINUTES
}

fn build_time_allocation(tasks: &[TaskRecord]) -> (TimeAllocationBreakdown, i64) {
    let mut by_type: HashMap<String, i64> = HashMap::new();
    let mut by_priority: HashMap<String, i64> = HashMap::new();
    let mut by_status: HashMap<String, i64> = HashMap::new();

    for task in tasks {
        let minutes = task_estimated_minutes(task);
        let type_key = task.task_type.as_deref().unwrap_or("other").to_lowercase();
        *by_type.entry(type_key).or_insert(0) += minutes;
        *by_priority.entry(task.priority.to_lowercase()).or_insert(0) += minutes;
        *by_status.entry(task.status.to_lowercase()).or_insert(0) += minutes;
    }

    let mut total: i64 = by_type.values().copied().sum();
    if total <= 0 {
        total = 1;
    }

    let mut by_type_entries: Vec<TimeAllocationTypeEntry> = by_type
        .into_iter()
        .map(|(kind, minutes)| TimeAllocationTypeEntry {
            kind,
            minutes,
            percentage: percentage(minutes, total),
        })
        .collect();
    by_type_entries.sort_by(|a, b| b.minutes.cmp(&a.minutes));

    let mut by_priority_entries: Vec<TimeAllocationPriorityEntry> = by_priority
        .into_iter()
        .map(|(priority, minutes)| TimeAllocationPriorityEntry {
            priority,
            minutes,
            percentage: percentage(minutes, total),
        })
        .collect();
    by_priority_entries.sort_by(|a, b| b.minutes.cmp(&a.minutes));

    let mut by_status_entries: Vec<TimeAllocationEntry> = by_status
        .into_iter()
        .map(|(label, minutes)| TimeAllocationEntry {
            label,
            minutes,
            percentage: percentage(minutes, total),
        })
        .collect();
    by_status_entries.sort_by(|a, b| b.minutes.cmp(&a.minutes));

    (
        TimeAllocationBreakdown {
            by_type: by_type_entries,
            by_priority: by_priority_entries,
            by_status: by_status_entries,
        },
        total,
    )
}

fn percentage(value: i64, total: i64) -> f64 {
    if total <= 0 {
        0.0
    } else {
        ((value as f64 / total as f64) * 1000.0).round() / 10.0
    }
}

fn build_efficiency_metrics(
    tasks: &[TaskRecord],
    blocks: &[PlanningTimeBlockRecord],
    total_focus_minutes: i64,
    estimated_total: i64,
) -> (AnalyticsEfficiency, Vec<EfficiencySuggestion>) {
    let mut on_time_count = 0i64;
    let mut due_completion_count = 0i64;
    let mut completion_deltas: Vec<f64> = Vec::new();
    let mut complexity_samples: Vec<f64> = Vec::new();

    for task in tasks {
        let due = parse_record_datetime(&task.due_at);
        let completed = parse_record_datetime(&task.completed_at);

        if let (Some(due_at), Some(done_at)) = (due, completed) {
            due_completion_count += 1;
            if done_at <= due_at {
                on_time_count += 1;
            }
            let delta_hours = (done_at - due_at).num_minutes().abs() as f64 / 60.0;
            completion_deltas.push(delta_hours);
        }

        if let Some(complexity) = task
            .ai
            .as_ref()
            .and_then(|ai| ai.complexity_score)
            .filter(|score| score.is_finite() && *score >= 0.0)
        {
            complexity_samples.push(complexity);
        }
    }

    let on_time_rate = if due_completion_count > 0 {
        clamp_ratio(on_time_count as f64 / due_completion_count as f64)
    } else {
        0.0
    };

    let estimated_total = estimated_total.max(1);
    let actual_minutes: i64 = blocks
        .iter()
        .filter_map(|block| {
            let start = parse_block_start(block);
            let end = parse_block_end(block);
            match (start, end) {
                (Some(start), Some(end)) if end > start => Some((end - start).num_minutes().max(0)),
                _ => None,
            }
        })
        .sum();

    let accuracy = if estimated_total > 0 {
        let diff = (actual_minutes - estimated_total).abs() as f64 / estimated_total as f64;
        clamp_ratio(1.0 - diff.min(1.0))
    } else {
        0.0
    };

    let complexity_correlation = if !complexity_samples.is_empty() && !completion_deltas.is_empty()
    {
        let avg_complexity = mean(&complexity_samples);
        let avg_delta = mean(&completion_deltas);
        let correlation = (1.0 - (avg_delta / 48.0)).clamp(0.0, 1.0);
        ((correlation + (avg_complexity / 10.0)).min(1.0) + 0.2).clamp(0.0, 1.0)
    } else {
        0.52
    };

    let efficiency = AnalyticsEfficiency {
        estimate_accuracy: accuracy,
        on_time_rate,
        complexity_correlation,
        suggestions: Vec::new(),
    };

    let mut suggestions = Vec::new();
    let focus_confidence = ((0.6 + on_time_rate * 0.4).clamp(0.0, 1.0) * 100.0).round() / 100.0;
    suggestions.push(EfficiencySuggestion {
        id: "optimize-focus".to_string(),
        title: "优化专注时间安排".to_string(),
        summary: format!(
            "过去周期共投入 {} 分钟专注时间，可将高优先级任务安排在完成率最高的时段。",
            total_focus_minutes
        ),
        related_task_id: None,
        related_plan_id: None,
        impact: if on_time_rate < 0.7 { "high" } else { "medium" }.to_string(),
        confidence: focus_confidence,
        category: "focus".to_string(),
    });

    let planning_confidence = ((0.55 + accuracy * 0.35).clamp(0.0, 1.0) * 100.0).round() / 100.0;
    suggestions.push(EfficiencySuggestion {
        id: "review-estimates".to_string(),
        title: "复盘任务预估".to_string(),
        summary: "部分任务实际耗时与预估存在偏差，建议在规划时记录更多上下文以提升准确率。"
            .to_string(),
        related_task_id: None,
        related_plan_id: None,
        impact: if accuracy < 0.75 { "high" } else { "medium" }.to_string(),
        confidence: planning_confidence,
        category: "planning".to_string(),
    });

    (efficiency, suggestions)
}

fn parse_block_start(block: &PlanningTimeBlockRecord) -> Option<DateTime<Utc>> {
    block
        .actual_start_at
        .as_ref()
        .or_else(|| Some(&block.start_at))
        .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
        .map(|dt| dt.with_timezone(&Utc))
}

fn parse_block_end(block: &PlanningTimeBlockRecord) -> Option<DateTime<Utc>> {
    block
        .actual_end_at
        .as_ref()
        .or_else(|| Some(&block.end_at))
        .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
        .map(|dt| dt.with_timezone(&Utc))
}

fn build_insights(
    total_completed: i64,
    completion_rate: f64,
    total_focus_minutes: i64,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Vec<InsightCard> {
    let generated_at = Utc::now().to_rfc3339();
    let period_label = format!("{} ~ {}", start.date_naive(), end.date_naive());

    let completion = InsightCard {
        id: "insight-completion-rate".to_string(),
        headline: "完成率趋势".to_string(),
        detail: format!(
            "周期内完成 {} 个任务，完成率 {:.1}%",
            total_completed,
            completion_rate * 100.0
        ),
        action_label: Some("查看任务".to_string()),
        action_href: Some("/tasks".to_string()),
        severity: if completion_rate >= 0.75 {
            "success".to_string()
        } else if completion_rate >= 0.5 {
            "warning".to_string()
        } else {
            "critical".to_string()
        },
        related_ids: None,
        generated_at: generated_at.clone(),
        source: "rule".to_string(),
    };

    let focus = InsightCard {
        id: "insight-focus-balance".to_string(),
        headline: "专注时间分布".to_string(),
        detail: format!(
            "{} 内共投入 {} 分钟专注时间，可在高能时段安排关键任务。",
            period_label, total_focus_minutes
        ),
        action_label: Some("查看日历".to_string()),
        action_href: Some("/calendar".to_string()),
        severity: "info".to_string(),
        related_ids: None,
        generated_at,
        source: "ai".to_string(),
    };

    vec![completion, focus]
}

fn predict_workload(tasks: &[TaskRecord]) -> i64 {
    let active_count = tasks
        .iter()
        .filter(|task| !matches!(task.status.as_str(), "done" | "archived"))
        .count();
    (active_count as f64 * 1.1).ceil() as i64
}

fn build_daily_stats(
    tasks: &[TaskRecord],
    blocks: &[PlanningTimeBlockRecord],
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Vec<(NaiveDate, DailyStats)> {
    let mut stats: HashMap<NaiveDate, DailyStats> = HashMap::new();
    let mut date = start.date_naive();
    while date <= end.date_naive() {
        stats.entry(date).or_default();
        date = date.succ_opt().unwrap();
    }

    for task in tasks {
        if let Some(completed) = parse_record_datetime(&task.completed_at) {
            if completed >= start && completed <= end {
                let entry = stats.entry(completed.date_naive()).or_default();
                entry.completed += 1;
            }
        }

        if let Some(due) = parse_record_datetime(&task.due_at) {
            if due >= start && due <= end {
                let entry = stats.entry(due.date_naive()).or_default();
                entry.due += 1;
            }
        }
    }

    let mut focus_by_day: HashMap<NaiveDate, i64> = HashMap::new();
    for block in blocks {
        if let (Some(start_at), Some(end_at)) = (parse_block_start(block), parse_block_end(block)) {
            if end_at <= start_at {
                continue;
            }
            let clamped_start = if start_at < start { start } else { start_at };
            let clamped_end = if end_at > end { end } else { end_at };
            if clamped_end <= clamped_start {
                continue;
            }
            let minutes = (clamped_end - clamped_start).num_minutes().max(0);
            let day = clamped_start.date_naive();
            *focus_by_day.entry(day).or_insert(0) += minutes;
        }
    }

    for (day, minutes) in focus_by_day {
        let entry = stats.entry(day).or_default();
        entry.focus_minutes += minutes;
    }

    let mut ordered: Vec<(NaiveDate, DailyStats)> = stats.into_iter().collect();
    ordered.sort_by_key(|(date, _)| *date);

    for (day, entry) in &mut ordered {
        let day_end = Utc.from_utc_datetime(&day.and_hms_opt(23, 59, 59).unwrap());
        let overdue = tasks
            .iter()
            .filter(|task| {
                let due = parse_record_datetime(&task.due_at);
                if let Some(due_at) = due {
                    if due_at > day_end {
                        return false;
                    }
                    if let Some(completed) = parse_record_datetime(&task.completed_at) {
                        completed > day_end
                    } else {
                        true
                    }
                } else {
                    false
                }
            })
            .count() as i64;
        entry.overdue = overdue;
    }

    ordered
}

fn build_history_points(
    daily: &[(NaiveDate, DailyStats)],
    grouping: AnalyticsGrouping,
) -> Vec<AnalyticsHistoryPoint> {
    match grouping {
        AnalyticsGrouping::Day => daily
            .iter()
            .map(|(date, stats)| AnalyticsHistoryPoint {
                date: date_to_iso(*date),
                productivity_score: productivity(stats.completed, stats.focus_minutes),
                completion_rate: completion_ratio(stats.completed, stats.due),
                focus_minutes: stats.focus_minutes,
                completed_tasks: stats.completed,
                overdue_tasks: stats.overdue,
            })
            .collect(),
        AnalyticsGrouping::Week => {
            let mut grouped: Vec<AnalyticsHistoryPoint> = Vec::new();
            let mut buffer: Vec<(NaiveDate, DailyStats)> = Vec::new();

            for (date, stats) in daily {
                buffer.push((*date, stats.clone()));
                if buffer.len() == 7 {
                    grouped.push(build_grouped_point(&buffer));
                    buffer.clear();
                }
            }

            if !buffer.is_empty() {
                grouped.push(build_grouped_point(&buffer));
            }

            grouped
        }
    }
}

fn build_grouped_point(buffer: &[(NaiveDate, DailyStats)]) -> AnalyticsHistoryPoint {
    let completed: i64 = buffer.iter().map(|(_, stats)| stats.completed).sum();
    let due: i64 = buffer.iter().map(|(_, stats)| stats.due).sum();
    let focus_minutes: i64 = buffer.iter().map(|(_, stats)| stats.focus_minutes).sum();
    let overdue = buffer.last().map(|(_, stats)| stats.overdue).unwrap_or(0);
    let date = buffer
        .first()
        .map(|(date, _)| *date)
        .unwrap_or_else(|| Utc::now().date_naive());

    AnalyticsHistoryPoint {
        date: date_to_iso(date),
        productivity_score: productivity(completed, focus_minutes),
        completion_rate: completion_ratio(completed, due),
        focus_minutes,
        completed_tasks: completed,
        overdue_tasks: overdue,
    }
}

fn completion_ratio(completed: i64, due: i64) -> f64 {
    if due > 0 {
        clamp_ratio(completed as f64 / due as f64)
    } else if completed > 0 {
        1.0
    } else {
        0.0
    }
}

fn productivity(completed: i64, focus_minutes: i64) -> f64 {
    let base = completed as f64 * 12.0 + focus_minutes as f64 / 6.0;
    base.clamp(30.0, 100.0)
}

fn date_to_iso(date: NaiveDate) -> String {
    Utc.from_utc_datetime(&date.and_hms_opt(0, 0, 0).unwrap())
        .to_rfc3339()
}

fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f64>() / values.len() as f64
    }
}

fn round_ratio(value: f64) -> f64 {
    let clamped = clamp_ratio(value);
    (clamped * 1000.0).round() / 1000.0
}

fn compute_focus_consistency(samples: &[f64]) -> f64 {
    let meaningful: Vec<f64> = samples
        .iter()
        .copied()
        .filter(|value| value.is_finite() && *value >= 0.0)
        .collect();

    if meaningful.is_empty() {
        return 0.0;
    }

    if meaningful.len() == 1 {
        return if meaningful[0] > 0.0 { 0.85 } else { 0.0 };
    }

    let average = mean(&meaningful);
    if average <= 0.0 {
        return 0.0;
    }

    let variance = meaningful
        .iter()
        .map(|value| (value - average).powi(2))
        .sum::<f64>()
        / meaningful.len() as f64;
    let std_dev = variance.sqrt();
    let normalized = 1.0 - (std_dev / (average + 1.0)).min(1.0);
    normalized.clamp(0.0, 1.0)
}

fn compute_rest_balance(focus_minutes: i64) -> f64 {
    if focus_minutes <= 0 {
        return 1.0;
    }

    let ideal_focus = 300.0;
    let delta = (focus_minutes as f64 - ideal_focus).abs();
    1.0 - (delta / ideal_focus).min(1.0)
}

fn compute_capacity_risk(tasks: &[TaskRecord], overdue_tasks: i64, estimated_minutes: i64) -> f64 {
    let active_count = tasks
        .iter()
        .filter(|task| !matches!(task.status.as_str(), "done" | "archived"))
        .count() as f64;

    let backlog_pressure = clamp_ratio(active_count / 18.0);
    let utilization_pressure = if estimated_minutes > 0 {
        clamp_ratio(estimated_minutes as f64 / 360.0)
    } else {
        0.0
    };
    let overdue_pressure = clamp_ratio(overdue_tasks as f64 / 8.0);

    (backlog_pressure * 0.4 + utilization_pressure * 0.4 + overdue_pressure * 0.2).clamp(0.0, 1.0)
}

fn render_markdown_report(overview: &AnalyticsOverviewResponse) -> String {
    let summary = &overview.overview.summary;
    let mut content = String::new();
    content.push_str("# CogniCal 分析报告\n\n");
    content.push_str(&format!(
        "生成时间：{}\n\n",
        overview.overview.meta.generated_at
    ));
    content.push_str(&format!(
        "分析范围：{}\n\n",
        overview.overview.range.as_str()
    ));
    content.push_str("## 概览指标\n");
    content.push_str(&format!(
        "- 完成任务：{}\n- 完成率：{:.1}%\n- 趋势变化：{:.1} 个百分点\n- 预计工作量：{}\n- 专注总时长：{} 分钟\n- 未完成任务：{}\n\n",
        summary.total_completed,
        summary.completion_rate * 100.0,
        summary.trend_delta,
        summary.workload_prediction,
        summary.focus_minutes,
        summary.overdue_tasks
    ));

    content.push_str("## 时间分配\n");
    for entry in &overview.overview.time_allocation.by_type {
        content.push_str(&format!(
            "- 类型 {}：{} 分钟 ({:.1}%)\n",
            entry.kind, entry.minutes, entry.percentage
        ));
    }
    content.push('\n');

    content.push_str("## 效率指标\n");
    content.push_str(&format!(
        "- 预估准确率：{:.1}%\n- 按时完成率：{:.1}%\n- 复杂度相关性：{:.1}%\n\n",
        overview.overview.efficiency.estimate_accuracy * 100.0,
        overview.overview.efficiency.on_time_rate * 100.0,
        overview.overview.efficiency.complexity_correlation * 100.0
    ));

    content.push_str("## 建议与洞察\n");
    for suggestion in &overview.overview.efficiency.suggestions {
        content.push_str(&format!(
            "- [{} - {}] {} (信心 {:.0}%)\n",
            suggestion.category,
            suggestion.impact,
            suggestion.summary,
            suggestion.confidence * 100.0
        ));
    }
    content.push('\n');

    for insight in &overview.overview.insights {
        content.push_str(&format!("- {}：{}\n", insight.headline, insight.detail));
    }

    content
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::task::TaskRecord;
    use chrono::NaiveDate;

    fn base_task(id: &str) -> TaskRecord {
        TaskRecord {
            id: id.to_string(),
            title: format!("Task {id}"),
            description: None,
            status: "Pending".to_string(),
            priority: "Medium".to_string(),
            planned_start_at: None,
            start_at: None,
            due_at: None,
            completed_at: None,
            estimated_minutes: None,
            estimated_hours: None,
            tags: Vec::new(),
            owner_id: None,
            task_type: None,
            is_recurring: false,
            recurrence: None,
            ai: None,
            external_links: Vec::new(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn task_estimated_minutes_respects_minimums_and_preferences() {
        let mut task = base_task("1");
        task.estimated_minutes = Some(10);
        assert_eq!(task_estimated_minutes(&task), MIN_ESTIMATED_MINUTES);

        task.estimated_minutes = Some(45);
        assert_eq!(task_estimated_minutes(&task), 45);

        task.estimated_minutes = None;
        task.estimated_hours = Some(0.2);
        assert_eq!(task_estimated_minutes(&task), MIN_ESTIMATED_MINUTES);

        task.estimated_hours = Some(1.5);
        assert_eq!(task_estimated_minutes(&task), 90);
    }

    #[test]
    fn build_time_allocation_aggregates_minutes_by_category() {
        let mut focus_primary = base_task("focus-1");
        focus_primary.task_type = Some("Focus".to_string());
        focus_primary.priority = "High".to_string();
        focus_primary.status = "Completed".to_string();
        focus_primary.estimated_minutes = Some(60);

        let mut focus_support = base_task("focus-2");
        focus_support.task_type = Some("Focus".to_string());
        focus_support.priority = "Low".to_string();
        focus_support.status = "InProgress".to_string();
        focus_support.estimated_hours = Some(0.5);

        let mut admin = base_task("admin");
        admin.task_type = Some("Admin".to_string());
        admin.priority = "Medium".to_string();
        admin.status = "Completed".to_string();
        admin.estimated_minutes = None;

        let mut uncategorized = base_task("other");
        uncategorized.task_type = None;
        uncategorized.priority = "High".to_string();
        uncategorized.status = "Pending".to_string();
        uncategorized.estimated_minutes = None;

        let tasks = vec![focus_primary, focus_support, admin, uncategorized];

        let (allocation, total) = build_time_allocation(&tasks);

        assert_eq!(
            total,
            60 + 30 + MIN_ESTIMATED_MINUTES + MIN_ESTIMATED_MINUTES
        );

        let focus = allocation
            .by_type
            .iter()
            .find(|entry| entry.kind == "focus")
            .expect("focus allocation");
        assert_eq!(focus.minutes, 90);
        assert_eq!(focus.percentage, 75.0);

        let admin_entry = allocation
            .by_type
            .iter()
            .find(|entry| entry.kind == "admin")
            .expect("admin allocation");
        assert_eq!(admin_entry.minutes, MIN_ESTIMATED_MINUTES);

        let other_entry = allocation
            .by_type
            .iter()
            .find(|entry| entry.kind == "other")
            .expect("other allocation");
        assert_eq!(other_entry.minutes, MIN_ESTIMATED_MINUTES);

        let completed_status = allocation
            .by_status
            .iter()
            .find(|entry| entry.label == "completed")
            .expect("completed status");
        assert_eq!(completed_status.minutes, 60 + MIN_ESTIMATED_MINUTES);

        let high_priority = allocation
            .by_priority
            .iter()
            .find(|entry| entry.priority == "high")
            .expect("high priority");
        assert_eq!(high_priority.minutes, 60 + MIN_ESTIMATED_MINUTES);
    }

    #[test]
    fn build_history_points_groups_weeks_and_preserves_overdue() {
        let base_date = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
        let mut daily: Vec<(NaiveDate, DailyStats)> = Vec::new();

        for offset in 0..9 {
            let mut stats = DailyStats {
                completed: 2,
                due: 4,
                focus_minutes: 60,
                overdue: 0,
            };

            if offset == 6 {
                stats.overdue = 5;
            }

            if offset == 8 {
                stats.overdue = 2;
            }

            let date = base_date
                .checked_add_signed(chrono::Duration::days(offset))
                .unwrap();
            daily.push((date, stats));
        }

        let points = build_history_points(&daily, AnalyticsGrouping::Week);

        assert_eq!(points.len(), 2);

        let first = &points[0];
        assert!(first.date.starts_with("2024-03-01"));
        assert_eq!(first.completed_tasks, 14);
        assert_eq!(first.focus_minutes, 420);
        assert_eq!(first.overdue_tasks, 5);
        assert_eq!(first.completion_rate, 0.5);
        assert_eq!(first.productivity_score, 100.0);

        let second = &points[1];
        assert_eq!(second.completed_tasks, 4);
        assert_eq!(second.focus_minutes, 120);
        assert_eq!(second.overdue_tasks, 2);
        assert_eq!(second.completion_rate, 0.5);
        assert_eq!(second.productivity_score, 68.0);
    }

    #[test]
    fn completion_ratio_handles_zero_due_tasks() {
        assert_eq!(completion_ratio(0, 0), 0.0);
        assert_eq!(completion_ratio(3, 0), 1.0);
        assert_eq!(completion_ratio(1, 2), 0.5);
    }
}
