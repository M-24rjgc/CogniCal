use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use chrono::{DateTime, Duration, TimeZone, Utc};
use tracing::{debug, error, info};

use crate::db::repositories::task_repository::TaskRepository;
use crate::db::repositories::workload_repository::WorkloadRepository;
use crate::db::DbPool;
use crate::error::AppResult;
use crate::models::workload::{
    ContributingTaskSummary, WorkloadForecastRecord, WorkloadForecastResponse, WorkloadHorizon,
    WorkloadRiskLevel,
};
use crate::services::task_service::TaskService;

const DEFAULT_CAPACITY_THRESHOLD_HOURS: f64 = 40.0;
const LOW_CONFIDENCE_THRESHOLD: f64 = 0.4;
#[allow(dead_code)]
const MIN_HISTORICAL_DAYS: i64 = 7;
const WARNING_THRESHOLD_MULTIPLIER: f64 = 0.8;
const CRITICAL_THRESHOLD_MULTIPLIER: f64 = 1.0;

/// Service for forecasting workload and detecting capacity risks.
pub struct WorkloadForecastService {
    db: DbPool,
    #[allow(dead_code)]
    task_service: Arc<TaskService>,
    job_started: AtomicBool,
}

impl WorkloadForecastService {
    pub fn new(db: DbPool, task_service: Arc<TaskService>) -> Self {
        Self {
            db,
            task_service,
            job_started: AtomicBool::new(false),
        }
    }

    /// Generate forecasts for all horizons (7d, 14d, 30d).
    pub fn generate_forecasts(
        &self,
        capacity_threshold_hours: Option<f64>,
    ) -> AppResult<Vec<WorkloadForecastResponse>> {
        let threshold = capacity_threshold_hours.unwrap_or(DEFAULT_CAPACITY_THRESHOLD_HOURS);
        let now = Utc::now();

        let horizons = vec![
            WorkloadHorizon::SevenDays,
            WorkloadHorizon::FourteenDays,
            WorkloadHorizon::ThirtyDays,
        ];

        let mut results = Vec::new();

        for horizon in horizons {
            let forecast = self.generate_forecast_for_horizon(horizon, threshold, &now)?;
            results.push(forecast);
        }

        Ok(results)
    }

    /// Generate forecast for a specific horizon.
    fn generate_forecast_for_horizon(
        &self,
        horizon: WorkloadHorizon,
        capacity_threshold: f64,
        now: &DateTime<Utc>,
    ) -> AppResult<WorkloadForecastResponse> {
        let days = match horizon {
            WorkloadHorizon::SevenDays => 7,
            WorkloadHorizon::FourteenDays => 14,
            WorkloadHorizon::ThirtyDays => 30,
        };

        let end_date = *now + Duration::days(days);

        // Fetch pending and in-progress tasks
        let conn = self.db.get_connection()?;
        let tasks = TaskRepository::list_all(&conn)?;

        let pending_tasks: Vec<_> = tasks
            .into_iter()
            .filter(|task| {
                (task.status == "todo" || task.status == "in-progress")
                    && task
                        .due_at
                        .as_ref()
                        .and_then(|due| DateTime::parse_from_rfc3339(due).ok())
                        .map(|due| due.with_timezone(&Utc) <= end_date)
                        .unwrap_or(false)
            })
            .collect();

        // Calculate total workload
        let mut total_hours = 0.0;
        let mut contributing_tasks = Vec::new();

        for task in pending_tasks {
            let hours = task
                .estimated_hours
                .or_else(|| task.estimated_minutes.map(|m| m as f64 / 60.0))
                .unwrap_or(1.0); // Default 1 hour if no estimate

            total_hours += hours;

            contributing_tasks.push(ContributingTaskSummary {
                task_id: task.id.clone(),
                title: task.title.clone(),
                estimated_hours: hours,
                due_at: task.due_at.clone(),
                priority: task.priority.as_str().to_string(),
            });
        }

        // Calculate confidence based on historical data availability
        let confidence = self.calculate_confidence(&conn)?;

        // Determine risk level
        let risk_level = self.determine_risk_level(total_hours, capacity_threshold, confidence);

        // Create forecast record
        let record = WorkloadForecastRecord {
            horizon,
            generated_at: now.to_rfc3339(),
            risk_level,
            total_hours,
            capacity_threshold,
            contributing_tasks: contributing_tasks.clone(),
            confidence,
        };

        // Save to database
        WorkloadRepository::upsert_forecast(&conn, &record)?;

        info!(
            target: "app::workload_forecast",
            "Generated forecast for {}: total_hours={:.1}, risk_level={:?}, confidence={:.2}",
            horizon.as_str(),
            total_hours,
            risk_level,
            confidence
        );

        Ok(WorkloadForecastResponse {
            horizon: horizon.as_str().to_string(),
            generated_at: record.generated_at,
            risk_level: risk_level.as_str().to_string(),
            total_hours,
            capacity_threshold,
            contributing_tasks,
            confidence,
            recommendations: self.generate_recommendations(
                &risk_level,
                total_hours,
                capacity_threshold,
            ),
        })
    }

    /// Calculate confidence based on historical data availability.
    fn calculate_confidence(&self, conn: &rusqlite::Connection) -> AppResult<f64> {
        // Count completed tasks in the last 30 days
        let thirty_days_ago = (Utc::now() - Duration::days(30)).to_rfc3339();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM tasks WHERE status = 'completed' AND completed_at >= ?",
                [&thirty_days_ago],
                |row| row.get(0),
            )
            .unwrap_or(0);

