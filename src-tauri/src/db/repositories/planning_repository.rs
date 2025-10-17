use std::convert::TryFrom;

use rusqlite::{named_params, Connection, OptionalExtension, Row};
use serde_json::Value as JsonValue;

use crate::error::{AppError, AppResult};
use crate::models::planning::{
    PlanningOptionRecord, PlanningSessionRecord, PlanningTimeBlockRecord, SchedulePreferencesRecord,
};

#[derive(Debug, Clone)]
pub struct PlanningSessionRow {
    pub id: String,
    pub task_ids: String,
    pub constraints: Option<String>,
    pub generated_at: String,
    pub status: String,
    pub selected_option_id: Option<String>,
    pub personalization_snapshot: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl PlanningSessionRow {
    pub fn from_record(record: &PlanningSessionRecord) -> AppResult<Self> {
        Ok(Self {
            id: record.id.clone(),
            task_ids: serialize_vec(&record.task_ids)?,
            constraints: serialize_json(record.constraints.as_ref())?,
            generated_at: record.generated_at.clone(),
            status: record.status.clone(),
            selected_option_id: record.selected_option_id.clone(),
            personalization_snapshot: serialize_json(record.personalization_snapshot.as_ref())?,
            created_at: record.created_at.clone(),
            updated_at: record.updated_at.clone(),
        })
    }

    pub fn into_record(self) -> AppResult<PlanningSessionRecord> {
        Ok(PlanningSessionRecord {
            id: self.id,
            task_ids: deserialize_vec(Some(self.task_ids))?,
            constraints: deserialize_json(self.constraints)?,
            generated_at: self.generated_at,
            status: self.status,
            selected_option_id: self.selected_option_id,
            personalization_snapshot: deserialize_json(self.personalization_snapshot)?,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

impl TryFrom<&Row<'_>> for PlanningSessionRow {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.get("id")?,
            task_ids: row.get("task_ids")?,
            constraints: row.get("constraints")?,
            generated_at: row.get("generated_at")?,
            status: row.get("status")?,
            selected_option_id: row.get("selected_option_id")?,
            personalization_snapshot: row.get("personalization_snapshot")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct PlanningOptionRow {
    pub id: String,
    pub session_id: String,
    pub rank: i64,
    pub score: Option<f64>,
    pub summary: Option<String>,
    pub cot_steps: Option<String>,
    pub risk_notes: Option<String>,
    pub is_fallback: bool,
    pub created_at: String,
}

impl PlanningOptionRow {
    pub fn from_record(record: &PlanningOptionRecord) -> AppResult<Self> {
        Ok(Self {
            id: record.id.clone(),
            session_id: record.session_id.clone(),
            rank: record.rank,
            score: record.score,
            summary: record.summary.clone(),
            cot_steps: serialize_json(record.cot_steps.as_ref())?,
            risk_notes: serialize_json(record.risk_notes.as_ref())?,
            is_fallback: record.is_fallback,
            created_at: record.created_at.clone(),
        })
    }

    pub fn into_record(self) -> AppResult<PlanningOptionRecord> {
        Ok(PlanningOptionRecord {
            id: self.id,
            session_id: self.session_id,
            rank: self.rank,
            score: self.score,
            summary: self.summary,
            cot_steps: deserialize_json(self.cot_steps)?,
            risk_notes: deserialize_json(self.risk_notes)?,
            is_fallback: self.is_fallback,
            created_at: self.created_at,
        })
    }
}

impl TryFrom<&Row<'_>> for PlanningOptionRow {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.get("id")?,
            session_id: row.get("session_id")?,
            rank: row.get("rank")?,
            score: row.get("score")?,
            summary: row.get("summary")?,
            cot_steps: row.get("cot_steps")?,
            risk_notes: row.get("risk_notes")?,
            is_fallback: row.get::<_, i64>("is_fallback")? != 0,
            created_at: row.get("created_at")?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct PlanningTimeBlockRow {
    pub id: String,
    pub option_id: String,
    pub task_id: String,
    pub start_at: String,
    pub end_at: String,
    pub flexibility: Option<String>,
    pub confidence: Option<f64>,
    pub conflict_flags: Option<String>,
    pub applied_at: Option<String>,
    pub actual_start_at: Option<String>,
    pub actual_end_at: Option<String>,
    pub status: String,
}

impl PlanningTimeBlockRow {
    pub fn from_record(record: &PlanningTimeBlockRecord) -> AppResult<Self> {
        Ok(Self {
            id: record.id.clone(),
            option_id: record.option_id.clone(),
            task_id: record.task_id.clone(),
            start_at: record.start_at.clone(),
            end_at: record.end_at.clone(),
            flexibility: record.flexibility.clone(),
            confidence: record.confidence,
            conflict_flags: serialize_json(record.conflict_flags.as_ref())?,
            applied_at: record.applied_at.clone(),
            actual_start_at: record.actual_start_at.clone(),
            actual_end_at: record.actual_end_at.clone(),
            status: record.status.clone(),
        })
    }

    pub fn into_record(self) -> AppResult<PlanningTimeBlockRecord> {
        Ok(PlanningTimeBlockRecord {
            id: self.id,
            option_id: self.option_id,
            task_id: self.task_id,
            start_at: self.start_at,
            end_at: self.end_at,
            flexibility: self.flexibility,
            confidence: self.confidence,
            conflict_flags: deserialize_json(self.conflict_flags)?,
            applied_at: self.applied_at,
            actual_start_at: self.actual_start_at,
            actual_end_at: self.actual_end_at,
            status: self.status,
        })
    }
}

impl TryFrom<&Row<'_>> for PlanningTimeBlockRow {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.get("id")?,
            option_id: row.get("option_id")?,
            task_id: row.get("task_id")?,
            start_at: row.get("start_at")?,
            end_at: row.get("end_at")?,
            flexibility: row.get("flexibility")?,
            confidence: row.get("confidence")?,
            conflict_flags: row.get("conflict_flags")?,
            applied_at: row.get("applied_at")?,
            actual_start_at: row.get("actual_start_at")?,
            actual_end_at: row.get("actual_end_at")?,
            status: row.get("status")?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct SchedulePreferencesRow {
    pub id: String,
    pub data: String,
    pub updated_at: String,
}

impl SchedulePreferencesRow {
    pub fn from_record(record: &SchedulePreferencesRecord) -> AppResult<Self> {
        Ok(Self {
            id: record.id.clone(),
            data: serialize_required_json(&record.data)?,
            updated_at: record.updated_at.clone(),
        })
    }

    pub fn into_record(self) -> AppResult<SchedulePreferencesRecord> {
        Ok(SchedulePreferencesRecord {
            id: self.id,
            data: deserialize_required_json(self.data)?,
            updated_at: self.updated_at,
        })
    }
}

impl TryFrom<&Row<'_>> for SchedulePreferencesRow {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.get("id")?,
            data: row.get("data")?,
            updated_at: row.get("updated_at")?,
        })
    }
}

pub struct PlanningRepository;

impl PlanningRepository {
    pub fn insert_session(conn: &Connection, row: &PlanningSessionRow) -> AppResult<()> {
        conn.execute(
            r#"
                INSERT INTO planning_sessions (
                    id,
                    task_ids,
                    constraints,
                    generated_at,
                    status,
                    selected_option_id,
                    personalization_snapshot,
                    created_at,
                    updated_at
                ) VALUES (
                    :id,
                    :task_ids,
                    :constraints,
                    :generated_at,
                    :status,
                    :selected_option_id,
                    :personalization_snapshot,
                    :created_at,
                    :updated_at
                )
            "#,
            named_params! {
                ":id": &row.id,
                ":task_ids": &row.task_ids,
                ":constraints": &row.constraints,
                ":generated_at": &row.generated_at,
                ":status": &row.status,
                ":selected_option_id": &row.selected_option_id,
                ":personalization_snapshot": &row.personalization_snapshot,
                ":created_at": &row.created_at,
                ":updated_at": &row.updated_at,
            },
        )?;

        Ok(())
    }

