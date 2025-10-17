use chrono::{Duration, Utc};
use cognical_app_lib::db::repositories::workload_repository::WorkloadRepository;
use cognical_app_lib::db::DbPool;
use cognical_app_lib::models::task::TaskCreateInput;
use cognical_app_lib::models::workload::WorkloadHorizon;
use cognical_app_lib::services::task_service::TaskService;
use cognical_app_lib::services::workload_forecast_service::WorkloadForecastService;
use std::sync::Arc;
use tempfile::NamedTempFile;

#[test]
fn test_workload_forecast_generation() {
    // Setup test database
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path();
    let pool = DbPool::new(db_path).unwrap();

    // Create services
    let task_service = Arc::new(TaskService::new(pool.clone()));
    let forecast_service = WorkloadForecastService::new(pool.clone(), task_service.clone());

    // Create test tasks with due dates in next 7 days
    let now = Utc::now();

    for i in 0..5 {
        task_service
            .create_task(TaskCreateInput {
                title: format!("Test Task {}", i),
                description: Some("Test description".to_string()),
                status: Some("todo".to_string()),
                priority: Some("high".to_string()),
                planned_start_at: None,
                start_at: None,
                due_at: Some((now + Duration::days(i + 1)).to_rfc3339()),
                completed_at: None,
                estimated_minutes: None,
                estimated_hours: Some(8.0),
                tags: None,
                owner_id: None,
                task_type: None,
                is_recurring: None,
                recurrence: None,
                ai: None,
                external_links: None,
            })
            .unwrap();
    }

    // Generate forecasts
    let forecasts = forecast_service.generate_forecasts(Some(40.0)).unwrap();

    // Should generate 3 forecasts (7d, 14d, 30d)
    assert_eq!(forecasts.len(), 3);

    // Check 7-day forecast
    let seven_day = forecasts.iter().find(|f| f.horizon == "7d").unwrap();
    assert_eq!(seven_day.total_hours, 40.0); // 5 tasks * 8 hours
    assert!(seven_day.confidence >= 0.0 && seven_day.confidence <= 1.0);
    assert_eq!(seven_day.contributing_tasks.len(), 5);

    // Check risk level calculation
    assert!(matches!(
        seven_day.risk_level.as_str(),
        "ok" | "warning" | "critical"
    ));
}

#[test]
fn test_workload_forecast_confidence_calculation() {
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path();
    let pool = DbPool::new(db_path).unwrap();

    let task_service = Arc::new(TaskService::new(pool.clone()));
    let forecast_service = WorkloadForecastService::new(pool.clone(), task_service);

    // Generate forecast with no historical data
    let forecasts = forecast_service.generate_forecasts(None).unwrap();

    // Should have low confidence with no history
    for forecast in forecasts {
        // Confidence should be between 0.2 and 0.4 (low confidence range)
        assert!(forecast.confidence >= 0.2 && forecast.confidence <= 0.5);
    }
}

#[test]
fn test_workload_forecast_risk_levels() {
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path();
    let pool = DbPool::new(db_path).unwrap();

    let task_service = Arc::new(TaskService::new(pool.clone()));
    let forecast_service = WorkloadForecastService::new(pool.clone(), task_service.clone());

    let now = Utc::now();

    // Create tasks that exceed capacity (60 hours with 40h threshold)
    for i in 0..6 {
        task_service
            .create_task(TaskCreateInput {
                title: format!("Overload Task {}", i),
                description: None,
                status: Some("todo".to_string()),
                priority: Some("high".to_string()),
                planned_start_at: None,
                start_at: None,
                due_at: Some((now + Duration::days(i + 1)).to_rfc3339()),
                completed_at: None,
                estimated_minutes: None,
                estimated_hours: Some(10.0),
                tags: None,
                owner_id: None,
                task_type: None,
                is_recurring: None,
                recurrence: None,
                ai: None,
                external_links: None,
            })
            .unwrap();
    }

    // Generate forecasts with 30h threshold (lower to trigger warning)
    let forecasts = forecast_service.generate_forecasts(Some(30.0)).unwrap();

    let seven_day = forecasts.iter().find(|f| f.horizon == "7d").unwrap();
    assert_eq!(seven_day.total_hours, 60.0); // 6 tasks * 10 hours

    // With low confidence and 60/30 = 2.0 (> 1.5), should be warning
    assert!(
        seven_day.risk_level == "warning" || seven_day.risk_level == "critical",
        "Expected warning or critical with 200% utilization, got: {}",
        seven_day.risk_level
    );

    // Check risk level is not "ok" since we're overloaded
    assert_ne!(
        seven_day.risk_level, "ok",
        "Should not be ok with 200% utilization"
    );
}

#[test]
fn test_workload_forecast_persistence() {
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path();
    let pool = DbPool::new(db_path).unwrap();

    let task_service = Arc::new(TaskService::new(pool.clone()));
    let forecast_service = WorkloadForecastService::new(pool.clone(), task_service);

    // Generate and persist forecasts
    let forecasts = forecast_service.generate_forecasts(Some(40.0)).unwrap();
    assert_eq!(forecasts.len(), 3);

    // Retrieve persisted forecasts
    let conn = pool.get_connection().unwrap();
    let persisted_7d = WorkloadRepository::latest_for_horizon(&conn, WorkloadHorizon::SevenDays)
        .unwrap()
        .expect("Should have persisted 7-day forecast");

    let original_7d = forecasts.iter().find(|f| f.horizon == "7d").unwrap();
    assert_eq!(persisted_7d.total_hours, original_7d.total_hours);
    assert_eq!(persisted_7d.risk_level.as_str(), original_7d.risk_level);
}

#[test]
fn test_get_all_latest_forecasts() {
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path();
    let pool = DbPool::new(db_path).unwrap();

    let task_service = Arc::new(TaskService::new(pool.clone()));
    let forecast_service = WorkloadForecastService::new(pool.clone(), task_service);

    // Generate initial forecasts
    forecast_service.generate_forecasts(Some(40.0)).unwrap();

    // Retrieve all latest forecasts
    let all_latest = forecast_service.get_all_latest_forecasts().unwrap();

    // Should return 3 forecasts (7d, 14d, 30d)
    assert_eq!(all_latest.len(), 3);

    // Check each horizon is present
    assert!(all_latest.iter().any(|f| f.horizon == "7d"));
    assert!(all_latest.iter().any(|f| f.horizon == "14d"));
    assert!(all_latest.iter().any(|f| f.horizon == "30d"));
}
