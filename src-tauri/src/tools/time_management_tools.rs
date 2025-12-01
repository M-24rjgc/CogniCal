use crate::error::{AppError, AppResult};
use crate::services::schedule_service::ScheduleService;
use crate::services::task_service::TaskService;
use chrono::{Datelike, Local, LocalResult, TimeZone};
use serde::Deserialize;
use serde_json::{json, Value as JsonValue};
use std::sync::Arc;
use tracing::debug;

/// Unified time management tool schemas
/// These schemas replace the separate task_tools and calendar_tools

/// Get the schema for the list_time_items tool
pub fn list_time_items_schema() -> JsonValue {
    json!({
        "type": "object",
        "properties": {
            "date_range": {
                "type": "string",
                "enum": ["today", "week", "month", "custom"],
                "description": "Time range for listing items (default: today)"
            },
            "start_date": {
                "type": "string",
                "format": "date",
                "description": "Start date for custom range in YYYY-MM-DD format"
            },
            "end_date": {
                "type": "string",
                "format": "date",
                "description": "End date for custom range in YYYY-MM-DD format"
            },
            "item_type": {
                "type": "string",
                "enum": ["time_block", "deadline", "all"],
                "description": "Filter by item type: time_block (scheduled events), deadline (due dates), or all"
            },
            "status_filter": {
                "type": "array",
                "items": {"type": "string"},
                "description": "Filter by task status: backlog, todo, in_progress, blocked, done, archived"
            }
        },
        "required": []
    })
}

/// Get the schema for the create_time_block tool
pub fn create_time_block_schema() -> JsonValue {
    json!({
        "type": "object",
        "properties": {
            "title": {
                "type": "string",
                "description": "Title of the time-blocked item (required, max 160 characters)"
            },
            "start_datetime": {
                "type": "string",
                "format": "date-time",
                "description": "Start time in RFC3339 format (e.g., 2024-12-31T14:00:00Z) (required)"
            },
            "duration_minutes": {
                "type": "integer",
                "description": "Duration in minutes (required, must be > 0)"
            },
            "description": {
                "type": "string",
                "description": "Detailed description of the time block (optional)"
            },
            "tags": {
                "type": "array",
                "items": {"type": "string"},
                "description": "Tags to categorize the time block (max 30 tags, each max 32 chars)"
            }
        },
        "required": ["title", "start_datetime", "duration_minutes"]
    })
}

/// Get the schema for the update_time_item tool
pub fn update_time_item_schema() -> JsonValue {
    json!({
        "type": "object",
        "properties": {
            "id": {
                "type": "string",
                "description": "ID of the time item to update (required)"
            },
            "title": {
                "type": "string",
                "description": "New title for the time item (optional)"
            },
            "start_datetime": {
                "type": "string",
                "format": "date-time",
                "description": "New start time in RFC3339 format (optional)"
            },
            "end_datetime": {
                "type": "string",
                "format": "date-time",
                "description": "New end time in RFC3339 format (optional)"
            },
            "duration_minutes": {
                "type": "integer",
                "description": "New duration in minutes (optional)"
            },
            "description": {
                "type": "string",
                "description": "New description (optional)"
            },
            "status": {
                "type": "string",
                "enum": ["todo", "in_progress", "done", "blocked"],
                "description": "New status of the time item (optional)"
            }
        },
        "required": ["id"]
    })
}

/// Get the schema for the search_time_items tool
pub fn search_time_items_schema() -> JsonValue {
    json!({
        "type": "object",
        "properties": {
            "query": {
                "type": "string",
                "description": "Search query to match against titles and descriptions (required)"
            },
            "date_range": {
                "type": "string",
                "enum": ["today", "week", "month", "all"],
                "description": "Time range to search within (default: all)"
            },
            "item_type": {
                "type": "string",
                "enum": ["time_block", "deadline", "all"],
                "description": "Filter by item type (default: all)"
            }
        },
        "required": ["query"]
    })
}

/// Get the schema for the quick_schedule tool
pub fn quick_schedule_schema() -> JsonValue {
    json!({
        "type": "object",
        "properties": {
            "title": {
                "type": "string",
                "description": "Title for the quick scheduled item (required)"
            },
            "when": {
                "type": "string",
                "description": "When to schedule: 'today_9am', 'tomorrow_2pm', 'next_week_monday', or RFC3339 datetime (required)"
            },
            "duration_minutes": {
                "type": "integer",
                "description": "Duration in minutes (default: 60)"
            },
            "description": {
                "type": "string",
                "description": "Description (optional)"
            }
        },
        "required": ["title", "when"]
    })
}

