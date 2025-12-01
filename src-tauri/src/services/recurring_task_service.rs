use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::recurring_task::{
    RecurringTaskTemplate, RecurringTaskTemplateCreate, RecurringTaskTemplateFilter,
    RecurringTaskTemplateUpdate, TaskInstance,
};
use crate::services::instance_generator::{GenerationConfig, InstanceGenerator};
use crate::services::rrule_parser::RRuleParser;

/// Cache for generated instances to avoid redundant generation
#[derive(Clone)]
struct InstanceCache {
    cache: Arc<RwLock<HashMap<String, Vec<TaskInstance>>>>,
    cache_timestamps: Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
}

impl InstanceCache {
    fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_timestamps: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn get(&self, template_id: &str) -> Option<Vec<TaskInstance>> {
        let cache = self.cache.read().unwrap();
        let timestamps = self.cache_timestamps.read().unwrap();
        
        // Check if cache is still valid (less than 5 minutes old)
        if let Some(timestamp) = timestamps.get(template_id) {
            let age = Utc::now().signed_duration_since(*timestamp);
            if age.num_minutes() < 5 {
                return cache.get(template_id).cloned();
            }
        }
        
        None
    }

    fn set(&self, template_id: String, instances: Vec<TaskInstance>) {
        let mut cache = self.cache.write().unwrap();
        let mut timestamps = self.cache_timestamps.write().unwrap();
        
        cache.insert(template_id.clone(), instances);
        timestamps.insert(template_id, Utc::now());
    }

    fn invalidate(&self, template_id: &str) {
        let mut cache = self.cache.write().unwrap();
        let mut timestamps = self.cache_timestamps.write().unwrap();
        
        cache.remove(template_id);
        timestamps.remove(template_id);
    }

    fn clear(&self) {
        let mut cache = self.cache.write().unwrap();
        let mut timestamps = self.cache_timestamps.write().unwrap();
        
        cache.clear();
        timestamps.clear();
    }
}

/// Service for managing recurring task templates and instances
#[derive(Clone)]
pub struct RecurringTaskService {
    db: DbPool,
    instance_cache: InstanceCache,
}

impl RecurringTaskService {
    /// Create a new recurring task service
    pub fn new(db: DbPool) -> Self {
        Self { 
            db,
            instance_cache: InstanceCache::new(),
        }
    }

