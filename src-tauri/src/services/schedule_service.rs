use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc, Weekday};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};
use crate::models::task::{TaskRecord, TaskUpdateInput};
use crate::services::task_service::TaskService;
use tracing::{debug, info};

/// Unified schedule service that treats tasks and calendar events as one
/// This service extends TaskService with calendar-like query capabilities
#[derive(Clone)]
pub struct ScheduleService {
    task_service: TaskService,
}

impl ScheduleService {
    pub fn new(task_service: TaskService) -> Self {
        Self { task_service }
    }

    /// Get all scheduled items (tasks with time components) for a date range
    /// This includes both time-blocked tasks and deadline tasks
    pub async fn get_schedule_for_range(
        &self,
        start_date: &str,
        end_date: &str,
    ) -> AppResult<Vec<ScheduledItem>> {
        debug!(start_date, end_date, "fetching schedule range");

        let all_tasks = self.task_service.list_tasks()?;
        let mut scheduled_items = Vec::new();

        for task in all_tasks {
            if let Some(item) = self.task_to_scheduled_item(task)? {
                // Check if this task falls within the requested date range
                if self.is_task_in_date_range(&item, start_date, end_date)? {
                    scheduled_items.push(item);
                }
            }
        }

        // Sort by start time or due time
        scheduled_items.sort_by(|a, b| {
            let a_time = a.start_time_or_due_time();
            let b_time = b.start_time_or_due_time();
            a_time.cmp(&b_time)
        });

        debug!(
            start_date,
            end_date,
            count = scheduled_items.len(),
            "schedule range fetched"
        );

        Ok(scheduled_items)
    }

    /// Create a time-blocked task (equivalent to calendar event)
    pub async fn create_time_block(
        &self,
        title: &str,
        start_datetime: &str,
        duration_minutes: i64,
        description: Option<&str>,
    ) -> AppResult<ScheduledItem> {
        info!(
            title,
            start_datetime, duration_minutes, "creating time block"
        );

        // Calculate end time
        let start_dt = DateTime::parse_from_rfc3339(start_datetime)
            .map_err(|e| AppError::validation(format!("Invalid start datetime: {}", e)))?;
        let end_dt = start_dt + Duration::minutes(duration_minutes);

        // Create task with time block
        let task_input = crate::models::task::TaskCreateInput {
            title: title.to_string(),
            description: description.map(|s| s.to_string()),
            status: Some("todo".to_string()),
            priority: Some("medium".to_string()),
            planned_start_at: Some(start_datetime.to_string()),
            start_at: Some(start_datetime.to_string()),
            due_at: Some(end_dt.to_rfc3339()),
            completed_at: None,
            estimated_minutes: Some(duration_minutes),
            estimated_hours: None,
            tags: None,
            owner_id: None,
            is_recurring: None,
            recurrence: None,
            task_type: Some("time_block".to_string()),
            ai: None,
            external_links: None,
        };

        let task_record = self.task_service.create_task(task_input)?;
        let scheduled_item = self.task_to_scheduled_item(task_record)?.ok_or_else(|| {
            AppError::Other("Failed to convert task to scheduled item".to_string())
        })?;

        info!(
            task_id = %scheduled_item.id,
            "time block created successfully"
        );

        Ok(scheduled_item)
    }

    /// Update a scheduled item's time
    pub async fn update_schedule_time(
        &self,
        id: &str,
        start_datetime: Option<&str>,
        end_datetime: Option<&str>,
        duration_minutes: Option<i64>,
    ) -> AppResult<ScheduledItem> {
        debug!(id, "updating schedule time");

        let _task = self.task_service.get_task(id)?;
        let mut update_input = TaskUpdateInput::default();

        // Handle different update scenarios
        if let Some(start_dt) = start_datetime {
            update_input.planned_start_at = Some(Some(start_dt.to_string()));
            update_input.start_at = Some(Some(start_dt.to_string()));
        }

        if let Some(end_dt) = end_datetime {
            update_input.due_at = Some(Some(end_dt.to_string()));
        }

        if let Some(duration) = duration_minutes {
            update_input.estimated_minutes = Some(Some(duration));
        }

        // Ensure we have either start time or both start and end time
        if update_input.planned_start_at.is_none() && update_input.due_at.is_none() {
            return Err(AppError::validation(
                "Must provide at least start time or end time for schedule update",
            ));
        }

        // If we have start time but no end time, and duration is provided, calculate end time
        if let (Some(Some(start_dt)), Some(duration)) =
            (&update_input.planned_start_at, duration_minutes)
        {
            if update_input.due_at.is_none() {
                let parsed_start = DateTime::parse_from_rfc3339(start_dt)
                    .map_err(|e| AppError::validation(format!("Invalid start datetime: {}", e)))?;
                let end_dt = parsed_start + Duration::minutes(duration);
                update_input.due_at = Some(Some(end_dt.to_rfc3339()));
            }
        }

        // If we have end time but no start time, and duration is provided, calculate start time
        let updated_task = if let (Some(Some(end_dt)), Some(duration)) =
            (&update_input.due_at, duration_minutes)
        {
            if update_input.planned_start_at.is_none() {
                let parsed_end = DateTime::parse_from_rfc3339(end_dt)
                    .map_err(|e| AppError::validation(format!("Invalid end datetime: {}", e)))?;
                let start_dt = parsed_end - Duration::minutes(duration);
                let mut updated_input = update_input.clone();
                updated_input.planned_start_at = Some(Some(start_dt.to_rfc3339()));
                updated_input.start_at = Some(Some(start_dt.to_rfc3339()));
                self.task_service.update_task(id, updated_input)?
            } else {
                self.task_service.update_task(id, update_input)?
            }
        } else {
            self.task_service.update_task(id, update_input)?
        };
        let scheduled_item = self.task_to_scheduled_item(updated_task)?.ok_or_else(|| {
            AppError::Other("Failed to convert updated task to scheduled item".to_string())
        })?;

        debug!(id, "schedule time updated successfully");

        Ok(scheduled_item)
    }