/// Tool implementations

pub async fn list_time_items_tool(
    schedule_service: Arc<ScheduleService>,
    args: JsonValue,
) -> AppResult<JsonValue> {
    debug!("list_time_items_tool invoked");

    let params: ListTimeItemsParams = serde_json::from_value(args)
        .map_err(|e| AppError::validation(format!("Failed to parse parameters: {}", e)))?;

    let items = match params.date_range.as_str() {
        "today" => schedule_service.get_today_schedule().await?,
        "week" => schedule_service.get_week_schedule().await?,
        "custom" => {
            if let (Some(start), Some(end)) = (params.start_date, params.end_date) {
                schedule_service
                    .get_schedule_for_range(&start, &end)
                    .await?
            } else {
                return Err(AppError::validation(
                    "Custom date range requires both start_date and end_date",
                ));
            }
        }
        _ => schedule_service.get_today_schedule().await?, // default to today
    };

    // Apply filters
    let filtered_items: Vec<_> = items
        .into_iter()
        .filter(|item| {
            let mut keep_item = true;

            // Filter by item type
            if let Some(ref item_type) = params.item_type {
                match item_type.as_str() {
                    "time_block" => {
                        keep_item = item.item_type
                            == crate::services::schedule_service::ScheduleItemType::TimeBlock
                    }
                    "deadline" => {
                        keep_item = item.item_type
                            == crate::services::schedule_service::ScheduleItemType::Deadline
                    }
                    _ => {}
                }
            }

            // Filter by status
            if let Some(ref statuses) = params.status_filter {
                if keep_item {
                    keep_item = statuses
                        .iter()
                        .any(|status| status.eq_ignore_ascii_case(&item.status));
                }
            }

            keep_item
        })
        .collect();

    let items_json: Vec<JsonValue> = filtered_items
        .iter()
        .map(|item| {
            json!({
                "id": item.id,
                "title": item.title,
                "description": item.description,
                "type": format!("{:?}", item.item_type).to_lowercase(),
                "start_at": item.start_at,
                "end_at": item.end_at,
                "status": item.status,
                "priority": item.priority,
                "tags": item.tags,
                "duration_minutes": item.duration_minutes(),
                "is_overdue": item.is_overdue(),
                "formatted_display": item.format_display()
            })
        })
        .collect();

    let summary = format!(
        "📅 时间管理项目 ({} 个)\n\n{}",
        filtered_items.len(),
        if filtered_items.is_empty() {
            "没有找到匹配的时间项目".to_string()
        } else {
            filtered_items
                .iter()
                .map(|item| format!("• {}", item.title))
                .collect::<Vec<_>>()
                .join("\n")
        }
    );

    Ok(json!({
        "success": true,
        "items": items_json,
        "summary": summary,
        "count": filtered_items.len()
    }))
}

pub async fn create_time_block_tool(
    schedule_service: Arc<ScheduleService>,
    args: JsonValue,
) -> AppResult<JsonValue> {
    debug!("create_time_block_tool invoked");

    let params: CreateTimeBlockParams = serde_json::from_value(args)
        .map_err(|e| AppError::validation(format!("Failed to parse parameters: {}", e)))?;

    if params.duration_minutes <= 0 {
        return Err(AppError::validation(
            "Duration must be greater than 0 minutes",
        ));
    }

    // Normalize the start datetime into the local timezone so UI always sees the same-day value
    let parsed_start = chrono::DateTime::parse_from_rfc3339(&params.start_datetime)
        .map_err(|e| AppError::validation(format!("Invalid start datetime: {}", e)))?;
    let naive_local = parsed_start.naive_local();
    let local_start = match Local.from_local_datetime(&naive_local) {
        LocalResult::Single(dt) => dt,
        LocalResult::Ambiguous(first, _) => first,
        LocalResult::None => Local.from_utc_datetime(&parsed_start.naive_utc()),
    };
    let normalized_start = local_start.to_rfc3339();

    let scheduled_item = schedule_service
        .create_time_block(
            &params.title,
            &normalized_start,
            params.duration_minutes,
            params.description.as_deref(),
        )
        .await?;

    let result = json!({
        "success": true,
        "id": scheduled_item.id,
        "title": scheduled_item.title,
        "start_at": scheduled_item.start_at,
        "end_at": scheduled_item.end_at,
        "duration_minutes": scheduled_item.duration_minutes(),
        "formatted_display": scheduled_item.format_display(),
        "message": format!("✅ 时间块 '{}' 创建成功!", scheduled_item.title)
    });

    debug!(task_id = %scheduled_item.id, "time block created successfully");
    Ok(result)
}

