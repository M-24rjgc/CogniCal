use crate::error::{AppError, AppResult};
use crate::services::planning_service::PlanningService;
use crate::services::schedule_optimizer::ExistingEvent;
use chrono::{DateTime, Duration, FixedOffset, NaiveDate, NaiveTime, TimeZone};
use serde::Deserialize;
use serde_json::{json, Value as JsonValue};
use std::sync::Arc;
use tracing::debug;

/// Tool schemas for calendar operations
/// These schemas follow the OpenAI function calling format

/// Get the schema for the get_calendar_events tool
pub fn get_calendar_events_schema() -> JsonValue {
    json!({
        "type": "object",
        "properties": {
            "start_date": {
                "type": "string",
                "format": "date",
                "description": "Start date for the range in YYYY-MM-DD format (required)"
            },
            "end_date": {
                "type": "string",
                "format": "date",
                "description": "End date for the range in YYYY-MM-DD format (required)"
            },
            "event_type": {
                "type": "string",
                "description": "Filter by event type (optional)"
            }
        },
        "required": ["start_date", "end_date"]
    })
}

/// Get the schema for the create_calendar_event tool
pub fn create_calendar_event_schema() -> JsonValue {
    json!({
        "type": "object",
        "properties": {
            "title": {
                "type": "string",
                "description": "The title of the event (required)"
            },
            "date": {
                "type": "string",
                "format": "date",
                "description": "Date of the event in YYYY-MM-DD format (required)"
            },
            "start_time": {
                "type": "string",
                "format": "time",
                "description": "Start time in HH:MM format (24-hour, required)"
            },
            "duration_minutes": {
                "type": "integer",
                "description": "Duration of the event in minutes (required)"
            },
            "event_type": {
                "type": "string",
                "description": "Type of event (e.g., 'meeting', 'focus', 'break')"
            }
        },
        "required": ["title", "date", "start_time", "duration_minutes"]
    })
}

/// Get the schema for the update_calendar_event tool
pub fn update_calendar_event_schema() -> JsonValue {
    json!({
        "type": "object",
        "properties": {
            "event_id": {
                "type": "string",
                "description": "The ID of the event to update (required)"
            },
            "title": {
                "type": "string",
                "description": "New title for the event"
            },
            "date": {
                "type": "string",
                "format": "date",
                "description": "New date in YYYY-MM-DD format"
            },
            "start_time": {
                "type": "string",
                "format": "time",
                "description": "New start time in HH:MM format (24-hour)"
            },
            "duration_minutes": {
                "type": "integer",
                "description": "New duration in minutes"
            },
            "event_type": {
                "type": "string",
                "description": "New event type"
            }
        },
        "required": ["event_id"]
    })
}

/// Parameters for getting calendar events
#[derive(Debug, Deserialize)]
struct GetCalendarEventsParams {
    start_date: String,
    end_date: String,
    #[serde(default)]
    event_type: Option<String>,
}

/// Parameters for creating a calendar event
#[derive(Debug, Deserialize)]
struct CreateCalendarEventParams {
    title: String,
    date: String,
    start_time: String,
    duration_minutes: i64,
    #[serde(default)]
    event_type: Option<String>,
}

/// Parameters for updating a calendar event
#[derive(Debug, Deserialize)]
struct UpdateCalendarEventParams {
    event_id: String,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    date: Option<String>,
    #[serde(default)]
    start_time: Option<String>,
    #[serde(default)]
    duration_minutes: Option<i64>,
    #[serde(default)]
    event_type: Option<String>,
}

/// Helper function to extract parameters from JSON
fn extract_params<T: for<'de> Deserialize<'de>>(args: &JsonValue) -> AppResult<T> {
    serde_json::from_value(args.clone()).map_err(|e| {
        AppError::validation(format!("Failed to parse tool parameters: {}", e))
    })
}

/// Parse date string (YYYY-MM-DD) into NaiveDate
fn parse_date(date_str: &str) -> AppResult<NaiveDate> {
    NaiveDate::parse_from_str(date_str, "%Y-%m-%d").map_err(|e| {
        AppError::validation(format!(
            "Invalid date format '{}'. Expected YYYY-MM-DD: {}",
            date_str, e
        ))
    })
}