    pub fn update_session(conn: &Connection, row: &PlanningSessionRow) -> AppResult<()> {
        let affected = conn.execute(
            r#"
                UPDATE planning_sessions SET
                    task_ids = :task_ids,
                    constraints = :constraints,
                    generated_at = :generated_at,
                    status = :status,
                    selected_option_id = :selected_option_id,
                    personalization_snapshot = :personalization_snapshot,
                    updated_at = :updated_at
                WHERE id = :id
            "#,
            named_params! {
                ":id": &row.id,
                ":task_ids": &row.task_ids,
                ":constraints": &row.constraints,
                ":generated_at": &row.generated_at,
                ":status": &row.status,
                ":selected_option_id": &row.selected_option_id,
                ":personalization_snapshot": &row.personalization_snapshot,
                ":updated_at": &row.updated_at,
            },
        )?;

        if affected == 0 {
            return Err(AppError::not_found());
        }

        Ok(())
    }

    pub fn delete_session(conn: &Connection, id: &str) -> AppResult<()> {
        let affected = conn.execute("DELETE FROM planning_sessions WHERE id = ?1", [id])?;
        if affected == 0 {
            return Err(AppError::not_found());
        }
        Ok(())
    }

    pub fn find_session_by_id(
        conn: &Connection,
        id: &str,
    ) -> AppResult<Option<PlanningSessionRow>> {
        let mut stmt = conn.prepare(
            r#"
            SELECT
                id,
                task_ids,
                constraints,
                generated_at,
                status,
                selected_option_id,
                personalization_snapshot,
                created_at,
                updated_at
            FROM planning_sessions
            WHERE id = ?1
        "#,
        )?;

        let row = stmt
            .query_row([id], |row| PlanningSessionRow::try_from(row))
            .optional()?;

        Ok(row)
    }

