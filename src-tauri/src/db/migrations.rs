use rusqlite::{Connection, Row};
use tracing::info;

use crate::error::AppResult;
use crate::models::settings::DashboardConfig;

const USER_VERSION: i32 = 7;
const KEY_DASHBOARD_CONFIG: &str = "dashboard_config";

pub fn run(conn: &Connection) -> AppResult<()> {
    let mut current_version: i32 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;

    if current_version < 1 {
        info!(target: "app::db", version = current_version, "running migration v1");
        migrate_to_v1(conn)?;
        current_version = 1;
        conn.execute(&format!("PRAGMA user_version = {}", current_version), [])?;
    }

    if current_version < 2 {
        info!(target: "app::db", version = current_version, "running migration v2");
        migrate_to_v2(conn)?;
        current_version = 2;
        conn.execute(&format!("PRAGMA user_version = {}", current_version), [])?;
    }

    if current_version < 3 {
        info!(target: "app::db", version = current_version, "running migration v3");
        migrate_to_v3(conn)?;
        current_version = 3;
        conn.execute(&format!("PRAGMA user_version = {}", current_version), [])?;
    }

    if current_version < 4 {
        info!(target: "app::db", version = current_version, "running migration v4");
        migrate_to_v4(conn)?;
        current_version = 4;
        conn.execute(&format!("PRAGMA user_version = {}", current_version), [])?;
    }

    if current_version < 5 {
        info!(target: "app::db", version = current_version, "running migration v5");
        migrate_to_v5(conn)?;
        current_version = 5;
        conn.execute(&format!("PRAGMA user_version = {}", current_version), [])?;
    }

    if current_version < 6 {
        info!(target: "app::db", version = current_version, "running migration v6");
        migrate_to_v6(conn)?;
        current_version = 6;
        conn.execute(&format!("PRAGMA user_version = {}", current_version), [])?;
    }

    if current_version < 7 {
        info!(target: "app::db", version = current_version, "running migration v7");
        migrate_to_v7(conn)?;
        current_version = 7;
        conn.execute(&format!("PRAGMA user_version = {}", current_version), [])?;
    }

    if current_version != USER_VERSION {
        conn.execute(&format!("PRAGMA user_version = {}", USER_VERSION), [])?;
    }

    Ok(())
}

fn migrate_to_v1(conn: &Connection) -> AppResult<()> {
    ensure_column(conn, "tasks", "planned_start_at", "TEXT")?;
    ensure_column(conn, "tasks", "estimated_hours", "REAL")?;
    ensure_column(conn, "tasks", "task_type", "TEXT")?;
    ensure_column(conn, "tasks", "ai_complexity_score", "REAL")?;
    ensure_column(conn, "tasks", "ai_suggested_start_at", "TEXT")?;
    ensure_column(conn, "tasks", "ai_focus_mode", "TEXT")?;
    ensure_column(conn, "tasks", "ai_efficiency_prediction", "TEXT")?;
    ensure_column(conn, "tasks", "ai_cot_steps", "TEXT")?;
    ensure_column(conn, "tasks", "ai_cot_summary", "TEXT")?;
    ensure_column(conn, "tasks", "ai_metadata", "TEXT")?;
    ensure_column(conn, "tasks", "ai_source", "TEXT")?;
    ensure_column(conn, "tasks", "ai_generated_at", "TEXT")?;

    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS ai_parse_cache (
            semantic_hash TEXT PRIMARY KEY,
            raw_input TEXT NOT NULL,
            output_json TEXT NOT NULL,
            created_at TEXT NOT NULL,
            expires_at TEXT NOT NULL,
            usage_count INTEGER NOT NULL DEFAULT 0
        );
        CREATE INDEX IF NOT EXISTS idx_ai_parse_cache_expires_at
            ON ai_parse_cache(expires_at);
        "#,
    )?;

    Ok(())
}

