use serde::Deserialize;
use tauri::{async_runtime, State};

use crate::error::AppError;
use crate::models::settings::AppSettings;
use crate::services::settings_service::SettingsUpdateInput;

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
}
