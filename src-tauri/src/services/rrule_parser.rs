#[cfg(test)]
#[allow(unused_imports)]
use chrono::Datelike;
use chrono::{DateTime, NaiveDate, Utc, Weekday};
use serde::de::{self, MapAccess, Visitor};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

use crate::error::{AppError, AppResult};

/// Frequency values for RRULE as defined in RFC 5545
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Frequency {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

impl FromStr for Frequency {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "DAILY" => Ok(Frequency::Daily),
            "WEEKLY" => Ok(Frequency::Weekly),
            "MONTHLY" => Ok(Frequency::Monthly),
            "YEARLY" => Ok(Frequency::Yearly),
            _ => Err(AppError::validation(&format!("Invalid frequency: {}", s))),
        }
    }
}

impl std::fmt::Display for Frequency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Frequency::Daily => write!(f, "DAILY"),
            Frequency::Weekly => write!(f, "WEEKLY"),
            Frequency::Monthly => write!(f, "MONTHLY"),
            Frequency::Yearly => write!(f, "YEARLY"),
        }
    }
}

/// Represents a BYDAY entry with optional ordinal position (e.g. 1MO, -1FR)
#[derive(Debug, Clone, PartialEq)]
pub struct ByDayEntry {
    pub weekday: Weekday,
    pub position: Option<i8>,
}

impl ByDayEntry {
    pub fn new(weekday: Weekday, position: Option<i8>) -> Self {
        Self { weekday, position }
    }
}

impl Serialize for ByDayEntry {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let len = if self.position.is_some() { 2 } else { 1 };
        let mut state = serializer.serialize_struct("ByDayEntry", len)?;
        state.serialize_field("weekday", weekday_to_string(self.weekday))?;
        if let Some(position) = self.position {
            state.serialize_field("position", &position)?;
        }
        state.end()
    }
}

struct ByDayEntryVisitor;

impl<'de> Visitor<'de> for ByDayEntryVisitor {
    type Value = ByDayEntry;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an object with weekday and optional position")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut weekday: Option<String> = None;
        let mut position: Option<i8> = None;

        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "weekday" => {
                    if weekday.is_some() {
                        return Err(de::Error::duplicate_field("weekday"));
                    }
                    weekday = Some(map.next_value()?);
                }
                "position" => {
                    if position.is_some() {
                        return Err(de::Error::duplicate_field("position"));
                    }
                    position = Some(map.next_value()?);
                }
                _ => {
                    return Err(de::Error::unknown_field(&key, &["weekday", "position"]));
                }
            }
        }

        let weekday = weekday.ok_or_else(|| de::Error::missing_field("weekday"))?;
        let weekday = string_to_weekday(&weekday)
            .map_err(|err| de::Error::custom(format!("invalid weekday: {err}")))?;

        Ok(ByDayEntry { weekday, position })
    }
}

impl<'de> Deserialize<'de> for ByDayEntry {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        const FIELDS: &[&str] = &["weekday", "position"];
        deserializer.deserialize_struct("ByDayEntry", FIELDS, ByDayEntryVisitor)
    }
}

fn weekday_to_string(weekday: Weekday) -> &'static str {
    match weekday {
        Weekday::Sun => "SU",
        Weekday::Mon => "MO",
        Weekday::Tue => "TU",
        Weekday::Wed => "WE",
        Weekday::Thu => "TH",
        Weekday::Fri => "FR",
        Weekday::Sat => "SA",
    }
}

fn string_to_weekday(code: &str) -> AppResult<Weekday> {
    match code {
        "SU" => Ok(Weekday::Sun),
        "MO" => Ok(Weekday::Mon),
        "TU" => Ok(Weekday::Tue),
        "WE" => Ok(Weekday::Wed),
        "TH" => Ok(Weekday::Thu),
        "FR" => Ok(Weekday::Fri),
        "SA" => Ok(Weekday::Sat),
        _ => Err(AppError::validation(&format!("Invalid weekday: {code}"))),
    }
}

