use chrono::{DateTime, Duration, FixedOffset, NaiveTime, Timelike};
use serde_json::json;

use crate::error::{AppError, AppResult};

pub fn parse_datetime(value: &str) -> AppResult<DateTime<FixedOffset>> {
    DateTime::parse_from_rfc3339(value).map_err(|err| {
        AppError::validation_with_details(
            "无效的时间格式",
            json!({"value": value, "error": err.to_string()}),
        )
    })
}

pub fn parse_optional_datetime(value: Option<&String>) -> AppResult<Option<DateTime<FixedOffset>>> {
    match value {
        Some(raw) => Ok(Some(parse_datetime(raw)?)),
        Option::None => Ok(Option::None),
    }
}

pub fn format_datetime(dt: DateTime<FixedOffset>) -> String {
    dt.to_rfc3339()
}

pub fn add_minutes(dt: DateTime<FixedOffset>, minutes: i64) -> AppResult<DateTime<FixedOffset>> {
    dt.checked_add_signed(Duration::minutes(minutes))
        .ok_or_else(|| AppError::validation("时间计算超出范围"))
}

pub fn duration_minutes(
    start: DateTime<FixedOffset>,
    end: DateTime<FixedOffset>,
) -> AppResult<i64> {
    let total = end.signed_duration_since(start).num_minutes();
    if total < 0 {
        Err(AppError::validation("结束时间必须晚于开始时间"))
    } else {
        Ok(total)
    }
}

pub fn overlaps(
    a_start: DateTime<FixedOffset>,
    a_end: DateTime<FixedOffset>,
    b_start: DateTime<FixedOffset>,
    b_end: DateTime<FixedOffset>,
) -> AppResult<bool> {
    if a_end <= a_start {
        return Err(AppError::validation("时间范围无效"));
    }
    if b_end <= b_start {
        return Err(AppError::validation("时间范围无效"));
    }
    Ok(a_start < b_end && b_start < a_end)
}

pub fn ensure_window(start: DateTime<FixedOffset>, end: DateTime<FixedOffset>) -> AppResult<()> {
    if end <= start {
        Err(AppError::validation("时间窗口结束时间必须晚于开始"))
    } else {
        Ok(())
    }
}

pub fn minutes_from_midnight(time: NaiveTime) -> i64 {
    (time.hour() as i64) * 60 + (time.minute() as i64)
}

pub fn clamp_time_to_window(
    current: DateTime<FixedOffset>,
    window_start: DateTime<FixedOffset>,
) -> DateTime<FixedOffset> {
    if current < window_start {
        window_start
    } else {
        current
    }
}

pub fn midnight_minutes_of(dt: DateTime<FixedOffset>) -> i64 {
    let time = dt.time();
    (time.hour() as i64) * 60 + (time.minute() as i64)
}

pub fn same_day(a: DateTime<FixedOffset>, b: DateTime<FixedOffset>) -> bool {
    a.date_naive() == b.date_naive()
}

pub fn to_naive_time(total_minutes: u32) -> NaiveTime {
    let hours = (total_minutes / 60) as u32;
    let minutes = total_minutes % 60;
    NaiveTime::from_hms_opt(hours, minutes, 0)
        .unwrap_or_else(|| NaiveTime::from_hms_opt(0, 0, 0).expect("00:00 must be valid"))
}
