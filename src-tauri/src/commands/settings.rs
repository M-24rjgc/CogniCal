use std::collections::{BTreeMap, HashMap};

use serde::Deserialize;
use tauri::{async_runtime, State};

use crate::error::AppError;
use crate::models::settings::{AppSettings, DashboardConfig};
use crate::services::settings_service::{DashboardConfigUpdateInput, SettingsUpdateInput};

use super::{AppState, CommandError, CommandResult};

#[tauri::command]
pub async fn settings_get(state: State<'_, AppState>) -> CommandResult<AppSettings> {
    let app_state = state.inner().clone();
    run_blocking(move || app_state.settings().get()).await
}

#[tauri::command]
pub async fn settings_update(
    state: State<'_, AppState>,
    payload: SettingsUpdatePayload,
) -> CommandResult<AppSettings> {
    let app_state = state.inner().clone();
    let input = payload.into_input();
    run_blocking(move || app_state.settings().update(input)).await
}

#[tauri::command]
pub async fn settings_clear_api_key(state: State<'_, AppState>) -> CommandResult<AppSettings> {
    let app_state = state.inner().clone();
    run_blocking(move || {
        let service = app_state.settings();
        service.clear_sensitive()?;
        service.get()
    })
    .await
}

#[tauri::command]
pub async fn dashboard_config_get(state: State<'_, AppState>) -> CommandResult<DashboardConfig> {
    let app_state = state.inner().clone();
    run_blocking(move || app_state.settings().get_dashboard_config()).await
}

#[tauri::command]
pub async fn dashboard_config_update(
    state: State<'_, AppState>,
    payload: DashboardConfigUpdatePayload,
) -> CommandResult<DashboardConfig> {
    let app_state = state.inner().clone();
    let input = payload.into_input();
    run_blocking(move || app_state.settings().update_dashboard_config(input)).await
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsUpdatePayload {
    #[serde(default)]
    deepseek_api_key: Option<String>,
    #[serde(default)]
    remove_deepseek_key: Option<bool>,
    #[serde(default)]
    workday_start_minute: Option<i16>,
    #[serde(default)]
    workday_end_minute: Option<i16>,
    #[serde(default)]
    theme: Option<String>,
    #[serde(default)]
    ai_feedback_opt_out: Option<bool>,
}

impl SettingsUpdatePayload {
    fn into_input(self) -> SettingsUpdateInput {
        let deepseek_api_key = if self.remove_deepseek_key == Some(true) {
            Some(None)
        } else {
            self.deepseek_api_key.map(Some)
        };

        SettingsUpdateInput {
            deepseek_api_key,
            workday_start_minute: self.workday_start_minute,
            workday_end_minute: self.workday_end_minute,
            theme: self.theme,
            ai_feedback_opt_out: self.ai_feedback_opt_out,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardConfigUpdatePayload {
    #[serde(default)]
    modules: Option<HashMap<String, bool>>,
    #[serde(default)]
    last_updated_at: Option<Option<String>>,
}

impl DashboardConfigUpdatePayload {
    fn into_input(self) -> DashboardConfigUpdateInput {
        DashboardConfigUpdateInput {
            modules: self
                .modules
                .map(|modules| modules.into_iter().collect::<BTreeMap<_, _>>()),
            last_updated_at: self.last_updated_at,
        }
    }
}

async fn run_blocking<T: Send + 'static>(
    task: impl FnOnce() -> Result<T, AppError> + Send + 'static,
) -> CommandResult<T> {
    async_runtime::spawn_blocking(task)
        .await
        .map_err(|err| CommandError::new("UNKNOWN", format!("设置操作执行失败: {err}"), None))?
        .map_err(CommandError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_remove_deepseek_key_flag() {
        // When removeDeepseekKey is true, should convert to Some(None)
        let payload = SettingsUpdatePayload {
            deepseek_api_key: None,
            remove_deepseek_key: Some(true),
            workday_start_minute: None,
            workday_end_minute: None,
            theme: None,
            ai_feedback_opt_out: None,
        };

        let input = payload.into_input();
        assert_eq!(input.deepseek_api_key, Some(None));
    }

    #[test]
    fn test_set_deepseek_key() {
        // When deepseekApiKey is provided, should convert to Some(Some(value))
        let payload = SettingsUpdatePayload {
            deepseek_api_key: Some("sk-test-key".to_string()),
            remove_deepseek_key: None,
            workday_start_minute: None,
            workday_end_minute: None,
            theme: None,
            ai_feedback_opt_out: None,
        };

        let input = payload.into_input();
        assert_eq!(
            input.deepseek_api_key,
            Some(Some("sk-test-key".to_string()))
        );
    }

    #[test]
    fn test_no_change_deepseek_key() {
        // When neither is provided, should be None (no change)
        let payload = SettingsUpdatePayload {
            deepseek_api_key: None,
            remove_deepseek_key: None,
            workday_start_minute: None,
            workday_end_minute: None,
            theme: None,
            ai_feedback_opt_out: None,
        };

        let input = payload.into_input();
        assert_eq!(input.deepseek_api_key, None);
    }

    #[test]
    fn test_remove_takes_precedence() {
        // If both are provided (shouldn't happen due to validation),
        // remove should take precedence
        let payload = SettingsUpdatePayload {
            deepseek_api_key: Some("sk-test-key".to_string()),
            remove_deepseek_key: Some(true),
            workday_start_minute: None,
            workday_end_minute: None,
            theme: None,
            ai_feedback_opt_out: None,
        };

        let input = payload.into_input();
        assert_eq!(input.deepseek_api_key, Some(None));
    }

    #[test]
    fn test_dashboard_config_payload_modules() {
        let mut modules = HashMap::new();
        modules.insert("quick-actions".to_string(), false);
        let payload = DashboardConfigUpdatePayload {
            modules: Some(modules),
            last_updated_at: None,
        };

        let input = payload.into_input();
        let overrides = input.modules.expect("modules should be present");
        assert_eq!(overrides.get("quick-actions"), Some(&false));
        assert!(input.last_updated_at.is_none());
    }

    #[test]
    fn test_dashboard_config_payload_null_timestamp() {
        let payload = DashboardConfigUpdatePayload {
            modules: None,
            last_updated_at: Some(None),
        };

        let input = payload.into_input();
        assert!(input.modules.is_none());
        assert_eq!(input.last_updated_at, Some(None));
    }
}