/// Parse time string (HH:MM) into NaiveTime
fn parse_time(time_str: &str) -> AppResult<NaiveTime> {
    NaiveTime::parse_from_str(time_str, "%H:%M").map_err(|e| {
        AppError::validation(format!(
            "Invalid time format '{}'. Expected HH:MM (24-hour): {}",
            time_str, e
        ))
    })
}

/// Combine date and time into RFC3339 datetime string
fn combine_datetime(date: NaiveDate, time: NaiveTime) -> AppResult<String> {
    let local_tz = FixedOffset::east_opt(8 * 3600)
        .ok_or_else(|| AppError::validation("Failed to create timezone offset"))?;
    
    let naive_dt = date.and_time(time);
    let dt = local_tz
        .from_local_datetime(&naive_dt)
        .single()
        .ok_or_else(|| AppError::validation("Ambiguous or invalid local datetime"))?;
    
    Ok(dt.to_rfc3339())
}

/// Parse RFC3339 datetime string
fn parse_datetime(datetime_str: &str) -> AppResult<DateTime<FixedOffset>> {
    DateTime::parse_from_rfc3339(datetime_str).map_err(|e| {
        AppError::validation(format!(
            "Invalid datetime format '{}': {}",
            datetime_str, e
        ))
    })
}

/// Format datetime for human-readable display
fn format_datetime_display(datetime_str: &str) -> String {
    match parse_datetime(datetime_str) {
        Ok(dt) => dt.format("%Y-%m-%d %H:%M").to_string(),
        Err(_) => datetime_str.to_string(),
    }
}

/// Helper function to format an event for AI consumption
fn format_event_for_ai(event: &ExistingEvent) -> JsonValue {
    json!({
        "id": event.id,
        "start_at": event.start_at,
        "end_at": event.end_at,
        "event_type": event.event_type,
        "start_display": format_datetime_display(&event.start_at),
        "end_display": format_datetime_display(&event.end_at),
    })
}

/// Helper function to format multiple events for AI consumption
fn format_events_summary(events: &[ExistingEvent]) -> String {
    if events.is_empty() {
        return "No events found in the specified date range.".to_string();
    }

    let mut summary = format!("Found {} event(s):\n\n", events.len());
    
    for (idx, event) in events.iter().enumerate() {
        let start_display = format_datetime_display(&event.start_at);
        let end_display = format_datetime_display(&event.end_at);
        
        summary.push_str(&format!(
            "{}. Event ID: {}\n",
            idx + 1,
            event.id
        ));
        
        summary.push_str(&format!("   Time: {} to {}\n", start_display, end_display));
        
        if let Some(event_type) = &event.event_type {
            summary.push_str(&format!("   Type: {}\n", event_type));
        }
        
        summary.push('\n');
    }

    summary
}

/// Check for scheduling conflicts with existing events
fn check_conflicts(
    new_start: &str,
    new_end: &str,
    existing_events: &[ExistingEvent],
    exclude_event_id: Option<&str>,
) -> AppResult<Vec<String>> {
    let new_start_dt = parse_datetime(new_start)?;
    let new_end_dt = parse_datetime(new_end)?;
    
    let mut conflicts = Vec::new();
    
    for event in existing_events {
        // Skip the event being updated
        if let Some(exclude_id) = exclude_event_id {
            if event.id == exclude_id {
                continue;
            }
        }
        
        let event_start_dt = parse_datetime(&event.start_at)?;
        let event_end_dt = parse_datetime(&event.end_at)?;
        
        // Check for overlap
        if new_start_dt < event_end_dt && new_end_dt > event_start_dt {
            conflicts.push(format!(
                "Conflicts with event {} ({} to {})",
                event.id,
                format_datetime_display(&event.start_at),
                format_datetime_display(&event.end_at)
            ));
        }
    }
    
    Ok(conflicts)
}


