use once_cell::sync::OnceCell;
use tauri::Manager;
use tracing_subscriber::{
    fmt, fmt::time::UtcTime, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

use crate::error::{AppError, AppResult};

static LOGGER_INIT: OnceCell<()> = OnceCell::new();
static LOGGER_GUARD: OnceCell<tracing_appender::non_blocking::WorkerGuard> = OnceCell::new();

const DEFAULT_LOG_DIRECTIVES: &str = "info,app::ai=debug,app::ai::cache=debug,app::db=info";

pub fn init_logging(app: &tauri::AppHandle) -> AppResult<()> {
    LOGGER_INIT
        .get_or_try_init(|| {
            let mut log_dir = app
                .path()
                .app_data_dir()
                .map_err(|err| AppError::other(format!("无法解析应用数据目录: {err}")))?;
            log_dir.push("logs");

            std::fs::create_dir_all(&log_dir)?;

            let file_appender = tracing_appender::rolling::daily(&log_dir, "cognical.log");
            let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

            let env_filter = EnvFilter::try_from_default_env()
                .or_else(|_| EnvFilter::try_new(DEFAULT_LOG_DIRECTIVES))
                .map_err(|err| AppError::other(format!("解析日志级别失败: {err}")))?;

            LOGGER_GUARD
                .set(guard)
                .map_err(|_| AppError::other("日志已初始化"))?;

            tracing_subscriber::registry()
                .with(env_filter)
                .with(
                    fmt::layer()
                        .with_writer(non_blocking)
                        .with_ansi(false)
                        .with_target(true)
                        .with_timer(UtcTime::rfc_3339()),
                )
                .with(
                    fmt::layer()
                        .with_target(false)
                        .with_timer(UtcTime::rfc_3339()),
                )
                .init();

            Ok(())
        })
        .map(|_| ())
}
