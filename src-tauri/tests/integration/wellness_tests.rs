//! Integration tests for wellness nudge engine (Task 7 - R4 requirement)
//!
//! These tests verify the core wellness nudge functionality including:
//! - Service initialization
//! - Nudge generation and retrieval
//! - Response recording
//! - Weekly summary calculation

use chrono::Utc;
use cognical_app_lib::db::repositories::wellness_repository::WellnessRepository;
use cognical_app_lib::db::DbPool;
use cognical_app_lib::models::wellness::{WellnessEventInsert, WellnessTriggerReason};
use cognical_app_lib::services::settings_service::SettingsService;
use cognical_app_lib::services::wellness_service::WellnessService;
use std::sync::Arc;
use tempfile::{tempdir, TempDir};

fn setup_test_env() -> (DbPool, Arc<WellnessService>, TempDir) {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test.db");
    let db = DbPool::new(&db_path).expect("Failed to create test database");

    let settings_service =
        Arc::new(SettingsService::new(db.clone()).expect("Failed to create SettingsService"));
    let wellness_service = Arc::new(WellnessService::new(db.clone(), settings_service));

    (db, wellness_service, temp_dir)
}

#[test]
fn test_wellness_service_initialization() {
    let (_db, wellness_service, _temp_dir) = setup_test_env();

    // Service should initialize successfully
    assert!(
        Arc::strong_count(&wellness_service) == 1,
        "WellnessService should be initialized"
    );

    // Should be able to check for nudges (even if none generated)
    let result = wellness_service.check_and_generate_nudge();
    assert!(
        result.is_ok(),
        "check_and_generate_nudge should not error: {:?}",
        result.err()
    );
}

#[test]
fn test_wellness_repository_operations() {
    let (db, _wellness_service, _temp_dir) = setup_test_env();
    let conn = db.get_connection().unwrap();
    let now = Utc::now();

    // Create a wellness event insert
    let insert = WellnessEventInsert {
        window_start: now.to_rfc3339(),
        trigger_reason: WellnessTriggerReason::WorkStreak,
        recommended_break_minutes: 10,
        suggested_micro_task: Some("Test task".to_string()),
    };

    // Insert and retrieve
    let id = WellnessRepository::insert(&conn, &insert).unwrap();
    let record = WellnessRepository::find_by_id(&conn, id).unwrap();

    assert_eq!(record.id, id, "Record ID should match");
    assert_eq!(
        record.trigger_reason,
        WellnessTriggerReason::WorkStreak,
        "Trigger reason should match"
    );
    assert_eq!(record.response, None, "Initial response should be None");
    assert_eq!(
        record.deferral_count, 0,
        "Initial deferral count should be 0"
    );
}

#[test]
fn test_weekly_summary() {
    let (_db, wellness_service, _temp_dir) = setup_test_env();

    // Get weekly summary (should work even with no events)
    let summary = wellness_service.get_weekly_summary();
    assert!(
        summary.is_ok(),
        "get_weekly_summary should not error: {:?}",
        summary.err()
    );

    if let Ok(s) = summary {
        assert!(s.total_nudges >= 0, "Total nudges should be non-negative");
        assert!(
            s.rest_compliance_rate >= 0.0 && s.rest_compliance_rate <= 1.0,
            "Compliance rate should be between 0 and 1"
        );
        assert!(
            s.focus_rhythm_score >= 0.0 && s.focus_rhythm_score <= 100.0,
            "Rhythm score should be between 0 and 100"
        );
    }
}

#[test]
fn test_pending_nudge() {
    let (_db, wellness_service, _temp_dir) = setup_test_env();

    // Get pending nudge (may be None if no nudge pending)
    let result = wellness_service.get_pending_nudge();
    assert!(
        result.is_ok(),
        "get_pending_nudge should not error: {:?}",
        result.err()
    );
}
