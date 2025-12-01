use cognical_app_lib::db::{DbPool, migrations};
use tempfile::tempdir;
use chrono::Utc;

#[test]
fn test_recurring_task_tables_creation() {
    let dir = tempdir().expect("temp dir");
    let db_path = dir.path().join("test.sqlite");
    let pool = DbPool::new(db_path).expect("db pool");
    
    pool.with_connection(|conn| {
        // Verify recurring_task_templates table exists and has correct structure
        let mut stmt = conn.prepare("PRAGMA table_info(recurring_task_templates)")?;
        let column_info: Vec<(String, String)> = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(1)?, row.get::<_, String>(2)?)) // name, type
        })?.collect::<Result<Vec<_>, _>>()?;
        
        // Check required columns exist
        let column_names: Vec<&str> = column_info.iter().map(|(name, _)| name.as_str()).collect();
        assert!(column_names.contains(&"id"));
        assert!(column_names.contains(&"title"));
        assert!(column_names.contains(&"recurrence_rule"));
        assert!(column_names.contains(&"is_active"));
        assert!(column_names.contains(&"created_at"));
        assert!(column_names.contains(&"updated_at"));
        
        // Verify task_instances table exists and has correct structure
        let mut stmt = conn.prepare("PRAGMA table_info(task_instances)")?;
        let column_info: Vec<(String, String)> = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(1)?, row.get::<_, String>(2)?))
        })?.collect::<Result<Vec<_>, _>>()?;
        
        let column_names: Vec<&str> = column_info.iter().map(|(name, _)| name.as_str()).collect();
        assert!(column_names.contains(&"id"));
        assert!(column_names.contains(&"template_id"));
        assert!(column_names.contains(&"instance_date"));
        assert!(column_names.contains(&"is_exception"));
        
        Ok(())
    }).expect("table structure verification");
}

#[test]
fn test_task_dependency_tables_creation() {
    let dir = tempdir().expect("temp dir");
    let db_path = dir.path().join("test.sqlite");
    let pool = DbPool::new(db_path).expect("db pool");
    
    pool.with_connection(|conn| {
        // Verify task_dependencies table exists and has correct structure
        let mut stmt = conn.prepare("PRAGMA table_info(task_dependencies)")?;
        let column_info: Vec<(String, String)> = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(1)?, row.get::<_, String>(2)?))
        })?.collect::<Result<Vec<_>, _>>()?;
        
        let column_names: Vec<&str> = column_info.iter().map(|(name, _)| name.as_str()).collect();
        assert!(column_names.contains(&"id"));
        assert!(column_names.contains(&"predecessor_id"));
        assert!(column_names.contains(&"successor_id"));
        assert!(column_names.contains(&"dependency_type"));
        assert!(column_names.contains(&"created_at"));
        
        // Verify ready_tasks view exists
        let view_exists: bool = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='view' AND name='ready_tasks'",
            [],
            |row| row.get(0)
        )?;
        assert!(view_exists);
        
        Ok(())
    }).expect("dependency table verification");
}

#[test]
fn test_foreign_key_constraints() {
    let dir = tempdir().expect("temp dir");
    let db_path = dir.path().join("test.sqlite");
    let pool = DbPool::new(db_path).expect("db pool");
    
    pool.with_connection(|conn| {
        // Create a recurring task template
        conn.execute(
            "INSERT INTO recurring_task_templates (id, title, recurrence_rule, created_at, updated_at) 
             VALUES ('template1', 'Test Template', 'FREQ=DAILY', ?, ?)",
            (Utc::now().to_rfc3339(), Utc::now().to_rfc3339())
        )?;
        
        // Create a task instance referencing the template
        conn.execute(
            "INSERT INTO task_instances (id, template_id, instance_date, title, created_at, updated_at)
             VALUES ('instance1', 'template1', '2025-01-01', 'Test Instance', ?, ?)",
            (Utc::now().to_rfc3339(), Utc::now().to_rfc3339())
        )?;
        
        // Verify the instance was created
        let count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM task_instances WHERE template_id = 'template1'",
            [],
            |row| row.get(0)
        )?;
        assert_eq!(count, 1);
        
        // Test cascade delete - deleting template should delete instances
        conn.execute("DELETE FROM recurring_task_templates WHERE id = 'template1'", [])?;
        
        let count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM task_instances WHERE template_id = 'template1'",
            [],
            |row| row.get(0)
        )?;
        assert_eq!(count, 0);
        
        Ok(())
    }).expect("foreign key constraint test");
}

