use chrono::Utc;
use rusqlite::params;
use uuid::Uuid;

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::goal::{
    CreateGoalRequest, Goal, GoalStatus, GoalTaskAssociation, GoalWithProgress, UpdateGoalRequest,
};

pub struct GoalService {
    db: DbPool,
}

impl GoalService {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub fn create_goal(&self, request: CreateGoalRequest) -> AppResult<Goal> {
        let now = Utc::now();
        let id = Uuid::new_v4().to_string();

        self.db.with_connection(|conn| {
        // Validate parent goal exists if specified
        if let Some(ref parent_id) = request.parent_goal_id {
            let exists: bool = conn.query_row(
                "SELECT EXISTS(SELECT 1 FROM goals WHERE id = ?)",
                params![parent_id],
                |row| row.get(0),
            )?;
            if !exists {
                return Err(AppError::database(format!(
                    "Parent goal not found: {}",
                    parent_id
                )));
            }
        }

        conn.execute(
            r#"
            INSERT INTO goals (id, title, description, parent_goal_id, status, priority, target_date, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            params![
                &id,
                &request.title,
                &request.description,
                &request.parent_goal_id,
                "not_started",
                &request.priority,
                request.target_date.map(|d| d.to_rfc3339()),
                now.to_rfc3339(),
                now.to_rfc3339(),
            ],
        )?;

        Ok(())
        })?;

        self.get_goal(&id)
    }

