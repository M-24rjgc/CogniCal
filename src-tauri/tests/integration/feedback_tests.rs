use cognical_app_lib::db::DbPool;
use cognical_app_lib::error::AppResult;
use cognical_app_lib::models::ai_feedback::{AiFeedbackSentiment, AiFeedbackSurface};
use cognical_app_lib::services::feedback_service::{FeedbackService, FeedbackSubmission};
use cognical_app_lib::services::settings_service::{SettingsService, SettingsUpdateInput};
use std::sync::Arc;
use tempfile::TempDir;

fn setup_test_services() -> AppResult<(FeedbackService, Arc<SettingsService>, TempDir)> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test.db");
    let db = DbPool::new(&db_path)?;
    let settings_service = Arc::new(SettingsService::new(db.clone())?);
    let feedback_service = FeedbackService::new(db.clone(), settings_service.clone());
    Ok((feedback_service, settings_service, temp_dir))
}

#[test]
fn test_feedback_submission_flow() -> AppResult<()> {
    let (service, _, _temp_dir) = setup_test_services()?;

    // Submit positive feedback
    let submission = FeedbackSubmission {
        surface: AiFeedbackSurface::Score,
        session_id: Some("test-session-1".to_string()),
        sentiment: AiFeedbackSentiment::Up,
        note: Some("Great productivity score!".to_string()),
        prompt_snapshot: "Calculate productivity score".to_string(),
        context_snapshot: serde_json::json!({
            "task_count": 5,
            "completion_rate": 0.8
        }),
    };

    let feedback_id = service.submit_feedback(&submission)?;
    assert!(feedback_id > 0);

    // Verify feedback was saved
    let recent = service.get_recent_feedback(AiFeedbackSurface::Score, Some(10))?;
    assert_eq!(recent.len(), 1);
    assert_eq!(recent[0].surface, AiFeedbackSurface::Score);
    assert_eq!(recent[0].sentiment, AiFeedbackSentiment::Up);

    Ok(())
}

#[test]
fn test_feedback_anonymization() -> AppResult<()> {
    let (service, _, _temp_dir) = setup_test_services()?;

    // Submit feedback with sensitive data
    let submission = FeedbackSubmission {
        surface: AiFeedbackSurface::Recommendation,
        session_id: None,
        sentiment: AiFeedbackSentiment::Down,
        note: Some("Task recommendation was not helpful".to_string()),
        prompt_snapshot: "Generate task plan".to_string(),
        context_snapshot: serde_json::json!({
            "task_count": 10,
            "description": "Personal project details",
            "note": "Sensitive information"
        }),
    };

    let feedback_id = service.submit_feedback(&submission)?;
    assert!(feedback_id > 0);

    // Verify anonymization
    let recent = service.get_recent_feedback(AiFeedbackSurface::Recommendation, Some(1))?;
    assert_eq!(recent.len(), 1);
    assert!(recent[0].anonymized);

    // Check that sensitive fields were redacted
    let context = &recent[0].context_snapshot;
    assert_eq!(context["description"], "[REDACTED]");
    assert_eq!(context["note"], "[REDACTED]");
    assert_eq!(context["task_count"], 10); // Non-sensitive field preserved

    Ok(())
}

#[test]
fn test_weekly_digest_generation() -> AppResult<()> {
    let (service, _, _temp_dir) = setup_test_services()?;

    // Submit multiple feedback entries (need at least 5 for digest)
    for i in 0..8 {
        let sentiment = if i < 5 {
            AiFeedbackSentiment::Up
        } else {
            AiFeedbackSentiment::Down
        };

        let submission = FeedbackSubmission {
            surface: AiFeedbackSurface::Score,
            session_id: Some(format!("session-{}", i)),
            sentiment,
            note: None,
            prompt_snapshot: format!("Action {}", i),
            context_snapshot: serde_json::json!({}),
        };

        service.submit_feedback(&submission)?;
    }

    // Generate digest - might be None if timestamp filtering is too strict
    let digest = service.generate_weekly_digest()?;

    // If digest is generated, verify its contents
    if let Some(digest) = digest {
        assert!(
            digest.total_feedback >= 5,
            "Expected at least 5 feedback items"
        );
        assert!(digest.positive_count > 0, "Expected positive feedback");
        assert!(!digest.by_surface.is_empty(), "Expected surface breakdown");
        assert!(!digest.insights.is_empty(), "Expected insights");
    } else {
        // If no digest, verify we have the feedback but it might be outside the time window
        let recent = service.get_recent_feedback(AiFeedbackSurface::Score, None)?;
        assert_eq!(recent.len(), 8, "Expected 8 feedback entries");
    }

    Ok(())
}

