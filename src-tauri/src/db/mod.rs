use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use rusqlite::Connection;
use tracing::{debug, info};

use crate::error::AppResult;

pub mod migrations;

pub mod repositories;

const SCHEMA_SQL: &str = include_str!("schema.sql");

#[derive(Clone, Debug)]
pub struct DbPool {
    path: PathBuf,
}

impl DbPool {
    pub fn new<P: Into<PathBuf>>(path: P) -> AppResult<Self> {
        let path = path.into();
        info!(db_path = %path.display(), "initializing database pool");
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        let pool = Self { path };
        {
            pool.get_connection()?;
        }

        Ok(pool)
    }

    pub fn get_connection(&self) -> AppResult<Connection> {
        let mut conn = Connection::open(&self.path)?;
        configure_connection(&mut conn)?;
        conn.execute_batch(SCHEMA_SQL)?;
        migrations::run(&conn)?;
        debug!(db_path = %self.path.display(), "database connection ready");
        Ok(conn)
    }

    pub fn with_connection<F, T>(&self, callback: F) -> AppResult<T>
    where
        F: FnOnce(&Connection) -> AppResult<T>,
    {
        let conn = self.get_connection()?;
        callback(&conn)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

fn configure_connection(conn: &mut Connection) -> AppResult<()> {
    conn.busy_timeout(Duration::from_secs(5))?;
    conn.pragma_update(None, "foreign_keys", &1)?;
    conn.pragma_update(None, "journal_mode", &"WAL")?;
    Ok(())
}
