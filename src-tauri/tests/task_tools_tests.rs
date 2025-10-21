use cognical_app_lib::db::DbPool;
use cognical_app_lib::services::task_service::TaskService;
use cognical_app_lib::tools::task_tools::*;
use serde_json::json;
use std::sync::Arc;
use tempfile::tempdir;

/// Helper function to set up a test environment with TaskService
fn setup_test_service() -> (Arc<TaskService>, tempfile::TempDir) {
    let dir = tempdir().expect("Failed to create temp dir");
    let db_path = dir.path().join("test_tasks.sqlite");
    let pool = DbPool::new(&db_path).expect("Failed to create DB pool");
    let service = Arc::new(TaskService::new(pool));
    (service, dir)
}

#[tokio::test]
async fn test_create_task_tool_success() {
    let (service, _dir) = setup_test_service();

    let args = json!({
        "title": "Test Task",
        "description": "This is a test task",
        "priority": "high",
        "status": "todo",
        "tags": ["test", "automation"]
    });

    let result = create_task_tool(service.clone(), args).await;
    assert!(result.is_ok(), "create_task_tool should succeed");

    let result_json = result.unwrap();
    assert_eq!(result_json["success"], true);
    assert!(result_json["message"].as_str().unwrap().contains("created successfully"));
    assert!(result_json["task"]["id"].is_string());
    assert_eq!(result_json["task"]["title"], "Test Task");
    assert_eq!(result_json["task"]["priority"], "high");
}

#[tokio::test]
async fn test_create_task_tool_missing_title() {
    let (service, _dir) = setup_test_service();

    let args = json!({
        "description": "Task without title"
    });

    let result = create_task_tool(service.clone(), args).await;
    assert!(result.is_err(), "create_task_tool should fail without title");
}

#[tokio::test]
async fn test_create_task_tool_invalid_priority() {
    let (service, _dir) = setup_test_service();

    let args = json!({
        "title": "Test Task",
        "priority": "invalid_priority"
    });

    let result = create_task_tool(service.clone(), args).await;
    assert!(result.is_err(), "create_task_tool should fail with invalid priority");
}

#[tokio::test]
async fn test_update_task_tool_success() {
    let (service, _dir) = setup_test_service();

    // First create a task
    let create_args = json!({
        "title": "Original Title",
        "priority": "low"
    });

    let create_result = create_task_tool(service.clone(), create_args).await.unwrap();
    let task_id = create_result["task"]["id"].as_str().unwrap();

    // Now update it
    let update_args = json!({
        "task_id": task_id,
        "title": "Updated Title",
        "priority": "high",
        "status": "in_progress"
    });

    let result = update_task_tool(service.clone(), update_args).await;
    assert!(result.is_ok(), "update_task_tool should succeed");

    let result_json = result.unwrap();
    assert_eq!(result_json["success"], true);
    assert_eq!(result_json["task"]["title"], "Updated Title");
    assert_eq!(result_json["task"]["priority"], "high");
    assert_eq!(result_json["task"]["status"], "in_progress");
}

#[tokio::test]
async fn test_update_task_tool_not_found() {
    let (service, _dir) = setup_test_service();

    let args = json!({
        "task_id": "non-existent-id",
        "title": "Updated Title"
    });

    let result = update_task_tool(service.clone(), args).await;
    assert!(result.is_err(), "update_task_tool should fail for non-existent task");
    
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("not found"), "Error should mention task not found");
}

#[tokio::test]
async fn test_delete_task_tool_success() {
    let (service, _dir) = setup_test_service();

    // First create a task
    let create_args = json!({
        "title": "Task to Delete"
    });

    let create_result = create_task_tool(service.clone(), create_args).await.unwrap();
    let task_id = create_result["task"]["id"].as_str().unwrap();

    // Now delete it
    let delete_args = json!({
        "task_id": task_id
    });

    let result = delete_task_tool(service.clone(), delete_args).await;
    assert!(result.is_ok(), "delete_task_tool should succeed");

    let result_json = result.unwrap();
    assert_eq!(result_json["success"], true);
    assert!(result_json["message"].as_str().unwrap().contains("deleted successfully"));
}

#[tokio::test]
async fn test_delete_task_tool_not_found() {
    let (service, _dir) = setup_test_service();

    let args = json!({
        "task_id": "non-existent-id"
    });

    let result = delete_task_tool(service.clone(), args).await;
    assert!(result.is_err(), "delete_task_tool should fail for non-existent task");
}

#[tokio::test]
async fn test_list_tasks_tool_empty() {
    let (service, _dir) = setup_test_service();

    let args = json!({});

    let result = list_tasks_tool(service.clone(), args).await;
    assert!(result.is_ok(), "list_tasks_tool should succeed");

    let result_json = result.unwrap();
    assert_eq!(result_json["success"], true);
    assert_eq!(result_json["count"], 0);
    assert!(result_json["message"].as_str().unwrap().contains("No tasks found"));
}

#[tokio::test]
async fn test_list_tasks_tool_with_tasks() {
    let (service, _dir) = setup_test_service();

    // Create multiple tasks
    create_task_tool(service.clone(), json!({"title": "Task 1", "priority": "high", "status": "todo"})).await.unwrap();
    create_task_tool(service.clone(), json!({"title": "Task 2", "priority": "low", "status": "done"})).await.unwrap();
    create_task_tool(service.clone(), json!({"title": "Task 3", "priority": "high", "status": "in_progress"})).await.unwrap();

    let args = json!({});

    let result = list_tasks_tool(service.clone(), args).await;
    assert!(result.is_ok(), "list_tasks_tool should succeed");

    let result_json = result.unwrap();
    assert_eq!(result_json["success"], true);
    assert_eq!(result_json["count"], 3);
    assert!(result_json["message"].as_str().unwrap().contains("Found 3 task"));
}