    /// Convert TaskRecord to ScheduledItem
    fn task_to_scheduled_item(&self, task: TaskRecord) -> AppResult<Option<ScheduledItem>> {
        // Only convert tasks that have time components
        if task.planned_start_at.is_none() && task.due_at.is_none() {
            return Ok(None);
        }

        let item_type = if task.planned_start_at.is_some() && task.due_at.is_some() {
            ScheduleItemType::TimeBlock
        } else if task.due_at.is_some() {
            ScheduleItemType::Deadline
        } else {
            ScheduleItemType::StartOnly
        };

        let scheduled_item = ScheduledItem {
            id: task.id,
            title: task.title,
            description: task.description,
            item_type,
            start_at: task.planned_start_at,
            end_at: task.due_at,
            status: task.status,
            priority: task.priority,
            tags: task.tags,
            task_type: task.task_type,
            created_at: task.created_at,
            updated_at: task.updated_at,
        };

        Ok(Some(scheduled_item))
    }

    /// Check if a scheduled item falls within a date range
    fn is_task_in_date_range(
        &self,
        item: &ScheduledItem,
        start_date: &str,
        end_date: &str,
    ) -> AppResult<bool> {
        let start_date_parsed = NaiveDate::parse_from_str(start_date, "%Y-%m-%d")
            .map_err(|e| AppError::validation(format!("Invalid start date format: {}", e)))?;
        let end_date_parsed = NaiveDate::parse_from_str(end_date, "%Y-%m-%d")
            .map_err(|e| AppError::validation(format!("Invalid end date format: {}", e)))?;

        // Check if the task's time range overlaps with the requested range
        if let (Some(start_at), Some(end_at)) = (&item.start_at, &item.end_at) {
            let task_start = DateTime::parse_from_rfc3339(start_at)
                .map_err(|e| AppError::validation(format!("Invalid task start time: {}", e)))?;
            let task_end = DateTime::parse_from_rfc3339(end_at)
                .map_err(|e| AppError::validation(format!("Invalid task end time: {}", e)))?;

            let range_start = start_date_parsed.and_hms_opt(0, 0, 0).unwrap().and_utc();
            let range_end = end_date_parsed.and_hms_opt(23, 59, 59).unwrap().and_utc();

            // Check for overlap
            Ok(task_start <= range_end && task_end >= range_start)
        } else if let Some(end_at) = &item.end_at {
            let task_due = DateTime::parse_from_rfc3339(end_at)
                .map_err(|e| AppError::validation(format!("Invalid task due time: {}", e)))?;

            let range_start = start_date_parsed.and_hms_opt(0, 0, 0).unwrap().and_utc();
            let range_end = end_date_parsed.and_hms_opt(23, 59, 59).unwrap().and_utc();

            Ok(task_due >= range_start && task_due <= range_end)
        } else if let Some(start_at) = &item.start_at {
            let task_start = DateTime::parse_from_rfc3339(start_at)
                .map_err(|e| AppError::validation(format!("Invalid task start time: {}", e)))?;

            let range_start = start_date_parsed.and_hms_opt(0, 0, 0).unwrap().and_utc();
            let range_end = end_date_parsed.and_hms_opt(23, 59, 59).unwrap().and_utc();

            Ok(task_start >= range_start && task_start <= range_end)
        } else {
            Ok(false)
        }
    }