pub async fn update_time_item_tool(
    schedule_service: Arc<ScheduleService>,
    args: JsonValue,
) -> AppResult<JsonValue> {
    debug!("update_time_item_tool invoked");

    let params: UpdateTimeItemParams = serde_json::from_value(args)
        .map_err(|e| AppError::validation(format!("Failed to parse parameters: {}", e)))?;

    let updated_item = schedule_service
        .update_schedule_time(
            &params.id,
            params.start_datetime.as_deref(),
            params.end_datetime.as_deref(),
            params.duration_minutes,
        )
        .await?;

    let result = json!({
        "success": true,
        "id": updated_item.id,
        "title": updated_item.title,
        "start_at": updated_item.start_at,
        "end_at": updated_item.end_at,
        "duration_minutes": updated_item.duration_minutes(),
        "formatted_display": updated_item.format_display(),
        "message": format!("✅ 时间项目 '{}' 更新成功!", updated_item.title)
    });

    debug!(task_id = %updated_item.id, "time item updated successfully");
    Ok(result)
}

pub async fn search_time_items_tool(
    schedule_service: Arc<ScheduleService>,
    args: JsonValue,
) -> AppResult<JsonValue> {
    debug!("search_time_items_tool invoked");

    let params: SearchTimeItemsParams = serde_json::from_value(args)
        .map_err(|e| AppError::validation(format!("Failed to parse parameters: {}", e)))?;

    // Get items for the specified date range
    let items = match params.date_range.as_str() {
        "today" => schedule_service.get_today_schedule().await?,
        "week" => schedule_service.get_week_schedule().await?,
        "all" => {
            // For "all", we'll search within the current month
            let today = chrono::Utc::now().date_naive();
            let month_start = today.with_day(1).unwrap_or(today);
            let month_end = month_start + chrono::Duration::days(31);
            schedule_service
                .get_schedule_for_range(
                    &month_start.format("%Y-%m-%d").to_string(),
                    &month_end.format("%Y-%m-%d").to_string(),
                )
                .await?
        }
        _ => schedule_service.get_today_schedule().await?, // default
    };

    // Search by query (simple text matching)
    let query_lower = params.query.to_lowercase();
    let matched_items: Vec<_> = items
        .into_iter()
        .filter(|item| {
            item.title.to_lowercase().contains(&query_lower)
                || item
                    .description
                    .as_ref()
                    .map(|d| d.to_lowercase().contains(&query_lower))
                    .unwrap_or(false)
                || item
                    .tags
                    .iter()
                    .any(|tag| tag.to_lowercase().contains(&query_lower))
        })
        .collect();

    // Apply item type filter
    let filtered_items: Vec<_> = if params.item_type.as_str() != "all" {
        matched_items
            .into_iter()
            .filter(|item| match params.item_type.as_str() {
                "time_block" => matches!(
                    item.item_type,
                    crate::services::schedule_service::ScheduleItemType::TimeBlock
                ),
                "deadline" => matches!(
                    item.item_type,
                    crate::services::schedule_service::ScheduleItemType::Deadline
                ),
                _ => true,
            })
            .collect()
    } else {
        matched_items
    };

    let items_json: Vec<JsonValue> = filtered_items
        .iter()
        .map(|item| {
            json!({
                "id": item.id,
                "title": item.title,
                "description": item.description,
                "type": format!("{:?}", item.item_type).to_lowercase(),
                "start_at": item.start_at,
                "end_at": item.end_at,
                "status": item.status,
                "tags": item.tags,
                "formatted_display": item.format_display()
            })
        })
        .collect();

    let summary = format!(
        "🔍 搜索结果: '{}' (找到 {} 个结果)\n\n{}",
        params.query,
        filtered_items.len(),
        if filtered_items.is_empty() {
            "没有找到匹配的时间项目".to_string()
        } else {
            filtered_items
                .iter()
                .map(|item| format!("• {}", item.title))
                .collect::<Vec<_>>()
                .join("\n")
        }
    );

    Ok(json!({
        "success": true,
        "query": params.query,
        "items": items_json,
        "summary": summary,
        "count": filtered_items.len()
    }))
}