fn migrate_to_v2(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS planning_sessions (
            id TEXT PRIMARY KEY,
            task_ids TEXT NOT NULL,
            constraints TEXT,
            generated_at TEXT NOT NULL,
            status TEXT NOT NULL,
            selected_option_id TEXT,
            personalization_snapshot TEXT,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        CREATE INDEX IF NOT EXISTS idx_planning_sessions_status ON planning_sessions(status);
        CREATE INDEX IF NOT EXISTS idx_planning_sessions_generated_at ON planning_sessions(generated_at);

        CREATE TABLE IF NOT EXISTS planning_options (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            rank INTEGER NOT NULL,
            score REAL,
            summary TEXT,
            cot_steps TEXT,
            risk_notes TEXT,
            is_fallback INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (session_id) REFERENCES planning_sessions(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_planning_options_session_id ON planning_options(session_id);
        CREATE INDEX IF NOT EXISTS idx_planning_options_rank ON planning_options(rank);

        CREATE TABLE IF NOT EXISTS planning_time_blocks (
            id TEXT PRIMARY KEY,
            option_id TEXT NOT NULL,
            task_id TEXT NOT NULL,
            start_at TEXT NOT NULL,
            end_at TEXT NOT NULL,
            flexibility TEXT,
            confidence REAL,
            conflict_flags TEXT,
            applied_at TEXT,
            actual_start_at TEXT,
            actual_end_at TEXT,
            status TEXT NOT NULL DEFAULT 'planned',
            FOREIGN KEY (option_id) REFERENCES planning_options(id) ON DELETE CASCADE,
            FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_planning_time_blocks_option_id ON planning_time_blocks(option_id);
        CREATE INDEX IF NOT EXISTS idx_planning_time_blocks_task_id ON planning_time_blocks(task_id);
        CREATE INDEX IF NOT EXISTS idx_planning_time_blocks_status ON planning_time_blocks(status);

        CREATE TABLE IF NOT EXISTS schedule_preferences (
            id TEXT PRIMARY KEY,
            data TEXT NOT NULL,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        "#,
    )?;

    conn.execute(
        "INSERT OR IGNORE INTO schedule_preferences (id, data, updated_at) VALUES (?1, ?2, datetime('now'))",
        ("default", "{}"),
    )?;

    Ok(())
}

fn migrate_to_v3(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS analytics_snapshots (
            snapshot_date TEXT PRIMARY KEY,
            total_tasks_completed INTEGER NOT NULL,
            completion_rate REAL NOT NULL,
            overdue_tasks INTEGER NOT NULL,
            total_focus_minutes INTEGER NOT NULL,
            productivity_score REAL NOT NULL,
            efficiency_rating REAL NOT NULL,
            time_spent_work REAL NOT NULL,
            time_spent_study REAL NOT NULL,
            time_spent_life REAL NOT NULL,
            time_spent_other REAL NOT NULL,
            on_time_ratio REAL NOT NULL,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        CREATE INDEX IF NOT EXISTS idx_analytics_snapshots_created_at
            ON analytics_snapshots(created_at);

        CREATE TABLE IF NOT EXISTS app_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        CREATE INDEX IF NOT EXISTS idx_app_settings_updated_at
            ON app_settings(updated_at);
        "#,
    )?;

    Ok(())
}

fn migrate_to_v4(conn: &Connection) -> AppResult<()> {
    ensure_column(
        conn,
        "analytics_snapshots",
        "focus_consistency",
        "REAL NOT NULL DEFAULT 0",
    )?;
    ensure_column(
        conn,
        "analytics_snapshots",
        "rest_balance",
        "REAL NOT NULL DEFAULT 0",
    )?;
    ensure_column(
        conn,
        "analytics_snapshots",
        "capacity_risk",
        "REAL NOT NULL DEFAULT 0",
    )?;

    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS productivity_scores (
            snapshot_date TEXT PRIMARY KEY,
            composite_score REAL NOT NULL,
            dimension_scores TEXT NOT NULL,
            weight_breakdown TEXT NOT NULL,
            explanation TEXT,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS recommendation_sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            generated_at TEXT NOT NULL,
            context_hash TEXT NOT NULL,
            plans TEXT NOT NULL,
            source TEXT NOT NULL CHECK(source IN ('deepseek', 'cached', 'heuristic')),
            network_status TEXT NOT NULL CHECK(network_status IN ('online', 'offline')),
            expires_at TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_recommendation_sessions_generated_at
            ON recommendation_sessions(generated_at);
        CREATE INDEX IF NOT EXISTS idx_recommendation_sessions_context_hash
            ON recommendation_sessions(context_hash);

        CREATE TABLE IF NOT EXISTS recommendation_decisions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id INTEGER NOT NULL,
            user_action TEXT NOT NULL CHECK(user_action IN ('accepted', 'rejected', 'adjusted')),
            adjustment_payload TEXT,
            responded_at TEXT NOT NULL,
            preference_tags TEXT,
            FOREIGN KEY (session_id) REFERENCES recommendation_sessions(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_recommendation_decisions_session_id
            ON recommendation_decisions(session_id);

        CREATE TABLE IF NOT EXISTS workload_forecasts (
            horizon TEXT NOT NULL CHECK(horizon IN ('7d', '14d', '30d')),
            generated_at TEXT NOT NULL,
            risk_level TEXT NOT NULL CHECK(risk_level IN ('ok', 'warning', 'critical')),
            total_hours REAL NOT NULL,
            capacity_threshold REAL NOT NULL,
            contributing_tasks TEXT NOT NULL,
            confidence REAL NOT NULL,
            PRIMARY KEY (horizon, generated_at)
        );
        CREATE INDEX IF NOT EXISTS idx_workload_forecasts_generated_at
            ON workload_forecasts(generated_at);

        CREATE TABLE IF NOT EXISTS wellness_events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            window_start TEXT NOT NULL,
            trigger_reason TEXT NOT NULL CHECK(trigger_reason IN ('focus_streak', 'work_streak')),
            recommended_break_minutes INTEGER NOT NULL,
            suggested_micro_task TEXT,
            response TEXT CHECK(response IN ('completed', 'snoozed', 'ignored')),
            response_at TEXT,
            deferral_count INTEGER NOT NULL DEFAULT 0
        );
        CREATE INDEX IF NOT EXISTS idx_wellness_events_window_start
            ON wellness_events(window_start);

        CREATE TABLE IF NOT EXISTS ai_feedback (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            surface TEXT NOT NULL CHECK(surface IN ('score', 'recommendation', 'forecast')),
            session_id INTEGER,
            sentiment TEXT NOT NULL CHECK(sentiment IN ('up', 'down')),
            note TEXT,
            prompt_snapshot TEXT NOT NULL,
            context_snapshot TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            anonymized INTEGER NOT NULL DEFAULT 1
        );
        CREATE INDEX IF NOT EXISTS idx_ai_feedback_created_at
            ON ai_feedback(created_at);
        CREATE INDEX IF NOT EXISTS idx_ai_feedback_surface
            ON ai_feedback(surface);

        CREATE TABLE IF NOT EXISTS community_exports (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            generated_at TEXT NOT NULL,
            payload_path TEXT NOT NULL,
            metrics_summary TEXT NOT NULL,
            includes_feedback INTEGER NOT NULL DEFAULT 0,
            checksum TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_community_exports_generated_at
            ON community_exports(generated_at);
        "#,
    )?;

    Ok(())
}

fn migrate_to_v5(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS ai_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        CREATE INDEX IF NOT EXISTS idx_ai_settings_updated_at
            ON ai_settings(updated_at);

        CREATE TABLE IF NOT EXISTS ai_cache (
            cache_key TEXT PRIMARY KEY,
            operation TEXT NOT NULL CHECK(operation IN ('parse','recommend','schedule')),
            semantic_hash TEXT NOT NULL,
            raw_input TEXT NOT NULL,
            response_json TEXT NOT NULL,
            created_at TEXT NOT NULL,
            expires_at TEXT NOT NULL,
            hit_count INTEGER NOT NULL DEFAULT 0,
            metadata_json TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_ai_cache_operation
            ON ai_cache(operation);
        CREATE INDEX IF NOT EXISTS idx_ai_cache_semantic_hash
            ON ai_cache(semantic_hash);
        CREATE INDEX IF NOT EXISTS idx_ai_cache_expires_at
            ON ai_cache(expires_at);
        "#,
    )?;

    Ok(())
}

fn migrate_to_v6(conn: &Connection) -> AppResult<()> {
    let default_value = serde_json::to_string(&DashboardConfig::default())?;
    conn.execute(
        r#"
        INSERT INTO app_settings (key, value, updated_at)
        VALUES (?1, ?2, CURRENT_TIMESTAMP)
        ON CONFLICT(key) DO NOTHING
        "#,
        (KEY_DASHBOARD_CONFIG, default_value.as_str()),
    )?;

    Ok(())
}

fn migrate_to_v7(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS conversations (
            id TEXT PRIMARY KEY,
            user_id TEXT,
            started_at TEXT NOT NULL,
            last_message_at TEXT NOT NULL,
            message_count INTEGER DEFAULT 0,
            archived BOOLEAN DEFAULT FALSE
        );
        CREATE INDEX IF NOT EXISTS idx_conversations_user_id
            ON conversations(user_id);
        CREATE INDEX IF NOT EXISTS idx_conversations_last_message_at
            ON conversations(last_message_at);

        CREATE TABLE IF NOT EXISTS memory_config (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        "#,
    )?;

    Ok(())
}

fn ensure_column(conn: &Connection, table: &str, column: &str, definition: &str) -> AppResult<()> {
    if !column_exists(conn, table, column)? {
        let sql = format!("ALTER TABLE {table} ADD COLUMN {column} {definition};");
        conn.execute(&sql, [])?;
    }
    Ok(())
}

fn column_exists(conn: &Connection, table: &str, column: &str) -> AppResult<bool> {
    let pragma = format!("PRAGMA table_info({table})");
    let mut stmt = conn.prepare(&pragma)?;
    let mut rows = stmt.query([])?;

    while let Some(row) = rows.next()? {
        if equals_name(&row, column)? {
            return Ok(true);
        }
    }

    Ok(false)
}

fn equals_name(row: &Row<'_>, column: &str) -> Result<bool, rusqlite::Error> {
    let name: String = row.get(1)?;
    Ok(name.eq_ignore_ascii_case(column))
}