#[test]
fn test_task_dependency_constraints() {
    let dir = tempdir().expect("temp dir");
    let db_path = dir.path().join("test.sqlite");
    let pool = DbPool::new(db_path).expect("db pool");
    
    pool.with_connection(|conn| {
        // Create test tasks
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO tasks (id, title, status, priority, created_at, updated_at) 
             VALUES ('task1', 'Task 1', 'todo', 'medium', ?, ?)",
            (now.clone(), now.clone())
        )?;
        conn.execute(
            "INSERT INTO tasks (id, title, status, priority, created_at, updated_at) 
             VALUES ('task2', 'Task 2', 'todo', 'medium', ?, ?)",
            (now.clone(), now.clone())
        )?;
        
        // Create dependency relationship
        conn.execute(
            "INSERT INTO task_dependencies (id, predecessor_id, successor_id, created_at)
             VALUES ('dep1', 'task1', 'task2', ?)",
            [now.clone()]
        )?;
        
        // Verify dependency was created
        let count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM task_dependencies WHERE predecessor_id = 'task1' AND successor_id = 'task2'",
            [],
            |row| row.get(0)
        )?;
        assert_eq!(count, 1);
        
        // Test unique constraint - should not allow duplicate dependency
        let result = conn.execute(
            "INSERT INTO task_dependencies (id, predecessor_id, successor_id, created_at)
             VALUES ('dep2', 'task1', 'task2', ?)",
            [now]
        );
        assert!(result.is_err()); // Should fail due to unique constraint
        
        Ok(())
    }).expect("dependency constraint test");
}

#[test]
fn test_ready_tasks_view() {
    let dir = tempdir().expect("temp dir");
    let db_path = dir.path().join("test.sqlite");
    let pool = DbPool::new(db_path).expect("db pool");
    
    pool.with_connection(|conn| {
        let now = Utc::now().to_rfc3339();
        
        // Create test tasks
        conn.execute(
            "INSERT INTO tasks (id, title, status, priority, created_at, updated_at) 
             VALUES ('task1', 'Task 1', 'completed', 'medium', ?, ?)",
            (now.clone(), now.clone())
        )?;
        conn.execute(
            "INSERT INTO tasks (id, title, status, priority, created_at, updated_at) 
             VALUES ('task2', 'Task 2', 'todo', 'medium', ?, ?)",
            (now.clone(), now.clone())
        )?;
        conn.execute(
            "INSERT INTO tasks (id, title, status, priority, created_at, updated_at) 
             VALUES ('task3', 'Task 3', 'todo', 'medium', ?, ?)",
            (now.clone(), now.clone())
        )?;
        
        // Create dependency: task2 depends on task1 (completed)
        conn.execute(
            "INSERT INTO task_dependencies (id, predecessor_id, successor_id, created_at)
             VALUES ('dep1', 'task1', 'task2', ?)",
            [now.clone()]
        )?;
        
        // Query ready_tasks view
        let mut stmt = conn.prepare("SELECT id, title FROM ready_tasks ORDER BY id")?;
        let ready_tasks: Vec<String> = stmt.query_map([], |row| {
            Ok(row.get::<_, String>(0)?)
        })?.collect::<Result<Vec<_>, _>>()?;
        
        // task2 should be ready (dependency completed), task3 should be ready (no dependencies)
        // task1 should not appear (completed status)
        assert!(ready_tasks.contains(&"task2".to_string()));
        assert!(ready_tasks.contains(&"task3".to_string()));
        assert!(!ready_tasks.contains(&"task1".to_string()));
        
        Ok(())
    }).expect("ready tasks view test");
}

