// End-to-end workflow integration tests
// Tests complete user workflows across multiple services

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
async fn test_recurring_task_basic_workflow() {
    let (_pool, recurring_service, _dep_service, _mem_service, _dir) = setup_test_environment().await;
    
    // Create a recurring task template
    let template_input = RecurringTaskTemplateCreate {
        title: "Weekly Team Meeting".to_string(),
        description: Some("Discuss project progress".to_string()),
        recurrence_rule_string: "FREQ=WEEKLY;INTERVAL=1;BYDAY=MO,WE,FR".to_string(),
        priority: Some("high".to_string()),
        tags: None,
        estimated_minutes: None,
    };
    
    let template = recurring_service
        .create_template(template_input)
        .expect("Failed to create recurring task");
    
    assert!(!template.id.is_empty());
    assert_eq!(template.title, "Weekly Team Meeting");
}

#[tokio::test]
async fn test_dependency_basic_workflow() {
    let (pool, _rec_service, dependency_service, _mem_service, _dir) = setup_test_environment().await;
    
    // Create test tasks
    pool.with_connection(|conn| {
        let now = Utc::now().to_rfc3339();
        for i in 1..=3 {
            conn.execute(
                "INSERT INTO tasks (id, title, status, priority, created_at, updated_at) 
                 VALUES (?, ?, ?, 'medium', ?, ?)",
                (format!("task{}", i), format!("Task {}", i), "todo", now.clone(), now.clone())
            )?;
        }
        Ok(())
    }).expect("test data setup");
    
    // Create dependency
    let input = DependencyCreateInput {
        predecessor_id: "task1".to_string(),
        successor_id: "task2".to_string(),
        dependency_type: None,
    };
    
    let result = dependency_service.add_dependency(input).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_memory_basic_workflow() {
    let (_pool, _rec_service, _dep_service, memory_service, _dir) = setup_test_environment().await;
    
    // Store conversation
    let doc_id = memory_service
        .store_conversation(
            "test_conv",
            "How do I create recurring tasks?",
            "You can create recurring tasks by setting up recurrence rules...",
            vec!["recurring tasks".to_string()],
        )
        .await
        .expect("Failed to store conversation");
    
    assert!(!doc_id.is_empty());
    
    // Search memory
    let context = memory_service
        .search_memory("recurring tasks", 5)
        .await
        .expect("Failed to search memory");
    
    assert!(!context.relevant_documents.is_empty());
}
