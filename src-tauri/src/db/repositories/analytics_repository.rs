use std::convert::TryFrom;

use chrono::NaiveDate;
use rusqlite::{named_params, Connection, OptionalExtension, Row};

use crate::error::AppResult;
use crate::models::analytics::AnalyticsSnapshotRecord;

#[derive(Debug, Clone)]
pub struct AnalyticsSnapshotRow {
    pub snapshot_date: String,
    pub total_tasks_completed: i64,
    pub completion_rate: f64,
    pub overdue_tasks: i64,
    pub total_focus_minutes: i64,
    pub productivity_score: f64,
    pub efficiency_rating: f64,
    pub time_spent_work: f64,
    pub time_spent_study: f64,
    pub time_spent_life: f64,
    pub time_spent_other: f64,
    pub on_time_ratio: f64,
    pub focus_consistency: f64,
    pub rest_balance: f64,
    pub capacity_risk: f64,
    pub created_at: String,
}

impl AnalyticsSnapshotRow {
    pub fn from_record(record: &AnalyticsSnapshotRecord) -> Self {
        Self {
            snapshot_date: record.snapshot_date.clone(),
            total_tasks_completed: record.total_tasks_completed,
            completion_rate: record.completion_rate,
            overdue_tasks: record.overdue_tasks,
            total_focus_minutes: record.total_focus_minutes,
            productivity_score: record.productivity_score,
            efficiency_rating: record.efficiency_rating,
            time_spent_work: record.time_spent_work,
            time_spent_study: record.time_spent_study,
            time_spent_life: record.time_spent_life,
            time_spent_other: record.time_spent_other,
            on_time_ratio: record.on_time_ratio,
            focus_consistency: record.focus_consistency,
            rest_balance: record.rest_balance,
            capacity_risk: record.capacity_risk,
            created_at: record.created_at.clone(),
        }
    }

    pub fn into_record(self) -> AnalyticsSnapshotRecord {
        AnalyticsSnapshotRecord {
            snapshot_date: self.snapshot_date,
            total_tasks_completed: self.total_tasks_completed,
            completion_rate: self.completion_rate,
            overdue_tasks: self.overdue_tasks,
            total_focus_minutes: self.total_focus_minutes,
            productivity_score: self.productivity_score,
            efficiency_rating: self.efficiency_rating,
            time_spent_work: self.time_spent_work,
            time_spent_study: self.time_spent_study,
            time_spent_life: self.time_spent_life,
            time_spent_other: self.time_spent_other,
            on_time_ratio: self.on_time_ratio,
            focus_consistency: self.focus_consistency,
            rest_balance: self.rest_balance,
            capacity_risk: self.capacity_risk,
            created_at: self.created_at,
        }
    }
}

impl TryFrom<&Row<'_>> for AnalyticsSnapshotRow {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            snapshot_date: row.get("snapshot_date")?,
            total_tasks_completed: row.get("total_tasks_completed")?,
            completion_rate: row.get("completion_rate")?,
            overdue_tasks: row.get("overdue_tasks")?,
            total_focus_minutes: row.get("total_focus_minutes")?,
            productivity_score: row.get("productivity_score")?,
            efficiency_rating: row.get("efficiency_rating")?,
            time_spent_work: row.get("time_spent_work")?,
            time_spent_study: row.get("time_spent_study")?,
            time_spent_life: row.get("time_spent_life")?,
            time_spent_other: row.get("time_spent_other")?,
            on_time_ratio: row.get("on_time_ratio")?,
            focus_consistency: row.get("focus_consistency")?,
            rest_balance: row.get("rest_balance")?,
            capacity_risk: row.get("capacity_risk")?,
            created_at: row.get("created_at")?,
        })
    }
}

pub struct AnalyticsRepository;

