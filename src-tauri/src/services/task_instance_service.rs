use chrono::{DateTime, Utc};

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::recurring_task::{
    TaskInstance, TaskInstanceFilter, TaskInstanceUpdate,
};

/// Options for editing task instances
#[derive(Debug, Clone)]
pub enum EditScope {
    /// Edit only this specific instance
    ThisInstance,
    /// Edit this instance and all future instances in the series
    ThisAndFuture,
    /// Edit all instances in the series (past, present, and future)
    AllInstances,
}

/// Options for deleting task instances
#[derive(Debug, Clone)]
pub enum DeleteScope {
    /// Delete only this specific instance
    ThisInstance,
    /// Delete this instance and all future instances
    ThisAndFuture,
    /// Delete all instances in the series
    AllInstances,
}

/// Service for managing task instance lifecycle
#[derive(Clone)]
pub struct TaskInstanceService {
    db: DbPool,
}

impl TaskInstanceService {
    /// Create a new task instance service
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    /// Get a task instance by ID
    pub fn get_instance(&self, id: &str) -> AppResult<TaskInstance> {
        self.db.with_connection(|conn| {
            let sql = r#"
                SELECT id, template_id, instance_date, title, description, status, 
                       priority, due_at, completed_at, is_exception, created_at, updated_at
                FROM task_instances 
                WHERE id = ?1
            "#;
            
            let mut stmt = conn.prepare(sql)?;
            let mut rows = stmt.query_map([id], |row| {
                let instance_date_str: String = row.get(2)?;
                let instance_date = DateTime::parse_from_rfc3339(&instance_date_str)
                    .map_err(|_e| rusqlite::Error::InvalidColumnType(2, "instance_date".to_string(), rusqlite::types::Type::Text))?
                    .with_timezone(&Utc);
                
                let due_at_str: Option<String> = row.get(7)?;
                let due_at = due_at_str.map(|s| DateTime::parse_from_rfc3339(&s)
                    .map_err(|_e| rusqlite::Error::InvalidColumnType(7, "due_at".to_string(), rusqlite::types::Type::Text))
                    .map(|dt| dt.with_timezone(&Utc)))
                    .transpose()?;
                
                let completed_at_str: Option<String> = row.get(8)?;
                let completed_at = completed_at_str.map(|s| DateTime::parse_from_rfc3339(&s)
                    .map_err(|_e| rusqlite::Error::InvalidColumnType(8, "completed_at".to_string(), rusqlite::types::Type::Text))
                    .map(|dt| dt.with_timezone(&Utc)))
                    .transpose()?;
                
                let created_at_str: String = row.get(10)?;
                let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                    .map_err(|_e| rusqlite::Error::InvalidColumnType(10, "created_at".to_string(), rusqlite::types::Type::Text))?
                    .with_timezone(&Utc);
                
                let updated_at_str: String = row.get(11)?;
                let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                    .map_err(|_e| rusqlite::Error::InvalidColumnType(11, "updated_at".to_string(), rusqlite::types::Type::Text))?
                    .with_timezone(&Utc);
                
                Ok(TaskInstance {
                    id: row.get(0)?,
                    template_id: row.get(1)?,
                    instance_date,
                    title: row.get(3)?,
                    description: row.get(4)?,
                    status: row.get(5)?,
                    priority: row.get(6)?,
                    due_at,
                    completed_at,
                    is_exception: row.get(9)?,
                    created_at,
                    updated_at,
                })
            })?;
            
            rows.next()
                .ok_or_else(|| AppError::not_found())?
                .map_err(AppError::from)
        })
    }

