use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use chrono::{DateTime, Duration, Timelike, Utc};
use tracing::{debug, info};

use crate::db::repositories::task_repository::TaskRepository;
use crate::db::repositories::wellness_repository::WellnessRepository;
use crate::db::DbPool;
use crate::error::AppResult;
use crate::models::wellness::{
    WellnessEventInsert, WellnessEventRecord, WellnessEventResponseUpdate, WellnessResponse,
    WellnessTriggerReason,
};
use crate::services::settings_service::SettingsService;

const DEFAULT_FOCUS_THRESHOLD_MINUTES: i64 = 90; // 90 minutes of continuous focus
const DEFAULT_WORK_STREAK_THRESHOLD_HOURS: f64 = 4.0; // 4 hours continuous work
const DEFAULT_REST_BREAK_MINUTES: i64 = 10; // Recommend 10-minute break
const MAX_DEFERRAL_COUNT: i64 = 3; // Max times user can snooze
const SNOOZE_INCREMENT_MINUTES: i64 = 15; // Snooze for 15 minutes

/// Service for wellness nudges and rest reminders
pub struct WellnessService {
    db: DbPool,
    settings_service: Arc<SettingsService>,
    nudge_job_running: Arc<AtomicBool>,
}

impl WellnessService {
    pub fn new(db: DbPool, settings_service: Arc<SettingsService>) -> Self {
        Self {
            db,
            settings_service,
            nudge_job_running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Check if current time is within quiet hours
    fn is_quiet_hours(&self, now: &DateTime<Utc>) -> AppResult<bool> {
        let settings = self.settings_service.get()?;

        let current_minute = now.hour() as i16 * 60 + now.minute() as i16;

        // If work hours are defined, consider outside work hours as quiet
        if settings.workday_start_minute > 0 && settings.workday_end_minute > 0 {
            if current_minute < settings.workday_start_minute
                || current_minute > settings.workday_end_minute
            {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Calculate exponential back-off delay based on deferral count
    fn calculate_backoff_minutes(deferral_count: i64) -> i64 {
        // Exponential back-off: 15, 30, 60, 120 minutes
        SNOOZE_INCREMENT_MINUTES * 2_i64.pow(deferral_count.min(3) as u32)
    }

    /// Generate wellness nudge based on current activity patterns
    pub fn check_and_generate_nudge(&self) -> AppResult<Option<WellnessEventRecord>> {
        let now = Utc::now();

        // Check if in quiet hours
        if self.is_quiet_hours(&now)? {
            debug!("Skipping wellness nudge: currently in quiet hours");
            return Ok(None);
        }

        // Check for existing pending nudges
        let conn = self.db.get_connection()?;
        let pending = WellnessRepository::list_pending(&conn, 1)?;

        if let Some(existing) = pending.first() {
            // Check if enough time has passed based on deferral count
            if let Ok(triggered) = DateTime::parse_from_rfc3339(&existing.window_start) {
                let backoff = Self::calculate_backoff_minutes(existing.deferral_count);
                let next_trigger = triggered + Duration::minutes(backoff);

                if now < next_trigger.with_timezone(&Utc) {
                    debug!(
                        "Skipping nudge: back-off period active ({} minutes remaining)",
                        (next_trigger.timestamp() - now.timestamp()) / 60
                    );
                    return Ok(None);
                }
            }

            // If max deferrals reached, consider it expired
            if existing.deferral_count >= MAX_DEFERRAL_COUNT {
                debug!(
                    "Existing nudge expired after {} deferrals",
                    existing.deferral_count
                );
                // Mark as ignored
                WellnessRepository::update_response(
                    &conn,
                    existing.id,
                    &WellnessEventResponseUpdate {
                        response: WellnessResponse::Ignored,
                        response_at: now.to_rfc3339(),
                        deferral_count: existing.deferral_count,
                    },
                )?;
            } else {
                return Ok(Some(existing.clone()));
            }
        }

        // Compute current work patterns
        let work_pattern = self.analyze_work_pattern()?;

        // Determine if nudge should be triggered
        let should_trigger = work_pattern.continuous_focus_minutes
            >= DEFAULT_FOCUS_THRESHOLD_MINUTES
            || work_pattern.work_streak_hours >= DEFAULT_WORK_STREAK_THRESHOLD_HOURS;

        if !should_trigger {
            return Ok(None);
        }

        // Determine trigger reason
        let (trigger_reason, _message) =
            if work_pattern.work_streak_hours >= DEFAULT_WORK_STREAK_THRESHOLD_HOURS {
                (
                    WellnessTriggerReason::WorkStreak,
                    format!(
                        "您已经连续工作 {:.1} 小时了，建议休息一下",
                        work_pattern.work_streak_hours
                    ),
                )
            } else {
                (
                    WellnessTriggerReason::FocusStreak,
                    format!(
                        "您已经专注 {} 分钟了，休息一下会更高效",
                        work_pattern.continuous_focus_minutes
                    ),
                )
            };

        // Create new nudge
        let insert = WellnessEventInsert {
            window_start: now.to_rfc3339(),
            trigger_reason,
            recommended_break_minutes: DEFAULT_REST_BREAK_MINUTES,
            suggested_micro_task: Some("喝杯水、伸展身体、眺望远方".to_string()),
        };

        let id = WellnessRepository::insert(&conn, &insert)?;
        let record = WellnessRepository::find_by_id(&conn, id)?;

        info!(
            "Generated wellness nudge: {:?} (id: {})",
            trigger_reason, id
        );

        Ok(Some(record))
    }

    /// Analyze current work patterns
    fn analyze_work_pattern(&self) -> AppResult<WorkPattern> {
        let conn = self.db.get_connection()?;
        let now = Utc::now();

        // Get recent tasks (last 4 hours)
        let _start_time = now - Duration::hours(4);
        let tasks = TaskRepository::list_all(&conn)?;

        // Filter tasks that were worked on today
        let today_start = now.date_naive().and_hms_opt(0, 0, 0).unwrap();
        let today_tasks: Vec<_> = tasks
            .into_iter()
            .filter(|task| {
                if let Some(start_at) = &task.start_at {
                    if let Ok(started) = DateTime::parse_from_rfc3339(start_at) {
                        let started_naive = started.naive_utc();
                        return started_naive >= today_start;
                    }
                }
                false
            })
            .collect();

        // Calculate work streak (simplified)
        let work_streak_hours = if !today_tasks.is_empty() {
            // Estimate based on tasks started today
            let hours_since_start = if let Some(first_task) = today_tasks.first() {
                if let Some(start_at) = &first_task.start_at {
                    if let Ok(started) = DateTime::parse_from_rfc3339(start_at) {
                        let diff = now.signed_duration_since(started.with_timezone(&Utc));
                        diff.num_minutes() as f64 / 60.0
                    } else {
                        0.0
                    }
                } else {
                    0.0
                }
            } else {
                0.0
            };
            hours_since_start.min(8.0) // Cap at 8 hours
        } else {
            0.0
        };

        // Calculate continuous focus (simplified - based on recent task activity)
        let continuous_focus_minutes = if !today_tasks.is_empty() {
            // Estimate based on number of in-progress tasks
            let active_tasks = today_tasks
                .iter()
                .filter(|t| t.status == "in-progress")
                .count();

            if active_tasks > 0 {
                (work_streak_hours * 60.0) as i64
            } else {
                0
            }
        } else {
            0
        };

        Ok(WorkPattern {
            continuous_focus_minutes,
            work_streak_hours,
        })
    }

    /// Record user response to wellness nudge
    pub fn respond_to_nudge(
        &self,
        id: i64,
        response: WellnessResponse,
    ) -> AppResult<WellnessEventRecord> {
        let conn = self.db.get_connection()?;
        let now = Utc::now();

        // Get current event
        let event = WellnessRepository::find_by_id(&conn, id)?;

        // Calculate deferral count
        let deferral_count = match response {
            WellnessResponse::Snoozed => event.deferral_count + 1,
            _ => event.deferral_count,
        };

        let update = WellnessEventResponseUpdate {
            response,
            response_at: now.to_rfc3339(),
            deferral_count,
        };

        WellnessRepository::update_response(&conn, id, &update)?;

        let updated = WellnessRepository::find_by_id(&conn, id)?;

        info!("User responded to wellness nudge {}: {:?}", id, response);

        Ok(updated)
    }

    /// Get current pending nudge
    pub fn get_pending_nudge(&self) -> AppResult<Option<WellnessEventRecord>> {
        let conn = self.db.get_connection()?;
        let pending = WellnessRepository::list_pending(&conn, 1)?;
        Ok(pending.into_iter().next())
    }

    /// Get weekly wellness summary
    pub fn get_weekly_summary(&self) -> AppResult<WeeklySummary> {
        let conn = self.db.get_connection()?;
        let now = Utc::now();

        // Get recent events (last 100, then filter to 7 days)
        let week_start = now - Duration::days(7);
        let all_events = WellnessRepository::list_recent(&conn, 100)?;
        let events: Vec<_> = all_events
            .into_iter()
            .filter(|e| {
                if let Ok(ts) = DateTime::parse_from_rfc3339(&e.window_start) {
                    ts.with_timezone(&Utc) >= week_start
                } else {
                    false
                }
            })
            .collect();

        let total_nudges = events.len() as i32;
        let mut completed_count = 0;
        let mut snoozed_count = 0;
        let mut ignored_count = 0;
        let mut total_focus_minutes = 0.0;
        let mut max_work_streak_hours: f64 = 0.0;

        for event in &events {
            match event.response {
                Some(WellnessResponse::Completed) => completed_count += 1,
                Some(WellnessResponse::Snoozed) => snoozed_count += 1,
                Some(WellnessResponse::Ignored) => ignored_count += 1,
                _ => {}
            }

            // Calculate focus metrics based on trigger reason
            match event.trigger_reason {
                WellnessTriggerReason::FocusStreak => {
                    // Estimate focus duration
                    total_focus_minutes += 90.0; // DEFAULT_FOCUS_THRESHOLD_MINUTES
                }
                WellnessTriggerReason::WorkStreak => {
                    max_work_streak_hours = max_work_streak_hours.max(4.0); // DEFAULT_WORK_STREAK_THRESHOLD_HOURS
                }
            }
        }

        let average_focus_minutes = if total_nudges > 0 {
            total_focus_minutes / total_nudges as f64
        } else {
            0.0
        };

        let rest_compliance_rate = if total_nudges > 0 {
            completed_count as f64 / total_nudges as f64
        } else {
            0.0
        };

        // Simple rhythm score based on compliance
        let focus_rhythm_score = (rest_compliance_rate * 100.0).min(100.0);

        // Generate recommendations
        let recommendations = self.generate_wellness_recommendations(
            rest_compliance_rate,
            snoozed_count,
            ignored_count,
        );

        Ok(WeeklySummary {
            week_start: week_start.to_rfc3339(),
            week_end: now.to_rfc3339(),
            total_nudges,
            completed_count,
            snoozed_count,
            ignored_count,
            average_focus_minutes,
            max_work_streak_hours,
            rest_compliance_rate,
            focus_rhythm_score,
            peak_hours: vec![], // TODO: Implement peak hours analysis
            recommendations,
        })
    }

    fn generate_wellness_recommendations(
        &self,
        compliance_rate: f64,
        snoozed_count: i32,
        ignored_count: i32,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if compliance_rate < 0.3 {
            recommendations.push("建议提高休息合规率，定期休息有助于保持长期高效".to_string());
        } else if compliance_rate > 0.8 {
            recommendations.push("很好地保持了工作与休息的平衡！".to_string());
        }

        if snoozed_count > 5 {
            recommendations.push("您经常推迟休息提醒，考虑调整工作节奏".to_string());
        }

        if ignored_count > 3 {
            recommendations.push("注意到您忽略了多个休息提醒，请关注身体健康".to_string());
        }

        if recommendations.is_empty() {
            recommendations.push("继续保持良好的工作节奏".to_string());
        }

        recommendations
    }

    /// Start periodic nudge checking (called from background job)
    pub fn ensure_nudge_job(&self) -> AppResult<()> {
        if self.nudge_job_running.load(Ordering::SeqCst) {
            return Ok(());
        }

        self.nudge_job_running.store(true, Ordering::SeqCst);
        info!("Wellness nudge job initialized");
        Ok(())
    }
}

struct WorkPattern {
    continuous_focus_minutes: i64,
    work_streak_hours: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeeklySummary {
    pub week_start: String,
    pub week_end: String,
    pub total_nudges: i32,
    pub completed_count: i32,
    pub snoozed_count: i32,
    pub ignored_count: i32,
    pub average_focus_minutes: f64,
    pub max_work_streak_hours: f64,
    pub rest_compliance_rate: f64,
    pub focus_rhythm_score: f64,
    pub peak_hours: Vec<i32>,
    pub recommendations: Vec<String>,
}