pub async fn quick_schedule_tool(
    schedule_service: Arc<ScheduleService>,
    args: JsonValue,
) -> AppResult<JsonValue> {
    debug!("quick_schedule_tool invoked");

    let params: QuickScheduleParams = serde_json::from_value(args)
        .map_err(|e| AppError::validation(format!("Failed to parse parameters: {}", e)))?;

    let duration = params.duration_minutes.unwrap_or(60);
    let start_datetime = parse_quick_schedule_time(&params.when)?;

    let scheduled_item = schedule_service
        .create_time_block(
            &params.title,
            &start_datetime,
            duration,
            params.description.as_deref(),
        )
        .await?;

    let result = json!({
        "success": true,
        "id": scheduled_item.id,
        "title": scheduled_item.title,
        "start_at": scheduled_item.start_at,
        "end_at": scheduled_item.end_at,
        "formatted_display": scheduled_item.format_display(),
        "message": format!("⚡ 快速安排 '{}' 成功!", scheduled_item.title)
    });

    debug!(task_id = %scheduled_item.id, "quick schedule created successfully");
    Ok(result)
}

/// Parse quick schedule time strings like "today_9am", "tomorrow_2pm", etc.
fn parse_quick_schedule_time(when: &str) -> AppResult<String> {
    let now = chrono::Utc::now();
    let (date_part, time_part) = when.split_once('_').ok_or_else(|| {
        AppError::validation("Invalid format. Use 'today_9am', 'tomorrow_2pm', or RFC3339 datetime")
    })?;

    match date_part {
        "today" => {
            let target_date = now.date_naive();
            parse_time_part(target_date, time_part)
        }
        "tomorrow" => {
            let target_date = now.date_naive() + chrono::Duration::days(1);
            parse_time_part(target_date, time_part)
        }
        "next_week_monday" => {
            let weekday = now.weekday();
            let days_since_monday = match weekday {
                chrono::Weekday::Mon => 0,
                chrono::Weekday::Tue => 1,
                chrono::Weekday::Wed => 2,
                chrono::Weekday::Thu => 3,
                chrono::Weekday::Fri => 4,
                chrono::Weekday::Sat => 5,
                chrono::Weekday::Sun => 6,
            };
            let days_until_monday = (7 - days_since_monday as i64) % 7;
            let target_date = if days_until_monday == 0 {
                now.date_naive() + chrono::Duration::days(7)
            } else {
                now.date_naive() + chrono::Duration::days(days_until_monday)
            };
            parse_time_part(target_date, time_part)
        }
        _ => {
            // Try to parse as RFC3339 datetime directly
            chrono::DateTime::parse_from_rfc3339(when)
                .map(|dt| dt.to_rfc3339())
                .map_err(|_| AppError::validation("Invalid datetime format. Use 'today_9am', 'tomorrow_2pm', or RFC3339 datetime"))
        }
    }
}

fn parse_time_part(date_naive: chrono::NaiveDate, time_part: &str) -> AppResult<String> {
    let time_map = [
        ("9am", 9, 0),
        ("10am", 10, 0),
        ("11am", 11, 0),
        ("12pm", 12, 0),
        ("1pm", 13, 0),
        ("2pm", 14, 0),
        ("3pm", 15, 0),
        ("4pm", 16, 0),
        ("5pm", 17, 0),
        ("6pm", 18, 0),
    ];

    if let Some((hour, minute)) = time_map
        .iter()
        .find(|(_t, _, _)| *_t == time_part)
        .map(|(_t, h, m)| (*h, *m))
    {
        let naive_dt = date_naive.and_hms_opt(hour, minute, 0).unwrap();
        let timezone = chrono::FixedOffset::east_opt(8 * 3600).unwrap(); // UTC+8
        let dt = timezone
            .from_local_datetime(&naive_dt)
            .single()
            .ok_or_else(|| AppError::validation("Invalid datetime"))?;
        Ok(dt.to_rfc3339())
    } else {
        Err(AppError::validation(format!(
            "Invalid time '{}'. Use: 9am, 10am, 11am, 12pm, 1pm, 2pm, 3pm, 4pm, 5pm, 6pm",
            time_part
        )))
    }
}

