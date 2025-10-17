use std::collections::HashMap;
use std::path::Path;
use std::sync::RwLock;

use base64::{engine::general_purpose::STANDARD as Base64, Engine as _};
use chrono::Utc;
use sha2::{Digest, Sha256};
use tracing::warn;

use crate::db::repositories::ai_settings_repository::AiSettingsRepository;
use crate::db::repositories::settings_repository::{AppSettingRow, SettingsRepository};
use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::settings::AppSettings;
use crate::utils::crypto::CryptoVault;

const KEY_DEEPSEEK_API: &str = "deepseek_api_key";
const KEY_WORKDAY_START: &str = "workday_start_minute";
const KEY_WORKDAY_END: &str = "workday_end_minute";
const KEY_THEME: &str = "theme";
const KEY_AI_FEEDBACK_OPT_OUT: &str = "ai_feedback_opt_out";

const DEFAULT_WORKDAY_START: i16 = 9 * 60;
const DEFAULT_WORKDAY_END: i16 = 18 * 60;
const DEFAULT_THEME: &str = "system";
const THEME_OPTIONS: [&str; 3] = ["system", "light", "dark"];

#[derive(Debug, Default, Clone)]
pub struct SettingsUpdateInput {
    pub deepseek_api_key: Option<Option<String>>,
    pub workday_start_minute: Option<i16>,
    pub workday_end_minute: Option<i16>,
    pub theme: Option<String>,
    pub ai_feedback_opt_out: Option<bool>,
}

pub struct SettingsService {
    db: DbPool,
    vault: CryptoVault,
    legacy_secret: [u8; 32],
    cache: RwLock<Option<AppSettings>>,
}

impl SettingsService {
    pub fn new(db: DbPool) -> AppResult<Self> {
        let vault = CryptoVault::from_database_path(db.path())?;
        let legacy_secret = derive_legacy_secret(db.path());
        Ok(Self {
            db,
            vault,
            legacy_secret,
            cache: RwLock::new(None),
        })
    }

    pub fn get(&self) -> AppResult<AppSettings> {
        if let Ok(guard) = self.cache.read() {
            if let Some(settings) = guard.as_ref() {
                return Ok(settings.clone());
            }
        }

        let settings = self.load_settings_from_db()?;
        if let Ok(mut guard) = self.cache.write() {
            *guard = Some(settings.clone());
        }
        Ok(settings)
    }

    pub fn update(&self, input: SettingsUpdateInput) -> AppResult<AppSettings> {
        let mut current = self.get()?;

        if let Some(workday_start) = input.workday_start_minute {
            ensure_valid_minute(workday_start)?;
            current.workday_start_minute = workday_start;
        }

        if let Some(workday_end) = input.workday_end_minute {
            ensure_valid_minute(workday_end)?;
            current.workday_end_minute = workday_end;
        }

        if current.workday_start_minute >= current.workday_end_minute {
            return Err(AppError::validation(
                "工作时间段无效：开始时间必须早于结束时间",
            ));
        }

        if let Some(theme) = input.theme.as_ref() {
            let normalized = theme.trim().to_lowercase();
            if !normalized.is_empty() && !THEME_OPTIONS.contains(&normalized.as_str()) {
                return Err(AppError::validation("主题仅支持 system、light 或 dark"));
            }
            if normalized.is_empty() {
                return Err(AppError::validation("主题不能为空"));
            }
            current.theme = normalized;
        }

        if let Some(opt_out) = input.ai_feedback_opt_out {
            current.ai_feedback_opt_out = Some(opt_out);
        }

        let api_key_instruction = self.prepare_api_key_instruction(&input)?;
        if let Some(masked) = api_key_instruction.masked.clone() {
            current.deepseek_api_key = Some(masked);
        } else if matches!(api_key_instruction.action, ApiKeyAction::Clear) {
            current.deepseek_api_key = None;
        }

        let now = Utc::now().to_rfc3339();
        self.persist_changes(&input, &api_key_instruction)?;
        current.updated_at = now;

        if let Ok(mut guard) = self.cache.write() {
            *guard = Some(current.clone());
        }

        Ok(current)
    }