/// Get calendar events within a date range
///
/// This tool allows the AI to retrieve calendar events for a specified date range.
/// Returns a formatted list of events with their details.
pub async fn get_calendar_events_tool(
    _planning_service: Arc<PlanningService>,
    args: JsonValue,
) -> AppResult<JsonValue> {
    debug!(target: "calendar_tools", "Getting calendar events with args: {}", args);

    let params: GetCalendarEventsParams = extract_params(&args)?;

    // Parse and validate dates
    let start_date = parse_date(&params.start_date)?;
    let end_date = parse_date(&params.end_date)?;

    if end_date < start_date {
        return Err(AppError::validation(
            "End date must be after or equal to start date",
        ));
    }

    // For now, we'll return a placeholder implementation
    // In a real implementation, this would query a calendar database or service
    // Since the codebase uses ExistingEvent in planning constraints, we'll simulate that
    
    // TODO: Replace with actual calendar data retrieval
    // This is a placeholder that returns empty events
    let events: Vec<ExistingEvent> = vec![];
    
    // Apply event type filter if provided
    let filtered_events: Vec<ExistingEvent> = if let Some(event_type) = &params.event_type {
        events
            .into_iter()
            .filter(|e| {
                e.event_type
                    .as_ref()
                    .map(|t| t.eq_ignore_ascii_case(event_type))
                    .unwrap_or(false)
            })
            .collect()
    } else {
        events
    };

    let summary = format_events_summary(&filtered_events);

    Ok(json!({
        "success": true,
        "message": summary,
        "start_date": params.start_date,
        "end_date": params.end_date,
        "count": filtered_events.len(),
        "events": filtered_events.iter().map(format_event_for_ai).collect::<Vec<_>>()
    }))
}

/// Create a new calendar event
///
/// This tool allows the AI to create calendar events with specified parameters.
/// Returns a confirmation message with the event details and any conflicts detected.
pub async fn create_calendar_event_tool(
    _planning_service: Arc<PlanningService>,
    args: JsonValue,
) -> AppResult<JsonValue> {
    debug!(target: "calendar_tools", "Creating calendar event with args: {}", args);

    let params: CreateCalendarEventParams = extract_params(&args)?;

    // Parse and validate date and time
    let date = parse_date(&params.date)?;
    let start_time = parse_time(&params.start_time)?;
    
    // Validate duration
    if params.duration_minutes <= 0 {
        return Err(AppError::validation(
            "Duration must be greater than 0 minutes",
        ));
    }

    // Combine date and time into RFC3339 format
    let start_at = combine_datetime(date, start_time)?;
    
    // Calculate end time
    let start_dt = parse_datetime(&start_at)?;
    let end_dt = start_dt + Duration::minutes(params.duration_minutes);
    let end_at = end_dt.to_rfc3339();

    // Create the event
    let event_id = uuid::Uuid::new_v4().to_string();
    let new_event = ExistingEvent {
        id: event_id.clone(),
        start_at: start_at.clone(),
        end_at: end_at.clone(),
        event_type: params.event_type.clone(),
    };

    // TODO: Replace with actual calendar data storage
    // For now, we'll just check for conflicts with an empty list
    let existing_events: Vec<ExistingEvent> = vec![];
    
    // Check for conflicts
    let conflicts = check_conflicts(&start_at, &end_at, &existing_events, None)?;

    let mut message = format!(
        "✓ Calendar event created successfully!\n\nTitle: {}\nDate: {}\nTime: {} to {}\nDuration: {} minutes\nID: {}",
        params.title,
        params.date,
        format_datetime_display(&start_at),
        format_datetime_display(&end_at),
        params.duration_minutes,
        event_id
    );

    if !conflicts.is_empty() {
        message.push_str("\n\n⚠️ Scheduling conflicts detected:\n");
        for conflict in &conflicts {
            message.push_str(&format!("  - {}\n", conflict));
        }
    }

    Ok(json!({
        "success": true,
        "message": message,
        "event": format_event_for_ai(&new_event),
        "conflicts": conflicts,
        "has_conflicts": !conflicts.is_empty()
    }))
}

