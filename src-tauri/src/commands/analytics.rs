use tauri::{async_runtime, State};

use crate::error::AppError;
use crate::models::analytics::{
    AnalyticsExportParams, AnalyticsExportResult, AnalyticsHistoryResponse,
    AnalyticsOverviewResponse, AnalyticsQueryParams,
};
use crate::models::productivity::{ProductivityScoreHistoryResponse, ProductivityScoreRecord};

use super::{AppState, CommandError, CommandResult};

#[tauri::command]
pub async fn analytics_overview_fetch(
    state: State<'_, AppState>,
    params: Option<AnalyticsQueryParams>,
) -> CommandResult<AnalyticsOverviewResponse> {
    let app_state = state.inner().clone();
    let payload = params.unwrap_or_default();
    run_blocking(move || app_state.analytics().fetch_overview(payload)).await
}

#[tauri::command]
pub async fn analytics_history_fetch(
    state: State<'_, AppState>,
    params: Option<AnalyticsQueryParams>,
) -> CommandResult<AnalyticsHistoryResponse> {
    let app_state = state.inner().clone();
    let payload = params.unwrap_or_default();
    run_blocking(move || app_state.analytics().fetch_history(payload)).await
}

#[tauri::command]
pub async fn analytics_report_export(
    state: State<'_, AppState>,
    params: AnalyticsExportParams,
) -> CommandResult<AnalyticsExportResult> {
    let app_state = state.inner().clone();
    run_blocking(move || app_state.analytics().export_report(params)).await
}

#[tauri::command]
pub async fn analytics_get_productivity_score(
    state: State<'_, AppState>,
    date: Option<String>,
) -> CommandResult<ProductivityScoreRecord> {
    let app_state = state.inner().clone();
    let target_date = date.unwrap_or_else(|| chrono::Utc::now().format("%Y-%m-%d").to_string());

    run_blocking(move || {
        app_state
            .productivity_score_service()
            .calculate_score_for_date(&target_date)
    })
    .await
}

#[tauri::command]
pub async fn analytics_get_productivity_score_history(
    state: State<'_, AppState>,
    start_date: String,
    end_date: String,
) -> CommandResult<ProductivityScoreHistoryResponse> {
    let app_state = state.inner().clone();

    run_blocking(move || {
        let scores = app_state
            .productivity_score_service()
            .get_score_history(&start_date, &end_date)?;
        let total_scores = scores.len();

        Ok(ProductivityScoreHistoryResponse {
            scores,
            start_date,
            end_date,
            total_scores,
        })
    })
    .await
}

#[tauri::command]
pub async fn analytics_get_latest_productivity_score(
    state: State<'_, AppState>,
) -> CommandResult<Option<ProductivityScoreRecord>> {
    let app_state = state.inner().clone();

    run_blocking(move || app_state.productivity_score_service().get_latest_score()).await
}

#[tauri::command]
pub async fn analytics_get_workload_forecast(
    state: State<'_, AppState>,
    capacity_threshold_hours: Option<f64>,
) -> CommandResult<Vec<crate::models::workload::WorkloadForecastResponse>> {
    let app_state = state.inner().clone();

    run_blocking(move || {
        app_state
            .workload_forecast()
            .generate_forecasts(capacity_threshold_hours)
    })
    .await
}

#[tauri::command]
pub async fn analytics_get_latest_workload_forecasts(
    state: State<'_, AppState>,
) -> CommandResult<Vec<crate::models::workload::WorkloadForecastResponse>> {
    let app_state = state.inner().clone();

    run_blocking(move || app_state.workload_forecast().get_all_latest_forecasts()).await
}

async fn run_blocking<T: Send + 'static>(
    task: impl FnOnce() -> Result<T, AppError> + Send + 'static,
) -> CommandResult<T> {
    async_runtime::spawn_blocking(task)
        .await
        .map_err(|err| CommandError::new("UNKNOWN", format!("分析任务执行失败: {err}"), None))?
        .map_err(CommandError::from)
}