impl AnalyticsRepository {
    pub fn upsert_snapshot(conn: &Connection, row: &AnalyticsSnapshotRow) -> AppResult<()> {
        conn.execute(
            r#"
                INSERT INTO analytics_snapshots (
                    snapshot_date,
                    total_tasks_completed,
                    completion_rate,
                    overdue_tasks,
                    total_focus_minutes,
                    productivity_score,
                    efficiency_rating,
                    time_spent_work,
                    time_spent_study,
                    time_spent_life,
                    time_spent_other,
                    on_time_ratio,
                    focus_consistency,
                    rest_balance,
                    capacity_risk,
                    created_at
                ) VALUES (
                    :snapshot_date,
                    :total_tasks_completed,
                    :completion_rate,
                    :overdue_tasks,
                    :total_focus_minutes,
                    :productivity_score,
                    :efficiency_rating,
                    :time_spent_work,
                    :time_spent_study,
                    :time_spent_life,
                    :time_spent_other,
                    :on_time_ratio,
                    :focus_consistency,
                    :rest_balance,
                    :capacity_risk,
                    :created_at
                )
                ON CONFLICT(snapshot_date) DO UPDATE SET
                    total_tasks_completed = excluded.total_tasks_completed,
                    completion_rate = excluded.completion_rate,
                    overdue_tasks = excluded.overdue_tasks,
                    total_focus_minutes = excluded.total_focus_minutes,
                    productivity_score = excluded.productivity_score,
                    efficiency_rating = excluded.efficiency_rating,
                    time_spent_work = excluded.time_spent_work,
                    time_spent_study = excluded.time_spent_study,
                    time_spent_life = excluded.time_spent_life,
                    time_spent_other = excluded.time_spent_other,
                    on_time_ratio = excluded.on_time_ratio,
                    focus_consistency = excluded.focus_consistency,
                    rest_balance = excluded.rest_balance,
                    capacity_risk = excluded.capacity_risk,
                    created_at = excluded.created_at
            "#,
            named_params! {
                ":snapshot_date": &row.snapshot_date,
                ":total_tasks_completed": &row.total_tasks_completed,
                ":completion_rate": &row.completion_rate,
                ":overdue_tasks": &row.overdue_tasks,
                ":total_focus_minutes": &row.total_focus_minutes,
                ":productivity_score": &row.productivity_score,
                ":efficiency_rating": &row.efficiency_rating,
                ":time_spent_work": &row.time_spent_work,
                ":time_spent_study": &row.time_spent_study,
                ":time_spent_life": &row.time_spent_life,
                ":time_spent_other": &row.time_spent_other,
                ":on_time_ratio": &row.on_time_ratio,
                ":focus_consistency": &row.focus_consistency,
                ":rest_balance": &row.rest_balance,
                ":capacity_risk": &row.capacity_risk,
                ":created_at": &row.created_at,
            },
        )?;

        Ok(())
    }

    pub fn find_by_date(
        conn: &Connection,
        snapshot_date: &NaiveDate,
    ) -> AppResult<Option<AnalyticsSnapshotRow>> {
        let mut stmt = conn.prepare(
            r#"
            SELECT
                snapshot_date,
                total_tasks_completed,
                completion_rate,
                overdue_tasks,
                total_focus_minutes,
                productivity_score,
                efficiency_rating,
                time_spent_work,
                time_spent_study,
                time_spent_life,
                time_spent_other,
                on_time_ratio,
                focus_consistency,
                rest_balance,
                capacity_risk,
                created_at
            FROM analytics_snapshots
            WHERE snapshot_date = ?1
        "#,
        )?;

        let row = stmt
            .query_row([snapshot_date.to_string()], |row| {
                AnalyticsSnapshotRow::try_from(row)
            })
            .optional()?;

        Ok(row)
    }

    pub fn list_recent_snapshots(
        conn: &Connection,
        limit: usize,
    ) -> AppResult<Vec<AnalyticsSnapshotRow>> {
        let mut stmt = conn.prepare(
            r#"
            SELECT
                snapshot_date,
                total_tasks_completed,
                completion_rate,
                overdue_tasks,
                total_focus_minutes,
                productivity_score,
                efficiency_rating,
                time_spent_work,
                time_spent_study,
                time_spent_life,
                time_spent_other,
                on_time_ratio,
                focus_consistency,
                rest_balance,
                capacity_risk,
                created_at
            FROM analytics_snapshots
            ORDER BY snapshot_date DESC
            LIMIT ?1
        "#,
        )?;

        let rows = stmt
            .query_map([limit as i64], |row| AnalyticsSnapshotRow::try_from(row))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(rows)
    }

    pub fn delete_before(conn: &Connection, cutoff: &NaiveDate) -> AppResult<usize> {
        let deleted = conn.execute(
            "DELETE FROM analytics_snapshots WHERE snapshot_date < ?1",
            [cutoff.to_string()],
        )?;
        Ok(deleted as usize)
    }
}