    pub fn clear_sensitive(&self) -> AppResult<()> {
        self.db.with_connection(|conn| {
            AiSettingsRepository::delete(conn, KEY_DEEPSEEK_API)?;
            SettingsRepository::delete(conn, KEY_DEEPSEEK_API)?;
            Ok(())
        })?;

        if let Err(err) = self.vault.clear_master_secret() {
            warn!(
                target: "app::settings",
                error = %err,
                "failed to clear master secret from system keyring"
            );
        }

        if let Ok(mut guard) = self.cache.write() {
            if let Some(settings) = guard.as_mut() {
                settings.deepseek_api_key = None;
                settings.updated_at = Utc::now().to_rfc3339();
            }
        }

        Ok(())
    }

    fn persist_changes(
        &self,
        input: &SettingsUpdateInput,
        api_instr: &ApiKeyInstruction,
    ) -> AppResult<()> {
        let workday_start = input.workday_start_minute;
        let workday_end = input.workday_end_minute;
        let theme = input
            .theme
            .as_ref()
            .map(|value| value.trim().to_lowercase());
        let ai_feedback_opt_out = input.ai_feedback_opt_out;

        self.db.with_connection(|conn| {
            match api_instr.action {
                ApiKeyAction::Set => {
                    if let Some(cipher) = api_instr.ciphertext.as_ref() {
                        AiSettingsRepository::upsert(conn, KEY_DEEPSEEK_API, cipher)?;
                        SettingsRepository::delete(conn, KEY_DEEPSEEK_API)?;
                    }
                }
                ApiKeyAction::Clear => {
                    AiSettingsRepository::delete(conn, KEY_DEEPSEEK_API)?;
                    SettingsRepository::delete(conn, KEY_DEEPSEEK_API)?;
                }
                ApiKeyAction::NoChange => {}
            }

            if let Some(value) = workday_start {
                SettingsRepository::upsert(conn, KEY_WORKDAY_START, &value.to_string())?;
            }

            if let Some(value) = workday_end {
                SettingsRepository::upsert(conn, KEY_WORKDAY_END, &value.to_string())?;
            }

            if let Some(value) = theme {
                SettingsRepository::upsert(conn, KEY_THEME, &value)?;
            }

            if let Some(value) = ai_feedback_opt_out {
                SettingsRepository::upsert(conn, KEY_AI_FEEDBACK_OPT_OUT, &value.to_string())?;
            }

            Ok(())
        })
    }

    fn prepare_api_key_instruction(
        &self,
        input: &SettingsUpdateInput,
    ) -> AppResult<ApiKeyInstruction> {
        match &input.deepseek_api_key {
            None => Ok(ApiKeyInstruction::no_change()),
            Some(None) => Ok(ApiKeyInstruction::clear()),
            Some(Some(value)) => {
                let trimmed = value.trim();
                if trimmed.is_empty() {
                    return Err(AppError::validation("DeepSeek API Key 不能为空"));
                }
                let cipher = self.encrypt_api_key(trimmed)?;
                let masked = Some(mask_api_key(trimmed));
                Ok(ApiKeyInstruction::set(cipher, masked))
            }
        }
    }