#[tokio::test]
async fn test_list_tasks_tool_with_status_filter() {
    let (service, _dir) = setup_test_service();

    // Create tasks with different statuses
    create_task_tool(service.clone(), json!({"title": "Task 1", "status": "todo"})).await.unwrap();
    create_task_tool(service.clone(), json!({"title": "Task 2", "status": "done"})).await.unwrap();
    create_task_tool(service.clone(), json!({"title": "Task 3", "status": "todo"})).await.unwrap();

    let args = json!({
        "status": "todo"
    });

    let result = list_tasks_tool(service.clone(), args).await;
    assert!(result.is_ok(), "list_tasks_tool should succeed");

    let result_json = result.unwrap();
    assert_eq!(result_json["success"], true);
    assert_eq!(result_json["count"], 2);
}

#[tokio::test]
async fn test_list_tasks_tool_with_priority_filter() {
    let (service, _dir) = setup_test_service();

    // Create tasks with different priorities
    create_task_tool(service.clone(), json!({"title": "Task 1", "priority": "high"})).await.unwrap();
    create_task_tool(service.clone(), json!({"title": "Task 2", "priority": "low"})).await.unwrap();
    create_task_tool(service.clone(), json!({"title": "Task 3", "priority": "high"})).await.unwrap();

    let args = json!({
        "priority": "high"
    });

    let result = list_tasks_tool(service.clone(), args).await;
    assert!(result.is_ok(), "list_tasks_tool should succeed");

    let result_json = result.unwrap();
    assert_eq!(result_json["success"], true);
    assert_eq!(result_json["count"], 2);
}

#[tokio::test]
async fn test_list_tasks_tool_with_tag_filter() {
    let (service, _dir) = setup_test_service();

    // Create tasks with different tags
    create_task_tool(service.clone(), json!({"title": "Task 1", "tags": ["urgent", "bug"]})).await.unwrap();
    create_task_tool(service.clone(), json!({"title": "Task 2", "tags": ["feature"]})).await.unwrap();
    create_task_tool(service.clone(), json!({"title": "Task 3", "tags": ["urgent", "feature"]})).await.unwrap();

    let args = json!({
        "tag": "urgent"
    });

    let result = list_tasks_tool(service.clone(), args).await;
    assert!(result.is_ok(), "list_tasks_tool should succeed");

    let result_json = result.unwrap();
    assert_eq!(result_json["success"], true);
    assert_eq!(result_json["count"], 2);
}

#[tokio::test]
async fn test_search_tasks_tool_by_title() {
    let (service, _dir) = setup_test_service();

    // Create tasks
    create_task_tool(service.clone(), json!({"title": "Fix login bug"})).await.unwrap();
    create_task_tool(service.clone(), json!({"title": "Add new feature"})).await.unwrap();
    create_task_tool(service.clone(), json!({"title": "Fix registration bug"})).await.unwrap();

    let args = json!({
        "query": "bug"
    });

    let result = search_tasks_tool(service.clone(), args).await;
    assert!(result.is_ok(), "search_tasks_tool should succeed");

    let result_json = result.unwrap();
    assert_eq!(result_json["success"], true);
    assert_eq!(result_json["count"], 2);
    assert_eq!(result_json["query"], "bug");
}

#[tokio::test]
async fn test_search_tasks_tool_by_description() {
    let (service, _dir) = setup_test_service();

    // Create tasks with descriptions
    create_task_tool(service.clone(), json!({"title": "Task 1", "description": "Implement authentication"})).await.unwrap();
    create_task_tool(service.clone(), json!({"title": "Task 2", "description": "Add logging"})).await.unwrap();
    create_task_tool(service.clone(), json!({"title": "Task 3", "description": "Implement authorization"})).await.unwrap();

    let args = json!({
        "query": "Implement"
    });

    let result = search_tasks_tool(service.clone(), args).await;
    assert!(result.is_ok(), "search_tasks_tool should succeed");

    let result_json = result.unwrap();
    assert_eq!(result_json["success"], true);
    assert_eq!(result_json["count"], 2);
}

#[tokio::test]
async fn test_search_tasks_tool_no_results() {
    let (service, _dir) = setup_test_service();

    // Create tasks
    create_task_tool(service.clone(), json!({"title": "Task 1"})).await.unwrap();
    create_task_tool(service.clone(), json!({"title": "Task 2"})).await.unwrap();

    let args = json!({
        "query": "nonexistent"
    });

    let result = search_tasks_tool(service.clone(), args).await;
    assert!(result.is_ok(), "search_tasks_tool should succeed");

    let result_json = result.unwrap();
    assert_eq!(result_json["success"], true);
    assert_eq!(result_json["count"], 0);
    assert!(result_json["message"].as_str().unwrap().contains("No tasks found"));
}

#[tokio::test]
async fn test_search_tasks_tool_with_filters() {
    let (service, _dir) = setup_test_service();

    // Create tasks
    create_task_tool(service.clone(), json!({"title": "Bug fix 1", "status": "todo", "priority": "high"})).await.unwrap();
    create_task_tool(service.clone(), json!({"title": "Bug fix 2", "status": "done", "priority": "low"})).await.unwrap();
    create_task_tool(service.clone(), json!({"title": "Bug fix 3", "status": "todo", "priority": "high"})).await.unwrap();

    let args = json!({
        "query": "Bug",
        "status": "todo",
        "priority": "high"
    });

    let result = search_tasks_tool(service.clone(), args).await;
    assert!(result.is_ok(), "search_tasks_tool should succeed");

    let result_json = result.unwrap();
    assert_eq!(result_json["success"], true);
    assert_eq!(result_json["count"], 2);
}
