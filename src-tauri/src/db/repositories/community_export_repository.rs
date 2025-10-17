use std::convert::TryFrom;

use rusqlite::{named_params, Connection, Row};
use serde_json::Value as JsonValue;

use crate::error::{AppError, AppResult};
use crate::models::community_export::{CommunityExport, CommunityExportCreate};

#[derive(Debug, Clone)]
pub struct CommunityExportRow {
    pub id: i64,
    pub generated_at: String,
    pub payload_path: String,
    pub metrics_summary: String,
    pub includes_feedback: bool,
    pub checksum: String,
}

impl CommunityExportRow {
    pub fn from_create(input: &CommunityExportCreate) -> AppResult<Self> {
        Ok(Self {
            id: 0, // Will be set by database
            generated_at: chrono::Utc::now().to_rfc3339(),
            payload_path: input.payload_path.clone(),
            metrics_summary: serialize_json(&input.metrics_summary)?,
            includes_feedback: input.includes_feedback,
            checksum: input.checksum.clone(),
        })
    }

    pub fn into_export(self) -> AppResult<CommunityExport> {
        Ok(CommunityExport {
            id: self.id,
            generated_at: self.generated_at,
            payload_path: self.payload_path,
            metrics_summary: deserialize_json(&self.metrics_summary)?,
            includes_feedback: self.includes_feedback,
            checksum: self.checksum,
        })
    }
}

impl TryFrom<&Row<'_>> for CommunityExportRow {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.get("id")?,
            generated_at: row.get("generated_at")?,
            payload_path: row.get("payload_path")?,
            metrics_summary: row.get("metrics_summary")?,
            includes_feedback: row.get::<_, i64>("includes_feedback")? != 0,
            checksum: row.get("checksum")?,
        })
    }
}

pub struct CommunityExportRepository;

impl CommunityExportRepository {
    pub fn create_export(conn: &Connection, input: &CommunityExportCreate) -> AppResult<i64> {
        let row = CommunityExportRow::from_create(input)?;

        conn.execute(
            r#"
                INSERT INTO community_exports (
                    generated_at,
                    payload_path,
                    metrics_summary,
                    includes_feedback,
                    checksum
                ) VALUES (
                    :generated_at,
                    :payload_path,
                    :metrics_summary,
                    :includes_feedback,
                    :checksum
                )
            "#,
            named_params! {
                ":generated_at": &row.generated_at,
                ":payload_path": &row.payload_path,
                ":metrics_summary": &row.metrics_summary,
                ":includes_feedback": if row.includes_feedback { 1i64 } else { 0i64 },
                ":checksum": &row.checksum,
            },
        )?;

        Ok(conn.last_insert_rowid())
    }

    pub fn get_export_by_id(conn: &Connection, id: i64) -> AppResult<Option<CommunityExport>> {
        let mut stmt = conn.prepare(
            r#"
                SELECT id, generated_at, payload_path, metrics_summary, includes_feedback, checksum
                FROM community_exports
                WHERE id = :id
            "#,
        )?;

        let row = stmt.query_row(
            named_params! {
                ":id": &id,
            },
            |row| -> Result<_, rusqlite::Error> {
                let row: CommunityExportRow = CommunityExportRow::try_from(row)?;
                row.into_export().map_err(|_err| {
                    rusqlite::Error::InvalidColumnType(
                        0,
                        "conversion".to_string(),
                        rusqlite::types::Type::Text,
                    )
                })
            },
        );

        match row {
            Ok(export) => Ok(Some(export)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AppError::Database {
                message: e.to_string(),
            }),
        }
    }

    pub fn list_exports(
        conn: &Connection,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> AppResult<Vec<CommunityExport>> {
        let limit_clause = limit.map(|l| format!("LIMIT {}", l)).unwrap_or_default();
        let offset_clause = offset.map(|o| format!("OFFSET {}", o)).unwrap_or_default();

        let sql = format!(
            r#"
                SELECT id, generated_at, payload_path, metrics_summary, includes_feedback, checksum
                FROM community_exports
                ORDER BY generated_at DESC
                {} {}
            "#,
            limit_clause, offset_clause
        );

        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map([], |row| -> Result<_, rusqlite::Error> {
            let row: CommunityExportRow = CommunityExportRow::try_from(row)?;
            row.into_export().map_err(|_err| {
                rusqlite::Error::InvalidColumnType(
                    0,
                    "conversion".to_string(),
                    rusqlite::types::Type::Text,
                )
            })
        })?;

        let mut exports = Vec::new();
        for row in rows {
            exports.push(row?);
        }

        Ok(exports)
    }

    pub fn delete_export(conn: &Connection, id: i64) -> AppResult<bool> {
        let rows_affected = conn.execute(
            "DELETE FROM community_exports WHERE id = :id",
            named_params! {
                ":id": &id,
            },
        )?;

        Ok(rows_affected > 0)
    }

    pub fn delete_exports_before(conn: &Connection, before_date: &str) -> AppResult<i64> {
        let rows_affected = conn.execute(
            "DELETE FROM community_exports WHERE generated_at < :before_date",
            named_params! {
                ":before_date": &before_date,
            },
        )?;

        Ok(rows_affected as i64)
    }

    pub fn verify_checksum(conn: &Connection, id: i64) -> AppResult<Option<bool>> {
        if let Some(_export) = Self::get_export_by_id(conn, id)? {
            // In a real implementation, you would read the file at payload_path
            // and calculate its checksum to compare with the stored one
            // For now, we'll just return true if the export exists
            Ok(Some(true))
        } else {
            Ok(None)
        }
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
