use std::convert::TryFrom;

use rusqlite::{named_params, Connection, OptionalExtension, Row};

use crate::error::AppResult;

#[derive(Debug, Clone)]
pub struct AppSettingRow {
    pub key: String,
    pub value: String,
    pub updated_at: String,
}

impl TryFrom<&Row<'_>> for AppSettingRow {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            key: row.get("key")?,
            value: row.get("value")?,
            updated_at: row.get("updated_at")?,
        })
    }
}

pub struct SettingsRepository;

impl SettingsRepository {
    pub fn get(conn: &Connection, key: &str) -> AppResult<Option<AppSettingRow>> {
        let mut stmt =
            conn.prepare("SELECT key, value, updated_at FROM app_settings WHERE key = ?1")?;

        let row = stmt
            .query_row([key], |row| AppSettingRow::try_from(row))
            .optional()?;

        Ok(row)
    }

    pub fn list(conn: &Connection) -> AppResult<Vec<AppSettingRow>> {
        let mut stmt =
            conn.prepare("SELECT key, value, updated_at FROM app_settings ORDER BY key ASC")?;

        let rows = stmt
            .query_map([], |row| AppSettingRow::try_from(row))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(rows)
    }

    pub fn upsert(conn: &Connection, key: &str, value: &str) -> AppResult<()> {
        conn.execute(
            r#"
                INSERT INTO app_settings (key, value)
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
        conn.execute("DELETE FROM app_settings WHERE key = ?1", [key])?;
        Ok(())
    }
}