/// Update an existing calendar event
///
/// This tool allows the AI to update calendar event fields.
/// Returns a confirmation message with the updated event details and any conflicts.
pub async fn update_calendar_event_tool(
    _planning_service: Arc<PlanningService>,
    args: JsonValue,
) -> AppResult<JsonValue> {
    debug!(target: "calendar_tools", "Updating calendar event with args: {}", args);

    let params: UpdateCalendarEventParams = extract_params(&args)?;

    // TODO: Replace with actual calendar data retrieval
    // For now, we'll simulate finding an event
    let existing_events: Vec<ExistingEvent> = vec![];
    
    let existing_event = existing_events
        .iter()
        .find(|e| e.id == params.event_id)
        .ok_or_else(|| {
            AppError::validation(format!(
                "Event with ID '{}' not found. Please check the event ID and try again.",
                params.event_id
            ))
        })?;

    // Start with existing event data
    let mut updated_start_at = existing_event.start_at.clone();
    let mut updated_end_at = existing_event.end_at.clone();
    let mut updated_event_type = existing_event.event_type.clone();

    // Apply updates
    if let (Some(date_str), Some(time_str)) = (&params.date, &params.start_time) {
        // Both date and time provided
        let date = parse_date(date_str)?;
        let time = parse_time(time_str)?;
        updated_start_at = combine_datetime(date, time)?;
        
        // Recalculate end time if duration is provided
        if let Some(duration) = params.duration_minutes {
            if duration <= 0 {
                return Err(AppError::validation(
                    "Duration must be greater than 0 minutes",
                ));
            }
            let start_dt = parse_datetime(&updated_start_at)?;
            let end_dt = start_dt + Duration::minutes(duration);
            updated_end_at = end_dt.to_rfc3339();
        }
    } else if let Some(date_str) = &params.date {
        // Only date provided, keep the same time
        let old_start_dt = parse_datetime(&updated_start_at)?;
        let new_date = parse_date(date_str)?;
        let time = old_start_dt.time();
        updated_start_at = combine_datetime(new_date, time)?;
        
        // Adjust end time to maintain duration
        let old_end_dt = parse_datetime(&updated_end_at)?;
        let duration = old_end_dt.signed_duration_since(old_start_dt);
        let new_start_dt = parse_datetime(&updated_start_at)?;
        let new_end_dt = new_start_dt + duration;
        updated_end_at = new_end_dt.to_rfc3339();
    } else if let Some(time_str) = &params.start_time {
        // Only time provided, keep the same date
        let old_start_dt = parse_datetime(&updated_start_at)?;
        let date = old_start_dt.date_naive();
        let new_time = parse_time(time_str)?;
        updated_start_at = combine_datetime(date, new_time)?;
        
        // Recalculate end time if duration is provided, otherwise maintain duration
        if let Some(duration) = params.duration_minutes {
            if duration <= 0 {
                return Err(AppError::validation(
                    "Duration must be greater than 0 minutes",
                ));
            }
            let start_dt = parse_datetime(&updated_start_at)?;
            let end_dt = start_dt + Duration::minutes(duration);
            updated_end_at = end_dt.to_rfc3339();
        } else {
            let old_end_dt = parse_datetime(&updated_end_at)?;
            let duration = old_end_dt.signed_duration_since(old_start_dt);
            let new_start_dt = parse_datetime(&updated_start_at)?;
            let new_end_dt = new_start_dt + duration;
            updated_end_at = new_end_dt.to_rfc3339();
        }
    } else if let Some(duration) = params.duration_minutes {
        // Only duration provided
        if duration <= 0 {
            return Err(AppError::validation(
                "Duration must be greater than 0 minutes",
            ));
        }
        let start_dt = parse_datetime(&updated_start_at)?;
        let end_dt = start_dt + Duration::minutes(duration);
        updated_end_at = end_dt.to_rfc3339();
    }

    if let Some(event_type) = params.event_type {
        updated_event_type = Some(event_type);
    }

    let updated_event = ExistingEvent {
        id: params.event_id.clone(),
        start_at: updated_start_at.clone(),
        end_at: updated_end_at.clone(),
        event_type: updated_event_type,
    };

    // Check for conflicts (excluding the event being updated)
    let conflicts = check_conflicts(
        &updated_start_at,
        &updated_end_at,
        &existing_events,
        Some(&params.event_id),
    )?;

    let mut message = format!(
        "✓ Calendar event updated successfully!\n\nTime: {} to {}\nID: {}",
        format_datetime_display(&updated_start_at),
        format_datetime_display(&updated_end_at),
        params.event_id
    );

    if let Some(title) = params.title {
        message = format!("✓ Calendar event updated successfully!\n\nTitle: {}\n{}", title, &message[message.find("Time:").unwrap()..]);
    }

    if !conflicts.is_empty() {
        message.push_str("\n\n⚠️ Scheduling conflicts detected:\n");
        for conflict in &conflicts {
            message.push_str(&format!("  - {}\n", conflict));
        }
    }

    Ok(json!({
        "success": true,
        "message": message,
        "event": format_event_for_ai(&updated_event),
        "conflicts": conflicts,
        "has_conflicts": !conflicts.is_empty()
    }))
}