    pub fn get_goal(&self, id: &str) -> AppResult<Goal> {
        self.db.with_connection(|conn| {
        Ok(conn.query_row(
            r#"
            SELECT id, title, description, parent_goal_id, status, priority, target_date, created_at, updated_at
            FROM goals
            WHERE id = ?
            "#,
            params![id],
            |row| {
                Ok(Goal {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    description: row.get(2)?,
                    parent_goal_id: row.get(3)?,
                    status: GoalStatus::from_str(&row.get::<_, String>(4)?).unwrap_or_else(|_| GoalStatus::NotStarted),
                    priority: row.get(5)?,
                    target_date: row.get::<_, Option<String>>(6)?.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))),
                    created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?).unwrap().with_timezone(&Utc),
                    updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?).unwrap().with_timezone(&Utc),
                })
            },
        )?)
        })
    }

    pub fn list_goals(&self, parent_goal_id: Option<String>) -> AppResult<Vec<Goal>> {
        self.db.with_connection(|conn| {
        let query = if parent_goal_id.is_some() {
            "SELECT id, title, description, parent_goal_id, status, priority, target_date, created_at, updated_at FROM goals WHERE parent_goal_id = ? ORDER BY created_at DESC"
        } else {
            "SELECT id, title, description, parent_goal_id, status, priority, target_date, created_at, updated_at FROM goals WHERE parent_goal_id IS NULL ORDER BY created_at DESC"
        };

        let mut stmt = conn.prepare(query)?;
        let goals = if let Some(parent_id) = parent_goal_id {
            stmt.query_map(params![parent_id], Self::map_goal_row)?
        } else {
            stmt.query_map([], Self::map_goal_row)?
        };

        Ok(goals.collect::<Result<Vec<_>, _>>()?)
        })
    }

    pub fn update_goal(&self, id: &str, request: UpdateGoalRequest) -> AppResult<Goal> {
        self.db.with_connection(|conn| {
            let now = Utc::now();

            // Build dynamic update query
            let mut updates = Vec::new();
            let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

            if let Some(title) = request.title {
                updates.push("title = ?");
                params_vec.push(Box::new(title));
            }
            if let Some(description) = request.description {
                updates.push("description = ?");
                params_vec.push(Box::new(description));
            }
            if let Some(status) = request.status {
                updates.push("status = ?");
                params_vec.push(Box::new(status.as_str().to_string()));
            }
            if let Some(priority) = request.priority {
                updates.push("priority = ?");
                params_vec.push(Box::new(priority));
            }
            if let Some(target_date) = request.target_date {
                updates.push("target_date = ?");
                params_vec.push(Box::new(target_date.to_rfc3339()));
            }

            if updates.is_empty() {
                return Ok(());
            }

            updates.push("updated_at = ?");
            params_vec.push(Box::new(now.to_rfc3339()));
            params_vec.push(Box::new(id.to_string()));

            let query = format!("UPDATE goals SET {} WHERE id = ?", updates.join(", "));

            let params_refs: Vec<&dyn rusqlite::ToSql> =
                params_vec.iter().map(|p| p.as_ref()).collect();
            conn.execute(&query, params_refs.as_slice())?;

            Ok(())
        })?;

        self.get_goal(id)
    }

    pub fn delete_goal(&self, id: &str) -> AppResult<()> {
        self.db.with_connection(|conn| {
            let rows_affected = conn.execute("DELETE FROM goals WHERE id = ?", params![id])?;

            if rows_affected == 0 {
                return Err(AppError::not_found());
            }

            Ok(())
        })
    }

    pub fn associate_task(&self, goal_id: &str, task_id: &str) -> AppResult<GoalTaskAssociation> {
        self.db.with_connection(|conn| {
        let now = Utc::now();
        let id = Uuid::new_v4().to_string();

        // Verify goal exists
        let goal_exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM goals WHERE id = ?)",
            params![goal_id],
            |row| row.get(0),
        )?;
        if !goal_exists {
            return Err(AppError::database(format!("Goal not found: {}", goal_id)));
        }

        // Verify task exists
        let task_exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM tasks WHERE id = ?)",
            params![task_id],
            |row| row.get(0),
        )?;
        if !task_exists {
            return Err(AppError::database(format!("Task not found: {}", task_id)));
        }

        conn.execute(
            "INSERT INTO goal_task_associations (id, goal_id, task_id, created_at) VALUES (?, ?, ?, ?)",
            params![id, goal_id, task_id, now.to_rfc3339()],
        )?;

        Ok(GoalTaskAssociation {
            id,
            goal_id: goal_id.to_string(),
            task_id: task_id.to_string(),
            created_at: now,
        })
        })
    }

    pub fn dissociate_task(&self, goal_id: &str, task_id: &str) -> AppResult<()> {
        self.db.with_connection(|conn| {
            let rows_affected = conn.execute(
                "DELETE FROM goal_task_associations WHERE goal_id = ? AND task_id = ?",
                params![goal_id, task_id],
            )?;

            if rows_affected == 0 {
                return Err(AppError::not_found());
            }

            Ok(())
        })
    }

    pub fn get_goal_tasks(&self, goal_id: &str) -> AppResult<Vec<String>> {
        self.db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT task_id FROM goal_task_associations WHERE goal_id = ? ORDER BY created_at",
            )?;

            let rows = stmt.query_map(params![goal_id], |row| row.get::<_, String>(0))?;
            let task_ids: Result<Vec<String>, rusqlite::Error> = rows.collect();

            Ok(task_ids?)
        })
    }

    pub fn get_goal_with_progress(&self, id: &str) -> AppResult<GoalWithProgress> {
        let goal = self.get_goal(id)?;
        let task_ids = self.get_goal_tasks(id)?;

        let (total_tasks, completed_tasks, in_progress_tasks, blocked_tasks) =
            self.db.with_connection(|conn| {
                if task_ids.is_empty() {
                    Ok((0, 0, 0, 0))
                } else {
                    let placeholders = task_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
                    let query = format!(
                        r#"
                SELECT 
                    COUNT(*),
                    SUM(CASE WHEN status = 'completed' THEN 1 ELSE 0 END),
                    SUM(CASE WHEN status = 'in_progress' THEN 1 ELSE 0 END),
                    SUM(CASE WHEN status = 'blocked' THEN 1 ELSE 0 END)
                FROM tasks 
                WHERE id IN ({})
                "#,
                        placeholders
                    );

                    let params_refs: Vec<&dyn rusqlite::ToSql> = task_ids
                        .iter()
                        .map(|id| id as &dyn rusqlite::ToSql)
                        .collect();
                    Ok(conn.query_row(&query, params_refs.as_slice(), |row| {
                        Ok((
                            row.get::<_, i32>(0)?,
                            row.get::<_, i32>(1)?,
                            row.get::<_, i32>(2)?,
                            row.get::<_, i32>(3)?,
                        ))
                    })?)
                }
            })?;

        let progress_percentage = if total_tasks > 0 {
            (completed_tasks as f32 / total_tasks as f32) * 100.0
        } else {
            0.0
        };

        // Calculate days until target date
        let days_until_target = goal.target_date.map(|target| {
            let now = Utc::now();
            (target - now).num_days()
        });

        // Determine if goal is on track
        // A goal is on track if progress is proportional to time elapsed
        let is_on_track = if let Some(target) = goal.target_date {
            let now = Utc::now();
            let total_duration = (target - goal.created_at).num_days() as f32;
            let elapsed_duration = (now - goal.created_at).num_days() as f32;

            if total_duration > 0.0 {
                let expected_progress = (elapsed_duration / total_duration) * 100.0;
                progress_percentage >= expected_progress * 0.8 // Allow 20% slack
            } else {
                progress_percentage >= 90.0 // If target is past, check if nearly complete
            }
        } else {
            true // No target date, so always "on track"
        };

        // Get child goals recursively
        let child_goals = self.list_goals(Some(id.to_string()))?;
        let child_goals_with_progress = child_goals
            .into_iter()
            .map(|child| self.get_goal_with_progress(&child.id))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(GoalWithProgress {
            goal,
            progress_percentage,
            total_tasks,
            completed_tasks,
            in_progress_tasks,
            blocked_tasks,
            child_goals: child_goals_with_progress,
            is_on_track,
            days_until_target,
        })
    }

    fn map_goal_row(row: &rusqlite::Row) -> Result<Goal, rusqlite::Error> {
        Ok(Goal {
            id: row.get(0)?,
            title: row.get(1)?,
            description: row.get(2)?,
            parent_goal_id: row.get(3)?,
            status: GoalStatus::from_str(&row.get::<_, String>(4)?)
                .unwrap_or_else(|_| GoalStatus::NotStarted),
            priority: row.get(5)?,
            target_date: row.get::<_, Option<String>>(6)?.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|dt| dt.with_timezone(&Utc))
            }),
            created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                .unwrap()
                .with_timezone(&Utc),
            updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                .unwrap()
                .with_timezone(&Utc),
        })
    }
}
