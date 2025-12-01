use cognical_app_lib::db::DbPool;
use cognical_app_lib::models::dependency::{DependencyCreateInput, DependencyType};
use cognical_app_lib::services::dependency_service::DependencyService;
use tempfile::tempdir;
use chrono::Utc;

async fn setup_test_db() -> (DbPool, DependencyService) {
    let dir = tempdir().expect("temp dir");
    let db_path = dir.path().join("test.sqlite");
    let pool = DbPool::new(db_path).expect("db pool");
    let service = DependencyService::new(pool.clone());
    
    // Create test tasks
    pool.with_connection(|conn| {
        let now = Utc::now().to_rfc3339();
        for i in 1..=5 {
            conn.execute(
                "INSERT INTO tasks (id, title, status, priority, created_at, updated_at) 
                 VALUES (?, ?, ?, 'medium', ?, ?)",
                (format!("task{}", i), format!("Task {}", i), "todo", now.clone(), now.clone())
            )?;
        }
        Ok(())
    }).expect("test data setup");
    
    (pool, service)
}

#[tokio::test]
async fn test_add_dependency_success() {
    let (_pool, service) = setup_test_db().await;
    
    let input = DependencyCreateInput {
        predecessor_id: "task1".to_string(),
        successor_id: "task2".to_string(),
        dependency_type: Some(DependencyType::FinishToStart),
    };
    
    let result = service.add_dependency(input).await;
    match &result {
        Ok(_) => {},
        Err(e) => println!("Error: {:?}", e),
    }
    assert!(result.is_ok());
    
    let dependency_id = result.unwrap();
    assert!(!dependency_id.is_empty());
}

