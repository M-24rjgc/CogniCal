use std::collections::HashSet;

use chrono::{DateTime, Utc};

use crate::db::repositories::task_repository::{TaskRepository, TaskRow};
use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::task::{
    TaskAiInsights, TaskCreateInput, TaskRecord, TaskRecurrence, TaskUpdateInput,
};
use tracing::{debug, info};

const VALID_STATUSES: &[&str] = &[
    "backlog",
    "todo",
    "in_progress",
    "blocked",
    "done",
    "archived",
];

const VALID_PRIORITIES: &[&str] = &["low", "medium", "high", "urgent"];

#[derive(Clone)]
pub struct TaskService {
    db: DbPool,
}

impl TaskService {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub fn create_task(&self, input: TaskCreateInput) -> AppResult<TaskRecord> {
        let mut record = build_record_from_create(input)?;
        let now = Utc::now().to_rfc3339();
        record.id = uuid::Uuid::new_v4().to_string();
        record.created_at = now.clone();
        record.updated_at = now;

        validate_record(&record)?;

        let row = TaskRow::from_record(&record)?;
        self.db
            .with_connection(|conn| TaskRepository::insert(conn, &row))?;
        info!(task_id = %record.id, "task created");
        Ok(record)
    }

    pub fn update_task(&self, id: &str, update: TaskUpdateInput) -> AppResult<TaskRecord> {
        let mut existing = self.get_task(id)?;
        apply_update(&mut existing, update)?;
        existing.updated_at = Utc::now().to_rfc3339();
        validate_record(&existing)?;

        let row = TaskRow::from_record(&existing)?;
        self.db
            .with_connection(|conn| TaskRepository::update(conn, &row))?;
        info!(task_id = %existing.id, "task updated");
        Ok(existing)
    }

    pub fn delete_task(&self, id: &str) -> AppResult<()> {
        self.db
            .with_connection(|conn| TaskRepository::delete(conn, id))
            .map(|_| ())?;
        info!(task_id = %id, "task deleted");
        Ok(())
    }

    pub fn get_task(&self, id: &str) -> AppResult<TaskRecord> {
        let row = self
            .db
            .with_connection(|conn| TaskRepository::find_by_id(conn, id))?
            .ok_or_else(AppError::not_found)?;
        let record = row.into_record()?;
        debug!(task_id = %record.id, "task fetched");
        Ok(record)
    }

    pub fn list_tasks(&self) -> AppResult<Vec<TaskRecord>> {
        let rows = self
            .db
            .with_connection(|conn| TaskRepository::list_all(conn))?;
        let tasks = rows
            .into_iter()
            .map(|row| row.into_record())
            .collect::<AppResult<Vec<_>>>()?;
        debug!(count = tasks.len(), "tasks listed");
        Ok(tasks)
    }

    pub fn pool(&self) -> &DbPool {
        &self.db
    }
}

fn build_record_from_create(mut input: TaskCreateInput) -> AppResult<TaskRecord> {
    let title = normalize_title(&input.title)?;
    let description = normalize_optional_string(input.description.take());
    let status = normalize_status(input.status.take())?;
    let priority = normalize_priority(input.priority.take())?;
    let planned_start_at = normalize_datetime_opt(input.planned_start_at.take())?;
    let start_at = normalize_datetime_opt(input.start_at.take())?;
    let due_at = normalize_datetime_opt(input.due_at.take())?;
    let completed_at = normalize_datetime_opt(input.completed_at.take())?;
    let estimated_minutes = normalize_estimated_minutes(input.estimated_minutes.take())?;
    let estimated_hours = normalize_estimated_hours(input.estimated_hours.take())?;
    let tags = normalize_string_vec(input.tags.take().unwrap_or_default())?;
    let external_links = normalize_links(input.external_links.take().unwrap_or_default())?;
    let owner_id = normalize_optional_string(input.owner_id.take());
    let is_recurring = input.is_recurring.unwrap_or(false);
    let recurrence = normalize_recurrence(is_recurring, input.recurrence.take())?;
    let task_type = normalize_optional_string(input.task_type.take());
    let ai = normalize_ai(input.ai.take())?;

    Ok(TaskRecord {
        id: String::new(),
        title,
        description,
        status,
        priority,
        planned_start_at,
        start_at,
        due_at,
        completed_at,
        estimated_minutes,
        estimated_hours,
        tags,
        owner_id,
        task_type,
        is_recurring,
        recurrence,
        ai,
        external_links,
        created_at: String::new(),
        updated_at: String::new(),
    })
}