    /// List task instances with optional filtering
    pub fn list_instances(&self, filter: Option<TaskInstanceFilter>) -> AppResult<Vec<TaskInstance>> {
        self.db.with_connection(|conn| {
            let mut sql = String::from(r#"
                SELECT id, template_id, instance_date, title, description, status, 
                       priority, due_at, completed_at, is_exception, created_at, updated_at
                FROM task_instances 
                WHERE 1=1
            "#);
            
            let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
            
            if let Some(filter) = filter {
                if let Some(template_id) = filter.template_id {
                    sql.push_str(" AND template_id = ?");
                    params.push(Box::new(template_id));
                }
                
                if let Some(status) = filter.status {
                    sql.push_str(" AND status = ?");
                    params.push(Box::new(status));
                }
                
                if let Some(instance_date_after) = filter.instance_date_after {
                    sql.push_str(" AND instance_date >= ?");
                    params.push(Box::new(instance_date_after.to_rfc3339()));
                }
                
                if let Some(instance_date_before) = filter.instance_date_before {
                    sql.push_str(" AND instance_date <= ?");
                    params.push(Box::new(instance_date_before.to_rfc3339()));
                }
                
                if let Some(due_after) = filter.due_after {
                    sql.push_str(" AND due_at >= ?");
                    params.push(Box::new(due_after.to_rfc3339()));
                }
                
                if let Some(due_before) = filter.due_before {
                    sql.push_str(" AND due_at <= ?");
                    params.push(Box::new(due_before.to_rfc3339()));
                }
                
                if let Some(is_exception) = filter.is_exception {
                    sql.push_str(" AND is_exception = ?");
                    params.push(Box::new(is_exception));
                }
            }
            
            sql.push_str(" ORDER BY instance_date ASC");
            
            let mut stmt = conn.prepare(&sql)?;
            let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
            
            let rows = stmt.query_map(param_refs.as_slice(), |row| {
                let instance_date_str: String = row.get(2)?;
                let instance_date = DateTime::parse_from_rfc3339(&instance_date_str)
                    .map_err(|_e| rusqlite::Error::InvalidColumnType(2, "instance_date".to_string(), rusqlite::types::Type::Text))?
                    .with_timezone(&Utc);
                
                let due_at_str: Option<String> = row.get(7)?;
                let due_at = due_at_str.map(|s| DateTime::parse_from_rfc3339(&s)
                    .map_err(|_e| rusqlite::Error::InvalidColumnType(7, "due_at".to_string(), rusqlite::types::Type::Text))
                    .map(|dt| dt.with_timezone(&Utc)))
                    .transpose()?;
                
                let completed_at_str: Option<String> = row.get(8)?;
                let completed_at = completed_at_str.map(|s| DateTime::parse_from_rfc3339(&s)
                    .map_err(|_e| rusqlite::Error::InvalidColumnType(8, "completed_at".to_string(), rusqlite::types::Type::Text))
                    .map(|dt| dt.with_timezone(&Utc)))
                    .transpose()?;
                
                let created_at_str: String = row.get(10)?;
                let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                    .map_err(|_e| rusqlite::Error::InvalidColumnType(10, "created_at".to_string(), rusqlite::types::Type::Text))?
                    .with_timezone(&Utc);
                
                let updated_at_str: String = row.get(11)?;
                let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                    .map_err(|_e| rusqlite::Error::InvalidColumnType(11, "updated_at".to_string(), rusqlite::types::Type::Text))?
                    .with_timezone(&Utc);
                
                Ok(TaskInstance {
                    id: row.get(0)?,
                    template_id: row.get(1)?,
                    instance_date,
                    title: row.get(3)?,
                    description: row.get(4)?,
                    status: row.get(5)?,
                    priority: row.get(6)?,
                    due_at,
                    completed_at,
                    is_exception: row.get(9)?,
                    created_at,
                    updated_at,
                })
            })?;
            
            let mut instances = Vec::new();
            for row in rows {
                instances.push(row?);
            }
            
            Ok(instances)
        })
    }

    /// Update a task instance with specified scope
    pub fn update_instance(
        &self,
        id: &str,
        update: TaskInstanceUpdate,
        scope: EditScope,
    ) -> AppResult<Vec<TaskInstance>> {
        let instance = self.get_instance(id)?;
        
        // Validate the update
        if let Some(ref title) = update.title {
            if title.trim().is_empty() {
                return Err(AppError::validation("Title cannot be empty"));
            }
            if title.len() > 200 {
                return Err(AppError::validation("Title cannot exceed 200 characters"));
            }
        }

        if let Some(ref description) = update.description {
            if let Some(ref desc) = description {
                if desc.len() > 1000 {
                    return Err(AppError::validation("Description cannot exceed 1000 characters"));
                }
            }
        }

        if let Some(ref status) = update.status {
            Self::validate_status(status)?;
        }

        if let Some(ref priority) = update.priority {
            Self::validate_priority(priority)?;
        }

        match scope {
            EditScope::ThisInstance => {
                let updated = self.update_single_instance(id, update)?;
                Ok(vec![updated])
            }
            EditScope::ThisAndFuture => {
                self.update_this_and_future_instances(&instance, update)
            }
            EditScope::AllInstances => {
                self.update_all_instances(&instance.template_id, update)
            }
        }
    }

    /// Delete a task instance with specified scope
    pub fn delete_instance(&self, id: &str, scope: DeleteScope) -> AppResult<usize> {
        let instance = self.get_instance(id)?;
        
        match scope {
            DeleteScope::ThisInstance => {
                self.delete_single_instance(id)
            }
            DeleteScope::ThisAndFuture => {
                self.delete_this_and_future_instances(&instance)
            }
            DeleteScope::AllInstances => {
                self.delete_all_instances(&instance.template_id)
            }
        }
    }

    /// Complete a task instance
    pub fn complete_instance(&self, id: &str) -> AppResult<TaskInstance> {
        let update = TaskInstanceUpdate {
            status: Some("completed".to_string()),
            completed_at: Some(Some(Utc::now())),
            ..Default::default()
        };
        
        let updated = self.update_single_instance(id, update)?;
        Ok(updated)
    }

    /// Mark a task instance as an exception (modified from template)
    pub fn mark_as_exception(&self, id: &str) -> AppResult<TaskInstance> {
        self.db.with_connection(|conn| {
            let sql = r#"
                UPDATE task_instances 
                SET is_exception = TRUE, updated_at = ?1
                WHERE id = ?2
            "#;
            
            let now = Utc::now();
            conn.execute(sql, (&now.to_rfc3339(), id))?;
            
            self.get_instance(id)
        })
    }

    /// Get instances for a specific template
    pub fn get_instances_for_template(&self, template_id: &str) -> AppResult<Vec<TaskInstance>> {
        let filter = TaskInstanceFilter {
            template_id: Some(template_id.to_string()),
            ..Default::default()
        };
        self.list_instances(Some(filter))
    }

    /// Get upcoming instances (due within the next N days)
    pub fn get_upcoming_instances(&self, days: u32) -> AppResult<Vec<TaskInstance>> {
        let now = Utc::now();
        let future_date = now + chrono::Duration::days(days as i64);
        
        let filter = TaskInstanceFilter {
            instance_date_after: Some(now),
            instance_date_before: Some(future_date),
            status: Some("todo".to_string()),
            ..Default::default()
        };
        
        self.list_instances(Some(filter))
    }

    /// Get overdue instances
    pub fn get_overdue_instances(&self) -> AppResult<Vec<TaskInstance>> {
        let now = Utc::now();
        
        let filter = TaskInstanceFilter {
            due_before: Some(now),
            status: Some("todo".to_string()),
            ..Default::default()
        };
        
        self.list_instances(Some(filter))
    }

    fn update_single_instance(&self, id: &str, update: TaskInstanceUpdate) -> AppResult<TaskInstance> {
        let mut instance = self.get_instance(id)?;
        instance.update(update);
        
        self.db.with_connection(|conn| {
            let sql = r#"
                UPDATE task_instances 
                SET title = ?1, description = ?2, status = ?3, priority = ?4, 
                    due_at = ?5, completed_at = ?6, is_exception = ?7, updated_at = ?8
                WHERE id = ?9
            "#;
            
            conn.execute(sql, (
                &instance.title,
                &instance.description,
                &instance.status,
                &instance.priority,
                instance.due_at.as_ref().map(|d| d.to_rfc3339()),
                instance.completed_at.as_ref().map(|d| d.to_rfc3339()),
                instance.is_exception,
                &instance.updated_at.to_rfc3339(),
                id,
            ))?;
            
            Ok(instance)
        })
    }

    fn update_this_and_future_instances(
        &self,
        instance: &TaskInstance,
        update: TaskInstanceUpdate,
    ) -> AppResult<Vec<TaskInstance>> {
        let filter = TaskInstanceFilter {
            template_id: Some(instance.template_id.clone()),
            instance_date_after: Some(instance.instance_date),
            ..Default::default()
        };
        
        let instances = self.list_instances(Some(filter))?;
        let mut updated_instances = Vec::new();
        
        for inst in instances {
            let updated = self.update_single_instance(&inst.id, update.clone())?;
            updated_instances.push(updated);
        }
        
        Ok(updated_instances)
    }

    fn update_all_instances(
        &self,
        template_id: &str,
        update: TaskInstanceUpdate,
    ) -> AppResult<Vec<TaskInstance>> {
        let filter = TaskInstanceFilter {
            template_id: Some(template_id.to_string()),
            ..Default::default()
        };
        
        let instances = self.list_instances(Some(filter))?;
        let mut updated_instances = Vec::new();
        
        for inst in instances {
            let updated = self.update_single_instance(&inst.id, update.clone())?;
            updated_instances.push(updated);
        }
        
        Ok(updated_instances)
    }

    fn delete_single_instance(&self, id: &str) -> AppResult<usize> {
        self.db.with_connection(|conn| {
            let affected = conn.execute("DELETE FROM task_instances WHERE id = ?1", [id])?;
            
            if affected == 0 {
                return Err(AppError::not_found());
            }
            
            Ok(affected)
        })
    }

    fn delete_this_and_future_instances(&self, instance: &TaskInstance) -> AppResult<usize> {
        self.db.with_connection(|conn| {
            let sql = r#"
                DELETE FROM task_instances 
                WHERE template_id = ?1 AND instance_date >= ?2
            "#;
            
            let affected = conn.execute(sql, (
                &instance.template_id,
                &instance.instance_date.to_rfc3339(),
            ))?;
            
            Ok(affected)
        })
    }

    fn delete_all_instances(&self, template_id: &str) -> AppResult<usize> {
        self.db.with_connection(|conn| {
            let affected = conn.execute(
                "DELETE FROM task_instances WHERE template_id = ?1",
                [template_id],
            )?;
            
            Ok(affected)
        })
    }

    fn validate_status(status: &str) -> AppResult<()> {
        match status.to_lowercase().as_str() {
            "todo" | "in_progress" | "completed" | "cancelled" => Ok(()),
            _ => Err(AppError::validation("Status must be one of: todo, in_progress, completed, cancelled")),
        }
    }

    fn validate_priority(priority: &str) -> AppResult<()> {
        match priority.to_lowercase().as_str() {
            "low" | "medium" | "high" | "urgent" => Ok(()),
            _ => Err(AppError::validation("Priority must be one of: low, medium, high, urgent")),
        }
    }

    /// Get database pool reference
    pub fn pool(&self) -> &DbPool {
        &self.db
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::DbPool;

    use tempfile::tempdir;

    fn setup_service() -> (TaskInstanceService, tempfile::TempDir) {
        let dir = tempdir().expect("temp dir");
        let db_path = dir.path().join("task_instances.sqlite");
        let pool = DbPool::new(db_path).expect("db pool");
        (TaskInstanceService::new(pool), dir)
    }

    fn create_test_instance(service: &TaskInstanceService, template_id: &str) -> TaskInstance {
        // First create a template to satisfy foreign key constraint
        service.db.with_connection(|conn| {
            let sql = r#"
                INSERT OR IGNORE INTO recurring_task_templates (
                    id, title, description, recurrence_rule, priority, tags, 
                    estimated_minutes, created_at, updated_at, is_active
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#;
            
            let now = Utc::now();
            conn.execute(sql, (
                template_id,
                "Test Template",
                Some("Test template description"),
                "FREQ=DAILY",
                "medium",
                "[]",
                Some(30i64),
                &now.to_rfc3339(),
                &now.to_rfc3339(),
                true,
            )).unwrap();
            
            Ok(())
        }).unwrap();

        let instance = TaskInstance {
            id: uuid::Uuid::new_v4().to_string(),
            template_id: template_id.to_string(),
            instance_date: Utc::now(),
            title: "Test Instance".to_string(),
            description: Some("Test description".to_string()),
            status: "todo".to_string(),
            priority: "medium".to_string(),
            due_at: None,
            completed_at: None,
            is_exception: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Store in database
        service.db.with_connection(|conn| {
            let sql = r#"
                INSERT INTO task_instances (
                    id, template_id, instance_date, title, description, status, 
                    priority, due_at, completed_at, is_exception, created_at, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
            "#;
            
            conn.execute(sql, (
                &instance.id,
                &instance.template_id,
                &instance.instance_date.to_rfc3339(),
                &instance.title,
                &instance.description,
                &instance.status,
                &instance.priority,
                instance.due_at.as_ref().map(|d| d.to_rfc3339()),
                instance.completed_at.as_ref().map(|d| d.to_rfc3339()),
                instance.is_exception,
                &instance.created_at.to_rfc3339(),
                &instance.updated_at.to_rfc3339(),
            )).unwrap();
            
            Ok(())
        }).unwrap();

        instance
    }

    #[test]
    fn test_get_instance() {
        let (service, _dir) = setup_service();
        let instance = create_test_instance(&service, "template_1");
        
        let retrieved = service.get_instance(&instance.id).unwrap();
        
        assert_eq!(retrieved.id, instance.id);
        assert_eq!(retrieved.title, "Test Instance");
        assert_eq!(retrieved.status, "todo");
    }

    #[test]
    fn test_update_single_instance() {
        let (service, _dir) = setup_service();
        let instance = create_test_instance(&service, "template_1");
        
        let update = TaskInstanceUpdate {
            title: Some("Updated Title".to_string()),
            status: Some("in_progress".to_string()),
            priority: Some("high".to_string()),
            ..Default::default()
        };
        
        let updated_instances = service.update_instance(&instance.id, update, EditScope::ThisInstance).unwrap();
        
        assert_eq!(updated_instances.len(), 1);
        let updated = &updated_instances[0];
        assert_eq!(updated.title, "Updated Title");
        assert_eq!(updated.status, "in_progress");
        assert_eq!(updated.priority, "high");
        assert!(updated.is_exception); // Should be marked as exception
    }

    #[test]
    fn test_complete_instance() {
        let (service, _dir) = setup_service();
        let instance = create_test_instance(&service, "template_1");
        
        let completed = service.complete_instance(&instance.id).unwrap();
        
        assert_eq!(completed.status, "completed");
        assert!(completed.completed_at.is_some());
    }

    #[test]
    fn test_delete_single_instance() {
        let (service, _dir) = setup_service();
        let instance = create_test_instance(&service, "template_1");
        
        let deleted_count = service.delete_instance(&instance.id, DeleteScope::ThisInstance).unwrap();
        
        assert_eq!(deleted_count, 1);
        
        let result = service.get_instance(&instance.id);
        assert!(result.is_err());
    }

    #[test]
    fn test_mark_as_exception() {
        let (service, _dir) = setup_service();
        let instance = create_test_instance(&service, "template_1");
        
        assert!(!instance.is_exception);
        
        let marked = service.mark_as_exception(&instance.id).unwrap();
        
        assert!(marked.is_exception);
    }

    #[test]
    fn test_list_instances_with_filter() {
        let (service, _dir) = setup_service();
        let _instance1 = create_test_instance(&service, "template_1");
        let _instance2 = create_test_instance(&service, "template_2");
        
        let filter = TaskInstanceFilter {
            template_id: Some("template_1".to_string()),
            ..Default::default()
        };
        
        let instances = service.list_instances(Some(filter)).unwrap();
        
        assert_eq!(instances.len(), 1);
        assert_eq!(instances[0].template_id, "template_1");
    }

    #[test]
    fn test_validation_errors() {
        let (service, _dir) = setup_service();
        let instance = create_test_instance(&service, "template_1");
        
        // Empty title
        let update = TaskInstanceUpdate {
            title: Some("".to_string()),
            ..Default::default()
        };
        
        let result = service.update_instance(&instance.id, update, EditScope::ThisInstance);
        assert!(result.is_err());
        
        // Invalid status
        let update = TaskInstanceUpdate {
            status: Some("invalid_status".to_string()),
            ..Default::default()
        };
        
        let result = service.update_instance(&instance.id, update, EditScope::ThisInstance);
        assert!(result.is_err());
        
        // Invalid priority
        let update = TaskInstanceUpdate {
            priority: Some("invalid_priority".to_string()),
            ..Default::default()
        };
        
        let result = service.update_instance(&instance.id, update, EditScope::ThisInstance);
        assert!(result.is_err());
    }
}