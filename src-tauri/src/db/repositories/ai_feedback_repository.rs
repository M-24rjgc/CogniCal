use std::convert::TryFrom;

use rusqlite::{named_params, Connection, Row};
use serde_json::Value as JsonValue;

use crate::error::{AppError, AppResult};
use crate::models::ai_feedback::{AiFeedback, AiFeedbackCreate};

#[derive(Debug, Clone)]
pub struct AiFeedbackRow {
    pub id: i64,
    pub surface: String,
    pub session_id: Option<String>,
    pub sentiment: String,
    pub note: Option<String>,
    pub prompt_snapshot: String,
    pub context_snapshot: String,
    pub created_at: String,
    pub anonymized: bool,
}

impl AiFeedbackRow {
    pub fn from_create(input: &AiFeedbackCreate) -> AppResult<Self> {
        Ok(Self {
            id: 0, // Will be set by database
            surface: input.surface.to_string(),
            session_id: input.session_id.clone(),
            sentiment: input.sentiment.to_string(),
            note: input.note.clone(),
            prompt_snapshot: input.prompt_snapshot.clone(),
            context_snapshot: serialize_json(&input.context_snapshot)?,
            created_at: chrono::Utc::now().to_rfc3339(),
            anonymized: input.anonymized,
        })
    }

    pub fn into_feedback(self) -> AppResult<AiFeedback> {
        Ok(AiFeedback {
            id: self.id,
            surface: self.surface.parse().map_err(|e| AppError::Database {
                message: format!("Invalid surface: {}", e),
            })?,
            session_id: self.session_id,
            sentiment: self.sentiment.parse().map_err(|e| AppError::Database {
                message: format!("Invalid sentiment: {}", e),
            })?,
            note: self.note,
            prompt_snapshot: self.prompt_snapshot,
            context_snapshot: deserialize_json(&self.context_snapshot)?,
            created_at: self.created_at,
            anonymized: self.anonymized,
        })
    }
}

impl TryFrom<&Row<'_>> for AiFeedbackRow {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.get("id")?,
            surface: row.get("surface")?,
            session_id: row.get("session_id")?,
            sentiment: row.get("sentiment")?,
            note: row.get("note")?,
            prompt_snapshot: row.get("prompt_snapshot")?,
            context_snapshot: row.get("context_snapshot")?,
            created_at: row.get("created_at")?,
            anonymized: row.get::<_, i64>("anonymized")? != 0,
        })
    }
}

pub struct AiFeedbackRepository;

impl AiFeedbackRepository {
    pub fn create_feedback(conn: &Connection, input: &AiFeedbackCreate) -> AppResult<i64> {
        let row = AiFeedbackRow::from_create(input)?;

        conn.execute(
            r#"
                INSERT INTO ai_feedback (
                    surface,
                    session_id,
                    sentiment,
                    note,
                    prompt_snapshot,
                    context_snapshot,
                    anonymized
                ) VALUES (
                    :surface,
                    :session_id,
                    :sentiment,
                    :note,
                    :prompt_snapshot,
                    :context_snapshot,
                    :anonymized
                )
            "#,
            named_params! {
                ":surface": &row.surface,
                ":session_id": &row.session_id,
                ":sentiment": &row.sentiment,
                ":note": &row.note,
                ":prompt_snapshot": &row.prompt_snapshot,
                ":context_snapshot": &row.context_snapshot,
                ":anonymized": if row.anonymized { 1i64 } else { 0i64 },
            },
        )?;

        Ok(conn.last_insert_rowid())
    }