// Parameter structs
#[derive(Debug, Deserialize)]
struct ListTimeItemsParams {
    #[serde(default = "default_today")]
    date_range: String,
    start_date: Option<String>,
    end_date: Option<String>,
    item_type: Option<String>,
    status_filter: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct CreateTimeBlockParams {
    title: String,
    start_datetime: String,
    duration_minutes: i64,
    description: Option<String>,
    #[allow(dead_code)]
    tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct UpdateTimeItemParams {
    id: String,
    #[allow(dead_code)]
    title: Option<String>,
    start_datetime: Option<String>,
    end_datetime: Option<String>,
    duration_minutes: Option<i64>,
    #[allow(dead_code)]
    description: Option<String>,
    #[allow(dead_code)]
    status: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SearchTimeItemsParams {
    query: String,
    #[serde(default = "default_all")]
    date_range: String,
    #[serde(default = "default_all")]
    item_type: String,
}

#[derive(Debug, Deserialize)]
struct QuickScheduleParams {
    title: String,
    when: String,
    duration_minutes: Option<i64>,
    description: Option<String>,
}

fn default_today() -> String {
    "today".to_string()
}

fn default_all() -> String {
    "all".to_string()
}

/// Register all time management tools
pub fn register_time_management_tools(
    registry: &mut crate::services::tool_registry::ToolRegistry,
    task_service: Arc<TaskService>,
) -> AppResult<()> {
    use crate::services::tool_registry::ToolHandler;
    use std::future::Future;
    use std::pin::Pin;

    let schedule_service = Arc::new(ScheduleService::new((*task_service).clone()));

    // Register list_time_items tool
    {
        let service = Arc::clone(&schedule_service);
        let handler: ToolHandler = Arc::new(move |args: JsonValue| {
            let service = Arc::clone(&service);
            Box::pin(async move { list_time_items_tool(service, args).await })
                as Pin<Box<dyn Future<Output = AppResult<JsonValue>> + Send>>
        });

        registry.register_tool(
            "list_time_items".to_string(),
            "List time-based items (scheduled tasks and events) for a date range. Use this when user asks to 'view schedule', 'show calendar', 'what's planned', 'time management', or needs to see scheduled items. Supports: today, week, month, or custom date ranges.".to_string(),
            list_time_items_schema(),
            handler,
        )?;
    }

    // Register create_time_block tool
    {
        let service = Arc::clone(&schedule_service);
        let handler: ToolHandler = Arc::new(move |args: JsonValue| {
            let service = Arc::clone(&service);
            Box::pin(async move { create_time_block_tool(service, args).await })
                as Pin<Box<dyn Future<Output = AppResult<JsonValue>> + Send>>
        });

        registry.register_tool(
            "create_time_block".to_string(),
            "Create a scheduled time block or calendar event. Use when user wants to schedule something at a specific time like 'schedule meeting at 2pm', 'book time for X', 'set up appointment', or create any time-based commitment. Requires title, start time (RFC3339), and duration.".to_string(),
            create_time_block_schema(),
            handler,
        )?;
    }

    // Register update_time_item tool
    {
        let service = Arc::clone(&schedule_service);
        let handler: ToolHandler = Arc::new(move |args: JsonValue| {
            let service = Arc::clone(&service);
            Box::pin(async move { update_time_item_tool(service, args).await })
                as Pin<Box<dyn Future<Output = AppResult<JsonValue>> + Send>>
        });

        registry.register_tool(
            "update_time_item".to_string(),
            "Update an existing scheduled time item (task or event). Use when user wants to reschedule, change duration, or modify details of an existing time-blocked item. Requires item ID.".to_string(),
            update_time_item_schema(),
            handler,
        )?;
    }

    // Register search_time_items tool
    {
        let service = Arc::clone(&schedule_service);
        let handler: ToolHandler = Arc::new(move |args: JsonValue| {
            let service = Arc::clone(&service);
            Box::pin(async move { search_time_items_tool(service, args).await })
                as Pin<Box<dyn Future<Output = AppResult<JsonValue>> + Send>>
        });

        registry.register_tool(
            "search_time_items".to_string(),
            "Search for time-based items by keyword. Use when user asks to 'find scheduled X', 'search for Y in calendar', 'look for Z in my schedule', or needs to locate specific time-blocked items. Searches titles, descriptions, and tags.".to_string(),
            search_time_items_schema(),
            handler,
        )?;
    }

    // Register quick_schedule tool
    {
        let service = Arc::clone(&schedule_service);
        let handler: ToolHandler = Arc::new(move |args: JsonValue| {
            let service = Arc::clone(&service);
            Box::pin(async move { quick_schedule_tool(service, args).await })
                as Pin<Box<dyn Future<Output = AppResult<JsonValue>> + Send>>
        });

        registry.register_tool(
            "quick_schedule".to_string(),
            "Quickly schedule an item with natural time expressions. Use when user says 'schedule X for today at 9am', 'book Y tomorrow at 2pm', 'set up Z next week monday', or wants fast scheduling. Supports: today_9am, tomorrow_2pm, next_week_monday, or full datetime.".to_string(),
            quick_schedule_schema(),
            handler,
        )?;
    }

    debug!("Registered 5 unified time management tools");
    Ok(())
}
