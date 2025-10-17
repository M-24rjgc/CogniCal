use std::collections::HashSet;

use chrono::DateTime;
use serde::{Deserialize, Serialize};
use tauri::{async_runtime, State};
use tracing::debug;

use crate::error::AppError;
use crate::models::task::{TaskCreateInput, TaskRecord, TaskUpdateInput};

use super::{AppState, CommandError, CommandResult};

const DEFAULT_PAGE_SIZE: usize = 20;
const MAX_PAGE_SIZE: usize = 200;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TaskListFilters {
    pub search: Option<String>,
    pub statuses: Option<Vec<String>>,
    pub priorities: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub owner_ids: Option<Vec<String>>,
    pub include_archived: Option<bool>,
    pub due_after: Option<String>,
    pub due_before: Option<String>,
    pub updated_after: Option<String>,
    pub updated_before: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub page: Option<usize>,
    pub page_size: Option<usize>,
}

impl Default for TaskListFilters {
    fn default() -> Self {
        Self {
            search: None,
            statuses: None,
            priorities: None,
            tags: None,
            owner_ids: None,
            include_archived: None,
            due_after: None,
            due_before: None,
            updated_after: None,
            updated_before: None,
            sort_by: Some("createdAt".to_string()),
            sort_order: Some("desc".to_string()),
            page: Some(1),
            page_size: Some(DEFAULT_PAGE_SIZE),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskListResponse {
    pub items: Vec<TaskRecord>,
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
}

#[tauri::command]
pub async fn tasks_list(
    state: State<'_, AppState>,
    filters: Option<TaskListFilters>,
) -> CommandResult<TaskListResponse> {
    let state = state.inner().clone();
    let filters = filters.unwrap_or_default();

    let records = run_blocking(move || state.tasks().list_tasks()).await?;
    let response = filter_and_paginate(records, filters);
    Ok(response)
}

#[tauri::command]
pub async fn tasks_create(
    state: State<'_, AppState>,
    payload: TaskCreateInput,
) -> CommandResult<TaskRecord> {
    let service = state.inner().clone();
    run_blocking(move || service.tasks().create_task(payload)).await
}

#[tauri::command]
pub async fn tasks_update(
    state: State<'_, AppState>,
    id: String,
    payload: TaskUpdateInput,
) -> CommandResult<TaskRecord> {
    let service = state.inner().clone();
    run_blocking(move || service.tasks().update_task(&id, payload)).await
}

#[tauri::command]
pub async fn tasks_delete(state: State<'_, AppState>, id: String) -> CommandResult<()> {
    let service = state.inner().clone();
    run_blocking(move || service.tasks().delete_task(&id)).await
}

async fn run_blocking<T: Send + 'static>(
    task: impl FnOnce() -> Result<T, AppError> + Send + 'static,
) -> CommandResult<T> {
    async_runtime::spawn_blocking(task)
        .await
        .map_err(|err| CommandError::new("UNKNOWN", format!("任务执行失败: {err}"), None))?
        .map_err(CommandError::from)
}

fn filter_and_paginate(records: Vec<TaskRecord>, filters: TaskListFilters) -> TaskListResponse {
    let include_archived = filters.include_archived.unwrap_or(false);
    let statuses = normalize_set(filters.statuses);
    let priorities = normalize_set(filters.priorities);
    let tags = normalize_set(filters.tags);
    let owner_ids = normalize_set(filters.owner_ids);
    let search = filters
        .search
        .map(|value| value.trim().to_lowercase())
        .filter(|v| !v.is_empty());

    let due_after = filters.due_after.as_deref().and_then(parse_timestamp);
    let due_before = filters.due_before.as_deref().and_then(parse_timestamp);
    let updated_after = filters.updated_after.as_deref().and_then(parse_timestamp);
    let updated_before = filters.updated_before.as_deref().and_then(parse_timestamp);

    let mut filtered: Vec<TaskRecord> = records
        .into_iter()
        .filter(|task| {
            match_filters(
                task,
                include_archived,
                &statuses,
                &priorities,
                &tags,
                &owner_ids,
                search.as_deref(),
                due_after,
                due_before,
                updated_after,
                updated_before,
            )
        })
        .collect();

    sort_tasks(
        &mut filtered,
        filters.sort_by.as_deref(),
        filters.sort_order.as_deref(),
    );

    let page = filters.page.unwrap_or(1).max(1);
    let page_size = filters
        .page_size
        .unwrap_or(DEFAULT_PAGE_SIZE)
        .clamp(1, MAX_PAGE_SIZE);

    let total = filtered.len();
    let start = (page - 1) * page_size;
    let end = (start + page_size).min(total);
    let items = if start >= total {
        Vec::new()
    } else {
        filtered[start..end].to_vec()
    };

    debug!(
        target: "app::command",
        total,
        page,
        page_size,
        returned = items.len(),
        "tasks_list"
    );

    TaskListResponse {
        items,
        total,
        page,
        page_size,
    }
}
fn match_filters(
    task: &TaskRecord,
    include_archived: bool,
    statuses: &HashSet<String>,
    priorities: &HashSet<String>,
    tags: &HashSet<String>,
    owner_ids: &HashSet<String>,
    search: Option<&str>,
    due_after: Option<i64>,
    due_before: Option<i64>,
    updated_after: Option<i64>,
    updated_before: Option<i64>,
) -> bool {
    if !include_archived && task.status == "archived" {
        return false;
    }

    if !statuses.is_empty() && !statuses.contains(&task.status) {
        return false;
    }

    if !priorities.is_empty() && !priorities.contains(&task.priority) {
        return false;
    }

    if !tags.is_empty()
        && task
            .tags
            .iter()
            .all(|tag| !tags.contains(&tag.to_lowercase()))
    {
        return false;
    }

    if !owner_ids.is_empty() {
        match task.owner_id.as_ref() {
            Some(owner) if owner_ids.contains(&owner.to_lowercase()) => {}
            _ => return false,
        }
    }

    if let Some(search) = search {
        let in_title = task.title.to_lowercase().contains(search);
        let in_description = task
            .description
            .as_ref()
            .map(|desc| desc.to_lowercase().contains(search))
            .unwrap_or(false);
        if !in_title && !in_description {
            return false;
        }
    }

    if (due_after.is_some() || due_before.is_some()) && task.due_at.is_none() {
        return false;
    }

    if let Some(boundary) = due_after {
        let due_ts = parse_timestamp_opt(task.due_at.as_deref());
        if due_ts.map(|ts| ts < boundary).unwrap_or(true) {
            return false;
        }
    }

    if let Some(boundary) = due_before {
        let due_ts = parse_timestamp_opt(task.due_at.as_deref());
        if due_ts.map(|ts| ts > boundary).unwrap_or(true) {
            return false;
        }
    }

    if let Some(boundary) = updated_after {
        let updated_ts = parse_timestamp_opt(Some(task.updated_at.as_str()));
        if updated_ts.map(|ts| ts < boundary).unwrap_or(true) {
            return false;
        }
    }

    if let Some(boundary) = updated_before {
        let updated_ts = parse_timestamp_opt(Some(task.updated_at.as_str()));
        if updated_ts.map(|ts| ts > boundary).unwrap_or(true) {
            return false;
        }
    }

    true
}

fn sort_tasks(tasks: &mut [TaskRecord], sort_by: Option<&str>, sort_order: Option<&str>) {
    let order_desc = sort_order.unwrap_or("desc").eq_ignore_ascii_case("desc");
    let key = sort_by.unwrap_or("createdAt");

    tasks.sort_by(|a, b| {
        let ordering = match key {
            "createdAt" => compare_timestamp(&a.created_at, &b.created_at),
            "updatedAt" => compare_timestamp(&a.updated_at, &b.updated_at),
            "dueAt" => compare_option_timestamp(a.due_at.as_ref(), b.due_at.as_ref()),
            "priority" => priority_rank(&a.priority).cmp(&priority_rank(&b.priority)),
            "status" => status_rank(&a.status).cmp(&status_rank(&b.status)),
            _ => compare_timestamp(&a.created_at, &b.created_at),
        };

        if order_desc {
            ordering.reverse()
        } else {
            ordering
        }
    });
}

fn compare_timestamp(a: &str, b: &str) -> std::cmp::Ordering {
    let ts_a = parse_timestamp(a).unwrap_or_default();
    let ts_b = parse_timestamp(b).unwrap_or_default();
    ts_a.cmp(&ts_b)
}

fn compare_option_timestamp(a: Option<&String>, b: Option<&String>) -> std::cmp::Ordering {
    if let (Some(a_ts), Some(b_ts)) = (a, b) {
        return compare_timestamp(a_ts, b_ts);
    }

    match (a.is_some(), b.is_some()) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => std::cmp::Ordering::Equal,
    }
}

fn parse_timestamp(value: &str) -> Option<i64> {
    DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.timestamp_millis())
        .ok()
}

fn parse_timestamp_opt(value: Option<&str>) -> Option<i64> {
    value.and_then(parse_timestamp)
}

fn priority_rank(priority: &str) -> usize {
    match priority {
        "urgent" => 0,
        "high" => 1,
        "medium" => 2,
        "low" => 3,
        _ => 4,
    }
}

fn status_rank(status: &str) -> usize {
    match status {
        "backlog" => 0,
        "todo" => 1,
        "in_progress" => 2,
        "blocked" => 3,
        "done" => 4,
        "archived" => 5,
        _ => 6,
    }
}

fn normalize_set(values: Option<Vec<String>>) -> HashSet<String> {
    values
        .unwrap_or_default()
        .into_iter()
        .map(|value| value.to_lowercase())
        .collect()
}
