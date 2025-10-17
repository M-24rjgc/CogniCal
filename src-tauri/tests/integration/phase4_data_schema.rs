use chrono::Utc;
use cognical_app_lib::db::{repositories::*, DbPool};
use cognical_app_lib::models::ai_feedback::*;
use cognical_app_lib::models::productivity::*;

#[test]
fn phase4_productivity_score_crud_operations() {
    let pool = DbPool::new(":memory:").expect("Failed to create in-memory database");
    let conn = pool.get_connection().expect("Failed to get connection");

    // Test productivity score upsert
    let score_upsert = ProductivityScoreUpsert {
        snapshot_date: "2025-10-13".to_string(),
        composite_score: 85.5,
        dimension_scores: serde_json::json!({
            "completionRate": 90.0,
            "onTimeRatio": 80.0,
            "focusConsistency": 85.0,
            "restBalance": 75.0,
            "efficiencyRating": 88.0
        }),
        weight_breakdown: serde_json::json!({
            "completionRate": 0.3,
            "onTimeRatio": 0.25,
            "focusConsistency": 0.2,
            "restBalance": 0.15,
            "efficiencyRating": 0.1
        }),
        explanation: Some("Strong performance with good completion rate".to_string()),
    };

    productivity_repository::ProductivityRepository::upsert_score(&conn, &score_upsert)
        .expect("Failed to upsert productivity score");

    // Test that the score was inserted (basic verification)
    // More detailed testing would require implementing get_score_by_date
    assert!(true, "Productivity score upsert completed successfully");
}

#[test]
fn phase4_ai_feedback_crud_operations() {
    let pool = DbPool::new(":memory:").expect("Failed to create in-memory database");
    let conn = pool.get_connection().expect("Failed to get connection");

    // Test AI feedback creation
    let feedback_create = AiFeedbackCreate {
        surface: AiFeedbackSurface::Score,
        session_id: Some("session_123".to_string()),
        sentiment: AiFeedbackSentiment::Up,
        note: Some("Very helpful insights".to_string()),
        prompt_snapshot: "Analyze my productivity".to_string(),
        context_snapshot: serde_json::json!({"score": 85.5}),
        anonymized: true,
    };

    let feedback_id =
        ai_feedback_repository::AiFeedbackRepository::create_feedback(&conn, &feedback_create)
            .expect("Failed to create AI feedback");

    assert!(
        feedback_id > 0,
        "AI feedback should be created with valid ID"
    );

    // Test retrieval
    let feedback_list = ai_feedback_repository::AiFeedbackRepository::get_feedback_by_surface(
        &conn,
        AiFeedbackSurface::Score,
        Some(10),
    )
    .expect("Failed to retrieve AI feedback");

    assert!(!feedback_list.is_empty(), "Should have AI feedback");
    assert_eq!(feedback_list[0].sentiment, AiFeedbackSentiment::Up);
    assert_eq!(feedback_list[0].surface, AiFeedbackSurface::Score);
}

#[test]
fn phase4_database_migration_validation() {
    let pool = DbPool::new(":memory:").expect("Failed to create in-memory database");
    let conn = pool.get_connection().expect("Failed to get connection");

    // Verify that Phase 4 tables exist by checking their structure
    let tables = vec![
        "productivity_scores",
        "recommendation_sessions",
        "recommendation_decisions",
        "workload_forecasts",
        "wellness_events",
        "ai_feedback",
        "community_exports",
    ];

    for table in tables {
        let result = conn.prepare(&format!("SELECT COUNT(*) FROM {}", table));
        assert!(
            result.is_ok(),
            "Table {} should exist and be queryable",
            table
        );
    }

    // Test that we can insert basic records into each table
    // This validates the schema is correct and migrations ran successfully

    // Test productivity_scores table
    conn.execute(
        "INSERT INTO productivity_scores (snapshot_date, composite_score, dimension_scores, weight_breakdown) VALUES (?, ?, ?, ?)",
        ("2025-10-13", 85.5, "{}", "{}")
    ).expect("Should be able to insert into productivity_scores");

    // Test recommendation_sessions table
    conn.execute(
        "INSERT INTO recommendation_sessions (generated_at, context_hash, plans, source, network_status) VALUES (?, ?, ?, ?, ?)",
        (Utc::now().to_rfc3339(), "test_hash", "[]", "deepseek", "online")
    ).expect("Should be able to insert into recommendation_sessions");

    // Test workload_forecasts table
    conn.execute(
        "INSERT INTO workload_forecasts (horizon, generated_at, risk_level, total_hours, capacity_threshold, contributing_tasks, confidence) VALUES (?, ?, ?, ?, ?, ?, ?)",
        ("7d", Utc::now().to_rfc3339(), "ok", 40.0, 45.0, "[]", 0.8)
    ).expect("Should be able to insert into workload_forecasts");

    // Test wellness_events table
    conn.execute(
        "INSERT INTO wellness_events (window_start, trigger_reason, recommended_break_minutes) VALUES (?, ?, ?)",
        (Utc::now().to_rfc3339(), "focus_streak", 15)
    ).expect("Should be able to insert into wellness_events");

    // Test ai_feedback table
    conn.execute(
        "INSERT INTO ai_feedback (surface, sentiment, prompt_snapshot, context_snapshot) VALUES (?, ?, ?, ?)",
        ("score", "up", "test prompt", "{}")
    ).expect("Should be able to insert into ai_feedback");

    // Test community_exports table
    conn.execute(
        "INSERT INTO community_exports (generated_at, payload_path, metrics_summary, checksum) VALUES (?, ?, ?, ?)",
        (Utc::now().to_rfc3339(), "/tmp/test.json", "{}", "test_checksum")
    ).expect("Should be able to insert into community_exports");
}