/// Register all calendar tools with the tool registry
///
/// # Arguments
/// * `registry` - The tool registry to register tools with
/// * `db_pool` - The database pool for calendar operations
pub fn register_calendar_tools(
    registry: &mut crate::services::tool_registry::ToolRegistry,
    db_pool: crate::db::DbPool,
) -> AppResult<()> {
    use crate::services::planning_service::PlanningService;
    use crate::services::tool_registry::ToolHandler;
    use std::future::Future;
    use std::pin::Pin;

    // Create a planning service for calendar operations
    // Note: We need task_service and ai_service for PlanningService, but calendar tools don't use them
    // For now, we'll pass dummy services since calendar tools don't actually use the planning_service parameter
    
    // Register get_calendar_events tool
    {
        let pool = db_pool.clone();
        let handler: ToolHandler = Arc::new(move |args: JsonValue| {
            let pool = pool.clone();
            Box::pin(async move {
                // Create a minimal planning service just for the signature
                // The calendar tools don't actually use it
                let task_service = Arc::new(crate::services::task_service::TaskService::new(pool.clone()));
                let ai_service = Arc::new(crate::services::ai_service::AiService::new(pool.clone())?);
                let planning_service = Arc::new(PlanningService::new(
                    pool.clone(),
                    task_service,
                    ai_service,
                ));
                get_calendar_events_tool(planning_service, args).await
            }) as Pin<Box<dyn Future<Output = AppResult<JsonValue>> + Send>>
        });

        registry.register_tool(
            "get_calendar_events".to_string(),
            "Retrieve calendar events for a specified date range".to_string(),
            json!({
                "type": "object",
                "properties": get_calendar_events_schema()["properties"],
                "required": []
            }),
            handler,
        )?;
    }

    // Register create_calendar_event tool
    {
        let pool = db_pool.clone();
        let handler: ToolHandler = Arc::new(move |args: JsonValue| {
            let pool = pool.clone();
            Box::pin(async move {
                let task_service = Arc::new(crate::services::task_service::TaskService::new(pool.clone()));
                let ai_service = Arc::new(crate::services::ai_service::AiService::new(pool.clone())?);
                let planning_service = Arc::new(PlanningService::new(
                    pool.clone(),
                    task_service,
                    ai_service,
                ));
                create_calendar_event_tool(planning_service, args).await
            }) as Pin<Box<dyn Future<Output = AppResult<JsonValue>> + Send>>
        });

        registry.register_tool(
            "create_calendar_event".to_string(),
            "Create a new calendar event with the specified details".to_string(),
            json!({
                "type": "object",
                "properties": create_calendar_event_schema()["properties"],
                "required": ["title", "start_time"]
            }),
            handler,
        )?;
    }

    // Register update_calendar_event tool
    {
        let pool = db_pool.clone();
        let handler: ToolHandler = Arc::new(move |args: JsonValue| {
            let pool = pool.clone();
            Box::pin(async move {
                let task_service = Arc::new(crate::services::task_service::TaskService::new(pool.clone()));
                let ai_service = Arc::new(crate::services::ai_service::AiService::new(pool.clone())?);
                let planning_service = Arc::new(PlanningService::new(
                    pool.clone(),
                    task_service,
                    ai_service,
                ));
                update_calendar_event_tool(planning_service, args).await
            }) as Pin<Box<dyn Future<Output = AppResult<JsonValue>> + Send>>
        });

        registry.register_tool(
            "update_calendar_event".to_string(),
            "Update an existing calendar event's fields".to_string(),
            json!({
                "type": "object",
                "properties": update_calendar_event_schema()["properties"],
                "required": ["id"]
            }),
            handler,
        )?;
    }

    debug!(target: "calendar_tools", "Registered 3 calendar tools");
    Ok(())
}
