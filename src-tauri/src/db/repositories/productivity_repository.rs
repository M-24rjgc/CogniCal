use std::convert::TryFrom;

use chrono::NaiveDate;
use rusqlite::{named_params, Connection, OptionalExtension, Row};
use serde_json::Value as JsonValue;

use crate::error::{AppError, AppResult};
use crate::models::productivity::{ProductivityScoreRecord, ProductivityScoreUpsert};

#[derive(Debug, Clone)]
pub struct ProductivityScoreRow {
    pub snapshot_date: String,
    pub composite_score: f64,
    pub dimension_scores: String,
    pub weight_breakdown: String,
    pub explanation: Option<String>,
    pub created_at: String,
}

impl ProductivityScoreRow {
    pub fn from_upsert(input: &ProductivityScoreUpsert, created_at: String) -> AppResult<Self> {
        Ok(Self {
            snapshot_date: input.snapshot_date.clone(),
            composite_score: input.composite_score,
            dimension_scores: serialize_json(&input.dimension_scores)?,
            weight_breakdown: serialize_json(&input.weight_breakdown)?,
            explanation: input
                .explanation
                .as_ref()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
            created_at,
        })
    }

    pub fn into_record(self) -> AppResult<ProductivityScoreRecord> {
        Ok(ProductivityScoreRecord {
            snapshot_date: self.snapshot_date,
            composite_score: self.composite_score,
            dimension_scores: deserialize_json(&self.dimension_scores)?,
            weight_breakdown: deserialize_json(&self.weight_breakdown)?,
            explanation: self.explanation,
            created_at: self.created_at,
        })
    }
}

impl TryFrom<&Row<'_>> for ProductivityScoreRow {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            snapshot_date: row.get("snapshot_date")?,
            composite_score: row.get("composite_score")?,
            dimension_scores: row.get("dimension_scores")?,
            weight_breakdown: row.get("weight_breakdown")?,
            explanation: row.get("explanation")?,
            created_at: row.get("created_at")?,
        })
    }
}

pub struct ProductivityRepository;

impl ProductivityRepository {
    pub fn upsert_score(conn: &Connection, input: &ProductivityScoreUpsert) -> AppResult<()> {
        let created_at = chrono::Utc::now().to_rfc3339();
        let row = ProductivityScoreRow::from_upsert(input, created_at.clone())?;

        conn.execute(
            r#"
                INSERT INTO productivity_scores (
                    snapshot_date,
                    composite_score,
                    dimension_scores,
                    weight_breakdown,
                    explanation,
                    created_at
                ) VALUES (
                    :snapshot_date,
                    :composite_score,
                    :dimension_scores,
                    :weight_breakdown,
                    :explanation,
                    :created_at
                )
                ON CONFLICT(snapshot_date) DO UPDATE SET
                    composite_score = excluded.composite_score,
                    dimension_scores = excluded.dimension_scores,
                    weight_breakdown = excluded.weight_breakdown,
                    explanation = excluded.explanation,
                    created_at = excluded.created_at
            "#,
            named_params! {
                ":snapshot_date": &row.snapshot_date,
                ":composite_score": &row.composite_score,
                ":dimension_scores": &row.dimension_scores,
                ":weight_breakdown": &row.weight_breakdown,
                ":explanation": &row.explanation,
                ":created_at": &row.created_at,
            },
        )?;

        Ok(())
    }

    pub fn find_by_date(
        conn: &Connection,
        snapshot_date: &NaiveDate,
    ) -> AppResult<Option<ProductivityScoreRecord>> {
        let mut stmt = conn.prepare(
            r#"
                SELECT
                    snapshot_date,
                    composite_score,
                    dimension_scores,
                    weight_breakdown,
                    explanation,
                    created_at
                FROM productivity_scores
                WHERE snapshot_date = :snapshot_date
            "#,
        )?;

        let row = stmt
            .query_row(
                named_params! {":snapshot_date": snapshot_date.to_string()},
                |row| ProductivityScoreRow::try_from(row),
            )
            .optional()?;

        row.map(|row| row.into_record()).transpose()
    }

    pub fn list_recent(conn: &Connection, limit: usize) -> AppResult<Vec<ProductivityScoreRecord>> {
        let mut stmt = conn.prepare(
            r#"
                SELECT
                    snapshot_date,
                    composite_score,
                    dimension_scores,
                    weight_breakdown,
                    explanation,
                    created_at
                FROM productivity_scores
                ORDER BY snapshot_date DESC
                LIMIT :limit
            "#,
        )?;

        let rows = stmt
            .query_map(named_params! {":limit": limit as i64}, |row| {
                ProductivityScoreRow::try_from(row)
            })?
            .map(|row| {
                row.map_err(AppError::from)
                    .and_then(|row| row.into_record())
            })
            .collect::<AppResult<Vec<_>>>()?;

        Ok(rows)
    }

    pub fn delete_before(conn: &Connection, cutoff: &NaiveDate) -> AppResult<usize> {
        let deleted = conn.execute(
            r#"
                DELETE FROM productivity_scores
                WHERE snapshot_date < :cutoff
            "#,
            named_params! {":cutoff": cutoff.to_string()},
        )?;

        Ok(deleted as usize)
    }
}

fn serialize_json(value: &JsonValue) -> AppResult<String> {
    serde_json::to_string(value).map_err(AppError::from)
}

fn deserialize_json(raw: &str) -> AppResult<JsonValue> {
    serde_json::from_str(raw).map_err(AppError::from)
}