    fn load_settings_from_db(&self) -> AppResult<AppSettings> {
        self.db.with_connection(|conn| {
            let rows = SettingsRepository::list(conn)?;
            let mut map: HashMap<String, AppSettingRow> = HashMap::new();
            let mut latest_updated_at: Option<String> = None;

            for row in rows {
                latest_updated_at = match latest_updated_at {
                    Some(ref current) if current >= &row.updated_at => Some(current.clone()),
                    _ => Some(row.updated_at.clone()),
                };
                map.insert(row.key.clone(), row);
            }

            let ai_row = AiSettingsRepository::get(conn, KEY_DEEPSEEK_API)?;
            if let Some(row) = ai_row.as_ref() {
                latest_updated_at = match latest_updated_at {
                    Some(ref current) if current >= &row.updated_at => Some(current.clone()),
                    _ => Some(row.updated_at.clone()),
                };
            }

            let deepseek_api_key = if let Some(row) = ai_row {
                match self.decrypt_api_key(&row.value) {
                    Ok(plain) => Some(mask_api_key(&plain)),
                    Err(err) => {
                        warn!(
                            target: "app::settings",
                            error = %err,
                            "failed to decrypt stored api key"
                        );
                        None
                    }
                }
            } else if let Some(row) = map.remove(KEY_DEEPSEEK_API) {
                match self.decrypt_legacy_api_key(&row.value) {
                    Ok(plain) => {
                        let masked = mask_api_key(&plain);
                        match self.encrypt_api_key(&plain) {
                            Ok(cipher) => {
                                if let Err(err) =
                                    AiSettingsRepository::upsert(conn, KEY_DEEPSEEK_API, &cipher)
                                {
                                    warn!(
                                        target: "app::settings",
                                        error = %err,
                                        "failed to migrate api key to secure storage"
                                    );
                                } else {
                                    if let Err(err) =
                                        SettingsRepository::delete(conn, KEY_DEEPSEEK_API)
                                    {
                                        warn!(
                                            target: "app::settings",
                                            error = %err,
                                            "failed to remove legacy api key entry"
                                        );
                                    }
                                    latest_updated_at = Some(Utc::now().to_rfc3339());
                                }
                            }
                            Err(err) => {
                                warn!(
                                    target: "app::settings",
                                    error = %err,
                                    "failed to re-encrypt api key during migration"
                                );
                            }
                        }
                        Some(masked)
                    }
                    Err(err) => {
                        warn!(
                            target: "app::settings",
                            error = %err,
                            "failed to decrypt legacy stored api key"
                        );
                        None
                    }
                }
            } else {
                None
            };

            let workday_start = map
                .get(KEY_WORKDAY_START)
                .and_then(|row| row.value.parse::<i16>().ok())
                .unwrap_or(DEFAULT_WORKDAY_START);

            let workday_end = map
                .get(KEY_WORKDAY_END)
                .and_then(|row| row.value.parse::<i16>().ok())
                .unwrap_or(DEFAULT_WORKDAY_END);

            if workday_start >= workday_end {
                warn!(
                    target: "app::settings",
                    start = workday_start,
                    end = workday_end,
                    "stored workday range invalid, falling back to defaults"
                );
            }

            let theme = map
                .get(KEY_THEME)
                .map(|row| row.value.to_lowercase())
                .filter(|value| THEME_OPTIONS.contains(&value.as_str()))
                .unwrap_or_else(|| DEFAULT_THEME.to_string());

            let ai_feedback_opt_out = map
                .get(KEY_AI_FEEDBACK_OPT_OUT)
                .and_then(|row| row.value.parse::<bool>().ok());

            let updated_at = latest_updated_at.unwrap_or_else(|| Utc::now().to_rfc3339());

            Ok(AppSettings {
                deepseek_api_key,
                workday_start_minute: if workday_start < workday_end {
                    workday_start
                } else {
                    DEFAULT_WORKDAY_START
                },
                workday_end_minute: if workday_start < workday_end {
                    workday_end
                } else {
                    DEFAULT_WORKDAY_END
                },
                theme,
                updated_at,
                ai_feedback_opt_out,
            })
        })
    }

    fn encrypt_api_key(&self, plaintext: &str) -> AppResult<String> {
        self.vault.encrypt(plaintext.as_bytes())
    }

    fn decrypt_api_key(&self, ciphertext: &str) -> AppResult<String> {
        if !ciphertext.starts_with("v1:") {
            return self.decrypt_legacy_api_key(ciphertext);
        }

        let plain = self.vault.decrypt(ciphertext)?;
        String::from_utf8(plain).map_err(|_| AppError::other("密钥内容包含非法字符"))
    }

    fn decrypt_legacy_api_key(&self, ciphertext: &str) -> AppResult<String> {
        let decoded = Base64
            .decode(ciphertext.as_bytes())
            .map_err(|_| AppError::other("密钥内容解析失败"))?;
        let plain = xor_with_secret(&decoded, &self.legacy_secret);
        String::from_utf8(plain).map_err(|_| AppError::other("密钥内容包含非法字符"))
    }
}

fn derive_legacy_secret(path: &Path) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"cognical.settings.v1");
    hasher.update(path.to_string_lossy().as_bytes());
    let result = hasher.finalize();
    let mut buf = [0u8; 32];
    buf.copy_from_slice(&result);
    buf
}

fn xor_with_secret(data: &[u8], secret: &[u8]) -> Vec<u8> {
    data.iter()
        .enumerate()
        .map(|(idx, byte)| byte ^ secret[idx % secret.len()])
        .collect()
}