    pub fn list_recent_sessions(
        conn: &Connection,
        limit: usize,
    ) -> AppResult<Vec<PlanningSessionRow>> {
        let mut stmt = conn.prepare(
            r#"
            SELECT
                id,
                task_ids,
                constraints,
                generated_at,
                status,
                selected_option_id,
                personalization_snapshot,
                created_at,
                updated_at
            FROM planning_sessions
            ORDER BY generated_at DESC
            LIMIT ?1
        "#,
        )?;

        let rows = stmt
            .query_map([limit as i64], |row| PlanningSessionRow::try_from(row))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(rows)
    }

    pub fn insert_option(conn: &Connection, row: &PlanningOptionRow) -> AppResult<()> {
        conn.execute(
            r#"
                INSERT INTO planning_options (
                    id,
                    session_id,
                    rank,
                    score,
                    summary,
                    cot_steps,
                    risk_notes,
                    is_fallback,
                    created_at
                ) VALUES (
                    :id,
                    :session_id,
                    :rank,
                    :score,
                    :summary,
                    :cot_steps,
                    :risk_notes,
                    :is_fallback,
                    :created_at
                )
            "#,
            named_params! {
                ":id": &row.id,
                ":session_id": &row.session_id,
                ":rank": &row.rank,
                ":score": &row.score,
                ":summary": &row.summary,
                ":cot_steps": &row.cot_steps,
                ":risk_notes": &row.risk_notes,
                ":is_fallback": row.is_fallback as i64,
                ":created_at": &row.created_at,
            },
        )?;

        Ok(())
    }

