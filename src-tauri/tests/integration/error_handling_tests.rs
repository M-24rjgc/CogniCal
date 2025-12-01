// Error handling and edge case tests

use cognical_app_lib::db::DbPool;
use cognical_app_lib::models::dependency::DependencyCreateInput;
use cognical_app_lib::models::recurring_task::RecurringTaskTemplateCreate;
use cognical_app_lib::services::dependency_service::DependencyService;
use cognical_app_lib::services::recurring_task_service::RecurringTaskService;
use cognical_app_lib::services::memory_service::MemoryService;
use tempfile::tempdir;
use chrono::Utc;

async fn setup_test_environment() -> (DbPool, RecurringTaskService, DependencyService, MemoryService, tempfile::TempDir) {
    let dir = tempdir().expect("temp dir");
    let db_path = dir.path().join("test.sqlite");
    let pool = DbPool::new(db_path).expect("db pool");
    
    let recurring_service = RecurringTaskService::new(pool.clone());
    let dependency_service = DependencyService::new(pool.clone());
    
    let memory_dir = dir.path().join("memory");
    let memory_service = MemoryService::new(memory_dir).expect("memory service");
    
    (pool, recurring_service, dependency_service, memory_service, dir)
}

#[tokio::test]
async fn test_invalid_rrule_handling() {
    let (_pool, recurring_service, _dep_service, _mem_service, _dir) = setup_test_environment().await;
    
    let invalid_input = RecurringTaskTemplateCreate {
        title: "Invalid task".to_string(),
        description: None,
        recurrence_rule_string: "INVALID_RRULE".to_string(),
        priority: None,
        tags: None,
        estimated_minutes: None,
    };
    
    let result = recurring_service.create_template(invalid_input);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_self_dependency_prevention() {
    let (pool, _rec_service, dependency_service, _mem_service, _dir) = setup_test_environment().await;
    
    // Create a task
    pool.with_connection(|conn| {
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO tasks (id, title, status, priority, created_at, updated_at) 
             VALUES ('task1', 'Task 1', 'todo', 'medium', ?, ?)",
            (now.clone(), now)
        )?;
        Ok(())
    }).expect("test data setup");
    
    // Try to create self-dependency
    let input = DependencyCreateInput {
        predecessor_id: "task1".to_string(),
        successor_id: "task1".to_string(),
        dependency_type: None,
    };
    
    let result = dependency_service.add_dependency(input).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_circular_dependency_prevention() {
    let (pool, _rec_service, dependency_service, _mem_service, _dir) = setup_test_environment().await;
    
    // Create tasks
    pool.with_connection(|conn| {
        let now = Utc::now().to_rfc3339();
        for i in 1..=3 {
            conn.execute(
                "INSERT INTO tasks (id, title, status, priority, created_at, updated_at) 
                 VALUES (?, ?, 'todo', 'medium', ?, ?)",
                (format!("task{}", i), format!("Task {}", i), now.clone(), now.clone())
            )?;
        }
        Ok(())
    }).expect("test data setup");
    
    // Create chain: task1 -> task2 -> task3
    let input1 = DependencyCreateInput {
        predecessor_id: "task1".to_string(),
        successor_id: "task2".to_string(),
        dependency_type: None,
    };
    dependency_service.add_dependency(input1).await.expect("Failed to add dependency");
    
    let input2 = DependencyCreateInput {
        predecessor_id: "task2".to_string(),
        successor_id: "task3".to_string(),
        dependency_type: None,
    };
    dependency_service.add_dependency(input2).await.expect("Failed to add dependency");
    
    // Try to create cycle: task3 -> task1
    let input3 = DependencyCreateInput {
        predecessor_id: "task3".to_string(),
        successor_id: "task1".to_string(),
        dependency_type: None,
    };
    
    let result = dependency_service.add_dependency(input3).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_dependency_with_nonexistent_tasks() {
    let (_pool, _rec_service, dependency_service, _mem_service, _dir) = setup_test_environment().await;
    
    let input = DependencyCreateInput {
        predecessor_id: "nonexistent_task_1".to_string(),
        successor_id: "nonexistent_task_2".to_string(),
        dependency_type: None,
    };
    
    let result = dependency_service.add_dependency(input).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_memory_search_with_no_documents() {
    let (_pool, _rec_service, _dep_service, memory_service, _dir) = setup_test_environment().await;
    
    let result = memory_service.search_memory("anything", 5).await;
    
    assert!(result.is_ok());
    let context = result.unwrap();
    assert!(context.relevant_documents.is_empty());
}