fn format_by_day_entry(entry: &ByDayEntry) -> String {
    match entry.position {
        Some(position) => format!("{}{}", position, weekday_to_string(entry.weekday)),
        None => weekday_to_string(entry.weekday).to_string(),
    }
}

/// Parsed recurrence rule following RFC 5545 RRULE standard
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecurrenceRule {
    pub freq: Frequency,
    pub interval: Option<u32>,
    pub count: Option<u32>,
    pub until: Option<DateTime<Utc>>,
    pub by_day: Option<Vec<ByDayEntry>>,
    pub by_month_day: Option<Vec<i8>>,
    pub by_month: Option<Vec<u8>>,
}

impl RecurrenceRule {
    /// Create a new recurrence rule with the given frequency
    pub fn new(freq: Frequency) -> Self {
        Self {
            freq,
            interval: None,
            count: None,
            until: None,
            by_day: None,
            by_month_day: None,
            by_month: None,
        }
    }

    /// Set the interval for the recurrence rule
    pub fn with_interval(mut self, interval: u32) -> AppResult<Self> {
        if interval == 0 {
            return Err(AppError::validation("Interval must be greater than 0"));
        }
        if interval > 999 {
            return Err(AppError::validation("Interval must be less than 1000"));
        }
        self.interval = Some(interval);
        Ok(self)
    }

    /// Set the count for the recurrence rule
    pub fn with_count(mut self, count: u32) -> AppResult<Self> {
        if count == 0 {
            return Err(AppError::validation("Count must be greater than 0"));
        }
        if count > 9999 {
            return Err(AppError::validation("Count must be less than 10000"));
        }
        self.count = Some(count);
        Ok(self)
    }

    /// Set the until date for the recurrence rule
    pub fn with_until(mut self, until: DateTime<Utc>) -> Self {
        self.until = Some(until);
        self
    }

    /// Set the weekdays for the recurrence rule
    pub fn with_by_day(mut self, weekdays: Vec<ByDayEntry>) -> AppResult<Self> {
        if weekdays.is_empty() {
            return Err(AppError::validation("BYDAY cannot be empty"));
        }
        if weekdays.len() > 31 {
            return Err(AppError::validation(
                "BYDAY cannot have more than 31 entries",
            ));
        }
        for entry in &weekdays {
            if let Some(position) = entry.position {
                if position == 0 || position < -53 || position > 53 {
                    return Err(AppError::validation(
                        "BYDAY position must be between -53 and 53, excluding 0",
                    ));
                }
            }
        }
        self.by_day = Some(weekdays);
        Ok(self)
    }

    /// Set the month days for the recurrence rule
    pub fn with_by_month_day(mut self, month_days: Vec<i8>) -> AppResult<Self> {
        if month_days.is_empty() {
            return Err(AppError::validation("BYMONTHDAY cannot be empty"));
        }
        for &day in &month_days {
            if day == 0 || day < -31 || day > 31 {
                return Err(AppError::validation(
                    "BYMONTHDAY values must be between -31 and 31, excluding 0",
                ));
            }
        }
        self.by_month_day = Some(month_days);
        Ok(self)
    }

    /// Set the months for the recurrence rule
    pub fn with_by_month(mut self, months: Vec<u8>) -> AppResult<Self> {
        if months.is_empty() {
            return Err(AppError::validation("BYMONTH cannot be empty"));
        }
        for &month in &months {
            if month == 0 || month > 12 {
                return Err(AppError::validation(
                    "BYMONTH values must be between 1 and 12",
                ));
            }
        }
        self.by_month = Some(months);
        Ok(self)
    }

