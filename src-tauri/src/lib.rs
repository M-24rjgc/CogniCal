pub mod commands;
pub mod db;
pub mod error;
pub mod models;
pub mod services;
pub mod tools;
pub mod utils;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    if let Err(error) = try_run() {
        eprintln!("failed to launch application: {error}");
    }
}

fn try_run() -> Result<(), Box<dyn std::error::Error>> {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let handle = app.handle();

            crate::utils::logger::init_logging(&handle)
                .map_err(|err| Box::new(err) as Box<dyn std::error::Error>)?;

            let mut data_dir = handle
                .path()
                .app_data_dir()
                .map_err(|err| Box::new(err) as Box<dyn std::error::Error>)?;

            std::fs::create_dir_all(&data_dir)?;
            data_dir.push("cognical.sqlite");

            let pool = crate::db::DbPool::new(&data_dir)
                .map_err(|err| Box::new(err) as Box<dyn std::error::Error>)?;

            let state = crate::commands::AppState::new(pool)
                .map_err(|err| Box::new(err) as Box<dyn std::error::Error>)?;
            app.manage(state);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            crate::commands::analytics::analytics_history_fetch,
            crate::commands::analytics::analytics_overview_fetch,
            crate::commands::analytics::analytics_report_export,
            crate::commands::analytics::analytics_get_productivity_score,
            crate::commands::analytics::analytics_get_productivity_score_history,
            crate::commands::analytics::analytics_get_latest_productivity_score,
            crate::commands::analytics::analytics_get_workload_forecast,
            crate::commands::analytics::analytics_get_latest_workload_forecasts,
            crate::commands::ai_commands::tasks_parse_ai,
            crate::commands::ai_commands::ai_generate_recommendations,
            crate::commands::ai_commands::ai_plan_schedule,
            crate::commands::ai_commands::ai_status,
            crate::commands::ai_commands::ai_chat,
            crate::commands::ai_commands::ai_agent_chat,


            crate::commands::planning::planning_apply,
            crate::commands::planning::planning_generate,
            crate::commands::planning::planning_preferences_get,
            crate::commands::planning::planning_preferences_update,
            crate::commands::planning::planning_resolve_conflict,
            // Removed: recommendations commands - feature deleted
            // crate::commands::planning::recommendations_generate,
            // crate::commands::planning::recommendations_record_decision,
            crate::commands::task::tasks_list,
            crate::commands::task::tasks_create,
            crate::commands::task::tasks_update,
            crate::commands::task::tasks_delete,
            crate::commands::settings::settings_get,
            crate::commands::settings::settings_update,
            crate::commands::settings::settings_clear_api_key,
            crate::commands::settings::dashboard_config_get,
            crate::commands::settings::dashboard_config_update,
            crate::commands::cache::cache_clear_all,
            crate::commands::wellness::wellness_check_nudge,
            crate::commands::wellness::wellness_get_pending,
            crate::commands::wellness::wellness_respond,
            crate::commands::wellness::wellness_get_weekly_summary,
            crate::commands::feedback::feedback_submit,
            crate::commands::feedback::feedback_get_recent,
            crate::commands::feedback::feedback_get_session,
            crate::commands::feedback::feedback_get_weekly_digest,
            crate::commands::feedback::feedback_check_opt_out,
            crate::commands::feedback::feedback_purge_all,
            crate::commands::feedback::feedback_get_stats,
            crate::commands::community::community_get_project_info,
            crate::commands::community::community_detect_plugins,
            crate::commands::community::community_generate_export_bundle,
            crate::commands::community::community_save_export_to_file,
            crate::commands::community::community_list_exports,
        ])
        .run(tauri::generate_context!())?;

    Ok(())
}