        // Confidence increases with historical data
        // 0 tasks = 0.2, 10 tasks = 0.5, 30+ tasks = 0.9
        let confidence = if count == 0 {
            0.2
        } else if count < 10 {
            0.2 + (count as f64 * 0.03)
        } else if count < 30 {
            0.5 + ((count - 10) as f64 * 0.02)
        } else {
            0.9
        };

        Ok(confidence.min(1.0))
    }

    /// Determine risk level based on workload and capacity.
    fn determine_risk_level(
        &self,
        total_hours: f64,
        capacity_threshold: f64,
        confidence: f64,
    ) -> WorkloadRiskLevel {
        if confidence < LOW_CONFIDENCE_THRESHOLD {
            // Low confidence, use heuristic: if significantly over capacity, flag as warning
            if total_hours > capacity_threshold * 1.5 {
                return WorkloadRiskLevel::Warning;
            }
            return WorkloadRiskLevel::Ok;
        }

        let utilization = total_hours / capacity_threshold;

        if utilization >= CRITICAL_THRESHOLD_MULTIPLIER {
            WorkloadRiskLevel::Critical
        } else if utilization >= WARNING_THRESHOLD_MULTIPLIER {
            WorkloadRiskLevel::Warning
        } else {
            WorkloadRiskLevel::Ok
        }
    }

    /// Generate recommendations based on risk level.
    fn generate_recommendations(
        &self,
        risk_level: &WorkloadRiskLevel,
        total_hours: f64,
        capacity_threshold: f64,
    ) -> Vec<String> {
        match risk_level {
            WorkloadRiskLevel::Critical => vec![
                "⚠️ 工作负载严重超载，建议立即采取行动".to_string(),
                format!(
                    "当前预计工时 {:.1} 小时，超出容量阈值 {:.1} 小时",
                    total_hours, capacity_threshold
                ),
                "建议操作：".to_string(),
                "  • 重新评估任务优先级，推迟低优先级任务".to_string(),
                "  • 考虑委托部分任务给团队成员".to_string(),
                "  • 与相关方协商延长截止日期".to_string(),
                "  • 使用 AI 规划功能优化任务安排".to_string(),
            ],
            WorkloadRiskLevel::Warning => vec![
                "⚠️ 工作负载接近容量上限，需要关注".to_string(),
                format!(
                    "当前预计工时 {:.1} 小时，接近容量阈值 {:.1} 小时",
                    total_hours, capacity_threshold
                ),
                "建议操作：".to_string(),
                "  • 审查任务列表，确保估时准确".to_string(),
                "  • 预留缓冲时间应对意外情况".to_string(),
                "  • 考虑将大任务拆分为更小的部分".to_string(),
            ],
            WorkloadRiskLevel::Ok => vec![
                "✅ 工作负载在健康范围内".to_string(),
                format!(
                    "当前预计工时 {:.1} 小时，低于容量阈值 {:.1} 小时",
                    total_hours, capacity_threshold
                ),
                "继续保持良好的工作节奏！".to_string(),
            ],
        }
    }

    /// Get the latest forecast for a specific horizon.
    pub fn get_latest_forecast(
        &self,
        horizon: WorkloadHorizon,
    ) -> AppResult<Option<WorkloadForecastResponse>> {
        let conn = self.db.get_connection()?;
        let record = WorkloadRepository::latest_for_horizon(&conn, horizon)?;

        Ok(record.map(|r| WorkloadForecastResponse {
            horizon: r.horizon.as_str().to_string(),
            generated_at: r.generated_at,
            risk_level: r.risk_level.as_str().to_string(),
            total_hours: r.total_hours,
            capacity_threshold: r.capacity_threshold,
            contributing_tasks: r.contributing_tasks,
            confidence: r.confidence,
            recommendations: self.generate_recommendations(
                &r.risk_level,
                r.total_hours,
                r.capacity_threshold,
            ),
        }))
    }

    /// Get all latest forecasts.
    pub fn get_all_latest_forecasts(&self) -> AppResult<Vec<WorkloadForecastResponse>> {
        let horizons = vec![
            WorkloadHorizon::SevenDays,
            WorkloadHorizon::FourteenDays,
            WorkloadHorizon::ThirtyDays,
        ];

        let mut results = Vec::new();

        for horizon in horizons {
            if let Some(forecast) = self.get_latest_forecast(horizon)? {
                results.push(forecast);
            }
        }

        Ok(results)
    }

    /// Start the nightly forecast job.
    pub fn ensure_nightly_job(self: &Arc<Self>) -> AppResult<()> {
        if self
            .job_started
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            let service = Arc::clone(self);
            std::thread::spawn(move || {
                service.run_nightly_job();
            });
            info!(target: "app::workload_forecast", "Nightly forecast job started");
        }
        Ok(())
    }

    /// Run the nightly forecast job loop.
    fn run_nightly_job(&self) {
        loop {
            let now = Utc::now();
            let next_midnight = (now + Duration::days(1))
                .date_naive()
                .and_hms_opt(0, 5, 0) // Run at 00:05 AM
                .unwrap();
            let next_run = Utc.from_utc_datetime(&next_midnight);
            let wait_duration = (next_run - now)
                .to_std()
                .unwrap_or(std::time::Duration::from_secs(3600));

            debug!(
                target: "app::workload_forecast",
                "Waiting {} seconds until next forecast run",
                wait_duration.as_secs()
            );

            std::thread::sleep(wait_duration);

            // Run the forecast generation
            match self.generate_forecasts(None) {
                Ok(forecasts) => {
                    info!(
                        target: "app::workload_forecast",
                        "Nightly forecast completed: generated {} forecasts",
                        forecasts.len()
                    );
                }
                Err(err) => {
                    error!(
                        target: "app::workload_forecast",
                        "Nightly forecast failed: {}",
                        err
                    );
                }
            }
        }
    }
}