    /// Validate the recurrence rule for logical consistency
    pub fn validate(&self) -> AppResult<()> {
        // Cannot have both COUNT and UNTIL
        if self.count.is_some() && self.until.is_some() {
            return Err(AppError::validation("Cannot specify both COUNT and UNTIL"));
        }

        // Validate frequency-specific constraints
        match self.freq {
            Frequency::Daily => {
                if self.by_day.is_some() || self.by_month_day.is_some() || self.by_month.is_some() {
                    return Err(AppError::validation(
                        "DAILY frequency cannot use BYDAY, BYMONTHDAY, or BYMONTH",
                    ));
                }
            }
            Frequency::Weekly => {
                if self.by_month_day.is_some() || self.by_month.is_some() {
                    return Err(AppError::validation(
                        "WEEKLY frequency cannot use BYMONTHDAY or BYMONTH",
                    ));
                }
                if let Some(by_day) = &self.by_day {
                    if by_day.iter().any(|entry| entry.position.is_some()) {
                        return Err(AppError::validation(
                            "WEEKLY frequency cannot use positional BYDAY values",
                        ));
                    }
                }
            }
            Frequency::Monthly => {
                if self.by_day.is_some() && self.by_month_day.is_some() {
                    return Err(AppError::validation(
                        "MONTHLY frequency cannot use both BYDAY and BYMONTHDAY",
                    ));
                }
                if self.by_month.is_some() {
                    return Err(AppError::validation("MONTHLY frequency cannot use BYMONTH"));
                }
            }
            Frequency::Yearly => {
                // YEARLY can use any combination of BY* rules
            }
        }

        Ok(())
    }
}

/// RRULE parser for parsing iCalendar recurrence rules
pub struct RRuleParser;

impl RRuleParser {
    /// Parse an RRULE string into a RecurrenceRule struct
    pub fn parse(rrule_string: &str) -> AppResult<RecurrenceRule> {
        let rrule_string = rrule_string.trim();

        // Remove RRULE: prefix if present
        let rrule_string = if rrule_string.starts_with("RRULE:") {
            &rrule_string[6..]
        } else {
            rrule_string
        };

        if rrule_string.is_empty() {
            return Err(AppError::validation("RRULE string cannot be empty"));
        }

        let mut params = HashMap::new();

        // Parse key=value pairs separated by semicolons
        for part in rrule_string.split(';') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            let mut split = part.splitn(2, '=');
            let key = split
                .next()
                .ok_or_else(|| AppError::validation("Invalid RRULE format"))?;
            let value = split
                .next()
                .ok_or_else(|| AppError::validation("Invalid RRULE format"))?;

            params.insert(key.to_uppercase(), value.to_string());
        }

        // FREQ is required
        let freq_str = params
            .get("FREQ")
            .ok_or_else(|| AppError::validation("FREQ parameter is required"))?;
        let freq = Frequency::from_str(freq_str)?;

        let mut rule = RecurrenceRule::new(freq);

        // Parse optional parameters
        if let Some(interval_str) = params.get("INTERVAL") {
            let interval = interval_str
                .parse::<u32>()
                .map_err(|_| AppError::validation("Invalid INTERVAL value"))?;
            rule = rule.with_interval(interval)?;
        }

        if let Some(count_str) = params.get("COUNT") {
            let count = count_str
                .parse::<u32>()
                .map_err(|_| AppError::validation("Invalid COUNT value"))?;
            rule = rule.with_count(count)?;
        }

        if let Some(until_str) = params.get("UNTIL") {
            let until = Self::parse_until_date(until_str)?;
            rule = rule.with_until(until);
        }

        if let Some(by_day_str) = params.get("BYDAY") {
            let by_day_entries = Self::parse_by_day(by_day_str)?;
            rule = rule.with_by_day(by_day_entries)?;
        }

        if let Some(by_month_day_str) = params.get("BYMONTHDAY") {
            let month_days = Self::parse_by_month_day(by_month_day_str)?;
            rule = rule.with_by_month_day(month_days)?;
        }

        if let Some(by_month_str) = params.get("BYMONTH") {
            let months = Self::parse_by_month(by_month_str)?;
            rule = rule.with_by_month(months)?;
        }

        // Validate the complete rule
        rule.validate()?;

