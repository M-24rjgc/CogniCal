use std::sync::Arc;

use chrono::{Duration, FixedOffset, NaiveDate, TimeZone};
use cognical_app_lib::db::DbPool;
use cognical_app_lib::models::task::TaskCreateInput;
use cognical_app_lib::services::ai_service::AiService;
use cognical_app_lib::services::planning_service::{
    ApplyPlanInput, GeneratePlanInput, PlanningService, ResolveConflictInput, TimeBlockOverride,
};
use cognical_app_lib::services::schedule_optimizer::{
    ExistingEvent, ScheduleConstraints, TimeWindow,
};
use cognical_app_lib::services::schedule_utils;
use cognical_app_lib::services::task_service::TaskService;
use tempfile::tempdir;

#[tokio::test]
async fn planning_generate_apply_resolve_flow() {
    let dir = tempdir().expect("temp dir");
    let db_path = dir.path().join("planning.sqlite");
    let pool = DbPool::new(&db_path).expect("db pool");

    let task_service = Arc::new(TaskService::new(pool.clone()));
    let ai_service = Arc::new(AiService::new(pool.clone()).expect("ai service"));
    let planning_service = PlanningService::new(
        pool.clone(),
        Arc::clone(&task_service),
        Arc::clone(&ai_service),
    );

    let tz = FixedOffset::east_opt(0).expect("offset");
    let base_day = tz
        .from_local_datetime(
            &NaiveDate::from_ymd_opt(2025, 5, 1)
                .expect("base date")
                .and_hms_opt(9, 0, 0)
                .expect("base time"),
        )
        .single()
        .expect("base day");

    let task_a = task_service
        .create_task(TaskCreateInput {
            title: "Spec Review".into(),
            description: None,
            status: Some("todo".into()),
            priority: Some("high".into()),
            planned_start_at: Some(schedule_utils::format_datetime(base_day)),
            start_at: None,
            due_at: Some(schedule_utils::format_datetime(
                base_day + Duration::hours(6),
            )),
            completed_at: None,
            estimated_minutes: Some(120),
            estimated_hours: None,
            tags: None,
            owner_id: None,
            is_recurring: None,
            recurrence: None,
            task_type: None,
            ai: None,
            external_links: None,
        })
        .expect("create task A");

    let task_b_start = base_day + Duration::hours(2);
    let task_b = task_service
        .create_task(TaskCreateInput {
            title: "API Implementation".into(),
            description: None,
            status: Some("todo".into()),
            priority: Some("medium".into()),
            planned_start_at: Some(schedule_utils::format_datetime(task_b_start)),
            start_at: None,
            due_at: Some(schedule_utils::format_datetime(
                task_b_start + Duration::hours(5),
            )),
            completed_at: None,
            estimated_minutes: Some(150),
            estimated_hours: None,
            tags: None,
            owner_id: None,
            is_recurring: None,
            recurrence: None,
            task_type: None,
            ai: None,
            external_links: None,
        })
        .expect("create task B");

    let constraints = ScheduleConstraints {
        available_windows: vec![TimeWindow {
            start_at: schedule_utils::format_datetime(base_day),
            end_at: schedule_utils::format_datetime(base_day + Duration::hours(8)),
        }],
        existing_events: vec![ExistingEvent {
            id: "event-1".into(),
            start_at: schedule_utils::format_datetime(base_day + Duration::hours(1)),
            end_at: schedule_utils::format_datetime(base_day + Duration::hours(2)),
            event_type: Some("meeting".into()),
        }],
        max_focus_minutes_per_day: Some(480),
        ..Default::default()
    };

    let session = planning_service
        .generate_plan(GeneratePlanInput {
            task_ids: vec![task_a.id.clone(), task_b.id.clone()],
            constraints: Some(constraints.clone()),
            preference_id: Some("default".into()),
            seed: Some(11),
        })
        .await
        .expect("generate plan");

    assert!(!session.options.is_empty());
    assert!(session
        .conflicts
        .iter()
        .any(|conflict| conflict.conflict_type == "calendar-overlap"));

    let option_id = session.options[0].option.id.clone();
    let applied = planning_service
        .apply_option(ApplyPlanInput {
            session_id: session.session.id.clone(),
            option_id: option_id.clone(),
            overrides: Vec::new(),
        })
        .expect("apply option");

    assert!(!applied.option.blocks.is_empty());
    assert!(applied
        .conflicts
        .iter()
        .any(|conflict| conflict.conflict_type == "calendar-overlap"));

    let conflict = applied
        .conflicts
        .iter()
        .find(|conflict| conflict.conflict_type == "calendar-overlap")
        .expect("conflict to adjust");
    let block_id = conflict
        .related_block_id
        .as_ref()
        .expect("block id for conflict")
        .clone();

    let block = applied
        .option
        .blocks
        .iter()
        .find(|candidate| candidate.id == block_id)
        .expect("block lookup");
    let block_start = schedule_utils::parse_datetime(&block.start_at).expect("parse block start");
    let block_end = schedule_utils::parse_datetime(&block.end_at).expect("parse block end");
    let duration_minutes =
        schedule_utils::duration_minutes(block_start, block_end).expect("duration computation");

    let event_end =
        schedule_utils::parse_datetime(&constraints.existing_events[0].end_at).expect("event end");
    let new_start = event_end + Duration::minutes(30);
    let new_end = new_start + Duration::minutes(duration_minutes);

    let adjusted = planning_service
        .resolve_conflicts(ResolveConflictInput {
            session_id: session.session.id.clone(),
            option_id: option_id.clone(),
            adjustments: vec![TimeBlockOverride {
                block_id: block_id.clone(),
                start_at: Some(schedule_utils::format_datetime(new_start)),
                end_at: Some(schedule_utils::format_datetime(new_end)),
                flexibility: None,
            }],
        })
        .expect("resolve conflicts");

    let updated_option = adjusted
        .options
        .iter()
        .find(|opt| opt.option.id == option_id)
        .expect("option still present");
    assert!(updated_option
        .conflicts
        .iter()
        .all(|conflict| conflict.conflict_type != "calendar-overlap"));
    let updated_block = updated_option
        .blocks
        .iter()
        .find(|candidate| candidate.id == block_id)
        .expect("updated block");
    assert_eq!(
        updated_block.start_at,
        schedule_utils::format_datetime(new_start)
    );
    assert_eq!(
        updated_block.end_at,
        schedule_utils::format_datetime(new_end)
    );

    let refreshed_task = task_service
        .get_task(&task_a.id)
        .expect("expected task to exist after apply");
    assert!(
        refreshed_task.planned_start_at.is_some(),
        "expected planned start to be recorded"
    );
}
