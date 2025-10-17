use std::convert::TryFrom;

use rusqlite::{named_params, Connection, OptionalExtension, Row};

use crate::error::AppResult;

#[derive(Debug, Clone)]
pub struct AiSettingRow {
    pub key: String,
    pub value: String,
    pub updated_at: String,
}

impl TryFrom<&Row<'_>> for AiSettingRow {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            key: row.get("key")?,
            value: row.get("value")?,
            updated_at: row.get("updated_at")?,
        })
    }
}

pub struct AiSettingsRepository;

impl AiSettingsRepository {
    pub fn get(conn: &Connection, key: &str) -> AppResult<Option<AiSettingRow>> {
        let mut stmt =
            conn.prepare("SELECT key, value, updated_at FROM ai_settings WHERE key = ?1")?;

        let row = stmt
            .query_row([key], |row| AiSettingRow::try_from(row))
            .optional()?;

        Ok(row)
    }

    pub fn upsert(conn: &Connection, key: &str, value: &str) -> AppResult<()> {
        conn.execute(
            r#"
                INSERT INTO ai_settings (key, value)
                VALUES (:key, :value)
                ON CONFLICT(key) DO UPDATE SET
                    value = excluded.value,
                    updated_at = CURRENT_TIMESTAMP
            "#,
            named_params! {":key": key, ":value": value},
        )?;

        Ok(())
    }

    pub fn delete(conn: &Connection, key: &str) -> AppResult<()> {
        conn.execute("DELETE FROM ai_settings WHERE key = ?1", [key])?;
        Ok(())
    }
}
