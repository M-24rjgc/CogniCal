use rusqlite::{Connection, Row};
use tracing::{info, warn};
use chrono::{DateTime, Utc};

use crate::error::AppResult;
use crate::models::settings::DashboardConfig;

const USER_VERSION: i32 = 9;
const KEY_DASHBOARD_CONFIG: &str = "dashboard_config";

#[derive(Debug)]
pub struct MigrationInfo {
    pub version: i32,
    pub description: String,
    pub applied_at: DateTime<Utc>,
}



pub fn run(conn: &Connection) -> AppResult<()> {
    // Ensure migration history table exists
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS migration_history (
            version INTEGER PRIMARY KEY,
            description TEXT NOT NULL,
            applied_at TEXT NOT NULL,
            rollback_sql TEXT
        );
        "#,
    )?;
    
    let mut current_version: i32 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;

    if current_version < 1 {
        info!(target: "app::db", version = current_version, "running migration v1");
        migrate_to_v1(conn)?;
        current_version = 1;
        conn.execute(&format!("PRAGMA user_version = {}", current_version), [])?;
        record_migration(conn, 1, "Add AI-enhanced task fields and parse cache", None)?;
    }

    if current_version < 2 {
        info!(target: "app::db", version = current_version, "running migration v2");
        migrate_to_v2(conn)?;
        current_version = 2;
        conn.execute(&format!("PRAGMA user_version = {}", current_version), [])?;
        record_migration(conn, 2, "Add planning sessions and time blocks", None)?;
    }

    if current_version < 3 {
        info!(target: "app::db", version = current_version, "running migration v3");
        migrate_to_v3(conn)?;
        current_version = 3;
        conn.execute(&format!("PRAGMA user_version = {}", current_version), [])?;
        record_migration(conn, 3, "Add analytics snapshots and app settings", None)?;
    }

    if current_version < 4 {
        info!(target: "app::db", version = current_version, "running migration v4");
        migrate_to_v4(conn)?;
        current_version = 4;
        conn.execute(&format!("PRAGMA user_version = {}", current_version), [])?;
        record_migration(conn, 4, "Add productivity scores and recommendation system", None)?;
    }

    if current_version < 5 {
        info!(target: "app::db", version = current_version, "running migration v5");
        migrate_to_v5(conn)?;
        current_version = 5;
        conn.execute(&format!("PRAGMA user_version = {}", current_version), [])?;
        record_migration(conn, 5, "Add AI settings and enhanced cache", None)?;
    }

    if current_version < 6 {
        info!(target: "app::db", version = current_version, "running migration v6");
        migrate_to_v6(conn)?;
        current_version = 6;
        conn.execute(&format!("PRAGMA user_version = {}", current_version), [])?;
        record_migration(conn, 6, "Add default dashboard configuration", None)?;
    }

    if current_version < 7 {
        info!(target: "app::db", version = current_version, "running migration v7");
        migrate_to_v7(conn)?;
        current_version = 7;
        conn.execute(&format!("PRAGMA user_version = {}", current_version), [])?;
        record_migration(conn, 7, "Add conversations and memory config", Some(
            "DROP TABLE IF EXISTS conversations; DROP TABLE IF EXISTS memory_config;"
        ))?;
    }

    if current_version < 8 {
        info!(target: "app::db", version = current_version, "running migration v8");
        migrate_to_v8(conn)?;
        current_version = 8;
        conn.execute(&format!("PRAGMA user_version = {}", current_version), [])?;
        record_migration(conn, 8, "Add recurring tasks and task dependencies", Some(
            r#"
            DROP VIEW IF EXISTS ready_tasks;
            DROP TABLE IF EXISTS task_dependencies;
            DROP TABLE IF EXISTS task_instances;
            DROP TABLE IF EXISTS recurring_task_templates;
            "#
        ))?;
    }

    if current_version < 9 {
        info!(target: "app::db", version = current_version, "running migration v9");
        migrate_to_v9(conn)?;
        current_version = 9;
        conn.execute(&format!("PRAGMA user_version = {}", current_version), [])?;
        record_migration(conn, 9, "Add goals and goal-task associations", Some(
            r#"
            DROP TABLE IF EXISTS goal_task_associations;
            DROP TABLE IF EXISTS goals;
            "#
        ))?;
    }

    if current_version != USER_VERSION {
        conn.execute(&format!("PRAGMA user_version = {}", USER_VERSION), [])?;
    }

    Ok(())
}