    /// Create a new recurring task template
    pub fn create_template(&self, input: RecurringTaskTemplateCreate) -> AppResult<RecurringTaskTemplate> {
        // Parse and validate the recurrence rule
        let recurrence_rule = RRuleParser::parse(&input.recurrence_rule_string)?;
        
        // Validate input
        if input.title.trim().is_empty() {
            return Err(AppError::validation("Title cannot be empty"));
        }
        
        if input.title.len() > 200 {
            return Err(AppError::validation("Title cannot exceed 200 characters"));
        }

        if let Some(ref description) = input.description {
            if description.len() > 1000 {
                return Err(AppError::validation("Description cannot exceed 1000 characters"));
            }
        }

        if let Some(estimated_minutes) = input.estimated_minutes {
            if estimated_minutes <= 0 {
                return Err(AppError::validation("Estimated minutes must be positive"));
            }
            if estimated_minutes > 60 * 24 * 7 {
                return Err(AppError::validation("Estimated minutes cannot exceed one week"));
            }
        }

        // Create the template
        let mut template = RecurringTaskTemplate::new(input.title.trim().to_string(), recurrence_rule);
        
        if let Some(description) = input.description {
            template = template.with_description(Some(description.trim().to_string()));
        }
        
        if let Some(priority) = input.priority {
            let priority = Self::validate_priority(&priority)?;
            template = template.with_priority(priority);
        }
        
        if let Some(tags) = input.tags {
            let tags = Self::validate_tags(tags)?;
            template = template.with_tags(tags);
        }
        
        template = template.with_estimated_minutes(input.estimated_minutes);

        // Store in database
        self.db.with_connection(|conn| {
            let sql = r#"
                INSERT INTO recurring_task_templates (
                    id, title, description, recurrence_rule, priority, tags, 
                    estimated_minutes, created_at, updated_at, is_active
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#;
            
            let tags_json = serde_json::to_string(&template.tags)
                .map_err(|e| AppError::database(&format!("Failed to serialize tags: {}", e)))?;
            let recurrence_rule_string = RRuleParser::to_string(&template.recurrence_rule);
            
            conn.execute(sql, (
                &template.id,
                &template.title,
                &template.description,
                &recurrence_rule_string,
                &template.priority,
                &tags_json,
                template.estimated_minutes,
                &template.created_at.to_rfc3339(),
                &template.updated_at.to_rfc3339(),
                template.is_active,
            ))?;
            
            Ok(())
        })?;

        // Generate initial instances
        self.generate_instances_for_template(&template.id)?;

        Ok(template)
    }

    /// Update a recurring task template
    pub fn update_template(&self, id: &str, update: RecurringTaskTemplateUpdate) -> AppResult<RecurringTaskTemplate> {
        // Invalidate cache when template is updated
        self.invalidate_template_cache(id);
        
        let mut template = self.get_template(id)?;
        
        // Validate updates
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

        if let Some(ref priority) = update.priority {
            Self::validate_priority(priority)?;
        }

        if let Some(ref tags) = update.tags {
            Self::validate_tags(tags.clone())?;
        }

        // Apply updates
        template.update(update);

        // Store in database
        self.db.with_connection(|conn| {
            let sql = r#"
                UPDATE recurring_task_templates 
                SET title = ?1, description = ?2, recurrence_rule = ?3, priority = ?4, 
                    tags = ?5, estimated_minutes = ?6, updated_at = ?7, is_active = ?8
                WHERE id = ?9
            "#;
            
            let tags_json = serde_json::to_string(&template.tags)
                .map_err(|e| AppError::database(&format!("Failed to serialize tags: {}", e)))?;
            let recurrence_rule_string = RRuleParser::to_string(&template.recurrence_rule);
            
            conn.execute(sql, (
                &template.title,
                &template.description,
                &recurrence_rule_string,
                &template.priority,
                &tags_json,
                template.estimated_minutes,
                &template.updated_at.to_rfc3339(),
                template.is_active,
                id,
            ))?;
            
            Ok(())
        })?;

        Ok(template)
    }

    /// Get a recurring task template by ID
    pub fn get_template(&self, id: &str) -> AppResult<RecurringTaskTemplate> {
        self.db.with_connection(|conn| {
            let sql = r#"
                SELECT id, title, description, recurrence_rule, priority, tags, 
                       estimated_minutes, created_at, updated_at, is_active
                FROM recurring_task_templates 
                WHERE id = ?1
            "#;
            
            let mut stmt = conn.prepare(sql)?;
            let mut rows = stmt.query_map([id], |row| {
                let tags_json: String = row.get(5)?;
                let tags: Vec<String> = serde_json::from_str(&tags_json)
                    .map_err(|_e| rusqlite::Error::InvalidColumnType(5, "tags".to_string(), rusqlite::types::Type::Text))?;
                
                let recurrence_rule_string: String = row.get(3)?;
                let recurrence_rule = RRuleParser::parse(&recurrence_rule_string)
                    .map_err(|_e| rusqlite::Error::InvalidColumnType(3, "recurrence_rule".to_string(), rusqlite::types::Type::Text))?;
                
                let created_at_str: String = row.get(7)?;
                let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                    .map_err(|_e| rusqlite::Error::InvalidColumnType(7, "created_at".to_string(), rusqlite::types::Type::Text))?
                    .with_timezone(&Utc);
                
                let updated_at_str: String = row.get(8)?;
                let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                    .map_err(|_e| rusqlite::Error::InvalidColumnType(8, "updated_at".to_string(), rusqlite::types::Type::Text))?
                    .with_timezone(&Utc);
                
                Ok(RecurringTaskTemplate {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    description: row.get(2)?,
                    recurrence_rule,
                    priority: row.get(4)?,
                    tags,
                    estimated_minutes: row.get(6)?,
                    created_at,
                    updated_at,
                    is_active: row.get(9)?,
                })
            })?;
            
            rows.next()
                .ok_or_else(|| AppError::not_found())?
                .map_err(AppError::from)
        })
    }

    /// List recurring task templates with optional filtering
    pub fn list_templates(&self, filter: Option<RecurringTaskTemplateFilter>) -> AppResult<Vec<RecurringTaskTemplate>> {
        self.db.with_connection(|conn| {
            let mut sql = String::from(r#"
                SELECT id, title, description, recurrence_rule, priority, tags, 
                       estimated_minutes, created_at, updated_at, is_active
                FROM recurring_task_templates 
                WHERE 1=1
            "#);
            
            let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
            
            if let Some(filter) = filter {
                if let Some(is_active) = filter.is_active {
                    sql.push_str(" AND is_active = ?");
                    params.push(Box::new(is_active));
                }
                
                if let Some(created_after) = filter.created_after {
                    sql.push_str(" AND created_at >= ?");
                    params.push(Box::new(created_after.to_rfc3339()));
                }
                
                if let Some(created_before) = filter.created_before {
                    sql.push_str(" AND created_at <= ?");
                    params.push(Box::new(created_before.to_rfc3339()));
                }
                
                if let Some(title_contains) = filter.title_contains {
                    sql.push_str(" AND title LIKE ?");
                    params.push(Box::new(format!("%{}%", title_contains)));
                }
            }
            
            sql.push_str(" ORDER BY created_at DESC");
            
            let mut stmt = conn.prepare(&sql)?;
            let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
            
            let rows = stmt.query_map(param_refs.as_slice(), |row| {
                let tags_json: String = row.get(5)?;
                let tags: Vec<String> = serde_json::from_str(&tags_json)
                    .map_err(|_e| rusqlite::Error::InvalidColumnType(5, "tags".to_string(), rusqlite::types::Type::Text))?;
                
                let recurrence_rule_string: String = row.get(3)?;
                let recurrence_rule = RRuleParser::parse(&recurrence_rule_string)
                    .map_err(|_e| rusqlite::Error::InvalidColumnType(3, "recurrence_rule".to_string(), rusqlite::types::Type::Text))?;
                
                let created_at_str: String = row.get(7)?;
                let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                    .map_err(|_e| rusqlite::Error::InvalidColumnType(7, "created_at".to_string(), rusqlite::types::Type::Text))?
                    .with_timezone(&Utc);
                
                let updated_at_str: String = row.get(8)?;
                let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                    .map_err(|_e| rusqlite::Error::InvalidColumnType(8, "updated_at".to_string(), rusqlite::types::Type::Text))?
                    .with_timezone(&Utc);
                
                Ok(RecurringTaskTemplate {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    description: row.get(2)?,
                    recurrence_rule,
                    priority: row.get(4)?,
                    tags,
                    estimated_minutes: row.get(6)?,
                    created_at,
                    updated_at,
                    is_active: row.get(9)?,
                })
            })?;
            
            let mut templates = Vec::new();
            for row in rows {
                templates.push(row?);
            }
            
            Ok(templates)
        })
    }

    /// Delete a recurring task template and all its instances
    pub fn delete_template(&self, id: &str) -> AppResult<()> {
        // Invalidate cache when template is deleted
        self.invalidate_template_cache(id);
        
        self.db.with_connection(|conn| {
            // Delete instances first (foreign key constraint)
            conn.execute("DELETE FROM task_instances WHERE template_id = ?1", [id])?;
            
            // Delete template
            let affected = conn.execute("DELETE FROM recurring_task_templates WHERE id = ?1", [id])?;
            
            if affected == 0 {
                return Err(AppError::not_found());
            }
            
            Ok(())
        })
    }

    /// Activate a recurring task template
    pub fn activate_template(&self, id: &str) -> AppResult<RecurringTaskTemplate> {
        let update = RecurringTaskTemplateUpdate {
            is_active: Some(true),
            ..Default::default()
        };
        let template = self.update_template(id, update)?;
        
        // Generate instances for newly activated template
        self.generate_instances_for_template(id)?;
        
        Ok(template)
    }

    /// Deactivate a recurring task template
    pub fn deactivate_template(&self, id: &str) -> AppResult<RecurringTaskTemplate> {
        let update = RecurringTaskTemplateUpdate {
            is_active: Some(false),
            ..Default::default()
        };
        self.update_template(id, update)
    }

    /// Clone a recurring task template
    pub fn clone_template(&self, id: &str) -> AppResult<RecurringTaskTemplate> {
        let original = self.get_template(id)?;
        let cloned = original.clone_template();
        
        // Store the cloned template
        self.db.with_connection(|conn| {
            let sql = r#"
                INSERT INTO recurring_task_templates (
                    id, title, description, recurrence_rule, priority, tags, 
                    estimated_minutes, created_at, updated_at, is_active
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#;
            
            let tags_json = serde_json::to_string(&cloned.tags)
                .map_err(|e| AppError::database(&format!("Failed to serialize tags: {}", e)))?;
            let recurrence_rule_string = RRuleParser::to_string(&cloned.recurrence_rule);
            
            conn.execute(sql, (
                &cloned.id,
                &cloned.title,
                &cloned.description,
                &recurrence_rule_string,
                &cloned.priority,
                &tags_json,
                cloned.estimated_minutes,
                &cloned.created_at.to_rfc3339(),
                &cloned.updated_at.to_rfc3339(),
                cloned.is_active,
            ))?;
            
            Ok(())
        })?;

        // Generate instances for the cloned template
        self.generate_instances_for_template(&cloned.id)?;

        Ok(cloned)
    }

    /// Generate instances for a template within the configured horizon
    pub fn generate_instances_for_template(&self, template_id: &str) -> AppResult<Vec<TaskInstance>> {
        // Check cache first
        if let Some(cached_instances) = self.instance_cache.get(template_id) {
            return Ok(cached_instances);
        }

        let template = self.get_template(template_id)?;
        
        if !template.is_active {
            return Ok(Vec::new());
        }

        let config = GenerationConfig::default();
        let instances = InstanceGenerator::generate_instances(
            &template.id,
            &template.title,
            &template.recurrence_rule,
            &config,
        )?;

        // Store instances in database using batch transaction
        self.batch_store_instances(&instances)?;

        // Cache the generated instances
        self.instance_cache.set(template_id.to_string(), instances.clone());

        Ok(instances)
    }

    /// Batch store instances using a single transaction for better performance
    fn batch_store_instances(&self, instances: &[TaskInstance]) -> AppResult<()> {
        if instances.is_empty() {
            return Ok(());
        }

        // Use a separate connection for the transaction
        let mut conn = self.db.get_connection()?;
        let tx = conn.transaction()?;
        
        {
            let mut stmt = tx.prepare(
                r#"
                INSERT OR REPLACE INTO task_instances (
                    id, template_id, instance_date, title, description, status, 
                    priority, due_at, completed_at, is_exception, created_at, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                "#
            )?;

            for instance in instances {
                stmt.execute((
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
                ))?;
            }
        }

        tx.commit()?;
        Ok(())
    }

    /// Generate instances for all active templates (background task)
    pub fn generate_instances_for_all_templates(&self) -> AppResult<usize> {
        let templates = self.list_templates(Some(RecurringTaskTemplateFilter {
            is_active: Some(true),
            ..Default::default()
        }))?;

        let mut total_generated = 0;

        for template in templates {
            match self.generate_instances_for_template(&template.id) {
                Ok(instances) => {
                    total_generated += instances.len();
                }
                Err(e) => {
                    eprintln!("Failed to generate instances for template {}: {}", template.id, e);
                }
            }
        }

        Ok(total_generated)
    }

    /// Pre-generate instances for upcoming period (background task)
    pub fn pregenerate_instances(&self, days_ahead: u32) -> AppResult<usize> {
        let templates = self.list_templates(Some(RecurringTaskTemplateFilter {
            is_active: Some(true),
            ..Default::default()
        }))?;

        let mut total_generated = 0;
        let config = GenerationConfig {
            horizon_days: days_ahead,
            ..Default::default()
        };

        for template in templates {
            let instances = InstanceGenerator::generate_instances(
                &template.id,
                &template.title,
                &template.recurrence_rule,
                &config,
            )?;

            self.batch_store_instances(&instances)?;
            self.instance_cache.set(template.id.clone(), instances.clone());
            total_generated += instances.len();
        }

        Ok(total_generated)
    }

    /// Clear instance cache
    pub fn clear_instance_cache(&self) {
        self.instance_cache.clear();
    }

    /// Invalidate cache for a specific template
    pub fn invalidate_template_cache(&self, template_id: &str) {
        self.instance_cache.invalidate(template_id);
    }



    /// Get database pool reference
    pub fn pool(&self) -> &DbPool {
        &self.db
    }

    fn validate_priority(priority: &str) -> AppResult<String> {
        let priority = priority.to_lowercase();
        match priority.as_str() {
            "low" | "medium" | "high" | "urgent" => Ok(priority),
            _ => Err(AppError::validation("Priority must be one of: low, medium, high, urgent")),
        }
    }

    fn validate_tags(tags: Vec<String>) -> AppResult<Vec<String>> {
        if tags.len() > 20 {
            return Err(AppError::validation("Cannot have more than 20 tags"));
        }

        let mut validated_tags = Vec::new();
        for tag in tags {
            let tag = tag.trim().to_string();
            if tag.is_empty() {
                continue;
            }
            if tag.len() > 50 {
                return Err(AppError::validation("Tag cannot exceed 50 characters"));
            }
            validated_tags.push(tag);
        }

        Ok(validated_tags)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::DbPool;
    use tempfile::tempdir;

    fn setup_service() -> (RecurringTaskService, tempfile::TempDir) {
        let dir = tempdir().expect("temp dir");
        let db_path = dir.path().join("recurring_tasks.sqlite");
        let pool = DbPool::new(db_path).expect("db pool");
        (RecurringTaskService::new(pool), dir)
    }

    #[test]
    fn test_create_template() {
        let (service, _dir) = setup_service();
        
        let input = RecurringTaskTemplateCreate {
            title: "Daily Standup".to_string(),
            description: Some("Daily team standup meeting".to_string()),
            recurrence_rule_string: "FREQ=DAILY;INTERVAL=1".to_string(),
            priority: Some("high".to_string()),
            tags: Some(vec!["meeting".to_string(), "team".to_string()]),
            estimated_minutes: Some(30),
        };
        
        let template = service.create_template(input).unwrap();
        
        assert_eq!(template.title, "Daily Standup");
        assert_eq!(template.description, Some("Daily team standup meeting".to_string()));
        assert_eq!(template.priority, "high");
        assert_eq!(template.tags, vec!["meeting", "team"]);
        assert_eq!(template.estimated_minutes, Some(30));
        assert!(template.is_active);
    }

    #[test]
    fn test_update_template() {
        let (service, _dir) = setup_service();
        
        let input = RecurringTaskTemplateCreate {
            title: "Weekly Review".to_string(),
            description: None,
            recurrence_rule_string: "FREQ=WEEKLY;BYDAY=FR".to_string(),
            priority: None,
            tags: None,
            estimated_minutes: None,
        };
        
        let template = service.create_template(input).unwrap();
        
        let update = RecurringTaskTemplateUpdate {
            title: Some("Weekly Team Review".to_string()),
            description: Some(Some("Review team progress and plan next week".to_string())),
            priority: Some("high".to_string()),
            ..Default::default()
        };
        
        let updated = service.update_template(&template.id, update).unwrap();
        
        assert_eq!(updated.title, "Weekly Team Review");
        assert_eq!(updated.description, Some("Review team progress and plan next week".to_string()));
        assert_eq!(updated.priority, "high");
    }

    #[test]
    fn test_activate_deactivate_template() {
        let (service, _dir) = setup_service();
        
        let input = RecurringTaskTemplateCreate {
            title: "Test Task".to_string(),
            description: None,
            recurrence_rule_string: "FREQ=DAILY".to_string(),
            priority: None,
            tags: None,
            estimated_minutes: None,
        };
        
        let template = service.create_template(input).unwrap();
        assert!(template.is_active);
        
        let deactivated = service.deactivate_template(&template.id).unwrap();
        assert!(!deactivated.is_active);
        
        let activated = service.activate_template(&template.id).unwrap();
        assert!(activated.is_active);
    }

    #[test]
    fn test_clone_template() {
        let (service, _dir) = setup_service();
        
        let input = RecurringTaskTemplateCreate {
            title: "Original Task".to_string(),
            description: Some("Original description".to_string()),
            recurrence_rule_string: "FREQ=WEEKLY;BYDAY=MO".to_string(),
            priority: Some("medium".to_string()),
            tags: Some(vec!["original".to_string()]),
            estimated_minutes: Some(60),
        };
        
        let original = service.create_template(input).unwrap();
        let cloned = service.clone_template(&original.id).unwrap();
        
        assert_ne!(cloned.id, original.id);
        assert_eq!(cloned.title, "Original Task (Copy)");
        assert_eq!(cloned.description, original.description);
        assert_eq!(cloned.priority, original.priority);
        assert_eq!(cloned.tags, original.tags);
        assert_eq!(cloned.estimated_minutes, original.estimated_minutes);
        assert!(cloned.is_active);
    }

    #[test]
    fn test_validation_errors() {
        let (service, _dir) = setup_service();
        
        // Empty title
        let input = RecurringTaskTemplateCreate {
            title: "".to_string(),
            description: None,
            recurrence_rule_string: "FREQ=DAILY".to_string(),
            priority: None,
            tags: None,
            estimated_minutes: None,
        };
        
        assert!(service.create_template(input).is_err());
        
        // Invalid recurrence rule
        let input = RecurringTaskTemplateCreate {
            title: "Valid Title".to_string(),
            description: None,
            recurrence_rule_string: "INVALID_RULE".to_string(),
            priority: None,
            tags: None,
            estimated_minutes: None,
        };
        
        assert!(service.create_template(input).is_err());
        
        // Invalid priority
        let input = RecurringTaskTemplateCreate {
            title: "Valid Title".to_string(),
            description: None,
            recurrence_rule_string: "FREQ=DAILY".to_string(),
            priority: Some("invalid".to_string()),
            tags: None,
            estimated_minutes: None,
        };
        
        assert!(service.create_template(input).is_err());
    }
}