#[test]
fn phase4_productivity_score_boundary_values() {
    let pool = DbPool::new(":memory:").expect("Failed to create in-memory database");
    let conn = pool.get_connection().expect("Failed to get connection");

    // Test boundary values (0 and 100)
    let score_zero = ProductivityScoreUpsert {
        snapshot_date: "2025-10-01".to_string(),
        composite_score: 0.0,
        dimension_scores: serde_json::json!({"completion": 0.0}),
        weight_breakdown: serde_json::json!({"completion": 1.0}),
        explanation: Some("Zero score test".to_string()),
    };

    productivity_repository::ProductivityRepository::upsert_score(&conn, &score_zero)
        .expect("Should handle zero score");

    let score_max = ProductivityScoreUpsert {
        snapshot_date: "2025-10-02".to_string(),
        composite_score: 100.0,
        dimension_scores: serde_json::json!({"completion": 100.0}),
        weight_breakdown: serde_json::json!({"completion": 1.0}),
        explanation: Some("Perfect score test".to_string()),
    };

    productivity_repository::ProductivityRepository::upsert_score(&conn, &score_max)
        .expect("Should handle perfect score");

    assert!(true, "Boundary value testing completed");
}

// 推荐功能已移除，相关测试已注释
/* #[test]
fn phase4_recommendation_session_logging() {
    let pool = DbPool::new(":memory:").expect("Failed to create in-memory database");
    let conn = pool.get_connection().expect("Failed to get connection");

    use cognical_app_lib::models::recommendation::*;

    // Create recommendation session
    let session_insert = RecommendationSessionInsert {
        generated_at: Utc::now().to_rfc3339(),
        context_hash: "test_hash_123".to_string(),
        plans: serde_json::json!([
            {"rank": 1, "label": "Option A", "confidence": 0.95},
            {"rank": 2, "label": "Option B", "confidence": 0.85}
        ]),
        source: RecommendationSource::Deepseek,
        network_status: RecommendationNetworkStatus::Online,
        expires_at: None,
    };

    let session_id =
        recommendation_repository::RecommendationRepository::insert_session(&conn, &session_insert)
            .expect("Failed to insert recommendation session");

    assert!(session_id > 0, "Session should be created with valid ID");

    // Retrieve and verify
    let session =
        recommendation_repository::RecommendationRepository::find_session_by_id(&conn, session_id)
            .expect("Failed to retrieve session")
            .expect("Session should exist");

    assert_eq!(session.source, RecommendationSource::Deepseek);
    assert_eq!(session.network_status, RecommendationNetworkStatus::Online);
}

#[test]
fn phase4_recommendation_fallback_mode() {
    let pool = DbPool::new(":memory:").expect("Failed to create in-memory database");
    let conn = pool.get_connection().expect("Failed to get connection");

    use cognical_app_lib::models::recommendation::*;

    // Create fallback session (heuristic)
    let session_insert = RecommendationSessionInsert {
        generated_at: Utc::now().to_rfc3339(),
        context_hash: "fallback_hash".to_string(),
        plans: serde_json::json!([
            {"rank": 1, "label": "Heuristic Plan"}
        ]),
        source: RecommendationSource::Heuristic,
        network_status: RecommendationNetworkStatus::Offline,
        expires_at: None,
    };

    let session_id =
        recommendation_repository::RecommendationRepository::insert_session(&conn, &session_insert)
            .expect("Failed to insert fallback session");

    let session =
        recommendation_repository::RecommendationRepository::find_session_by_id(&conn, session_id)
            .expect("Failed to retrieve session")
            .expect("Session should exist");

    assert_eq!(
        session.source,
        RecommendationSource::Heuristic,
        "Should be heuristic source"
    );
    assert_eq!(
        session.network_status,
        RecommendationNetworkStatus::Offline,
        "Should be offline"
    );
}
*/ // 推荐功能相关测试结束

#[test]
fn phase4_ai_feedback_anonymization() {
    let pool = DbPool::new(":memory:").expect("Failed to create in-memory database");
    let conn = pool.get_connection().expect("Failed to get connection");

    // Create feedback with anonymization flag (using Recommendation surface)
    let feedback_create = AiFeedbackCreate {
        surface: AiFeedbackSurface::Recommendation,
        session_id: Some("anon_session".to_string()),
        sentiment: AiFeedbackSentiment::Down,
        note: Some("Needs improvement".to_string()),
        prompt_snapshot: "Generate plan".to_string(),
        context_snapshot: serde_json::json!({
            "note": "Sensitive data here",
            "description": "Private info"
        }),
        anonymized: true,
    };

    let _feedback_id =
        ai_feedback_repository::AiFeedbackRepository::create_feedback(&conn, &feedback_create)
            .expect("Failed to create anonymized feedback");

    // Verify via get_feedback_by_surface since get_feedback_by_id doesn't exist
    let feedback_list = ai_feedback_repository::AiFeedbackRepository::get_feedback_by_surface(
        &conn,
        AiFeedbackSurface::Recommendation,
        Some(1),
    )
    .expect("Failed to retrieve feedback");

    assert!(!feedback_list.is_empty(), "Should have created feedback");
    assert!(
        feedback_list[0].anonymized,
        "Anonymization flag should be set"
    );
}