    /// Get today's schedule
    pub async fn get_today_schedule(&self) -> AppResult<Vec<ScheduledItem>> {
        let today = Utc::now().date_naive();
        let tomorrow = today + Duration::days(1);

        self.get_schedule_for_range(
            &today.format("%Y-%m-%d").to_string(),
            &tomorrow.format("%Y-%m-%d").to_string(),
        )
        .await
    }

    /// Get this week's schedule
    pub async fn get_week_schedule(&self) -> AppResult<Vec<ScheduledItem>> {
        let now = Utc::now();
        let today = now.date_naive();
        let weekday = now.weekday();
        let days_since_monday = match weekday {
            Weekday::Mon => 0,
            Weekday::Tue => 1,
            Weekday::Wed => 2,
            Weekday::Thu => 3,
            Weekday::Fri => 4,
            Weekday::Sat => 5,
            Weekday::Sun => 6,
        };
        let week_start = today - Duration::days(days_since_monday as i64);
        let week_end = week_start + Duration::days(7);

        self.get_schedule_for_range(
            &week_start.format("%Y-%m-%d").to_string(),
            &week_end.format("%Y-%m-%d").to_string(),
        )
        .await
    }
}

/// Unified representation of time-based items (tasks with time components)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduledItem {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub item_type: ScheduleItemType,
    pub start_at: Option<String>, // RFC3339 format
    pub end_at: Option<String>,   // RFC3339 format
    pub status: String,
    pub priority: String,
    pub tags: Vec<String>,
    pub task_type: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ScheduleItemType {
    TimeBlock, // Has both start and end time (like calendar event)
    Deadline,  // Has only due time
    StartOnly, // Has only start time
}

impl ScheduledItem {
    /// Get the primary time reference (start time if available, otherwise due time)
    pub fn start_time_or_due_time(&self) -> chrono::DateTime<chrono::FixedOffset> {
        if let Some(start_at) = &self.start_at {
            DateTime::parse_from_rfc3339(start_at).unwrap_or_else(|_| {
                // Fallback to due time if start time is invalid
                DateTime::parse_from_rfc3339(self.end_at.as_ref().unwrap()).unwrap_or_else(|_| {
                    Utc::now().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap())
                })
            })
        } else if let Some(end_at) = &self.end_at {
            DateTime::parse_from_rfc3339(end_at).unwrap_or_else(|_| {
                Utc::now().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap())
            })
        } else {
            Utc::now().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap())
        }
    }

    /// Get duration in minutes (if calculable)
    pub fn duration_minutes(&self) -> Option<i64> {
        if let (Some(start_at), Some(end_at)) = (&self.start_at, &self.end_at) {
            if let (Ok(start), Ok(end)) = (
                DateTime::parse_from_rfc3339(start_at),
                DateTime::parse_from_rfc3339(end_at),
            ) {
                Some(end.signed_duration_since(start).num_minutes())
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Check if this item is overdue
    pub fn is_overdue(&self) -> bool {
        if let (Some(end_at), "todo") = (&self.end_at, self.status.as_str()) {
            if let Ok(end_time) = DateTime::parse_from_rfc3339(end_at) {
                return end_time
                    < Utc::now().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap());
            }
        }
        false
    }

    /// Format for human-readable display
    pub fn format_display(&self) -> String {
        let mut result = format!("📋 {}\n", self.title);

        match self.item_type {
            ScheduleItemType::TimeBlock => {
                if let (Some(start), Some(end)) = (&self.start_at, &self.end_at) {
                    if let (Ok(start_dt), Ok(end_dt)) = (
                        DateTime::parse_from_rfc3339(start),
                        DateTime::parse_from_rfc3339(end),
                    ) {
                        result.push_str(&format!(
                            "🕒 {} - {}\n",
                            start_dt.format("%Y-%m-%d %H:%M"),
                            end_dt.format("%H:%M")
                        ));
                    }
                }
            }
            ScheduleItemType::Deadline => {
                if let Some(end_at) = &self.end_at {
                    if let Ok(end_dt) = DateTime::parse_from_rfc3339(end_at) {
                        let overdue = self.is_overdue();
                        result.push_str(&format!(
                            "⏰ {}{}\n",
                            if overdue { "⚠️ " } else { "" },
                            end_dt.format("%Y-%m-%d %H:%M")
                        ));
                    }
                }
            }
            ScheduleItemType::StartOnly => {
                if let Some(start_at) = &self.start_at {
                    if let Ok(start_dt) = DateTime::parse_from_rfc3339(start_at) {
                        result.push_str(&format!("▶️ {}\n", start_dt.format("%Y-%m-%d %H:%M")));
                    }
                }
            }
        }

        if !self.tags.is_empty() {
            result.push_str(&format!("🏷️ {}\n", self.tags.join(", ")));
        }

        if let Some(desc) = &self.description {
            result.push_str(&format!("📝 {}\n", desc));
        }

        result
    }
}
