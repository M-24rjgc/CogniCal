use crate::commands::AppState;
use crate::services::community_service::{DetectedPlugin, ExportBundle, ProjectInfo};
use std::path::PathBuf;
use tauri::{async_runtime, State};

/// Get project information (license, repo, etc.)
#[tauri::command]
pub async fn community_get_project_info(state: State<'_, AppState>) -> Result<ProjectInfo, String> {
    let service = &state.community_service;
    service
        .get_project_info()
        .map_err(|e| format!("Failed to get project info: {}", e))
}

/// Detect installed plugins
#[tauri::command]
pub async fn community_detect_plugins(
    state: State<'_, AppState>,
) -> Result<Vec<DetectedPlugin>, String> {
    let service = &state.community_service;
    service
        .detect_plugins()
        .map_err(|e| format!("Failed to detect plugins: {}", e))
}

/// Generate export bundle
#[tauri::command]
pub async fn community_generate_export_bundle(
    state: State<'_, AppState>,
    include_feedback: bool,
) -> Result<ExportBundle, String> {
    let service = state.community_service.clone();
    async_runtime::spawn(async move { service.generate_export_bundle(include_feedback).await })
        .await
        .map_err(|e| format!("Task join error: {}", e))?
        .map_err(|e| format!("Failed to generate export bundle: {}", e))
}

/// Save export bundle to file
#[tauri::command]
pub async fn community_save_export_to_file(
    state: State<'_, AppState>,
    bundle_json: String,
    file_path: String,
) -> Result<i64, String> {
    let bundle: ExportBundle =
        serde_json::from_str(&bundle_json).map_err(|e| format!("Failed to parse bundle: {}", e))?;

    let path = PathBuf::from(&file_path);
    let service = state.community_service.clone();

    async_runtime::spawn(async move { service.save_export_to_file(&bundle, &path).await })
        .await
        .map_err(|e| format!("Task join error: {}", e))?
        .map_err(|e| format!("Failed to save export: {}", e))
}

/// List previous exports
#[tauri::command]
pub async fn community_list_exports(
    state: State<'_, AppState>,
) -> Result<Vec<(i64, String, String)>, String> {
    let service = &state.community_service;
    service
        .list_exports()
        .map_err(|e| format!("Failed to list exports: {}", e))
}
