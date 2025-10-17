use std::convert::TryFrom;

use rusqlite::{named_params, Connection, OptionalExtension, Row};

use crate::error::{AppError, AppResult};
use crate::models::wellness::{
    WellnessEventInsert, WellnessEventRecord, WellnessEventResponseUpdate, WellnessResponse,
    WellnessTriggerReason,
};

#[derive(Debug, Clone)]
pub struct WellnessEventRow {
    pub id: i64,
    pub window_start: String,
    pub trigger_reason: String,
    pub recommended_break_minutes: i64,
    pub suggested_micro_task: Option<String>,
    pub response: Option<String>,
    pub response_at: Option<String>,
    pub deferral_count: i64,
}

impl WellnessEventRow {
    pub fn from_insert(insert: &WellnessEventInsert) -> Self {
        Self {
            id: 0,
            window_start: insert.window_start.clone(),
            trigger_reason: insert.trigger_reason.as_str().to_string(),
            recommended_break_minutes: insert.recommended_break_minutes,
            suggested_micro_task: insert.suggested_micro_task.clone(),
            response: None,
            response_at: None,
            deferral_count: 0,
        }
    }

    pub fn into_record(self) -> AppResult<WellnessEventRecord> {
        let trigger_reason = WellnessTriggerReason::try_from(self.trigger_reason.as_str())
            .map_err(AppError::validation)?;

        let response = match self.response {
            Some(value) => {
                Some(WellnessResponse::try_from(value.as_str()).map_err(AppError::validation)?)
            }
            None => None,
        };

        Ok(WellnessEventRecord {
            id: self.id,
            window_start: self.window_start,
            trigger_reason,
            recommended_break_minutes: self.recommended_break_minutes,
            suggested_micro_task: self.suggested_micro_task,
            response,
            response_at: self.response_at,
            deferral_count: self.deferral_count,
        })
    }
}

impl TryFrom<&Row<'_>> for WellnessEventRow {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.get("id")?,
            window_start: row.get("window_start")?,
            trigger_reason: row.get("trigger_reason")?,
            recommended_break_minutes: row.get("recommended_break_minutes")?,
            suggested_micro_task: row.get("suggested_micro_task")?,
            response: row.get("response")?,
            response_at: row.get("response_at")?,
            deferral_count: row.get("deferral_count")?,
        })
    }
}

pub struct WellnessRepository;

impl WellnessRepository {
    pub fn insert(conn: &Connection, insert: &WellnessEventInsert) -> AppResult<i64> {
        let row = WellnessEventRow::from_insert(insert);

        conn.execute(
            r#"
                INSERT INTO wellness_events (
                    window_start,
                    trigger_reason,
                    recommended_break_minutes,
                    suggested_micro_task
                ) VALUES (
                    :window_start,
                    :trigger_reason,
                    :recommended_break_minutes,
                    :suggested_micro_task
                )
            "#,
            named_params! {
                ":window_start": &row.window_start,
                ":trigger_reason": &row.trigger_reason,
                ":recommended_break_minutes": &row.recommended_break_minutes,
                ":suggested_micro_task": &row.suggested_micro_task,
            },
        )?;

        Ok(conn.last_insert_rowid())
    }

    pub fn find_by_id(conn: &Connection, id: i64) -> AppResult<WellnessEventRecord> {
        let mut stmt = conn.prepare(
            r#"
                SELECT
                    id,
                    window_start,
                    trigger_reason,
                    recommended_break_minutes,
                    suggested_micro_task,
                    response,
                    response_at,
                    deferral_count
                FROM wellness_events
                WHERE id = :id
            "#,
        )?;

        let row = stmt
            .query_row(named_params! {":id": id}, |row| {
                WellnessEventRow::try_from(row)
            })
            .optional()?;

        match row {
            Some(row) => row.into_record(),
            None => Err(AppError::not_found()),
        }
    }

    pub fn list_recent(conn: &Connection, limit: usize) -> AppResult<Vec<WellnessEventRecord>> {
        let mut stmt = conn.prepare(
            r#"
                SELECT
                    id,
                    window_start,
                    trigger_reason,
                    recommended_break_minutes,
                    suggested_micro_task,
                    response,
                    response_at,
                    deferral_count
                FROM wellness_events
                ORDER BY window_start DESC
                LIMIT :limit
            "#,
        )?;

        let records = stmt
            .query_map(named_params! {":limit": limit as i64}, |row| {
                WellnessEventRow::try_from(row)
            })?
            .map(|row| {
                row.map_err(AppError::from)
                    .and_then(|row| row.into_record())
            })
            .collect::<AppResult<Vec<_>>>()?;

        Ok(records)
    }

    pub fn list_pending(conn: &Connection, limit: usize) -> AppResult<Vec<WellnessEventRecord>> {
        let mut stmt = conn.prepare(
            r#"
                SELECT
                    id,
                    window_start,
                    trigger_reason,
                    recommended_break_minutes,
                    suggested_micro_task,
                    response,
                    response_at,
                    deferral_count
                FROM wellness_events
                WHERE response IS NULL
                ORDER BY window_start ASC
                LIMIT :limit
            "#,
        )?;

        let records = stmt
            .query_map(named_params! {":limit": limit as i64}, |row| {
                WellnessEventRow::try_from(row)
            })?
            .map(|row| {
                row.map_err(AppError::from)
                    .and_then(|row| row.into_record())
            })
            .collect::<AppResult<Vec<_>>>()?;

        Ok(records)
    }

    pub fn update_response(
        conn: &Connection,
        id: i64,
        update: &WellnessEventResponseUpdate,
    ) -> AppResult<()> {
        let affected = conn.execute(
            r#"
                UPDATE wellness_events SET
                    response = :response,
                    response_at = :response_at,
                    deferral_count = :deferral_count
                WHERE id = :id
            "#,
            named_params! {
                ":id": id,
                ":response": update.response.as_str(),
                ":response_at": &update.response_at,
                ":deferral_count": update.deferral_count,
            },
        )?;

        if affected == 0 {
            return Err(AppError::not_found());
        }

        Ok(())
    }
}
