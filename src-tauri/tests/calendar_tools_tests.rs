use cognical_app_lib::db::DbPool;
use cognical_app_lib::services::ai_service::AiService;
use cognical_app_lib::services::planning_service::PlanningService;
use cognical_app_lib::services::task_service::TaskService;
use cognical_app_lib::tools::calendar_tools::*;
use serde_json::json;
use std::sync::Arc;
use tempfile::tempdir;

/// Helper function to set up a test environment with PlanningService
fn setup_test_service() -> (Arc<PlanningService>, tempfile::TempDir) {
    let dir = tempdir().expect("Failed to create temp dir");
    let db_path = dir.path().join("test_calendar.sqlite");
    let pool = DbPool::new(&db_path).expect("Failed to create DB pool");
    
    let task_service = Arc::new(TaskService::new(pool.clone()));
    let ai_service = Arc::new(AiService::new(pool.clone()).expect("Failed to create AI service"));
    let planning_service = Arc::new(PlanningService::new(
        pool,
        task_service,
        ai_service,
    ));
    
    (planning_service, dir)
}

#[tokio::test]
async fn test_get_calendar_events_tool_success() {
    let (service, _dir) = setup_test_service();

    let args = json!({
        "start_date": "2024-01-01",
        "end_date": "2024-01-31"
    });

    let result = get_calendar_events_tool(service.clone(), args).await;
    assert!(result.is_ok(), "get_calendar_events_tool should succeed");

    let result_json = result.unwrap();
    assert_eq!(result_json["success"], true);
    assert!(result_json["message"].is_string());
    assert_eq!(result_json["start_date"], "2024-01-01");
    assert_eq!(result_json["end_date"], "2024-01-31");
    assert!(result_json["count"].is_number());
    assert!(result_json["events"].is_array());
}

#[tokio::test]
async fn test_get_calendar_events_tool_invalid_date_format() {
    let (service, _dir) = setup_test_service();

    let args = json!({
        "start_date": "01-01-2024",  // Wrong format
        "end_date": "2024-01-31"
    });

    let result = get_calendar_events_tool(service.clone(), args).await;
    assert!(result.is_err(), "get_calendar_events_tool should fail with invalid date format");
}

#[tokio::test]
async fn test_get_calendar_events_tool_end_before_start() {
    let (service, _dir) = setup_test_service();

    let args = json!({
        "start_date": "2024-01-31",
        "end_date": "2024-01-01"  // End before start
    });

    let result = get_calendar_events_tool(service.clone(), args).await;
    assert!(result.is_err(), "get_calendar_events_tool should fail when end date is before start date");
}

#[tokio::test]
async fn test_get_calendar_events_tool_with_event_type_filter() {
    let (service, _dir) = setup_test_service();

    let args = json!({
        "start_date": "2024-01-01",
        "end_date": "2024-01-31",
        "event_type": "meeting"
    });

    let result = get_calendar_events_tool(service.clone(), args).await;
    assert!(result.is_ok(), "get_calendar_events_tool should succeed with event_type filter");

    let result_json = result.unwrap();
    assert_eq!(result_json["success"], true);
}

#[tokio::test]
async fn test_create_calendar_event_tool_success() {
    let (service, _dir) = setup_test_service();

    let args = json!({
        "title": "Team Meeting",
        "date": "2024-06-15",
        "start_time": "14:30",
        "duration_minutes": 60,
        "event_type": "meeting"
    });

    let result = create_calendar_event_tool(service.clone(), args).await;
    assert!(result.is_ok(), "create_calendar_event_tool should succeed");

    let result_json = result.unwrap();
    assert_eq!(result_json["success"], true);
    assert!(result_json["message"].as_str().unwrap().contains("created successfully"));
    assert!(result_json["event"]["id"].is_string());
    assert!(result_json["event"]["start_at"].is_string());
    assert!(result_json["event"]["end_at"].is_string());
    assert_eq!(result_json["has_conflicts"], false);
}

#[tokio::test]
async fn test_create_calendar_event_tool_invalid_date() {
    let (service, _dir) = setup_test_service();

    let args = json!({
        "title": "Team Meeting",
        "date": "15-06-2024",  // Wrong format
        "start_time": "14:30",
        "duration_minutes": 60
    });

    let result = create_calendar_event_tool(service.clone(), args).await;
    assert!(result.is_err(), "create_calendar_event_tool should fail with invalid date format");
}

#[tokio::test]
async fn test_create_calendar_event_tool_invalid_time() {
    let (service, _dir) = setup_test_service();

    let args = json!({
        "title": "Team Meeting",
        "date": "2024-06-15",
        "start_time": "2:30 PM",  // Wrong format
        "duration_minutes": 60
    });

    let result = create_calendar_event_tool(service.clone(), args).await;
    assert!(result.is_err(), "create_calendar_event_tool should fail with invalid time format");
}

#[tokio::test]
async fn test_create_calendar_event_tool_invalid_duration() {
    let (service, _dir) = setup_test_service();

    let args = json!({
        "title": "Team Meeting",
        "date": "2024-06-15",
        "start_time": "14:30",
        "duration_minutes": 0  // Invalid duration
    });

    let result = create_calendar_event_tool(service.clone(), args).await;
    assert!(result.is_err(), "create_calendar_event_tool should fail with zero duration");
}

#[tokio::test]
async fn test_create_calendar_event_tool_negative_duration() {
    let (service, _dir) = setup_test_service();

    let args = json!({
        "title": "Team Meeting",
        "date": "2024-06-15",
        "start_time": "14:30",
        "duration_minutes": -30  // Negative duration
    });

    let result = create_calendar_event_tool(service.clone(), args).await;
    assert!(result.is_err(), "create_calendar_event_tool should fail with negative duration");
}

#[tokio::test]
async fn test_update_calendar_event_tool_not_found() {
    let (service, _dir) = setup_test_service();

    let args = json!({
        "event_id": "non-existent-id",
        "title": "Updated Meeting"
    });

    let result = update_calendar_event_tool(service.clone(), args).await;
    assert!(result.is_err(), "update_calendar_event_tool should fail for non-existent event");
    
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("not found"), "Error should mention event not found");
}

#[tokio::test]
async fn test_date_time_parsing_edge_cases() {
    let (service, _dir) = setup_test_service();

    // Test midnight
    let args = json!({
        "title": "Midnight Event",
        "date": "2024-06-15",
        "start_time": "00:00",
        "duration_minutes": 30
    });

    let result = create_calendar_event_tool(service.clone(), args).await;
    assert!(result.is_ok(), "Should handle midnight time");

    // Test end of day
    let args = json!({
        "title": "Late Event",
        "date": "2024-06-15",
        "start_time": "23:30",
        "duration_minutes": 30
    });

    let result = create_calendar_event_tool(service.clone(), args).await;
    assert!(result.is_ok(), "Should handle late evening time");
}

#[tokio::test]
async fn test_calendar_event_schemas() {
    // Test that schemas are valid JSON
    let get_schema = get_calendar_events_schema();
    assert!(get_schema.is_object());
    assert!(get_schema["properties"].is_object());
    assert!(get_schema["required"].is_array());

    let create_schema = create_calendar_event_schema();
    assert!(create_schema.is_object());
    assert!(create_schema["properties"].is_object());
    assert!(create_schema["required"].is_array());

    let update_schema = update_calendar_event_schema();
    assert!(update_schema.is_object());
    assert!(update_schema["properties"].is_object());
    assert!(update_schema["required"].is_array());
}