fn apply_update(record: &mut TaskRecord, update: TaskUpdateInput) -> AppResult<()> {
    if let Some(title) = update.title {
        record.title = normalize_title(&title)?;
    }

    if let Some(description) = update.description {
        record.description = normalize_optional_string(description);
    }

    if let Some(status) = update.status {
        record.status = normalize_status(Some(status))?;
    }

    if let Some(priority) = update.priority {
        record.priority = normalize_priority(Some(priority))?;
    }

    if let Some(planned_start_at) = update.planned_start_at {
        record.planned_start_at = normalize_datetime_opt(planned_start_at)?;
    }

    if let Some(start_at) = update.start_at {
        record.start_at = normalize_datetime_opt(start_at)?;
    }

    if let Some(due_at) = update.due_at {
        record.due_at = normalize_datetime_opt(due_at)?;
    }

    if let Some(completed_at) = update.completed_at {
        record.completed_at = normalize_datetime_opt(completed_at)?;
    }

    if let Some(estimated_minutes) = update.estimated_minutes {
        record.estimated_minutes = normalize_estimated_minutes(estimated_minutes)?;
    }

    if let Some(estimated_hours) = update.estimated_hours {
        record.estimated_hours = normalize_estimated_hours(estimated_hours)?;
    }

    if let Some(tags) = update.tags {
        let values = tags.unwrap_or_default();
        record.tags = normalize_string_vec(values)?;
    }

    if let Some(owner_id) = update.owner_id {
        record.owner_id = normalize_optional_string(owner_id);
    }

    if let Some(is_recurring) = update.is_recurring {
        record.is_recurring = is_recurring;
        if !record.is_recurring {
            record.recurrence = None;
        }
    }

    if let Some(recurrence) = update.recurrence {
        record.recurrence = normalize_recurrence(record.is_recurring, recurrence)?;
    }

    if let Some(task_type) = update.task_type {
        record.task_type = normalize_optional_string(task_type);
    }

    if let Some(ai) = update.ai {
        record.ai = normalize_ai(ai)?;
    }

    if let Some(external_links) = update.external_links {
        let values = external_links.unwrap_or_default();
        record.external_links = normalize_links(values)?;
    }

    Ok(())
}

fn validate_record(record: &TaskRecord) -> AppResult<()> {
    if record.is_recurring && record.recurrence.is_none() {
        return Err(AppError::validation("循环任务必须提供重复规则"));
    }

    if !record.is_recurring && record.recurrence.is_some() {
        return Err(AppError::validation("非循环任务无需提供重复信息"));
    }

    if let (Some(start), Some(due)) = (record.start_at.as_ref(), record.due_at.as_ref()) {
        let start_dt = DateTime::parse_from_rfc3339(start)
            .map_err(|_| AppError::validation("开始时间格式非法"))?;
        let due_dt = DateTime::parse_from_rfc3339(due)
            .map_err(|_| AppError::validation("截止时间格式非法"))?;
        if due_dt < start_dt {
            return Err(AppError::validation("截止时间不能早于开始时间"));
        }
    }

    Ok(())
}

fn normalize_title(title: &str) -> AppResult<String> {
    let trimmed = title.trim();
    if trimmed.is_empty() {
        return Err(AppError::validation("标题不能为空"));
    }
    if trimmed.chars().count() > 160 {
        return Err(AppError::validation("标题长度需在 160 字以内"));
    }
    Ok(trimmed.to_string())
}

fn normalize_status(status: Option<String>) -> AppResult<String> {
    let value = status.unwrap_or_else(|| "todo".to_string()).to_lowercase();
    if VALID_STATUSES.contains(&value.as_str()) {
        Ok(value)
    } else {
        Err(AppError::validation("状态取值非法"))
    }
}

fn normalize_priority(priority: Option<String>) -> AppResult<String> {
    let value = priority
        .unwrap_or_else(|| "medium".to_string())
        .to_lowercase();
    if VALID_PRIORITIES.contains(&value.as_str()) {
        Ok(value)
    } else {
        Err(AppError::validation("优先级取值非法"))
    }
}

