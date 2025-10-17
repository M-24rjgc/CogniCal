use std::fs;
use std::sync::Arc;

use chrono::{Duration, FixedOffset, TimeZone, Utc};
use cognical_app_lib::db::DbPool;
use cognical_app_lib::models::analytics::{
    AnalyticsExportFormat, AnalyticsExportParams, AnalyticsGrouping, AnalyticsQueryParams,
    AnalyticsRangeKey,
};
use cognical_app_lib::models::task::TaskCreateInput;
use cognical_app_lib::services::analytics_service::AnalyticsService;
use cognical_app_lib::services::settings_service::{SettingsService, SettingsUpdateInput};
use cognical_app_lib::services::task_service::TaskService;
use tempfile::tempdir;

#[test]
fn analytics_overview_history_and_settings_flow() {
    let dir = tempdir().expect("temp dir");
    let db_path = dir.path().join("analytics.sqlite");
    let pool = DbPool::new(&db_path).expect("db pool");

    let task_service = Arc::new(TaskService::new(pool.clone()));
    let analytics_service =
        AnalyticsService::new(pool.clone(), Arc::clone(&task_service)).expect("analytics service");
    let settings_service = SettingsService::new(pool.clone()).expect("settings service");

    let tz = FixedOffset::east_opt(0).expect("offset");
    let first_day = tz
        .with_ymd_and_hms(2025, 5, 1, 9, 0, 0)
        .single()
        .expect("first day");
    let second_day = first_day + Duration::days(1);

    let completed_due_at = (first_day + Duration::hours(3)).with_timezone(&Utc);
    let completed_at = (first_day + Duration::hours(4)).with_timezone(&Utc);
    let pending_due_at = (second_day + Duration::hours(5)).with_timezone(&Utc);

    let completed_task = task_service
        .create_task(TaskCreateInput {
            title: "Retrospective".into(),
            description: Some("weekly recap".into()),
            status: Some("done".into()),
            priority: Some("high".into()),
            planned_start_at: Some(first_day.with_timezone(&Utc).to_rfc3339()),
            start_at: Some(
                (first_day + Duration::hours(1))
                    .with_timezone(&Utc)
                    .to_rfc3339(),
            ),
            due_at: Some(completed_due_at.to_rfc3339()),
            completed_at: Some(completed_at.to_rfc3339()),
            estimated_minutes: Some(90),
            estimated_hours: None,
            tags: None,
            owner_id: None,
            is_recurring: None,
            recurrence: None,
            task_type: Some("work".into()),
            ai: None,
            external_links: None,
        })
        .expect("create completed task");
    assert_eq!(completed_task.status, "done");

    let pending_task = task_service
        .create_task(TaskCreateInput {
            title: "Draft proposal".into(),
            description: None,
            status: Some("todo".into()),
            priority: Some("medium".into()),
            planned_start_at: Some(second_day.with_timezone(&Utc).to_rfc3339()),
            start_at: None,
            due_at: Some(pending_due_at.to_rfc3339()),
            completed_at: None,
            estimated_minutes: Some(30),
            estimated_hours: None,
            tags: None,
            owner_id: None,
            is_recurring: None,
            recurrence: None,
            task_type: Some("study".into()),
            ai: None,
            external_links: None,
        })
        .expect("create pending task");
    assert_eq!(pending_task.status, "todo");

    let range_start = (first_day - Duration::hours(1)).with_timezone(&Utc);
    let range_end = (second_day + Duration::hours(8)).with_timezone(&Utc);

    let params = AnalyticsQueryParams {
        range: AnalyticsRangeKey::ThirtyDays,
        from: Some(range_start.to_rfc3339()),
        to: Some(range_end.to_rfc3339()),
        grouping: Some(AnalyticsGrouping::Day),
    };

    let overview = analytics_service
        .fetch_overview(params.clone())
        .expect("overview response");
    assert_eq!(overview.overview.summary.total_completed, 1);
    assert_eq!(overview.overview.summary.overdue_tasks, 1);
    assert!((overview.overview.summary.completion_rate - 0.5).abs() < 0.001);
    assert_eq!(overview.overview.summary.workload_prediction, 2);
    assert_eq!(overview.history.points.len(), 2);
    assert_eq!(overview.history.points.last().unwrap().overdue_tasks, 1);

    let history = analytics_service
        .fetch_history(params.clone())
        .expect("history response");
    assert_eq!(history.points.len(), overview.history.points.len());

    let export = analytics_service
        .export_report(AnalyticsExportParams {
            range: AnalyticsRangeKey::ThirtyDays,
            format: AnalyticsExportFormat::Markdown,
            from: params.from.clone(),
            to: params.to.clone(),
        })
        .expect("export report");
    assert_eq!(export.format, AnalyticsExportFormat::Markdown);
    assert!(
        fs::metadata(&export.file_path).is_ok(),
        "report file exists"
    );
    let report = fs::read_to_string(&export.file_path).expect("read report file");
    assert!(report.contains("# CogniCal"));

    let updated_settings = settings_service
        .update(SettingsUpdateInput {
            deepseek_api_key: Some(Some("sk-phase3-abcdef123456".into())),
            workday_start_minute: Some(8 * 60),
            workday_end_minute: Some(17 * 60),
            theme: Some("dark".into()),
            ai_feedback_opt_out: None,
        })
        .expect("update settings");
    assert_eq!(updated_settings.theme, "dark");
    assert_eq!(updated_settings.workday_start_minute, 480);
    assert_eq!(updated_settings.workday_end_minute, 1020);
    let masked = updated_settings
        .deepseek_api_key
        .as_deref()
        .expect("masked api key");
    assert!(masked.ends_with("3456"));
    assert!(masked.chars().take(masked.len() - 4).all(|c| c == '*'));
    assert_eq!(masked.len(), "sk-phase3-abcdef123456".len());

    let fetched_settings = settings_service.get().expect("get settings");
    assert_eq!(fetched_settings.theme, "dark");
    let fetched_masked = fetched_settings
        .deepseek_api_key
        .as_deref()
        .expect("masked api key from get");
    assert!(fetched_masked.ends_with("3456"));
    assert!(fetched_masked
        .chars()
        .take(fetched_masked.len() - 4)
        .all(|c| c == '*'));
    assert_eq!(fetched_masked.len(), "sk-phase3-abcdef123456".len());
}