#[test]
fn test_database_indexes_exist() {
    let dir = tempdir().expect("temp dir");
    let db_path = dir.path().join("test.sqlite");
    let pool = DbPool::new(db_path).expect("db pool");
    
    pool.with_connection(|conn| {
        // Check that required indexes exist
        let indexes: Vec<String> = conn.prepare("SELECT name FROM sqlite_master WHERE type='index' AND name LIKE 'idx_%'")?
            .query_map([], |row| Ok(row.get::<_, String>(0)?))?
            .collect::<Result<Vec<_>, _>>()?;
        
        // Verify key indexes exist
        assert!(indexes.contains(&"idx_recurring_task_templates_is_active".to_string()));
        assert!(indexes.contains(&"idx_task_instances_template_date".to_string()));
        assert!(indexes.contains(&"idx_task_dependencies_predecessor".to_string()));
        assert!(indexes.contains(&"idx_task_dependencies_successor".to_string()));
        
        Ok(())
    }).expect("index verification");
}

#[test]
fn test_migration_history_tracking() {
    let dir = tempdir().expect("temp dir");
    let db_path = dir.path().join("test.sqlite");
    let pool = DbPool::new(db_path).expect("db pool");
    
    pool.with_connection(|conn| {
        // Verify migration history table exists
        let table_exists: bool = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='migration_history'",
            [],
            |row| row.get(0)
        )?;
        assert!(table_exists);
        
        // Get migration history
        let history = migrations::get_migration_history(conn)?;
        
        // Should have at least the v8 migration (recurring tasks and dependencies)
        assert!(!history.is_empty());
        
        // Check that v8 migration is recorded
        let v8_migration = history.iter().find(|m| m.version == 8);
        assert!(v8_migration.is_some());
        
        if let Some(migration) = v8_migration {
            assert!(migration.description.contains("recurring tasks"));
            assert!(migration.description.contains("dependencies"));
        }
        
        Ok(())
    }).expect("migration history test");
}

#[test]
fn test_sample_data_performance() {
    let dir = tempdir().expect("temp dir");
    let db_path = dir.path().join("test.sqlite");
    let pool = DbPool::new(db_path).expect("db pool");
    
    pool.with_connection(|conn| {
        let now = Utc::now().to_rfc3339();
        
        // Insert sample data to test index performance
        for i in 0..100 {
            // Create recurring task template
            conn.execute(
                "INSERT INTO recurring_task_templates (id, title, recurrence_rule, is_active, created_at, updated_at) 
                 VALUES (?, ?, 'FREQ=DAILY', ?, ?, ?)",
                (format!("template{}", i), format!("Template {}", i), true, now.clone(), now.clone())
            )?;
            
            // Create multiple instances for each template
            for j in 0..10 {
                conn.execute(
                    "INSERT INTO task_instances (id, template_id, instance_date, title, created_at, updated_at)
                     VALUES (?, ?, ?, ?, ?, ?)",
                    (
                        format!("instance{}_{}", i, j),
                        format!("template{}", i),
                        format!("2025-01-{:02}", (j % 28) + 1),
                        format!("Instance {} {}", i, j),
                        now.clone(),
                        now.clone()
                    )
                )?;
            }
        }
        
        // Test query performance with indexes
        let start = std::time::Instant::now();
        let count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM task_instances WHERE template_id = 'template50'",
            [],
            |row| row.get(0)
        )?;
        let duration = start.elapsed();
        
        assert_eq!(count, 10);
        // Query should be fast with proper indexing (under 10ms for this small dataset)
        assert!(duration.as_millis() < 10);
        
        Ok(())
    }).expect("performance test");
}