fn mask_api_key(value: &str) -> String {
    let chars: Vec<char> = value.chars().collect();
    if chars.len() <= 4 {
        return "*".repeat(chars.len());
    }
    let visible: String = chars[chars.len() - 4..].iter().collect();
    let masked_prefix = "*".repeat(chars.len() - 4);
    format!("{}{}", masked_prefix, visible)
}

fn ensure_valid_minute(value: i16) -> AppResult<()> {
    if !(0..=1440).contains(&value) {
        return Err(AppError::validation("工作时间必须在 0~1440 分钟之间"));
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct ApiKeyInstruction {
    action: ApiKeyAction,
    ciphertext: Option<String>,
    masked: Option<String>,
}

impl ApiKeyInstruction {
    fn no_change() -> Self {
        Self {
            action: ApiKeyAction::NoChange,
            ciphertext: None,
            masked: None,
        }
    }

    fn clear() -> Self {
        Self {
            action: ApiKeyAction::Clear,
            ciphertext: None,
            masked: None,
        }
    }

    fn set(ciphertext: String, masked: Option<String>) -> Self {
        Self {
            action: ApiKeyAction::Set,
            ciphertext: Some(ciphertext),
            masked,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ApiKeyAction {
    Set,
    Clear,
    NoChange,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_service() -> (SettingsService, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("settings.db");
        let pool = DbPool::new(&db_path).unwrap();
        let service = SettingsService::new(pool).unwrap();
        (service, temp_dir)
    }

    #[test]
    fn defaults_are_returned_when_no_settings_exist() {
        let (service, _guard) = setup_service();
        let settings = service.get().unwrap();

        assert_eq!(settings.workday_start_minute, DEFAULT_WORKDAY_START);
        assert_eq!(settings.workday_end_minute, DEFAULT_WORKDAY_END);
        assert_eq!(settings.theme, DEFAULT_THEME);
        assert!(settings.deepseek_api_key.is_none());

        service.clear_sensitive().unwrap();
    }

    #[test]
    fn update_persists_and_masks_api_key() {
        let (service, _guard) = setup_service();
        let input = SettingsUpdateInput {
            deepseek_api_key: Some(Some("sk-test-123456".to_string())),
            workday_start_minute: Some(8 * 60),
            workday_end_minute: Some(17 * 60),
            theme: Some("dark".to_string()),
            ai_feedback_opt_out: None,
        };

        let updated = service.update(input).unwrap();
        assert_eq!(updated.theme, "dark");
        assert_eq!(updated.workday_start_minute, 480);
        assert_eq!(updated.workday_end_minute, 1020);
        assert_eq!(updated.deepseek_api_key, Some("**********3456".to_string()));

        let settings = service.get().unwrap();
        assert_eq!(
            settings.deepseek_api_key,
            Some("**********3456".to_string())
        );

        service.clear_sensitive().unwrap();
    }

    #[test]
    fn clear_sensitive_removes_api_key() {
        let (service, _guard) = setup_service();
        service
            .update(SettingsUpdateInput {
                deepseek_api_key: Some(Some("sk-should-remove".to_string())),
                ..Default::default()
            })
            .unwrap();

        service.clear_sensitive().unwrap();
        let settings = service.get().unwrap();
        assert!(settings.deepseek_api_key.is_none());

        service.clear_sensitive().unwrap();
    }

    #[test]
    fn legacy_api_key_is_migrated_to_secure_storage() {
        let (service, _guard) = setup_service();

        let legacy_cipher =
            Base64.encode(xor_with_secret(b"sk-legacy-9876", &service.legacy_secret));

        service
            .db
            .with_connection(|conn| {
                SettingsRepository::upsert(conn, KEY_DEEPSEEK_API, &legacy_cipher)
            })
            .unwrap();

        let settings = service.get().unwrap();
        assert_eq!(
            settings.deepseek_api_key,
            Some("**********9876".to_string())
        );

        service
            .db
            .with_connection(|conn| {
                let ai_row = AiSettingsRepository::get(conn, KEY_DEEPSEEK_API)?;
                assert!(ai_row.is_some());
                let legacy_row = SettingsRepository::get(conn, KEY_DEEPSEEK_API)?;
                assert!(legacy_row.is_none());
                Ok(())
            })
            .unwrap();

        service.clear_sensitive().unwrap();
    }
}
