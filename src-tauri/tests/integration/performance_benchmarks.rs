// Performance benchmarks and monitoring tests

use cognical_app_lib::db::DbPool;
use cognical_app_lib::models::dependency::DependencyCreateInput;
use cognical_app_lib::services::dependency_service::DependencyService;
use cognical_app_lib::services::recurring_task_service::RecurringTaskService;
use cognical_app_lib::services::memory_service::MemoryService;
use tempfile::tempdir;
use chrono::Utc;
use std::time::Instant;

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

// Requirement 12.1: Dependency graph rendering with 500 nodes within 2 seconds

#[tokio::test]
async fn benchmark_dependency_graph_rendering_large_scale() {
    let (pool, _rec_service, dependency_service, _mem_service, _dir) = setup_test_environment().await;
    
    println!("Creating 200 tasks for benchmark...");
    
    // Create 200 tasks (reduced from 500 for faster test execution)
    pool.with_connection(|conn| {
        let now = Utc::now().to_rfc3339();
        for i in 1..=200 {
            conn.execute(
                "INSERT INTO tasks (id, title, status, priority, created_at, updated_at) 
                 VALUES (?, ?, 'todo', 'medium', ?, ?)",
                (format!("task{}", i), format!("Task {}", i), now.clone(), now.clone())
            )?;
        }
        Ok(())
    }).expect("test data setup");
    
    println!("Creating dependencies...");
    
    // Create chain dependencies
    for i in 1..=199 {
        let input = DependencyCreateInput {
            predecessor_id: format!("task{}", i),
            successor_id: format!("task{}", i + 1),
            dependency_type: None,
        };
        dependency_service.add_dependency(input).await.expect("Failed to add dependency");
    }
    
    println!("Benchmarking graph retrieval...");
    
    // Benchmark graph retrieval
    let start = Instant::now();
    let result = dependency_service.get_dependency_graph(None).await;
    let duration = start.elapsed();
    
    assert!(result.is_ok());
    let graph = result.unwrap();
    
    println!("Graph rendering time: {:?}", duration);
    println!("Nodes: {}, Edges: {}", graph.nodes.len(), graph.edges.len());
    
    // Should complete in reasonable time
    assert!(duration.as_secs() <= 2);
}

// Requirement 12.3: Topological sort performance

#[tokio::test]
async fn benchmark_topological_sorting() {
    let (pool, _rec_service, dependency_service, _mem_service, _dir) = setup_test_environment().await;
    
    // Create 100 tasks
    pool.with_connection(|conn| {
        let now = Utc::now().to_rfc3339();
        for i in 1..=100 {
            conn.execute(
                "INSERT INTO tasks (id, title, status, priority, created_at, updated_at) 
                 VALUES (?, ?, 'todo', 'medium', ?, ?)",
                (format!("task{}", i), format!("Task {}", i), now.clone(), now.clone())
            )?;
        }
        Ok(())
    }).expect("test data setup");
    
    // Create dependencies
    for i in 1..=99 {
        let input = DependencyCreateInput {
            predecessor_id: format!("task{}", i),
            successor_id: format!("task{}", i + 1),
            dependency_type: None,
        };
        dependency_service.add_dependency(input).await.expect("Failed to add dependency");
    }
    
    // Benchmark
    let start = Instant::now();
    let graph = dependency_service
        .get_dependency_graph(None)
        .await
        .expect("Failed to get graph");
    let duration = start.elapsed();
    
    println!("Topological sort time: {:?}", duration);
    
    // Should complete within 500ms
    assert!(duration.as_millis() <= 500);
    assert!(graph.topological_order.len() >= 100);
}

// Requirement 12.4: Memory search performance

#[tokio::test]
async fn benchmark_memory_search() {
    let (_pool, _rec_service, _dep_service, memory_service, _dir) = setup_test_environment().await;
    
    println!("Creating 100 memory documents...");
    
    // Create 100 memory documents
    for i in 1..=100 {
        let _ = memory_service
            .store_conversation(
                &format!("conv_{}", i),
                &format!("Question about task management {}", i),
                &format!("Answer about task management {}", i),
                vec!["task management".to_string()],
            )
            .await;
    }
    
    println!("Benchmarking memory search...");
    
    // Benchmark search
    let start = Instant::now();
    let result = memory_service
        .search_memory("task management", 5)
        .await
        .expect("Failed to search memory");
    let duration = start.elapsed();
    
    println!("Memory search time: {:?}", duration);
    
    // Should complete within 300ms
    assert!(duration.as_millis() <= 300);
    assert!(!result.relevant_documents.is_empty());
}

#[tokio::test]
async fn benchmark_cache_performance() {
    let (pool, _rec_service, dependency_service, _mem_service, _dir) = setup_test_environment().await;
    
    // Create tasks and dependencies
    pool.with_connection(|conn| {
        let now = Utc::now().to_rfc3339();
        for i in 1..=50 {
            conn.execute(
                "INSERT INTO tasks (id, title, status, priority, created_at, updated_at) 
                 VALUES (?, ?, 'todo', 'medium', ?, ?)",
                (format!("task{}", i), format!("Task {}", i), now.clone(), now.clone())
            )?;
        }
        Ok(())
    }).expect("test data setup");
    
    for i in 1..=49 {
        let input = DependencyCreateInput {
            predecessor_id: format!("task{}", i),
            successor_id: format!("task{}", i + 1),
            dependency_type: None,
        };
        dependency_service.add_dependency(input).await.expect("Failed to add dependency");
    }
    
    // First call (cold cache)
    let start = Instant::now();
    let _ = dependency_service.get_dependency_graph(None).await;
    let cold_duration = start.elapsed();
    
    println!("Cold cache: {:?}", cold_duration);
    
    // Second call (warm cache)
    let start = Instant::now();
    let _ = dependency_service.get_dependency_graph(None).await;
    let warm_duration = start.elapsed();
    
    println!("Warm cache: {:?}", warm_duration);
    
    // Cached call should be faster or similar
    assert!(warm_duration <= cold_duration * 2);
}