    pub fn update_option(conn: &Connection, row: &PlanningOptionRow) -> AppResult<()> {
        let affected = conn.execute(
            r#"
                UPDATE planning_options SET
                    session_id = :session_id,
                    rank = :rank,
                    score = :score,
                    summary = :summary,
                    cot_steps = :cot_steps,
                    risk_notes = :risk_notes,
                    is_fallback = :is_fallback
                WHERE id = :id
            "#,
            named_params! {
                ":id": &row.id,
                ":session_id": &row.session_id,
                ":rank": &row.rank,
                ":score": &row.score,
                ":summary": &row.summary,
                ":cot_steps": &row.cot_steps,
                ":risk_notes": &row.risk_notes,
                ":is_fallback": row.is_fallback as i64,
            },
        )?;

        if affected == 0 {
            return Err(AppError::not_found());
        }

        Ok(())
    }

    pub fn delete_options_for_session(conn: &Connection, session_id: &str) -> AppResult<()> {
        conn.execute(
            "DELETE FROM planning_options WHERE session_id = ?1",
            [session_id],
        )?;
        Ok(())
    }

    pub fn find_option_by_id(conn: &Connection, id: &str) -> AppResult<Option<PlanningOptionRow>> {
        let mut stmt = conn.prepare(
            r#"
            SELECT
                id,
                session_id,
                rank,
                score,
                summary,
                cot_steps,
                risk_notes,
                is_fallback,
                created_at
            FROM planning_options
            WHERE id = ?1
        "#,
        )?;

        let row = stmt
            .query_row([id], |row| PlanningOptionRow::try_from(row))
            .optional()?;

        Ok(row)
    }

    pub fn list_options_for_session(
        conn: &Connection,
        session_id: &str,
    ) -> AppResult<Vec<PlanningOptionRow>> {
        let mut stmt = conn.prepare(
            r#"
            SELECT
                id,
                session_id,
                rank,
                score,
                summary,
                cot_steps,
                risk_notes,
                is_fallback,
                created_at
            FROM planning_options
            WHERE session_id = ?1
            ORDER BY rank ASC
        "#,
        )?;

        let rows = stmt
            .query_map([session_id], |row| PlanningOptionRow::try_from(row))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(rows)
    }

    pub fn insert_time_block(conn: &Connection, row: &PlanningTimeBlockRow) -> AppResult<()> {
        conn.execute(
            r#"
                INSERT INTO planning_time_blocks (
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
                ) VALUES (
                    :id,
                    :option_id,
                    :task_id,
                    :start_at,
                    :end_at,
                    :flexibility,
                    :confidence,
                    :conflict_flags,
                    :applied_at,
                    :actual_start_at,
                    :actual_end_at,
                    :status
                )
            "#,
            named_params! {
                ":id": &row.id,
                ":option_id": &row.option_id,
                ":task_id": &row.task_id,
                ":start_at": &row.start_at,
                ":end_at": &row.end_at,
                ":flexibility": &row.flexibility,
                ":confidence": &row.confidence,
                ":conflict_flags": &row.conflict_flags,
                ":applied_at": &row.applied_at,
                ":actual_start_at": &row.actual_start_at,
                ":actual_end_at": &row.actual_end_at,
                ":status": &row.status,
            },
        )?;

        Ok(())
    }

    pub fn update_time_block(conn: &Connection, row: &PlanningTimeBlockRow) -> AppResult<()> {
        let affected = conn.execute(
            r#"
                UPDATE planning_time_blocks SET
                    option_id = :option_id,
                    task_id = :task_id,
                    start_at = :start_at,
                    end_at = :end_at,
                    flexibility = :flexibility,
                    confidence = :confidence,
                    conflict_flags = :conflict_flags,
                    applied_at = :applied_at,
                    actual_start_at = :actual_start_at,
                    actual_end_at = :actual_end_at,
                    status = :status
                WHERE id = :id
            "#,
            named_params! {
                ":id": &row.id,
                ":option_id": &row.option_id,
                ":task_id": &row.task_id,
                ":start_at": &row.start_at,
                ":end_at": &row.end_at,
                ":flexibility": &row.flexibility,
                ":confidence": &row.confidence,
                ":conflict_flags": &row.conflict_flags,
                ":applied_at": &row.applied_at,
                ":actual_start_at": &row.actual_start_at,
                ":actual_end_at": &row.actual_end_at,
                ":status": &row.status,
            },
        )?;

        if affected == 0 {
            return Err(AppError::not_found());
        }

        Ok(())
    }