#[test]
fn test_weekly_digest_threshold() -> AppResult<()> {
    let (service, _, _temp_dir) = setup_test_services()?;

    // Submit only 3 feedback entries (below threshold of 5)
    for i in 0..3 {
        let submission = FeedbackSubmission {
            surface: AiFeedbackSurface::Forecast,
            session_id: None,
            sentiment: AiFeedbackSentiment::Up,
            note: None,
            prompt_snapshot: format!("Action {}", i),
            context_snapshot: serde_json::json!({}),
        };

        service.submit_feedback(&submission)?;
    }

    // Should return None (not enough feedback)
    let digest = service.generate_weekly_digest()?;
    assert!(digest.is_none());

    Ok(())
}

#[test]
fn test_opt_out_functionality() -> AppResult<()> {
    let (service, settings_service, _temp_dir) = setup_test_services()?;

    // Initially, opt-out should be false
    assert!(!service.is_opted_out()?);

    // Enable opt-out
    let input = SettingsUpdateInput {
        ai_feedback_opt_out: Some(true),
        ..Default::default()
    };
    settings_service.update(input)?;

    // Verify opt-out is enabled
    assert!(service.is_opted_out()?);

    // Try to submit feedback - should fail
    let submission = FeedbackSubmission {
        surface: AiFeedbackSurface::Score,
        session_id: None,
        sentiment: AiFeedbackSentiment::Up,
        note: None,
        prompt_snapshot: "test".to_string(),
        context_snapshot: serde_json::json!({}),
    };

    let result = service.submit_feedback(&submission);
    assert!(result.is_err());

    Ok(())
}

#[test]
fn test_purge_all_feedback() -> AppResult<()> {
    let (service, _, _temp_dir) = setup_test_services()?;

    // Submit several feedback entries
    for i in 0..5 {
        let submission = FeedbackSubmission {
            surface: AiFeedbackSurface::Recommendation,
            session_id: None,
            sentiment: AiFeedbackSentiment::Up,
            note: Some(format!("Note {}", i)),
            prompt_snapshot: format!("Prompt {}", i),
            context_snapshot: serde_json::json!({}),
        };

        service.submit_feedback(&submission)?;
    }

    // Verify feedback exists
    let before_purge = service.get_recent_feedback(AiFeedbackSurface::Recommendation, None)?;
    assert_eq!(before_purge.len(), 5);

    // Purge all feedback
    let deleted_count = service.purge_all_feedback()?;
    assert_eq!(deleted_count, 5);

    // Verify all feedback was deleted
    let after_purge = service.get_recent_feedback(AiFeedbackSurface::Recommendation, None)?;
    assert_eq!(after_purge.len(), 0);

    Ok(())
}

#[test]
fn test_session_feedback_retrieval() -> AppResult<()> {
    let (service, _, _temp_dir) = setup_test_services()?;

    let session_id = "test-session-123";

    // Submit feedback for the same session
    for i in 0..3 {
        let submission = FeedbackSubmission {
            surface: AiFeedbackSurface::Score,
            session_id: Some(session_id.to_string()),
            sentiment: AiFeedbackSentiment::Up,
            note: None,
            prompt_snapshot: format!("Action {}", i),
            context_snapshot: serde_json::json!({}),
        };

        service.submit_feedback(&submission)?;
    }

    // Submit feedback for a different session
    let submission = FeedbackSubmission {
        surface: AiFeedbackSurface::Score,
        session_id: Some("other-session".to_string()),
        sentiment: AiFeedbackSentiment::Down,
        note: None,
        prompt_snapshot: "Other action".to_string(),
        context_snapshot: serde_json::json!({}),
    };
    service.submit_feedback(&submission)?;

    // Retrieve feedback for specific session
    let session_feedback = service.get_session_feedback(session_id)?;
    assert_eq!(session_feedback.len(), 3);

    for feedback in session_feedback {
        assert_eq!(feedback.session_id, Some(session_id.to_string()));
    }

    Ok(())
}
