use serde::{Deserialize, Serialize};
use tauri::{async_runtime, AppHandle, Emitter, State};
use tracing::warn;

use crate::error::AppError;
use crate::services::behavior_learning::{BehaviorLearningService, PreferenceSnapshot};
use crate::services::planning_service::{
    AppliedPlan, ApplyPlanInput, GeneratePlanInput, PlanningSessionView, ResolveConflictInput,
};
// Removed: recommendation_orchestrator imports - feature deleted
// use crate::services::recommendation_orchestrator::{
//     RecommendationConfig, RecommendationDecisionInput, RecommendationInput,
//     RecommendationOrchestrator, RecommendationResponse,
// };

use super::{AppState, CommandError, CommandResult};

const DEFAULT_PREFERENCE_ID: &str = "default";

#[tauri::command]
pub async fn planning_generate(
    app: AppHandle,
    state: State<'_, AppState>,
    payload: GeneratePlanInput,
) -> CommandResult<PlanningSessionView> {
    let state = state.inner().clone();
    let service = state.planning();
    
    // generate_plan is now async, so we call it directly
    let session = service.generate_plan(payload).await?;

    emit_event(&app, "planning://generated", &session);
    Ok(session)
}

#[tauri::command]
pub async fn planning_apply(
    app: AppHandle,
    state: State<'_, AppState>,
    payload: ApplyPlanInput,
) -> CommandResult<AppliedPlan> {
    let state = state.inner().clone();
    let applied = run_blocking(move || {
        let service = state.planning();
        service.apply_option(payload)
    })
    .await?;

    emit_event(&app, "planning://applied", &applied);
    Ok(applied)
}

#[tauri::command]
pub async fn planning_resolve_conflict(
    app: AppHandle,
    state: State<'_, AppState>,
    payload: ResolveConflictInput,
) -> CommandResult<PlanningSessionView> {
    let state = state.inner().clone();
    let updated = run_blocking(move || {
        let service = state.planning();
        service.resolve_conflicts(payload)
    })
    .await?;

    emit_event(&app, "planning://conflicts-resolved", &updated.conflicts);
    Ok(updated)
}

#[tauri::command]
pub async fn planning_preferences_get(
    state: State<'_, AppState>,
    preference_id: Option<String>,
) -> CommandResult<PreferenceSnapshot> {
    let state = state.inner().clone();
    let pref_id = preference_id.unwrap_or_else(|| DEFAULT_PREFERENCE_ID.to_string());

    run_blocking(move || {
        let pool = state.db();
        pool.with_connection(|conn| {
            let service = BehaviorLearningService::new(conn);
            service.load_preferences(&pref_id)
        })
    })
    .await
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanningPreferencesUpdatePayload {
    #[serde(default)]
    pub preference_id: Option<String>,
    pub snapshot: PreferenceSnapshot,
}

#[tauri::command]
pub async fn planning_preferences_update(
    app: AppHandle,
    state: State<'_, AppState>,
    payload: PlanningPreferencesUpdatePayload,
) -> CommandResult<()> {
    let state = state.inner().clone();
    let pref_id = payload
        .preference_id
        .unwrap_or_else(|| DEFAULT_PREFERENCE_ID.to_string());
    let snapshot = payload.snapshot;
    let pref_id_for_emit = pref_id.clone();

    run_blocking(move || {
        let pool = state.db();
        pool.with_connection(|conn| {
            let service = BehaviorLearningService::new(conn);
            service.save_preferences(&pref_id, &snapshot)
        })
    })
    .await?;

    emit_event(&app, "planning://preferences-updated", &pref_id_for_emit);
    Ok(())
}

// Removed: recommendations commands - feature deleted
// #[tauri::command]
// pub async fn recommendations_generate(...) { ... }
// #[tauri::command]
// pub async fn recommendations_record_decision(...) { ... }

async fn run_blocking<T: Send + 'static>(
    task: impl FnOnce() -> Result<T, AppError> + Send + 'static,
) -> CommandResult<T> {
    async_runtime::spawn_blocking(task)
        .await
        .map_err(|err| CommandError::new("UNKNOWN", format!("任务执行失败: {err}"), None))?
        .map_err(CommandError::from)
}

fn emit_event<T: Serialize>(app: &AppHandle, name: &str, payload: &T) {
    if let Err(error) = app.emit(name, payload) {
        warn!(target = "app::command", event = name, %error, "failed to emit planning event");
    }
}