    pub fn delete_time_blocks_for_option(conn: &Connection, option_id: &str) -> AppResult<()> {
        conn.execute(
            "DELETE FROM planning_time_blocks WHERE option_id = ?1",
            [option_id],
        )?;
        Ok(())
    }

    pub fn list_time_blocks_for_option(
        conn: &Connection,
        option_id: &str,
    ) -> AppResult<Vec<PlanningTimeBlockRow>> {
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
            WHERE option_id = ?1
            ORDER BY start_at ASC
        "#,
        )?;

        let rows = stmt
            .query_map([option_id], |row| PlanningTimeBlockRow::try_from(row))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(rows)
    }

    pub fn list_time_blocks_for_task(
        conn: &Connection,
        task_id: &str,
    ) -> AppResult<Vec<PlanningTimeBlockRow>> {
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
            WHERE task_id = ?1
            ORDER BY start_at ASC
        "#,
        )?;

        let rows = stmt
            .query_map([task_id], |row| PlanningTimeBlockRow::try_from(row))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(rows)
    }

    pub fn delete_time_blocks_for_session(conn: &Connection, session_id: &str) -> AppResult<()> {
        conn.execute(
            r#"
                DELETE FROM planning_time_blocks
                WHERE option_id IN (
                    SELECT id FROM planning_options WHERE session_id = ?1
                )
            "#,
            [session_id],
        )?;
        Ok(())
    }

    pub fn get_schedule_preferences(
        conn: &Connection,
        id: &str,
    ) -> AppResult<Option<SchedulePreferencesRow>> {
        let mut stmt = conn.prepare(
            r#"
            SELECT id, data, updated_at
            FROM schedule_preferences
            WHERE id = ?1
        "#,
        )?;

        let row = stmt
            .query_row([id], |row| SchedulePreferencesRow::try_from(row))
            .optional()?;

        Ok(row)
    }

    pub fn upsert_schedule_preferences(
        conn: &Connection,
        row: &SchedulePreferencesRow,
    ) -> AppResult<()> {
        conn.execute(
            r#"
                INSERT INTO schedule_preferences (id, data, updated_at)
                VALUES (:id, :data, :updated_at)
                ON CONFLICT(id) DO UPDATE SET
                    data = excluded.data,
                    updated_at = excluded.updated_at
            "#,
            named_params! {
                ":id": &row.id,
                ":data": &row.data,
                ":updated_at": &row.updated_at,
            },
        )?;

        Ok(())
    }
}

fn serialize_vec(values: &[String]) -> AppResult<String> {
    Ok(serde_json::to_string(values)?)
}

fn deserialize_vec(raw: Option<String>) -> AppResult<Vec<String>> {
    match raw {
        Some(value) if !value.is_empty() => Ok(serde_json::from_str(&value)?),
        _ => Ok(Vec::new()),
    }
}

fn serialize_json(value: Option<&JsonValue>) -> AppResult<Option<String>> {
    match value {
        Some(v) => Ok(Some(serde_json::to_string(v)?)),
        Option::None => Ok(None),
    }
}

fn deserialize_json(raw: Option<String>) -> AppResult<Option<JsonValue>> {
    match raw {
        Some(value) if !value.is_empty() => Ok(Some(serde_json::from_str(&value)?)),
        _ => Ok(None),
    }
}

fn serialize_required_json(value: &JsonValue) -> AppResult<String> {
    Ok(serde_json::to_string(value)?)
}

fn deserialize_required_json(raw: String) -> AppResult<JsonValue> {
    Ok(serde_json::from_str(&raw)?)
}