        Ok(rule)
    }

    /// Convert a RecurrenceRule back to an RRULE string
    pub fn to_string(rule: &RecurrenceRule) -> String {
        let mut parts = vec![format!("FREQ={}", rule.freq)];

        if let Some(interval) = rule.interval {
            parts.push(format!("INTERVAL={}", interval));
        }

        if let Some(count) = rule.count {
            parts.push(format!("COUNT={}", count));
        }

        if let Some(until) = rule.until {
            parts.push(format!("UNTIL={}", until.format("%Y%m%dT%H%M%SZ")));
        }

        if let Some(ref by_day) = rule.by_day {
            let day_strings: Vec<String> = by_day
                .iter()
                .map(|entry| format_by_day_entry(entry))
                .collect();
            parts.push(format!("BYDAY={}", day_strings.join(",")));
        }

        if let Some(ref by_month_day) = rule.by_month_day {
            let day_strings: Vec<String> = by_month_day.iter().map(|day| day.to_string()).collect();
            parts.push(format!("BYMONTHDAY={}", day_strings.join(",")));
        }

        if let Some(ref by_month) = rule.by_month {
            let month_strings: Vec<String> =
                by_month.iter().map(|month| month.to_string()).collect();
            parts.push(format!("BYMONTH={}", month_strings.join(",")));
        }

        parts.join(";")
    }

    fn parse_until_date(until_str: &str) -> AppResult<DateTime<Utc>> {
        // Support both date and datetime formats
        if until_str.len() == 8 {
            // YYYYMMDD format
            let year = until_str[0..4]
                .parse::<i32>()
                .map_err(|_| AppError::validation("Invalid UNTIL date format"))?;
            let month = until_str[4..6]
                .parse::<u32>()
                .map_err(|_| AppError::validation("Invalid UNTIL date format"))?;
            let day = until_str[6..8]
                .parse::<u32>()
                .map_err(|_| AppError::validation("Invalid UNTIL date format"))?;

            let naive_date = NaiveDate::from_ymd_opt(year, month, day)
                .ok_or_else(|| AppError::validation("Invalid UNTIL date"))?;
            let naive_datetime = naive_date
                .and_hms_opt(23, 59, 59)
                .ok_or_else(|| AppError::validation("Invalid UNTIL date"))?;

            Ok(DateTime::from_naive_utc_and_offset(naive_datetime, Utc))
        } else if until_str.len() == 16 && until_str.ends_with('Z') {
            // YYYYMMDDTHHMMSSZ format
            let date_part = &until_str[0..8];
            let time_part = &until_str[9..15];

            let year = date_part[0..4]
                .parse::<i32>()
                .map_err(|_| AppError::validation("Invalid UNTIL datetime format"))?;
            let month = date_part[4..6]
                .parse::<u32>()
                .map_err(|_| AppError::validation("Invalid UNTIL datetime format"))?;
            let day = date_part[6..8]
                .parse::<u32>()
                .map_err(|_| AppError::validation("Invalid UNTIL datetime format"))?;

            let hour = time_part[0..2]
                .parse::<u32>()
                .map_err(|_| AppError::validation("Invalid UNTIL datetime format"))?;
            let minute = time_part[2..4]
                .parse::<u32>()
                .map_err(|_| AppError::validation("Invalid UNTIL datetime format"))?;
            let second = time_part[4..6]
                .parse::<u32>()
                .map_err(|_| AppError::validation("Invalid UNTIL datetime format"))?;

            let naive_date = NaiveDate::from_ymd_opt(year, month, day)
                .ok_or_else(|| AppError::validation("Invalid UNTIL date"))?;
            let naive_datetime = naive_date
                .and_hms_opt(hour, minute, second)
                .ok_or_else(|| AppError::validation("Invalid UNTIL time"))?;

            Ok(DateTime::from_naive_utc_and_offset(naive_datetime, Utc))
        } else {
            Err(AppError::validation(
                "UNTIL must be in YYYYMMDD or YYYYMMDDTHHMMSSZ format",
            ))
        }
    }

    fn parse_by_day(by_day_str: &str) -> AppResult<Vec<ByDayEntry>> {
        let mut entries = Vec::new();

        for raw in by_day_str.split(',') {
            let token = raw.trim();
            if token.is_empty() {
                continue;
            }

            let normalized = token.to_uppercase();
            if normalized.len() < 2 {
                return Err(AppError::validation(&format!(
                    "Invalid BYDAY entry: {token}"
                )));
            }

            let (number_part, weekday_part) = normalized.split_at(normalized.len() - 2);
            let weekday = string_to_weekday(weekday_part)?;

            let position = if number_part.is_empty() {
                None
            } else {
                let value = number_part.parse::<i8>().map_err(|_| {
                    AppError::validation(&format!("Invalid BYDAY position: {token}"))
                })?;
                if value == 0 || value < -53 || value > 53 {
                    return Err(AppError::validation(&format!(
                        "BYDAY position must be between -53 and 53, excluding 0: {value}"
                    )));
                }
                Some(value)
            };

            entries.push(ByDayEntry::new(weekday, position));
        }

        if entries.is_empty() {
            return Err(AppError::validation("BYDAY cannot be empty"));
        }

        Ok(entries)
    }

    fn parse_by_month_day(by_month_day_str: &str) -> AppResult<Vec<i8>> {
        let mut month_days = Vec::new();

        for day_str in by_month_day_str.split(',') {
            let day_str = day_str.trim();
            let day = day_str
                .parse::<i8>()
                .map_err(|_| AppError::validation(&format!("Invalid month day: {}", day_str)))?;

            if day == 0 || day < -31 || day > 31 {
                return Err(AppError::validation(&format!(
                    "Month day must be between -31 and 31, excluding 0: {}",
                    day
                )));
            }

            month_days.push(day);
        }

        Ok(month_days)
    }

    fn parse_by_month(by_month_str: &str) -> AppResult<Vec<u8>> {
        let mut months = Vec::new();

        for month_str in by_month_str.split(',') {
            let month_str = month_str.trim();
            let month = month_str
                .parse::<u8>()
                .map_err(|_| AppError::validation(&format!("Invalid month: {}", month_str)))?;

            if month == 0 || month > 12 {
                return Err(AppError::validation(&format!(
                    "Month must be between 1 and 12: {}",
                    month
                )));
            }

            months.push(month);
        }

        Ok(months)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;

    #[test]
    fn test_parse_daily_rrule() {
        let rule = RRuleParser::parse("FREQ=DAILY").unwrap();
        assert_eq!(rule.freq, Frequency::Daily);
        assert_eq!(rule.interval, None);
    }

    #[test]
    fn test_parse_daily_with_interval() {
        let rule = RRuleParser::parse("FREQ=DAILY;INTERVAL=2").unwrap();
        assert_eq!(rule.freq, Frequency::Daily);
        assert_eq!(rule.interval, Some(2));
    }

    #[test]
    fn test_parse_weekly_with_byday() {
        let rule = RRuleParser::parse("FREQ=WEEKLY;BYDAY=MO,WE,FR").unwrap();
        assert_eq!(rule.freq, Frequency::Weekly);
        assert_eq!(
            rule.by_day,
            Some(vec![
                ByDayEntry::new(Weekday::Mon, None),
                ByDayEntry::new(Weekday::Wed, None),
                ByDayEntry::new(Weekday::Fri, None),
            ])
        );
    }

    #[test]
    fn test_parse_monthly_with_bymonthday() {
        let rule = RRuleParser::parse("FREQ=MONTHLY;BYMONTHDAY=15").unwrap();
        assert_eq!(rule.freq, Frequency::Monthly);
        assert_eq!(rule.by_month_day, Some(vec![15]));
    }

    #[test]
    fn test_parse_with_count() {
        let rule = RRuleParser::parse("FREQ=DAILY;COUNT=10").unwrap();
        assert_eq!(rule.freq, Frequency::Daily);
        assert_eq!(rule.count, Some(10));
    }

    #[test]
    fn test_parse_with_until_date() {
        let rule = RRuleParser::parse("FREQ=DAILY;UNTIL=20251231").unwrap();
        assert_eq!(rule.freq, Frequency::Daily);
        assert!(rule.until.is_some());

        let until = rule.until.unwrap();
        assert_eq!(until.year(), 2025);
        assert_eq!(until.month(), 12);
        assert_eq!(until.day(), 31);
    }

    #[test]
    fn test_parse_with_until_datetime() {
        let rule = RRuleParser::parse("FREQ=DAILY;UNTIL=20251231T235959Z").unwrap();
        assert_eq!(rule.freq, Frequency::Daily);
        assert!(rule.until.is_some());

        let until = rule.until.unwrap();
        assert_eq!(until.year(), 2025);
        assert_eq!(until.month(), 12);
        assert_eq!(until.day(), 31);
        assert_eq!(until.hour(), 23);
        assert_eq!(until.minute(), 59);
        assert_eq!(until.second(), 59);
    }

    #[test]
    fn test_parse_yearly_complex() {
        let rule = RRuleParser::parse("FREQ=YEARLY;BYMONTH=1,7;BYMONTHDAY=1,15").unwrap();
        assert_eq!(rule.freq, Frequency::Yearly);
        assert_eq!(rule.by_month, Some(vec![1, 7]));
        assert_eq!(rule.by_month_day, Some(vec![1, 15]));
    }

    #[test]
    fn test_to_string_daily() {
        let rule = RecurrenceRule::new(Frequency::Daily)
            .with_interval(2)
            .unwrap();
        let rrule_string = RRuleParser::to_string(&rule);
        assert_eq!(rrule_string, "FREQ=DAILY;INTERVAL=2");
    }

    #[test]
    fn test_to_string_weekly_with_byday() {
        let rule = RecurrenceRule::new(Frequency::Weekly)
            .with_by_day(vec![
                ByDayEntry::new(Weekday::Mon, None),
                ByDayEntry::new(Weekday::Wed, None),
                ByDayEntry::new(Weekday::Fri, None),
            ])
            .unwrap();
        let rrule_string = RRuleParser::to_string(&rule);
        assert_eq!(rrule_string, "FREQ=WEEKLY;BYDAY=MO,WE,FR");
    }

    #[test]
    fn test_validation_count_and_until() {
        let result = RRuleParser::parse("FREQ=DAILY;COUNT=10;UNTIL=20251231");
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_daily_with_byday() {
        let result = RRuleParser::parse("FREQ=DAILY;BYDAY=MO");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_monthly_with_positional_byday() {
        let rule = RRuleParser::parse("FREQ=MONTHLY;BYDAY=1MO,-1FR").unwrap();
        assert_eq!(rule.freq, Frequency::Monthly);
        assert_eq!(
            rule.by_day,
            Some(vec![
                ByDayEntry::new(Weekday::Mon, Some(1)),
                ByDayEntry::new(Weekday::Fri, Some(-1)),
            ])
        );
    }

    #[test]
    fn test_validation_weekly_with_positional_byday() {
        let result = RRuleParser::parse("FREQ=WEEKLY;BYDAY=1MO");
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_weekly_with_bymonthday() {
        let result = RRuleParser::parse("FREQ=WEEKLY;BYMONTHDAY=15");
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_invalid_weekday() {
        let result = RRuleParser::parse("FREQ=WEEKLY;BYDAY=XX");
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_invalid_month_day() {
        let result = RRuleParser::parse("FREQ=MONTHLY;BYMONTHDAY=0");
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_invalid_month() {
        let result = RRuleParser::parse("FREQ=YEARLY;BYMONTH=13");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_with_rrule_prefix() {
        let rule = RRuleParser::parse("RRULE:FREQ=DAILY;INTERVAL=2").unwrap();
        assert_eq!(rule.freq, Frequency::Daily);
        assert_eq!(rule.interval, Some(2));
    }

    #[test]
    fn test_parse_empty_string() {
        let result = RRuleParser::parse("");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_missing_freq() {
        let result = RRuleParser::parse("INTERVAL=2");
        assert!(result.is_err());
    }
}
