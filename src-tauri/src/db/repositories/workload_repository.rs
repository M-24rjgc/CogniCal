use std::convert::TryFrom;

use chrono::{DateTime, Utc};
use rusqlite::{named_params, Connection, OptionalExtension, Row};

use crate::error::{AppError, AppResult};
use crate::models::workload::{WorkloadForecastRecord, WorkloadHorizon, WorkloadRiskLevel};

#[derive(Debug, Clone)]
pub struct WorkloadForecastRow {
    pub horizon: String,
    pub generated_at: String,
    pub risk_level: String,
    pub total_hours: f64,
    pub capacity_threshold: f64,
    pub contributing_tasks: String,
    pub confidence: f64,
}

impl WorkloadForecastRow {
    pub fn from_record(record: &WorkloadForecastRecord) -> AppResult<Self> {
        Ok(Self {
            horizon: record.horizon.as_str().to_string(),
            generated_at: validate_datetime(&record.generated_at)?,
            risk_level: record.risk_level.as_str().to_string(),
            total_hours: record.total_hours,
            capacity_threshold: record.capacity_threshold,
            contributing_tasks: serialize_json(&record.contributing_tasks)?,
            confidence: record.confidence,
        })
    }

    pub fn into_record(self) -> AppResult<WorkloadForecastRecord> {
        Ok(WorkloadForecastRecord {
            horizon: WorkloadHorizon::try_from(self.horizon.as_str())
                .map_err(AppError::validation)?,
            generated_at: self.generated_at,
            risk_level: WorkloadRiskLevel::try_from(self.risk_level.as_str())
                .map_err(AppError::validation)?,
            total_hours: self.total_hours,
            capacity_threshold: self.capacity_threshold,
            contributing_tasks: deserialize_json(&self.contributing_tasks)?,
            confidence: self.confidence,
        })
    }
}

impl TryFrom<&Row<'_>> for WorkloadForecastRow {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            horizon: row.get("horizon")?,
            generated_at: row.get("generated_at")?,
            risk_level: row.get("risk_level")?,
            total_hours: row.get("total_hours")?,
            capacity_threshold: row.get("capacity_threshold")?,
            contributing_tasks: row.get("contributing_tasks")?,
            confidence: row.get("confidence")?,
        })
    }
}

pub struct WorkloadRepository;

impl WorkloadRepository {
    pub fn upsert_forecast(conn: &Connection, record: &WorkloadForecastRecord) -> AppResult<()> {
        let row = WorkloadForecastRow::from_record(record)?;

        conn.execute(
            r#"
                INSERT INTO workload_forecasts (
                    horizon,
                    generated_at,
                    risk_level,
                    total_hours,
                    capacity_threshold,
                    contributing_tasks,
                    confidence
                ) VALUES (
                    :horizon,
                    :generated_at,
                    :risk_level,
                    :total_hours,
                    :capacity_threshold,
                    :contributing_tasks,
                    :confidence
                )
                ON CONFLICT(horizon, generated_at) DO UPDATE SET
                    risk_level = excluded.risk_level,
                    total_hours = excluded.total_hours,
                    capacity_threshold = excluded.capacity_threshold,
                    contributing_tasks = excluded.contributing_tasks,
                    confidence = excluded.confidence
            "#,
            named_params! {
                ":horizon": &row.horizon,
                ":generated_at": &row.generated_at,
                ":risk_level": &row.risk_level,
                ":total_hours": &row.total_hours,
                ":capacity_threshold": &row.capacity_threshold,
                ":contributing_tasks": &row.contributing_tasks,
                ":confidence": &row.confidence,
            },
        )?;

        Ok(())
    }

    pub fn latest_for_horizon(
        conn: &Connection,
        horizon: WorkloadHorizon,
    ) -> AppResult<Option<WorkloadForecastRecord>> {
        let mut stmt = conn.prepare(
            r#"
                SELECT
                    horizon,
                    generated_at,
                    risk_level,
                    total_hours,
                    capacity_threshold,
                    contributing_tasks,
                    confidence
                FROM workload_forecasts
                WHERE horizon = :horizon
                ORDER BY generated_at DESC
                LIMIT 1
            "#,
        )?;

        let row = stmt
            .query_row(named_params! {":horizon": horizon.as_str()}, |row| {
                WorkloadForecastRow::try_from(row)
            })
            .optional()?;

        row.map(|row| row.into_record()).transpose()
    }

    pub fn list_for_horizon(
        conn: &Connection,
        horizon: WorkloadHorizon,
        limit: usize,
    ) -> AppResult<Vec<WorkloadForecastRecord>> {
        let mut stmt = conn.prepare(
            r#"
                SELECT
                    horizon,
                    generated_at,
                    risk_level,
                    total_hours,
                    capacity_threshold,
                    contributing_tasks,
                    confidence
                FROM workload_forecasts
                WHERE horizon = :horizon
                ORDER BY generated_at DESC
                LIMIT :limit
            "#,
        )?;

        let rows = stmt
            .query_map(
                named_params! {":horizon": horizon.as_str(), ":limit": limit as i64},
                |row| WorkloadForecastRow::try_from(row),
            )?
            .map(|row| {
                row.map_err(AppError::from)
                    .and_then(|row| row.into_record())
            })
            .collect::<AppResult<Vec<_>>>()?;

        Ok(rows)
    }

    pub fn delete_before(
        conn: &Connection,
        horizon: WorkloadHorizon,
        cutoff: &DateTime<Utc>,
    ) -> AppResult<usize> {
        let deleted = conn.execute(
            r#"
                DELETE FROM workload_forecasts
                WHERE horizon = :horizon AND generated_at < :cutoff
            "#,
            named_params! {
                ":horizon": horizon.as_str(),
                ":cutoff": cutoff.to_rfc3339(),
            },
        )?;

        Ok(deleted as usize)
    }
}

fn serialize_json<T: serde::Serialize>(value: &T) -> AppResult<String> {
    serde_json::to_string(value).map_err(AppError::from)
}

fn deserialize_json<T: serde::de::DeserializeOwned>(raw: &str) -> AppResult<T> {
    serde_json::from_str(raw).map_err(AppError::from)
}

fn validate_datetime(value: &str) -> AppResult<String> {
    DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.to_rfc3339())
        .map_err(|_| AppError::validation("时间格式非法"))
}
