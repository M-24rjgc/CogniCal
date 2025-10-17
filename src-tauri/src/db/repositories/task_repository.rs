use std::convert::TryFrom;

use rusqlite::{named_params, Connection, OptionalExtension, Row};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value as JsonValue;

use crate::error::{AppError, AppResult};
use crate::models::ai::{
    TaskAiReasoningStep, TaskAiSource, TaskEfficiencyPrediction, TaskFocusModeRecommendation,
};
use crate::models::task::{TaskAiInsights, TaskRecord, TaskRecurrence};

const BASE_SELECT: &str = r#"
    SELECT
        id,
        title,
        description,
        status,
        priority,
        planned_start_at,
        start_at,
        due_at,
        completed_at,
        estimated_minutes,
        estimated_hours,
        tags,
        owner_id,
        task_type,
        is_recurring,
        recurrence_rule,
        recurrence_until,
        ai_summary,
        ai_next_action,
        ai_confidence,
        ai_complexity_score,
        ai_suggested_start_at,
        ai_focus_mode,
        ai_efficiency_prediction,
        ai_cot_steps,
        ai_cot_summary,
        ai_metadata,
        ai_source,
        ai_generated_at,
        external_links,
        created_at,
        updated_at
    FROM tasks
"#;

#[derive(Debug, Clone)]
pub struct TaskRow {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub planned_start_at: Option<String>,
    pub start_at: Option<String>,
    pub due_at: Option<String>,
    pub completed_at: Option<String>,
    pub estimated_minutes: Option<i64>,
    pub estimated_hours: Option<f64>,
    pub tags: Option<String>,
    pub owner_id: Option<String>,
    pub task_type: Option<String>,
    pub is_recurring: bool,
    pub recurrence_rule: Option<String>,
    pub recurrence_until: Option<String>,
    pub ai_summary: Option<String>,
    pub ai_next_action: Option<String>,
    pub ai_confidence: Option<f64>,
    pub ai_complexity_score: Option<f64>,
    pub ai_suggested_start_at: Option<String>,
    pub ai_focus_mode: Option<String>,
    pub ai_efficiency_prediction: Option<String>,
    pub ai_cot_steps: Option<String>,
    pub ai_cot_summary: Option<String>,
    pub ai_metadata: Option<String>,
    pub ai_source: Option<String>,
    pub ai_generated_at: Option<String>,
    pub external_links: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl TaskRow {
    pub fn from_record(record: &TaskRecord) -> AppResult<Self> {
        Ok(Self {
            id: record.id.clone(),
            title: record.title.clone(),
            description: record.description.clone(),
            status: record.status.clone(),
            priority: record.priority.clone(),
            planned_start_at: record.planned_start_at.clone(),
            start_at: record.start_at.clone(),
            due_at: record.due_at.clone(),
            completed_at: record.completed_at.clone(),
            estimated_minutes: record.estimated_minutes,
            estimated_hours: record.estimated_hours,
            tags: serialize_vec(&record.tags)?,
            owner_id: record.owner_id.clone(),
            task_type: record.task_type.clone(),
            is_recurring: record.is_recurring,
            recurrence_rule: record.recurrence.as_ref().map(|r| r.rule.clone()),
            recurrence_until: record.recurrence.as_ref().and_then(|r| r.until.clone()),
            ai_summary: record.ai.as_ref().and_then(|ai| ai.summary.clone()),
            ai_next_action: record.ai.as_ref().and_then(|ai| ai.next_action.clone()),
            ai_confidence: record.ai.as_ref().and_then(|ai| ai.confidence),
            ai_complexity_score: record.ai.as_ref().and_then(|ai| ai.complexity_score),
            ai_suggested_start_at: record
                .ai
                .as_ref()
                .and_then(|ai| ai.suggested_start_at.clone()),
            ai_focus_mode: serialize_struct(
                record.ai.as_ref().and_then(|ai| ai.focus_mode.as_ref()),
            )?,
            ai_efficiency_prediction: serialize_struct(
                record
                    .ai
                    .as_ref()
                    .and_then(|ai| ai.efficiency_prediction.as_ref()),
            )?,
            ai_cot_steps: serialize_struct(
                record.ai.as_ref().and_then(|ai| ai.cot_steps.as_ref()),
            )?,
            ai_cot_summary: record.ai.as_ref().and_then(|ai| ai.cot_summary.clone()),
            ai_metadata: serialize_json(record.ai.as_ref().and_then(|ai| ai.metadata.as_ref()))?,
            ai_source: serialize_ai_source(record.ai.as_ref().and_then(|ai| ai.source)),
            ai_generated_at: record.ai.as_ref().and_then(|ai| ai.generated_at.clone()),
            external_links: serialize_vec(&record.external_links)?,
            created_at: record.created_at.clone(),
            updated_at: record.updated_at.clone(),
        })
    }

    pub fn into_record(self) -> AppResult<TaskRecord> {
        let recurrence = self.recurrence_rule.map(|rule| TaskRecurrence {
            rule,
            until: self.recurrence_until,
        });

        let ai = if self.ai_summary.is_some()
            || self.ai_next_action.is_some()
            || self.ai_confidence.is_some()
            || self.ai_complexity_score.is_some()
            || self.ai_suggested_start_at.is_some()
            || self.ai_focus_mode.is_some()
            || self.ai_efficiency_prediction.is_some()
            || self.ai_cot_steps.is_some()
            || self.ai_cot_summary.is_some()
            || self.ai_metadata.is_some()
            || self.ai_source.is_some()
            || self.ai_generated_at.is_some()
        {
            Some(TaskAiInsights {
                summary: self.ai_summary,
                next_action: self.ai_next_action,
                confidence: self.ai_confidence,
                metadata: deserialize_json(self.ai_metadata)?,
                complexity_score: self.ai_complexity_score,
                suggested_start_at: self.ai_suggested_start_at,
                focus_mode: deserialize_struct::<TaskFocusModeRecommendation>(self.ai_focus_mode)?,
                efficiency_prediction: deserialize_struct::<TaskEfficiencyPrediction>(
                    self.ai_efficiency_prediction,
                )?,
                cot_steps: deserialize_struct::<Vec<TaskAiReasoningStep>>(self.ai_cot_steps)?,
                cot_summary: self.ai_cot_summary,
                source: deserialize_ai_source(self.ai_source)?,
                generated_at: self.ai_generated_at,
            })
        } else {
            None
        };

        Ok(TaskRecord {
            id: self.id,
            title: self.title,
            description: self.description,
            status: self.status,
            priority: self.priority,
            planned_start_at: self.planned_start_at,
            start_at: self.start_at,
            due_at: self.due_at,
            completed_at: self.completed_at,
            estimated_minutes: self.estimated_minutes,
            estimated_hours: self.estimated_hours,
            tags: deserialize_vec(self.tags)?,
            owner_id: self.owner_id,
            task_type: self.task_type,
            is_recurring: self.is_recurring,
            recurrence,
            ai,
            external_links: deserialize_vec(self.external_links)?,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

impl TryFrom<&Row<'_>> for TaskRow {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(TaskRow {
            id: row.get("id")?,
            title: row.get("title")?,
            description: row.get("description")?,
            status: row.get("status")?,
            priority: row.get("priority")?,
            planned_start_at: row.get("planned_start_at")?,
            start_at: row.get("start_at")?,
            due_at: row.get("due_at")?,
            completed_at: row.get("completed_at")?,
            estimated_minutes: row.get("estimated_minutes")?,
            estimated_hours: row.get("estimated_hours")?,
            tags: row.get("tags")?,
            owner_id: row.get("owner_id")?,
            task_type: row.get("task_type")?,
            is_recurring: row.get::<_, i64>("is_recurring")? != 0,
            recurrence_rule: row.get("recurrence_rule")?,
            recurrence_until: row.get("recurrence_until")?,
            ai_summary: row.get("ai_summary")?,
            ai_next_action: row.get("ai_next_action")?,
            ai_confidence: row.get("ai_confidence")?,
            ai_complexity_score: row.get("ai_complexity_score")?,
            ai_suggested_start_at: row.get("ai_suggested_start_at")?,
            ai_focus_mode: row.get("ai_focus_mode")?,
            ai_efficiency_prediction: row.get("ai_efficiency_prediction")?,
            ai_cot_steps: row.get("ai_cot_steps")?,
            ai_cot_summary: row.get("ai_cot_summary")?,
            ai_metadata: row.get("ai_metadata")?,
            ai_source: row.get("ai_source")?,
            ai_generated_at: row.get("ai_generated_at")?,
            external_links: row.get("external_links")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    }
}

pub struct TaskRepository;

impl TaskRepository {
    pub fn insert(conn: &Connection, row: &TaskRow) -> AppResult<()> {
        conn.execute(
            r#"
                INSERT INTO tasks (
                    id,
                    title,
                    description,
                    status,
                    priority,
                    planned_start_at,
                    start_at,
                    due_at,
                    completed_at,
                    estimated_minutes,
                    estimated_hours,
                    tags,
                    owner_id,
                    task_type,
                    is_recurring,
                    recurrence_rule,
                    recurrence_until,
                    ai_summary,
                    ai_next_action,
                    ai_confidence,
                    ai_complexity_score,
                    ai_suggested_start_at,
                    ai_focus_mode,
                    ai_efficiency_prediction,
                    ai_cot_steps,
                    ai_cot_summary,
                    ai_metadata,
                    ai_source,
                    ai_generated_at,
                    external_links,
                    created_at,
                    updated_at
                ) VALUES (
                    :id,
                    :title,
                    :description,
                    :status,
                    :priority,
                    :planned_start_at,
                    :start_at,
                    :due_at,
                    :completed_at,
                    :estimated_minutes,
                    :estimated_hours,
                    :tags,
                    :owner_id,
                    :task_type,
                    :is_recurring,
                    :recurrence_rule,
                    :recurrence_until,
                    :ai_summary,
                    :ai_next_action,
                    :ai_confidence,
                    :ai_complexity_score,
                    :ai_suggested_start_at,
                    :ai_focus_mode,
                    :ai_efficiency_prediction,
                    :ai_cot_steps,
                    :ai_cot_summary,
                    :ai_metadata,
                    :ai_source,
                    :ai_generated_at,
                    :external_links,
                    :created_at,
                    :updated_at
                )
            "#,
            named_params! {
                ":id": &row.id,
                ":title": &row.title,
                ":description": &row.description,
                ":status": &row.status,
                ":priority": &row.priority,
                ":planned_start_at": &row.planned_start_at,
                ":start_at": &row.start_at,
                ":due_at": &row.due_at,
                ":completed_at": &row.completed_at,
                ":estimated_minutes": &row.estimated_minutes,
                ":estimated_hours": &row.estimated_hours,
                ":tags": &row.tags,
                ":owner_id": &row.owner_id,
                ":task_type": &row.task_type,
                ":is_recurring": row.is_recurring as i64,
                ":recurrence_rule": &row.recurrence_rule,
                ":recurrence_until": &row.recurrence_until,
                ":ai_summary": &row.ai_summary,
                ":ai_next_action": &row.ai_next_action,
                ":ai_confidence": &row.ai_confidence,
                ":ai_complexity_score": &row.ai_complexity_score,
                ":ai_suggested_start_at": &row.ai_suggested_start_at,
                ":ai_focus_mode": &row.ai_focus_mode,
                ":ai_efficiency_prediction": &row.ai_efficiency_prediction,
                ":ai_cot_steps": &row.ai_cot_steps,
                ":ai_cot_summary": &row.ai_cot_summary,
                ":ai_metadata": &row.ai_metadata,
                ":ai_source": &row.ai_source,
                ":ai_generated_at": &row.ai_generated_at,
                ":external_links": &row.external_links,
                ":created_at": &row.created_at,
                ":updated_at": &row.updated_at,
            },
        )?;

        Ok(())
    }

    pub fn update(conn: &Connection, row: &TaskRow) -> AppResult<()> {
        let affected = conn.execute(
            r#"
                UPDATE tasks SET
                    title = :title,
                    description = :description,
                    status = :status,
                    priority = :priority,
                    planned_start_at = :planned_start_at,
                    start_at = :start_at,
                    due_at = :due_at,
                    completed_at = :completed_at,
                    estimated_minutes = :estimated_minutes,
                    estimated_hours = :estimated_hours,
                    tags = :tags,
                    owner_id = :owner_id,
                    task_type = :task_type,
                    is_recurring = :is_recurring,
                    recurrence_rule = :recurrence_rule,
                    recurrence_until = :recurrence_until,
                    ai_summary = :ai_summary,
                    ai_next_action = :ai_next_action,
                    ai_confidence = :ai_confidence,
                    ai_complexity_score = :ai_complexity_score,
                    ai_suggested_start_at = :ai_suggested_start_at,
                    ai_focus_mode = :ai_focus_mode,
                    ai_efficiency_prediction = :ai_efficiency_prediction,
                    ai_cot_steps = :ai_cot_steps,
                    ai_cot_summary = :ai_cot_summary,
                    ai_metadata = :ai_metadata,
                    ai_source = :ai_source,
                    ai_generated_at = :ai_generated_at,
                    external_links = :external_links,
                    updated_at = :updated_at
                WHERE id = :id
            "#,
            named_params! {
                ":id": &row.id,
                ":title": &row.title,
                ":description": &row.description,
                ":status": &row.status,
                ":priority": &row.priority,
                ":planned_start_at": &row.planned_start_at,
                ":start_at": &row.start_at,
                ":due_at": &row.due_at,
                ":completed_at": &row.completed_at,
                ":estimated_minutes": &row.estimated_minutes,
                ":estimated_hours": &row.estimated_hours,
                ":tags": &row.tags,
                ":owner_id": &row.owner_id,
                ":task_type": &row.task_type,
                ":is_recurring": row.is_recurring as i64,
                ":recurrence_rule": &row.recurrence_rule,
                ":recurrence_until": &row.recurrence_until,
                ":ai_summary": &row.ai_summary,
                ":ai_next_action": &row.ai_next_action,
                ":ai_confidence": &row.ai_confidence,
                ":ai_complexity_score": &row.ai_complexity_score,
                ":ai_suggested_start_at": &row.ai_suggested_start_at,
                ":ai_focus_mode": &row.ai_focus_mode,
                ":ai_efficiency_prediction": &row.ai_efficiency_prediction,
                ":ai_cot_steps": &row.ai_cot_steps,
                ":ai_cot_summary": &row.ai_cot_summary,
                ":ai_metadata": &row.ai_metadata,
                ":ai_source": &row.ai_source,
                ":ai_generated_at": &row.ai_generated_at,
                ":external_links": &row.external_links,
                ":updated_at": &row.updated_at,
            },
        )?;

        if affected == 0 {
            return Err(AppError::not_found());
        }

        Ok(())
    }

    pub fn delete(conn: &Connection, id: &str) -> AppResult<()> {
        let affected = conn.execute("DELETE FROM tasks WHERE id = ?1", [id])?;
        if affected == 0 {
            return Err(AppError::not_found());
        }
        Ok(())
    }

    pub fn find_by_id(conn: &Connection, id: &str) -> AppResult<Option<TaskRow>> {
        let mut stmt = conn.prepare(&format!("{} WHERE id = ?1", BASE_SELECT))?;
        let row = stmt
            .query_row([id], |row| TaskRow::try_from(row))
            .optional()?;
        Ok(row)
    }

    pub fn list_all(conn: &Connection) -> AppResult<Vec<TaskRow>> {
        let mut stmt = conn.prepare(&format!("{} ORDER BY created_at DESC", BASE_SELECT))?;
        let rows = stmt
            .query_map([], |row| TaskRow::try_from(row))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }
}

fn serialize_vec(values: &[String]) -> AppResult<Option<String>> {
    if values.is_empty() {
        Ok(None)
    } else {
        Ok(Some(serde_json::to_string(values)?))
    }
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
        None => Ok(None),
    }
}

fn deserialize_json(raw: Option<String>) -> AppResult<Option<JsonValue>> {
    match raw {
        Some(value) if !value.is_empty() => Ok(Some(serde_json::from_str(&value)?)),
        _ => Ok(None),
    }
}

fn serialize_struct<T: Serialize>(value: Option<&T>) -> AppResult<Option<String>> {
    match value {
        Some(data) => Ok(Some(serde_json::to_string(data)?)),
        None => Ok(None),
    }
}

fn deserialize_struct<T: DeserializeOwned>(raw: Option<String>) -> AppResult<Option<T>> {
    match raw {
        Some(value) if !value.is_empty() => Ok(Some(serde_json::from_str(&value)?)),
        _ => Ok(None),
    }
}

fn serialize_ai_source(value: Option<TaskAiSource>) -> Option<String> {
    value.map(|source| match source {
        TaskAiSource::Live => "live".to_string(),
        TaskAiSource::Cache => "cache".to_string(),
    })
}

fn deserialize_ai_source(raw: Option<String>) -> AppResult<Option<TaskAiSource>> {
    let source = match raw.as_deref() {
        Some("live") => Some(TaskAiSource::Live),
        Some("cache") => Some(TaskAiSource::Cache),
        Some(_) => None,
        None => None,
    };
    Ok(source)
}