fn normalize_optional_string(value: Option<String>) -> Option<String> {
    value.and_then(|val| {
        let trimmed = val.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

fn normalize_datetime_opt(value: Option<String>) -> AppResult<Option<String>> {
    if let Some(value) = value {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Ok(None);
        }
        DateTime::parse_from_rfc3339(trimmed).map_err(|_| AppError::validation("时间格式非法"))?;
        Ok(Some(trimmed.to_string()))
    } else {
        Ok(None)
    }
}

fn normalize_estimated_minutes(value: Option<i64>) -> AppResult<Option<i64>> {
    if let Some(minutes) = value {
        if minutes <= 0 {
            return Err(AppError::validation("预估时长需大于 0"));
        }
        if minutes > 60 * 24 * 30 {
            return Err(AppError::validation("预估时长不能超过 30 天"));
        }
        Ok(Some(minutes))
    } else {
        Ok(None)
    }
}

fn normalize_estimated_hours(value: Option<f64>) -> AppResult<Option<f64>> {
    if let Some(hours) = value {
        if !hours.is_finite() || hours <= 0.0 {
            return Err(AppError::validation("预估小时需大于 0 且必须为有效数值"));
        }
        if hours > 24.0 * 30.0 {
            return Err(AppError::validation("预估小时不能超过 30 天"));
        }
        Ok(Some(hours))
    } else {
        Ok(None)
    }
}

fn normalize_string_vec(values: Vec<String>) -> AppResult<Vec<String>> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();

    for value in values {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.chars().count() > 32 {
            return Err(AppError::validation("单个标签长度需小于 32 字符"));
        }
        if seen.insert(trimmed.to_lowercase()) {
            result.push(trimmed.to_string());
        }
        if result.len() > 30 {
            return Err(AppError::validation("标签数量最多 30 个"));
        }
    }

    Ok(result)
}

fn normalize_links(values: Vec<String>) -> AppResult<Vec<String>> {
    let mut result = Vec::new();
    for value in values {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !(trimmed.starts_with("http://") || trimmed.starts_with("https://")) {
            return Err(AppError::validation("链接必须以 http:// 或 https:// 开头"));
        }
        result.push(trimmed.to_string());
        if result.len() > 20 {
            return Err(AppError::validation("链接数量最多 20 个"));
        }
    }
    Ok(result)
}

fn normalize_recurrence(
    is_recurring: bool,
    recurrence: Option<TaskRecurrence>,
) -> AppResult<Option<TaskRecurrence>> {
    if let Some(mut recurrence) = recurrence {
        if !is_recurring {
            return Err(AppError::validation("非循环任务无需提供重复信息"));
        }

        recurrence.rule = recurrence.rule.trim().to_string();
        if recurrence.rule.is_empty() {
            return Err(AppError::validation("重复规则不能为空"));
        }

        recurrence.until = normalize_datetime_opt(recurrence.until.take())?;

        Ok(Some(recurrence))
    } else {
        Ok(None)
    }
}

fn normalize_ai(ai: Option<TaskAiInsights>) -> AppResult<Option<TaskAiInsights>> {
    if let Some(mut ai) = ai {
        ai.summary = ai
            .summary
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        ai.next_action = ai
            .next_action
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        if let Some(confidence) = ai.confidence {
            if !(0.0..=1.0).contains(&confidence) {
                return Err(AppError::validation("置信度需在 0 到 1 之间"));
            }
        }
        Ok(Some(ai))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::DbPool;
    use tempfile::tempdir;

    fn setup_service() -> (TaskService, tempfile::TempDir) {
        let dir = tempdir().expect("temp dir");
        let db_path = dir.path().join("tasks.sqlite");
        let pool = DbPool::new(db_path).expect("db pool");
        (TaskService::new(pool), dir)
    }

    #[test]
    fn create_and_fetch_task() {
        let (service, _dir) = setup_service();
        let record = service
            .create_task(TaskCreateInput {
                title: "测试任务".into(),
                ..Default::default()
            })
            .expect("create task");

        assert!(!record.id.is_empty());
        assert_eq!(record.status, "todo");
        assert_eq!(record.priority, "medium");

        let fetched = service.get_task(&record.id).expect("get task");
        assert_eq!(fetched.id, record.id);
        assert_eq!(fetched.title, "测试任务");
    }

    #[test]
    fn update_task_fields() {
        let (service, _dir) = setup_service();
        let record = service
            .create_task(TaskCreateInput {
                title: "原始标题".into(),
                ..Default::default()
            })
            .expect("create task");

        let updated = service
            .update_task(
                &record.id,
                TaskUpdateInput {
                    title: Some("更新后的标题".into()),
                    status: Some("in_progress".into()),
                    priority: Some("high".into()),
                    tags: Some(Some(vec!["rust".into(), "database".into()])),
                    ..Default::default()
                },
            )
            .expect("update task");

        assert_eq!(updated.title, "更新后的标题");
        assert_eq!(updated.status, "in_progress");
        assert_eq!(updated.priority, "high");
        assert_eq!(updated.tags, vec!["rust", "database"]);
        assert_ne!(updated.updated_at, record.updated_at);
    }

    #[test]
    fn create_task_validates_status() {
        let (service, _dir) = setup_service();
        let result = service.create_task(TaskCreateInput {
            title: "任务".into(),
            status: Some("invalid".into()),
            ..Default::default()
        });

        assert!(matches!(result, Err(AppError::Validation { .. })));
    }

    #[test]
    fn delete_task_removes_record() {
        let (service, _dir) = setup_service();
        let record = service
            .create_task(TaskCreateInput {
                title: "删除测试".into(),
                ..Default::default()
            })
            .expect("create task");

        service.delete_task(&record.id).expect("delete task");
        let result = service.get_task(&record.id);
        assert!(matches!(result, Err(AppError::NotFound)));
    }
}