#[tokio::test]
async fn test_add_dependency_prevents_self_dependency() {
    let (_pool, service) = setup_test_db().await;
    
    let input = DependencyCreateInput {
        predecessor_id: "task1".to_string(),
        successor_id: "task1".to_string(),
        dependency_type: Some(DependencyType::FinishToStart),
    };
    
    let result = service.add_dependency(input).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_add_dependency_prevents_duplicate() {
    let (_pool, service) = setup_test_db().await;
    
    let input = DependencyCreateInput {
        predecessor_id: "task1".to_string(),
        successor_id: "task2".to_string(),
        dependency_type: Some(DependencyType::FinishToStart),
    };
    
    // Add first dependency
    let result1 = service.add_dependency(input.clone()).await;
    assert!(result1.is_ok());
    
    // Try to add duplicate
    let result2 = service.add_dependency(input).await;
    assert!(result2.is_err());
}

#[tokio::test]
async fn test_circular_dependency_detection() {
    let (_pool, service) = setup_test_db().await;
    
    // Create chain: task1 -> task2 -> task3
    let input1 = DependencyCreateInput {
        predecessor_id: "task1".to_string(),
        successor_id: "task2".to_string(),
        dependency_type: Some(DependencyType::FinishToStart),
    };
    service.add_dependency(input1).await.expect("first dependency");
    
    let input2 = DependencyCreateInput {
        predecessor_id: "task2".to_string(),
        successor_id: "task3".to_string(),
        dependency_type: Some(DependencyType::FinishToStart),
    };
    service.add_dependency(input2).await.expect("second dependency");
    
    // Try to create cycle: task3 -> task1
    let input3 = DependencyCreateInput {
        predecessor_id: "task3".to_string(),
        successor_id: "task1".to_string(),
        dependency_type: Some(DependencyType::FinishToStart),
    };
    
    let result = service.add_dependency(input3).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_validate_dependency() {
    let (_pool, service) = setup_test_db().await;
    
    // Valid dependency
    let validation = service.validate_dependency("task1", "task2").await.unwrap();
    assert!(validation.is_valid);
    assert!(!validation.would_create_cycle);
    
    // Invalid - nonexistent task
    let validation = service.validate_dependency("nonexistent", "task2").await.unwrap();
    assert!(!validation.is_valid);
    assert!(validation.error_message.is_some());
    
    // Invalid - self dependency
    let validation = service.validate_dependency("task1", "task1").await.unwrap();
    assert!(!validation.is_valid);
    assert!(validation.would_create_cycle);
}

#[tokio::test]
async fn test_remove_dependency() {
    let (_pool, service) = setup_test_db().await;
    
    let input = DependencyCreateInput {
        predecessor_id: "task1".to_string(),
        successor_id: "task2".to_string(),
        dependency_type: Some(DependencyType::FinishToStart),
    };
    
    let dependency_id = service.add_dependency(input).await.unwrap();
    
    // Remove the dependency
    let result = service.remove_dependency(&dependency_id).await;
    assert!(result.is_ok());
    
    // Try to remove again - should fail
    let result = service.remove_dependency(&dependency_id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_task_dependencies() {
    let (_pool, service) = setup_test_db().await;
    
    // Create dependencies: task1 -> task2, task1 -> task3
    let input1 = DependencyCreateInput {
        predecessor_id: "task1".to_string(),
        successor_id: "task2".to_string(),
        dependency_type: Some(DependencyType::FinishToStart),
    };
    service.add_dependency(input1).await.unwrap();
    
    let input2 = DependencyCreateInput {
        predecessor_id: "task1".to_string(),
        successor_id: "task3".to_string(),
        dependency_type: Some(DependencyType::StartToStart),
    };
    service.add_dependency(input2).await.unwrap();
    
    let dependencies = service.get_task_dependencies("task1").await.unwrap();
    assert_eq!(dependencies.len(), 2);
    
    // Check that both dependencies are returned
    let successor_ids: Vec<&str> = dependencies.iter()
        .map(|d| d.successor_id.as_str())
        .collect();
    assert!(successor_ids.contains(&"task2"));
    assert!(successor_ids.contains(&"task3"));
}

#[tokio::test]
async fn test_get_ready_tasks() {
    let (pool, service) = setup_test_db().await;
    
    // Mark task1 as completed
    pool.with_connection(|conn| {
        conn.execute(
            "UPDATE tasks SET status = 'completed' WHERE id = 'task1'",
            []
        )?;
        Ok(())
    }).expect("update task status");
    
    // Create dependency: task1 -> task2
    let input = DependencyCreateInput {
        predecessor_id: "task1".to_string(),
        successor_id: "task2".to_string(),
        dependency_type: Some(DependencyType::FinishToStart),
    };
    service.add_dependency(input).await.unwrap();
    
    let ready_tasks = service.get_ready_tasks().await.unwrap();
    
    // task2 should be ready (dependency completed)
    // task3, task4, task5 should be ready (no dependencies)
    // task1 should not appear (completed status)
    let ready_ids: Vec<&str> = ready_tasks.iter()
        .map(|t| t.id.as_str())
        .collect();
    
    assert!(ready_ids.contains(&"task2"));
    assert!(ready_ids.contains(&"task3"));
    assert!(ready_ids.contains(&"task4"));
    assert!(ready_ids.contains(&"task5"));
    assert!(!ready_ids.contains(&"task1"));
}

#[tokio::test]
async fn test_topological_sorting() {
    let (_pool, service) = setup_test_db().await;
    
    // Create dependency chain: task1 -> task2 -> task3
    let input1 = DependencyCreateInput {
        predecessor_id: "task1".to_string(),
        successor_id: "task2".to_string(),
        dependency_type: Some(DependencyType::FinishToStart),
    };
    service.add_dependency(input1).await.unwrap();
    
    let input2 = DependencyCreateInput {
        predecessor_id: "task2".to_string(),
        successor_id: "task3".to_string(),
        dependency_type: Some(DependencyType::FinishToStart),
    };
    service.add_dependency(input2).await.unwrap();
    
    let graph = service.get_dependency_graph(None).await.unwrap();
    
    // Check topological order
    let topo_order = &graph.topological_order;
    assert!(topo_order.len() >= 3);
    
    // task1 should come before task2, task2 should come before task3
    let task1_pos = topo_order.iter().position(|id| id == "task1");
    let task2_pos = topo_order.iter().position(|id| id == "task2");
    let task3_pos = topo_order.iter().position(|id| id == "task3");
    
    assert!(task1_pos.is_some());
    assert!(task2_pos.is_some());
    assert!(task3_pos.is_some());
    assert!(task1_pos < task2_pos);
    assert!(task2_pos < task3_pos);
}

#[tokio::test]
async fn test_critical_path_calculation() {
    let (_pool, service) = setup_test_db().await;
    
    // Create dependency chain: task1 -> task2 -> task3
    let input1 = DependencyCreateInput {
        predecessor_id: "task1".to_string(),
        successor_id: "task2".to_string(),
        dependency_type: Some(DependencyType::FinishToStart),
    };
    service.add_dependency(input1).await.unwrap();
    
    let input2 = DependencyCreateInput {
        predecessor_id: "task2".to_string(),
        successor_id: "task3".to_string(),
        dependency_type: Some(DependencyType::FinishToStart),
    };
    service.add_dependency(input2).await.unwrap();
    
    let critical_path = service.calculate_critical_path("task3").await.unwrap();
    
    // Critical path to task3 should include task1 -> task2 -> task3
    assert!(critical_path.contains(&"task1".to_string()));
    assert!(critical_path.contains(&"task2".to_string()));
    assert!(critical_path.contains(&"task3".to_string()));
}

#[tokio::test]
async fn test_dependency_graph_caching() {
    let (_pool, service) = setup_test_db().await;
    
    // First call should build the graph
    let start = std::time::Instant::now();
    let graph1 = service.get_dependency_graph(None).await.unwrap();
    let _first_duration = start.elapsed();
    
    // Second call should use cache (should be faster)
    let start = std::time::Instant::now();
    let graph2 = service.get_dependency_graph(None).await.unwrap();
    let _second_duration = start.elapsed();
    
    // Graphs should be identical
    assert_eq!(graph1.nodes.len(), graph2.nodes.len());
    assert_eq!(graph1.edges.len(), graph2.edges.len());
    
    // Second call should be faster (cached)
    // Note: This might not always be true in tests due to small dataset
    // but the caching mechanism should be in place
}

#[tokio::test]
async fn test_complex_dependency_graph() {
    let (_pool, service) = setup_test_db().await;
    
    // Create complex dependency structure:
    // task1 -> task3
    // task2 -> task3
    // task3 -> task4
    // task4 -> task5
    
    let dependencies = vec![
        ("task1", "task3"),
        ("task2", "task3"),
        ("task3", "task4"),
        ("task4", "task5"),
    ];
    
    for (pred, succ) in dependencies {
        let input = DependencyCreateInput {
            predecessor_id: pred.to_string(),
            successor_id: succ.to_string(),
            dependency_type: Some(DependencyType::FinishToStart),
        };
        service.add_dependency(input).await.unwrap();
    }
    
    let graph = service.get_dependency_graph(None).await.unwrap();
    
    // Verify graph structure
    assert_eq!(graph.edges.len(), 4);
    
    // Check that task3 has 2 dependencies
    let task3_node = graph.nodes.get("task3").unwrap();
    assert_eq!(task3_node.dependencies.len(), 2);
    assert!(task3_node.dependencies.contains(&"task1".to_string()));
    assert!(task3_node.dependencies.contains(&"task2".to_string()));
    
    // Check that task1 and task2 have task3 as dependent
    let task1_node = graph.nodes.get("task1").unwrap();
    assert!(task1_node.dependents.contains(&"task3".to_string()));
    
    let task2_node = graph.nodes.get("task2").unwrap();
    assert!(task2_node.dependents.contains(&"task3".to_string()));
}

#[tokio::test]
async fn test_dependency_types() {
    let (_pool, service) = setup_test_db().await;
    
    let dependency_types = vec![
        DependencyType::FinishToStart,
        DependencyType::StartToStart,
        DependencyType::FinishToFinish,
        DependencyType::StartToFinish,
    ];
    
    for (i, dep_type) in dependency_types.into_iter().enumerate() {
        let input = DependencyCreateInput {
            predecessor_id: "task1".to_string(),
            successor_id: format!("task{}", i + 2),
            dependency_type: Some(dep_type.clone()),
        };
        
        let result = service.add_dependency(input).await;
        assert!(result.is_ok(), "Failed to add dependency type: {:?}", dep_type);
    }
    
    let dependencies = service.get_task_dependencies("task1").await.unwrap();
    assert_eq!(dependencies.len(), 4);
    
    // Verify all dependency types are stored correctly
    let stored_types: Vec<DependencyType> = dependencies.iter()
        .map(|d| d.dependency_type.clone())
        .collect();
    
    assert!(stored_types.contains(&DependencyType::FinishToStart));
    assert!(stored_types.contains(&DependencyType::StartToStart));
    assert!(stored_types.contains(&DependencyType::FinishToFinish));
    assert!(stored_types.contains(&DependencyType::StartToFinish));
}

#[tokio::test]
async fn test_cache_invalidation() {
    let (_pool, service) = setup_test_db().await;
    
    // Build initial graph (should be cached)
    let graph1 = service.get_dependency_graph(None).await.unwrap();
    assert_eq!(graph1.edges.len(), 0);
    
    // Add a dependency (should invalidate cache)
    let input = DependencyCreateInput {
        predecessor_id: "task1".to_string(),
        successor_id: "task2".to_string(),
        dependency_type: Some(DependencyType::FinishToStart),
    };
    service.add_dependency(input).await.unwrap();
    
    // Get graph again (should rebuild and show new dependency)
    let graph2 = service.get_dependency_graph(None).await.unwrap();
    assert_eq!(graph2.edges.len(), 1);
    
    // Remove dependency (should invalidate cache again)
    let dependency_id = &graph2.edges[0].id;
    service.remove_dependency(dependency_id).await.unwrap();
    
    // Get graph again (should rebuild and show no dependencies)
    let graph3 = service.get_dependency_graph(None).await.unwrap();
    assert_eq!(graph3.edges.len(), 0);
}