fn record_migration(conn: &Connection, version: i32, description: &str, rollback_sql: Option<&str>) -> AppResult<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT OR REPLACE INTO migration_history (version, description, applied_at, rollback_sql) VALUES (?, ?, ?, ?)",
        (version, description, now, rollback_sql),
    )?;
    Ok(())
}

pub fn rollback_to_version(conn: &Connection, target_version: i32) -> AppResult<()> {
    let current_version: i32 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    
    if target_version >= current_version {
        warn!("Target version {} is not less than current version {}", target_version, current_version);
        return Ok(());
    }

    // Get rollback scripts for versions greater than target
    let mut stmt = conn.prepare(
        "SELECT version, rollback_sql FROM migration_history WHERE version > ? ORDER BY version DESC"
    )?;
    
    let rollback_iter = stmt.query_map([target_version], |row| {
        Ok((row.get::<_, i32>(0)?, row.get::<_, Option<String>>(1)?))
    })?;

    for rollback_result in rollback_iter {
        let (version, rollback_sql) = rollback_result?;
        if let Some(sql) = rollback_sql {
            info!("Rolling back migration v{}", version);
            conn.execute_batch(&sql)?;
        } else {
            warn!("No rollback script available for migration v{}", version);
        }
    }

    // Update version and remove rolled back migrations from history
    conn.execute(&format!("PRAGMA user_version = {}", target_version), [])?;
    conn.execute("DELETE FROM migration_history WHERE version > ?", [target_version])?;

    Ok(())
}

