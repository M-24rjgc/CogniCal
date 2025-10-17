use std::collections::HashMap;

use chrono::{DateTime, Datelike, FixedOffset};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use uuid::Uuid;

use crate::db::repositories::planning_repository::{PlanningRepository, SchedulePreferencesRow};
use crate::error::AppResult;
use crate::models::planning::SchedulePreferencesRecord;
use crate::services::schedule_utils;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct PreferenceSnapshot {
    pub focus_start_minute: Option<u32>,
    pub focus_end_minute: Option<u32>,
    pub buffer_minutes_between_blocks: i64,
    pub prefer_compact_schedule: bool,
    #[serde(default)]
    pub avoidance_windows: Vec<AvoidanceWindow>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AvoidanceWindow {
    pub weekday: u32,
    pub start_minute: u32,
    pub end_minute: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FeedbackEvent {
    pub task_id: String,
    pub planned_start_at: String,
    pub planned_end_at: String,
    pub actual_start_at: Option<String>,
    pub actual_end_at: Option<String>,
    pub completed: bool,
}

pub struct BehaviorLearningService<'a> {
    pub conn: &'a rusqlite::Connection,
}

impl<'a> BehaviorLearningService<'a> {
    pub fn new(conn: &'a rusqlite::Connection) -> Self {
        Self { conn }
    }

    pub fn load_preferences(&self, preference_id: &str) -> AppResult<PreferenceSnapshot> {
        let row = PlanningRepository::get_schedule_preferences(self.conn, preference_id)?;
        if let Some(row) = row {
            let record = row.into_record()?;
            Ok(self.parse_preferences(&record))
        } else {
            Ok(PreferenceSnapshot::default())
        }
    }

    pub fn save_preferences(
        &self,
        preference_id: &str,
        snapshot: &PreferenceSnapshot,
    ) -> AppResult<()> {
        let record = self.serialize_preferences(preference_id, snapshot);
        let row = SchedulePreferencesRow::from_record(&record)?;
        PlanningRepository::upsert_schedule_preferences(self.conn, &row)?;
        Ok(())
    }

    pub fn snapshot_for_planning(&self, preference_id: &str) -> AppResult<JsonValue> {
        let snapshot = self.load_preferences(preference_id)?;
        Ok(json!({
            "focusStartMinute": snapshot.focus_start_minute,
            "focusEndMinute": snapshot.focus_end_minute,
            "bufferMinutesBetweenBlocks": snapshot.buffer_minutes_between_blocks,
            "preferCompactSchedule": snapshot.prefer_compact_schedule,
            "avoidanceWindows": snapshot.avoidance_windows,
        }))
    }

    pub fn ingest_feedback(
        &self,
        preference_id: &str,
        feedback: &[FeedbackEvent],
    ) -> AppResult<()> {
        if feedback.is_empty() {
            return Ok(());
        }

        let mut snapshot = self.load_preferences(preference_id)?;

        let mut weekday_stats: HashMap<u32, Vec<FeedbackMetric>> = HashMap::new();

        for event in feedback {
            let planned_start = schedule_utils::parse_datetime(&event.planned_start_at)?;
            let planned_end = schedule_utils::parse_datetime(&event.planned_end_at)?;
            let weekday = planned_start.weekday().num_days_from_monday();

            weekday_stats
                .entry(weekday)
                .or_default()
                .push(FeedbackMetric {
                    completed: event.completed,
                    planned_start,
                    planned_end,
                });
        }

        self.update_focus_window(&mut snapshot, &weekday_stats);
        self.update_buffer_minutes(&mut snapshot, feedback)?;
        self.update_avoidance_windows(&mut snapshot, &weekday_stats)?;

        self.save_preferences(preference_id, &snapshot)
    }

    fn parse_preferences(&self, record: &SchedulePreferencesRecord) -> PreferenceSnapshot {
        let focus_start = record
            .data
            .get("focusStartMinute")
            .and_then(|value| value.as_u64())
            .map(|num| num as u32);
        let focus_end = record
            .data
            .get("focusEndMinute")
            .and_then(|value| value.as_u64())
            .map(|num| num as u32);
        let buffer = record
            .data
            .get("bufferMinutesBetweenBlocks")
            .and_then(|value| value.as_i64())
            .unwrap_or(15);
        let prefer_compact = record
            .data
            .get("preferCompactSchedule")
            .and_then(|value| value.as_bool())
            .unwrap_or(false);
        let avoidance_windows = record
            .data
            .get("avoidanceWindows")
            .and_then(|value| value.as_array())
            .map(|array| {
                array
                    .iter()
                    .filter_map(|item| {
                        Some(AvoidanceWindow {
                            weekday: item.get("weekday")?.as_u64()? as u32,
                            start_minute: item.get("startMinute")?.as_u64()? as u32,
                            end_minute: item.get("endMinute")?.as_u64()? as u32,
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        PreferenceSnapshot {
            focus_start_minute: focus_start,
            focus_end_minute: focus_end,
            buffer_minutes_between_blocks: buffer,
            prefer_compact_schedule: prefer_compact,
            avoidance_windows,
        }
    }

    fn serialize_preferences(
        &self,
        preference_id: &str,
        snapshot: &PreferenceSnapshot,
    ) -> SchedulePreferencesRecord {
        let avoidance = snapshot
            .avoidance_windows
            .iter()
            .map(|window| {
                json!({
                    "id": Uuid::new_v4().to_string(),
                    "weekday": window.weekday,
                    "startMinute": window.start_minute,
                    "endMinute": window.end_minute,
                })
            })
            .collect::<Vec<_>>();

        let data = json!({
            "focusStartMinute": snapshot.focus_start_minute,
            "focusEndMinute": snapshot.focus_end_minute,
            "bufferMinutesBetweenBlocks": snapshot.buffer_minutes_between_blocks,
            "preferCompactSchedule": snapshot.prefer_compact_schedule,
            "avoidanceWindows": avoidance,
        });

        SchedulePreferencesRecord {
            id: preference_id.to_string(),
            data,
            updated_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    fn update_focus_window(
        &self,
        snapshot: &mut PreferenceSnapshot,
        weekday_stats: &HashMap<u32, Vec<FeedbackMetric>>,
    ) {
        let mut best_window: Option<(u32, u32)> = None;
        let mut best_success_rate = 0.0;

        for (_weekday, metrics) in weekday_stats {
            if metrics.is_empty() {
                continue;
            }

            let total = metrics.len() as f32;
            let successes = metrics.iter().filter(|metric| metric.completed).count() as f32;
            let success_rate = successes / total;

            if success_rate > best_success_rate && success_rate >= 0.6 {
                best_success_rate = success_rate;

                let avg_start = metrics
                    .iter()
                    .map(|metric| schedule_utils::midnight_minutes_of(metric.planned_start) as u32)
                    .sum::<u32>()
                    / metrics.len() as u32;
                let avg_end = metrics
                    .iter()
                    .map(|metric| schedule_utils::midnight_minutes_of(metric.planned_end) as u32)
                    .sum::<u32>()
                    / metrics.len() as u32;
                best_window = Some((avg_start, avg_end));
            }
        }

        if let Some((start, end)) = best_window {
            snapshot.focus_start_minute = Some(start);
            snapshot.focus_end_minute = Some(end);
        }
    }

    fn update_buffer_minutes(
        &self,
        snapshot: &mut PreferenceSnapshot,
        feedback: &[FeedbackEvent],
    ) -> AppResult<()> {
        if feedback.is_empty() {
            return Ok(());
        }

        let mut delays = Vec::new();
        for event in feedback {
            if let (Some(actual_start), Some(actual_end)) =
                (event.actual_start_at.as_ref(), event.actual_end_at.as_ref())
            {
                let planned_start = schedule_utils::parse_datetime(&event.planned_start_at)?;
                let planned_end = schedule_utils::parse_datetime(&event.planned_end_at)?;
                let actual_start = schedule_utils::parse_datetime(actual_start)?;
                let actual_end = schedule_utils::parse_datetime(actual_end)?;

                let start_delay = (actual_start - planned_start).num_minutes();
                let end_delay = (actual_end - planned_end).num_minutes();
                if start_delay > 10 || end_delay > 10 {
                    delays.push(start_delay.max(end_delay));
                }
            }
        }

        if !delays.is_empty() {
            let median_delay = median(&mut delays);
            snapshot.buffer_minutes_between_blocks =
                (snapshot.buffer_minutes_between_blocks.max(10) + median_delay).min(90);
        }

        Ok(())
    }

    fn update_avoidance_windows(
        &self,
        snapshot: &mut PreferenceSnapshot,
        weekday_stats: &HashMap<u32, Vec<FeedbackMetric>>,
    ) -> AppResult<()> {
        let mut new_windows = Vec::new();

        for (weekday, metrics) in weekday_stats {
            let total = metrics.len();
            if total < 3 {
                continue;
            }

            let failed: Vec<&FeedbackMetric> =
                metrics.iter().filter(|metric| !metric.completed).collect();
            let failure_rate = failed.len() as f32 / total as f32;

            if failure_rate < 0.6 {
                continue;
            }

            let avg_start = failed
                .iter()
                .map(|metric| schedule_utils::midnight_minutes_of(metric.planned_start) as u32)
                .sum::<u32>()
                / failed.len() as u32;
            let avg_end = failed
                .iter()
                .map(|metric| schedule_utils::midnight_minutes_of(metric.planned_end) as u32)
                .sum::<u32>()
                / failed.len() as u32;

            new_windows.push(AvoidanceWindow {
                weekday: *weekday,
                start_minute: avg_start.saturating_sub(30),
                end_minute: avg_end + 30,
            });
        }

        if !new_windows.is_empty() {
            snapshot.avoidance_windows = merge_windows(&snapshot.avoidance_windows, &new_windows);
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
struct FeedbackMetric {
    completed: bool,
    planned_start: DateTime<FixedOffset>,
    planned_end: DateTime<FixedOffset>,
}

fn median(values: &mut Vec<i64>) -> i64 {
    values.sort_unstable();
    let len = values.len();
    if len % 2 == 1 {
        values[len / 2]
    } else {
        (values[len / 2 - 1] + values[len / 2]) / 2
    }
}

fn merge_windows(
    existing: &[AvoidanceWindow],
    new_windows: &[AvoidanceWindow],
) -> Vec<AvoidanceWindow> {
    let mut combined = existing.to_vec();
    combined.extend_from_slice(new_windows);
    combined.sort_by_key(|window| (window.weekday, window.start_minute));

    let mut merged: Vec<AvoidanceWindow> = Vec::new();
    for window in combined {
        if let Some(last) = merged.last_mut() {
            if last.weekday == window.weekday && window.start_minute <= last.end_minute {
                last.end_minute = last.end_minute.max(window.end_minute);
                continue;
            }
        }
        merged.push(window);
    }
    merged
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, TimeZone};
    use rusqlite::Connection;

    #[test]
    fn test_merge_windows() {
        let existing = vec![AvoidanceWindow {
            weekday: 1,
            start_minute: 60,
            end_minute: 120,
        }];
        let new_windows = vec![AvoidanceWindow {
            weekday: 1,
            start_minute: 90,
            end_minute: 150,
        }];
        let merged = merge_windows(&existing, &new_windows);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].start_minute, 60);
        assert_eq!(merged[0].end_minute, 150);
    }

    #[test]
    fn test_median() {
        let mut values = vec![5, 1, 9, 3, 7];
        assert_eq!(median(&mut values), 5);
    }

    #[test]
    fn test_load_default_preferences() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE schedule_preferences (id TEXT PRIMARY KEY, data TEXT NOT NULL, updated_at TEXT NOT NULL)",
            [],
        )
        .unwrap();

        let service = BehaviorLearningService::new(&conn);
        let snapshot = service.load_preferences("default").unwrap();
        assert_eq!(snapshot.focus_start_minute, None);
        assert_eq!(snapshot.buffer_minutes_between_blocks, 0);
    }

    #[test]
    fn ingest_feedback_updates_focus_buffer_and_avoidance() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE schedule_preferences (id TEXT PRIMARY KEY, data TEXT NOT NULL, updated_at TEXT NOT NULL)",
            [],
        )
        .unwrap();

        let service = BehaviorLearningService::new(&conn);
        let tz = FixedOffset::east_opt(0).unwrap();
        let monday = tz.with_ymd_and_hms(2025, 5, 5, 9, 0, 0).unwrap();

        let mut feedback = Vec::new();
        for i in 0..4 {
            let start = monday + Duration::minutes(i * 30);
            let end = start + Duration::minutes(45);
            feedback.push(FeedbackEvent {
                task_id: format!("task-success-{i}"),
                planned_start_at: schedule_utils::format_datetime(start),
                planned_end_at: schedule_utils::format_datetime(end),
                actual_start_at: Some(schedule_utils::format_datetime(
                    start + Duration::minutes(15),
                )),
                actual_end_at: Some(schedule_utils::format_datetime(end + Duration::minutes(20))),
                completed: true,
            });
        }

        let tuesday = monday + Duration::days(1);
        for i in 0..3 {
            let start = tuesday + Duration::minutes(i * 60);
            let end = start + Duration::minutes(30);
            feedback.push(FeedbackEvent {
                task_id: format!("task-failure-{i}"),
                planned_start_at: schedule_utils::format_datetime(start),
                planned_end_at: schedule_utils::format_datetime(end),
                actual_start_at: None,
                actual_end_at: None,
                completed: false,
            });
        }

        service
            .ingest_feedback("default", &feedback)
            .expect("ingest feedback");

        let snapshot = service.load_preferences("default").expect("load updated");
        assert!(
            snapshot.focus_start_minute.is_some(),
            "expected focus start to be inferred"
        );
        assert!(
            snapshot.focus_end_minute.is_some(),
            "expected focus end to be inferred"
        );
        assert!(snapshot.buffer_minutes_between_blocks >= 30);

        let avoidance = snapshot
            .avoidance_windows
            .iter()
            .find(|window| window.weekday == 1)
            .expect("expected avoidance window for Tuesday");
        let tuesday_start = schedule_utils::midnight_minutes_of(tuesday) as u32;
        assert!(avoidance.start_minute <= tuesday_start + 60);
        assert!(avoidance.end_minute >= tuesday_start + 90);
    }
}
