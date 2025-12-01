// Comprehensive integration tests

use cognical_app_lib::db::DbPool;
use cognical_app_lib::models::dependency::DependencyCreateInput;
use cognical_app_lib::models::recurring_task::RecurringTaskTemplateCreate;
use cognical_app_lib::services::dependency_service::DependencyService;
use cognical_app_lib::services::recurring_task_service::RecurringTaskService;
use cognical_app_lib::services::memory_service::MemoryService;
use tempfile::tempdir;
use chrono::Utc;

async fn setup_test_environment() -> (
    DbPool,
    RecurringTaskService,
    DependencyService,
    MemoryService,
    tempfile::TempDir,
) {
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
async fn test_memory_integration_with_task_management() {
    let (pool, _recurring_service, dependency_service, memory_service, _dir) =
        setup_test_environment().await;
    
    // User asks AI about setting up a project
    let _conv1 = memory_service
        .store_conversation(
            "project_setup_conv",
            "I need to set up a software development project",
            "Let's create a project structure with requirements, design, implementation, and testing...",
            vec!["project management".to_string()],
        )
        .await
        .expect("Failed to store conversation");
    
    // Create tasks based on AI guidance
    pool.with_connection(|conn| {
        let now = Utc::now().to_rfc3339();
        let tasks = vec![
            ("req", "Requirements", "todo"),
            ("design", "Design", "todo"),
            ("impl", "Implementation", "todo"),
        ];
        
        for (id, title, status) in tasks {
            conn.execute(
                "INSERT INTO tasks (id, title, status, priority, created_at, updated_at) 
                 VALUES (?, ?, ?, 'high', ?, ?)",
                (id, title, status, now.clone(), now.clone()),
            )?;
        }
        Ok(())
    })
    .expect("test data setup");
    
    // Create dependencies
    let deps = vec![("req", "design"), ("design", "impl")];
    
    for (pred, succ) in deps {
        let input = DependencyCreateInput {
            predecessor_id: pred.to_string(),
            successor_id: succ.to_string(),
            dependency_type: None,
        };
        
        dependency_service
            .add_dependency(input)
            .await
            .expect("Failed to add dependency");
    }
    
    // Store memory of dependency creation
    let _conv2 = memory_service
        .store_conversation(
            "dependencies_created",
            "I've set up the dependencies",
            "Great! Your project has a clear workflow now",
            vec!["project management".to_string()],
        )
        .await
        .expect("Failed to store conversation");
    
    // Later, user asks for context
    let context = memory_service
        .search_memory("project setup", 5)
        .await
        .expect("Failed to search memory");
    
    assert!(!context.relevant_documents.is_empty());
    
    // Verify project structure
    let graph = dependency_service
        .get_dependency_graph(None)
        .await
        .expect("Failed to get graph");
    
    assert_eq!(graph.edges.len(), 2);
}

#[tokio::test]
async fn test_complex_workflow_with_multiple_features() {
    let (pool, recurring_service, dependency_service, memory_service, _dir) =
        setup_test_environment().await;
    
    // Create recurring task
    let template_input = RecurringTaskTemplateCreate {
        title: "Daily standup".to_string(),
        description: None,
        recurrence_rule_string: "FREQ=DAILY;INTERVAL=1".to_string(),
        priority: Some("medium".to_string()),
        tags: None,
        estimated_minutes: None,
    };
    
    let _template = recurring_service
        .create_template(template_input)
        .expect("Failed to create template");
    
    // Create regular tasks
    pool.with_connection(|conn| {
        let now = Utc::now().to_rfc3339();
        for i in 1..=5 {
            conn.execute(
                "INSERT INTO tasks (id, title, status, priority, created_at, updated_at) 
                 VALUES (?, ?, 'todo', 'medium', ?, ?)",
                (format!("task{}", i), format!("Task {}", i), now.clone(), now.clone()),
            )?;
        }
        Ok(())
    })
    .expect("test data setup");
    
    // Create dependencies
    for i in 1..=4 {
        let input = DependencyCreateInput {
            predecessor_id: format!("task{}", i),
            successor_id: format!("task{}", i + 1),
            dependency_type: None,
        };
        dependency_service
            .add_dependency(input)
            .await
            .expect("Failed to add dependency");
    }
    
    // Store memory
    let _ = memory_service
        .store_conversation(
            "workflow_setup",
            "I've set up my workflow",
            "Excellent! You have recurring standups and a task chain",
            vec!["workflow".to_string()],
        )
        .await;
    
    // Verify everything works together
    let graph = dependency_service
        .get_dependency_graph(None)
        .await
        .expect("Failed to get graph");
    
    assert_eq!(graph.edges.len(), 4);
    
    let context = memory_service
        .search_memory("workflow", 5)
        .await
        .expect("Failed to search memory");
    
    assert!(!context.relevant_documents.is_empty());
}

#[tokio::test]
async fn test_sequential_operations() {
    let (pool, _recurring_service, dependency_service, memory_service, _dir) =
        setup_test_environment().await;
    
    // Create tasks
    pool.with_connection(|conn| {
        let now = Utc::now().to_rfc3339();
        for i in 1..=20 {
            conn.execute(
                "INSERT INTO tasks (id, title, status, priority, created_at, updated_at) 
                 VALUES (?, ?, 'todo', 'medium', ?, ?)",
                (format!("task{}", i), format!("Task {}", i), now.clone(), now.clone()),
            )?;
        }
        Ok(())
    })
    .expect("test data setup");
    
    // Sequential dependency creation
    for i in 1..=10 {
        let input = DependencyCreateInput {
            predecessor_id: format!("task{}", i),
            successor_id: format!("task{}", i + 1),
            dependency_type: None,
        };
        let result = dependency_service.add_dependency(input).await;
        assert!(result.is_ok());
    }
    
    // Sequential memory storage
    for i in 1..=5 {
        let result = memory_service
            .store_conversation(
                &format!("sequential_conv_{}", i),
                &format!("Question {}", i),
                &format!("Answer {}", i),
                vec!["sequential".to_string()],
            )
            .await;
        assert!(result.is_ok());
    }
    
    // Verify final state
    let graph = dependency_service
        .get_dependency_graph(None)
        .await
        .expect("Failed to get graph");
    
    assert_eq!(graph.edges.len(), 10);
    
    let context = memory_service
        .search_memory("sequential", 10)
        .await
        .expect("Failed to search memory");
    
    assert_eq!(context.relevant_documents.len(), 5);
}