pub fn get_migration_history(conn: &Connection) -> AppResult<Vec<MigrationInfo>> {
    let mut stmt = conn.prepare(
        "SELECT version, description, applied_at FROM migration_history ORDER BY version"
    )?;
    
    let migration_iter = stmt.query_map([], |row| {
        let applied_at_str: String = row.get(2)?;
        let applied_at = DateTime::parse_from_rfc3339(&applied_at_str)
            .map_err(|_| rusqlite::Error::InvalidColumnType(2, "applied_at".to_string(), rusqlite::types::Type::Text))?
            .with_timezone(&Utc);
        
        Ok(MigrationInfo {
            version: row.get(0)?,
            description: row.get(1)?,
            applied_at,
        })
    })?;

    let mut migrations = Vec::new();
    for migration in migration_iter {
        migrations.push(migration?);
    }
    Ok(migrations)
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

fn migrate_to_v8(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        r#"
        -- Recurring task templates table with RRULE support
        CREATE TABLE IF NOT EXISTS recurring_task_templates (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            description TEXT,
            recurrence_rule TEXT NOT NULL, -- RRULE string following RFC 5545
            priority TEXT DEFAULT 'medium',
            tags TEXT, -- JSON array
            estimated_minutes INTEGER,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            is_active BOOLEAN DEFAULT TRUE
        );

        -- Task instances generated from templates
        CREATE TABLE IF NOT EXISTS task_instances (
            id TEXT PRIMARY KEY,
            template_id TEXT NOT NULL,
            instance_date TEXT NOT NULL, -- ISO date for this occurrence
            title TEXT NOT NULL,
            description TEXT,
            status TEXT DEFAULT 'todo',
            priority TEXT DEFAULT 'medium',
            due_at TEXT,
            completed_at TEXT,
            is_exception BOOLEAN DEFAULT FALSE, -- Modified from template
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (template_id) REFERENCES recurring_task_templates(id) ON DELETE CASCADE
        );

        -- Task dependency relationships
        CREATE TABLE IF NOT EXISTS task_dependencies (
            id TEXT PRIMARY KEY,
            predecessor_id TEXT NOT NULL,
            successor_id TEXT NOT NULL,
            dependency_type TEXT DEFAULT 'finish_to_start',
            created_at TEXT NOT NULL,
            FOREIGN KEY (predecessor_id) REFERENCES tasks(id) ON DELETE CASCADE,
            FOREIGN KEY (successor_id) REFERENCES tasks(id) ON DELETE CASCADE,
            UNIQUE(predecessor_id, successor_id)
        );

        -- Indexes for efficient instance and template queries
        CREATE INDEX IF NOT EXISTS idx_recurring_task_templates_is_active 
            ON recurring_task_templates(is_active);
        CREATE INDEX IF NOT EXISTS idx_recurring_task_templates_created_at 
            ON recurring_task_templates(created_at);
        
        CREATE INDEX IF NOT EXISTS idx_task_instances_template_date 
            ON task_instances(template_id, instance_date);
        CREATE INDEX IF NOT EXISTS idx_task_instances_due_at 
            ON task_instances(due_at) WHERE due_at IS NOT NULL;
        CREATE INDEX IF NOT EXISTS idx_task_instances_status 
            ON task_instances(status);
        CREATE INDEX IF NOT EXISTS idx_task_instances_template_id 
            ON task_instances(template_id);

        -- Indexes for dependency queries
        CREATE INDEX IF NOT EXISTS idx_task_dependencies_predecessor 
            ON task_dependencies(predecessor_id);
        CREATE INDEX IF NOT EXISTS idx_task_dependencies_successor 
            ON task_dependencies(successor_id);
        CREATE INDEX IF NOT EXISTS idx_task_dependencies_created_at 
            ON task_dependencies(created_at);

        -- View for ready tasks (no incomplete dependencies)
        CREATE VIEW IF NOT EXISTS ready_tasks AS
        SELECT t.id, t.title, t.status, t.priority, t.due_at
        FROM tasks t
        WHERE t.status != 'completed'
        AND NOT EXISTS (
            SELECT 1 FROM task_dependencies td
            JOIN tasks pt ON td.predecessor_id = pt.id
            WHERE td.successor_id = t.id
            AND pt.status != 'completed'
        );
        "#,
    )?;

    Ok(())
}

fn migrate_to_v9(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        r#"
        -- Goals table for goal-oriented task breakdown
        CREATE TABLE IF NOT EXISTS goals (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            description TEXT,
            parent_goal_id TEXT,
            status TEXT DEFAULT 'not_started',
            priority TEXT DEFAULT 'medium',
            target_date TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (parent_goal_id) REFERENCES goals(id) ON DELETE CASCADE
        );

        -- Goal-task associations
        CREATE TABLE IF NOT EXISTS goal_task_associations (
            id TEXT PRIMARY KEY,
            goal_id TEXT NOT NULL,
            task_id TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY (goal_id) REFERENCES goals(id) ON DELETE CASCADE,
            FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,
            UNIQUE(goal_id, task_id)
        );

        -- Indexes for goal queries
        CREATE INDEX IF NOT EXISTS idx_goals_parent_goal_id 
            ON goals(parent_goal_id);
        CREATE INDEX IF NOT EXISTS idx_goals_status 
            ON goals(status);
        CREATE INDEX IF NOT EXISTS idx_goals_target_date 
            ON goals(target_date);
        CREATE INDEX IF NOT EXISTS idx_goals_created_at 
            ON goals(created_at);

        CREATE INDEX IF NOT EXISTS idx_goal_task_associations_goal_id 
            ON goal_task_associations(goal_id);
        CREATE INDEX IF NOT EXISTS idx_goal_task_associations_task_id 
            ON goal_task_associations(task_id);
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