    pub fn get_feedback_by_surface(
        conn: &Connection,
        surface: crate::models::ai_feedback::AiFeedbackSurface,
        limit: Option<i64>,
    ) -> AppResult<Vec<AiFeedback>> {
        let surface_str = surface.to_string();
        let limit_clause = limit.map(|l| format!("LIMIT {}", l)).unwrap_or_default();

        let sql = format!(
            r#"
                SELECT id, surface, session_id, sentiment, note, prompt_snapshot, context_snapshot, created_at, anonymized
                FROM ai_feedback
                WHERE surface = :surface
                ORDER BY created_at DESC
                {}
            "#,
            limit_clause
        );

        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(
            named_params! {
                ":surface": &surface_str,
            },
            |row| -> Result<_, rusqlite::Error> {
                let row: AiFeedbackRow = AiFeedbackRow::try_from(row)?;
                row.into_feedback().map_err(|_err| {
                    rusqlite::Error::InvalidColumnType(
                        0,
                        "conversion".to_string(),
                        rusqlite::types::Type::Text,
                    )
                })
            },
        )?;

        let mut feedback_list = Vec::new();
        for row in rows {
            feedback_list.push(row?);
        }

        Ok(feedback_list)
    }

    pub fn get_feedback_by_session(
        conn: &Connection,
        session_id: &str,
    ) -> AppResult<Vec<AiFeedback>> {
        let mut stmt = conn.prepare(
            r#"
                SELECT id, surface, session_id, sentiment, note, prompt_snapshot, context_snapshot, created_at, anonymized
                FROM ai_feedback
                WHERE session_id = :session_id
                ORDER BY created_at DESC
            "#,
        )?;

        let rows = stmt.query_map(
            named_params! {
                ":session_id": &session_id,
            },
            |row| -> Result<_, rusqlite::Error> {
                let row: AiFeedbackRow = AiFeedbackRow::try_from(row)?;
                row.into_feedback().map_err(|_err| {
                    rusqlite::Error::InvalidColumnType(
                        0,
                        "conversion".to_string(),
                        rusqlite::types::Type::Text,
                    )
                })
            },
        )?;

        let mut feedback_list = Vec::new();
        for row in rows {
            feedback_list.push(row?);
        }

        Ok(feedback_list)
    }

    pub fn delete_feedback_before(conn: &Connection, before_date: &str) -> AppResult<i64> {
        let rows_affected = conn.execute(
            "DELETE FROM ai_feedback WHERE created_at < :before_date",
            named_params! {
                ":before_date": &before_date,
            },
        )?;

        Ok(rows_affected as i64)
    }

    pub fn get_feedback_stats(
        conn: &Connection,
        surface: Option<crate::models::ai_feedback::AiFeedbackSurface>,
    ) -> AppResult<JsonValue> {
        let surface_filter = surface
            .map(|s| format!("AND surface = '{}'", s.to_string()))
            .unwrap_or_default();

        let sql = format!(
            r#"
                SELECT 
                    surface,
                    sentiment,
                    COUNT(*) as count,
                    AVG(CASE WHEN anonymized = 0 THEN 1.0 ELSE 0.0 END) as non_anonymized_ratio
                FROM ai_feedback
                WHERE 1=1 {}
                GROUP BY surface, sentiment
                ORDER BY surface, sentiment
            "#,
            surface_filter
        );

        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map([], |row| {
            Ok(serde_json::json!({
                "surface": row.get::<_, String>("surface")?,
                "sentiment": row.get::<_, String>("sentiment")?,
                "count": row.get::<_, i64>("count")?,
                "nonAnonymizedRatio": row.get::<_, f64>("non_anonymized_ratio")?,
            }))
        })?;

        let mut stats = Vec::new();
        for row in rows {
            stats.push(row?);
        }

        Ok(serde_json::json!({
            "bySurfaceAndSentiment": stats,
            "totalFeedback": stats.iter().map(|s| s["count"].as_i64().unwrap_or(0)).sum::<i64>()
        }))
    }
}

// Helper functions for JSON serialization/deserialization
fn serialize_json(value: &JsonValue) -> AppResult<String> {
    serde_json::to_string(value).map_err(|e| AppError::Database {
        message: format!("JSON serialization error: {}", e),
    })
}

fn deserialize_json(s: &str) -> AppResult<JsonValue> {
    serde_json::from_str(s).map_err(|e| AppError::Database {
        message: format!("JSON deserialization error: {}", e),
    })
}
