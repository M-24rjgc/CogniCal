CREATE TABLE IF NOT EXISTS tasks (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL,
    priority TEXT NOT NULL,
    planned_start_at TEXT,
    start_at TEXT,
    due_at TEXT,
    completed_at TEXT,
    estimated_minutes INTEGER,
    estimated_hours REAL,
    tags TEXT,
    owner_id TEXT,
    task_type TEXT,
    is_recurring INTEGER NOT NULL DEFAULT 0,
    recurrence_rule TEXT,
    recurrence_until TEXT,
    ai_summary TEXT,
    ai_next_action TEXT,
    ai_confidence REAL,
    ai_complexity_score REAL,
    ai_suggested_start_at TEXT,
    ai_focus_mode TEXT,
    ai_efficiency_prediction TEXT,
    ai_cot_steps TEXT,
    ai_cot_summary TEXT,
    ai_metadata TEXT,
    ai_source TEXT,
    ai_generated_at TEXT,
    external_links TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
CREATE INDEX IF NOT EXISTS idx_tasks_priority ON tasks(priority);
CREATE INDEX IF NOT EXISTS idx_tasks_due_at ON tasks(due_at);
CREATE INDEX IF NOT EXISTS idx_tasks_updated_at ON tasks(updated_at);

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
    focus_consistency REAL NOT NULL DEFAULT 0,
    rest_balance REAL NOT NULL DEFAULT 0,
    capacity_risk REAL NOT NULL DEFAULT 0,
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

CREATE TABLE IF NOT EXISTS migration_history (
    version INTEGER PRIMARY KEY,
    description TEXT NOT NULL,
    applied_at TEXT NOT NULL,
    rollback_sql TEXT
);

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
