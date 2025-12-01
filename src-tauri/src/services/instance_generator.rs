use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc, Weekday};
use std::collections::HashMap;

use crate::error::{AppError, AppResult};
use crate::models::recurring_task::TaskInstance;
use crate::services::rrule_parser::{ByDayEntry, Frequency, RecurrenceRule};

/// Configuration for instance generation
#[derive(Debug, Clone)]
pub struct GenerationConfig {
    /// Number of days ahead to generate instances
    pub horizon_days: u32,
    /// Maximum number of instances to generate per template
    pub max_instances: u32,
    /// Start date for generation (defaults to today)
    pub start_date: Option<DateTime<Utc>>,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            horizon_days: 30,
            max_instances: 1000,
            start_date: None,
        }
    }
}

/// Engine for generating task instances from recurrence rules
pub struct InstanceGenerator;

impl InstanceGenerator {
    /// Generate task instances for a given recurrence rule and time horizon
    pub fn generate_instances(
        template_id: &str,
        title: &str,
        rule: &RecurrenceRule,
        config: &GenerationConfig,
    ) -> AppResult<Vec<TaskInstance>> {
        let start_date = config.start_date.unwrap_or_else(Utc::now);
        let end_date = start_date + Duration::days(config.horizon_days as i64);

        let mut instances = Vec::new();

        let mut count = 0;

        // Handle COUNT limit
        let max_count = if let Some(rule_count) = rule.count {
            std::cmp::min(rule_count, config.max_instances)
        } else {
            config.max_instances
        };

        // Handle UNTIL limit
        let effective_end_date = if let Some(until) = rule.until {
            std::cmp::min(end_date, until)
        } else {
            end_date
        };

        // Start from the day after the start date for the first occurrence
        let mut search_date = start_date;

        while count < max_count {
            if let Some(next_occurrence) = Self::calculate_next_occurrence(rule, search_date)? {
                if next_occurrence > effective_end_date {
                    break;
                }

                let instance = TaskInstance {
                    id: uuid::Uuid::new_v4().to_string(),
                    template_id: template_id.to_string(),
                    instance_date: next_occurrence,
                    title: title.to_string(),
                    description: None,
                    status: "todo".to_string(),
                    priority: "medium".to_string(),
                    due_at: None,
                    completed_at: None,
                    is_exception: false,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                };
                instances.push(instance);
                count += 1;

                // Move search date to just after this occurrence for the next search
                search_date = next_occurrence + Duration::seconds(1);
            } else {
                break;
            }
        }

        Ok(instances)
    }

    /// Calculate the next occurrence of a recurrence rule from a given date
    pub fn calculate_next_occurrence(
        rule: &RecurrenceRule,
        from_date: DateTime<Utc>,
    ) -> AppResult<Option<DateTime<Utc>>> {
        match rule.freq {
            Frequency::Daily => Self::calculate_daily_occurrence(rule, from_date),
            Frequency::Weekly => Self::calculate_weekly_occurrence(rule, from_date),
            Frequency::Monthly => Self::calculate_monthly_occurrence(rule, from_date),
            Frequency::Yearly => Self::calculate_yearly_occurrence(rule, from_date),
        }
    }

    fn calculate_daily_occurrence(
        rule: &RecurrenceRule,
        from_date: DateTime<Utc>,
    ) -> AppResult<Option<DateTime<Utc>>> {
        let interval = rule.interval.unwrap_or(1);
        // For daily, the next occurrence is simply interval days from the from_date
        let next_date = from_date
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc))
            .unwrap_or(from_date)
            + Duration::days(interval as i64);
        Ok(Some(next_date))
    }

    fn calculate_weekly_occurrence(
        rule: &RecurrenceRule,
        from_date: DateTime<Utc>,
    ) -> AppResult<Option<DateTime<Utc>>> {
        let interval = rule.interval.unwrap_or(1);

        if let Some(ref by_day) = rule.by_day {
            let target_weekdays: Vec<Weekday> = by_day.iter().map(|entry| entry.weekday).collect();
            // Find the next occurrence on one of the specified weekdays
            let mut current = from_date;
            let mut weeks_added = 0;

            loop {
                let current_weekday = current.weekday();

                // Check if current day matches any of the specified weekdays
                if target_weekdays.contains(&current_weekday) && current > from_date {
                    return Ok(Some(current));
                }

                // Move to next day
                current = current + Duration::days(1);

                // If we've completed a week, check if we need to skip weeks based on interval
                if current.weekday() == from_date.weekday() && current > from_date {
                    weeks_added += 1;
                    if weeks_added < interval {
                        // Skip to the next interval week
                        current = current + Duration::weeks((interval - 1) as i64);
                    }
                }

                // Prevent infinite loops - limit to reasonable search range
                if current > from_date + Duration::weeks(52) {
                    return Ok(None);
                }
            }
        } else {
            // No specific weekdays, just add weeks
            let next_date = from_date + Duration::weeks(interval as i64);
            Ok(Some(next_date))
        }
    }

    fn calculate_monthly_occurrence(
        rule: &RecurrenceRule,
        from_date: DateTime<Utc>,
    ) -> AppResult<Option<DateTime<Utc>>> {
        let interval = rule.interval.unwrap_or(1);

        if let Some(ref by_month_day) = rule.by_month_day {
            // Find next occurrence on specified month days
            Self::find_next_monthly_by_day(from_date, by_month_day, interval)
        } else if let Some(ref by_day) = rule.by_day {
            // Find next occurrence on specified weekdays (e.g., first Monday)
            Self::find_next_monthly_by_weekday(from_date, by_day, interval)
        } else {
            // No specific constraints, just add months
            let next_month = if from_date.month() + interval > 12 {
                let years_to_add = (from_date.month() + interval - 1) / 12;
                let new_month = (from_date.month() + interval - 1) % 12 + 1;
                from_date
                    .with_year(from_date.year() + years_to_add as i32)
                    .and_then(|d| d.with_month(new_month))
            } else {
                from_date.with_month(from_date.month() + interval)
            };

            Ok(next_month)
        }
    }

    fn calculate_yearly_occurrence(
        rule: &RecurrenceRule,
        from_date: DateTime<Utc>,
    ) -> AppResult<Option<DateTime<Utc>>> {
        let interval = rule.interval.unwrap_or(1);
        let target_year = from_date.year() + interval as i32;

        // Handle BYMONTH and BYMONTHDAY constraints
        let default_month = [from_date.month() as u8];
        let default_day = [from_date.day() as i8];
        let months = rule
            .by_month
            .as_ref()
            .map(|m| m.as_slice())
            .unwrap_or(&default_month);
        let month_days = rule
            .by_month_day
            .as_ref()
            .map(|d| d.as_slice())
            .unwrap_or(&default_day);

        for &month in months {
            for &day in month_days {
                if let Some(candidate_date) =
                    Self::create_date_with_month_day(target_year, month, day)
                {
                    if candidate_date > from_date {
                        return Ok(Some(candidate_date));
                    }
                }
            }
        }

        Ok(None)
    }

    fn find_next_monthly_by_day(
        from_date: DateTime<Utc>,
        month_days: &[i8],
        interval: u32,
    ) -> AppResult<Option<DateTime<Utc>>> {
        let mut current_year = from_date.year();
        let mut current_month = from_date.month();

        // Try current month first, then advance
        for _ in 0..24 {
            // Limit search to 2 years
            for &day in month_days {
                if let Some(candidate_date) =
                    Self::create_date_with_month_day(current_year, current_month as u8, day)
                {
                    if candidate_date > from_date {
                        return Ok(Some(candidate_date));
                    }
                }
            }

            // Advance to next interval month
            current_month += interval;
            if current_month > 12 {
                current_year += ((current_month - 1) / 12) as i32;
                current_month = (current_month - 1) % 12 + 1;
            }
        }

        Ok(None)
    }

    fn find_next_monthly_by_weekday(
        from_date: DateTime<Utc>,
        entries: &[ByDayEntry],
        interval: u32,
    ) -> AppResult<Option<DateTime<Utc>>> {
        let mut current_year = from_date.year();
        let mut current_month = from_date.month();

        // Try current month first, then advance
        for _ in 0..60 {
            // Limit search to 5 years to avoid infinite loops
            let candidates =
                Self::build_monthly_weekday_candidates(current_year, current_month, entries)?;

            for candidate in &candidates {
                if *candidate > from_date {
                    return Ok(Some(*candidate));
                }
            }

            // Advance to next interval month
            let mut next_month = current_month + interval;
            let mut next_year = current_year;
            while next_month > 12 {
                next_month -= 12;
                next_year += 1;
            }

            current_year = next_year;
            current_month = next_month;
        }

        Ok(None)
    }

    fn build_monthly_weekday_candidates(
        year: i32,
        month: u32,
        entries: &[ByDayEntry],
    ) -> AppResult<Vec<DateTime<Utc>>> {
        let last_day = Self::last_day_of_month(year, month)
            .ok_or_else(|| AppError::validation("Invalid month when building BYDAY candidates"))?;

        let mut occurrences: HashMap<Weekday, Vec<NaiveDate>> = HashMap::new();
        for day in 1..=last_day {
            if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
                occurrences.entry(date.weekday()).or_default().push(date);
            }
        }

        let mut candidates: Vec<NaiveDate> = Vec::new();

        for entry in entries {
            if let Some(date_list) = occurrences.get(&entry.weekday) {
                if date_list.is_empty() {
                    continue;
                }

                if let Some(position) = entry.position {
                    let len = date_list.len();
                    let idx = if position > 0 {
                        let idx = position as usize - 1;
                        if idx >= len {
                            continue;
                        }
                        idx
                    } else {
                        let idx = len as i32 + position as i32;
                        if idx < 0 || idx as usize >= len {
                            continue;
                        }
                        idx as usize
                    };
                    candidates.push(date_list[idx]);
                } else {
                    candidates.extend(date_list.iter().copied());
                }
            }
        }

        candidates.sort();
        candidates.dedup();

        let mut result = Vec::new();
        for date in candidates {
            if let Some(datetime) = date.and_hms_opt(0, 0, 0) {
                result.push(DateTime::from_naive_utc_and_offset(datetime, Utc));
            }
        }

        Ok(result)
    }

    fn create_date_with_month_day(year: i32, month: u8, day: i8) -> Option<DateTime<Utc>> {
        if day > 0 {
            // Positive day (from start of month)
            NaiveDate::from_ymd_opt(year, month as u32, day as u32)
                .and_then(|d| d.and_hms_opt(0, 0, 0))
                .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc))
        } else {
            // Negative day (from end of month)
            let last_day_of_month = Self::last_day_of_month(year, month as u32)?;
            let target_day = (last_day_of_month as i8 + day + 1) as u32;

            if target_day >= 1 && target_day <= last_day_of_month {
                NaiveDate::from_ymd_opt(year, month as u32, target_day)
                    .and_then(|d| d.and_hms_opt(0, 0, 0))
                    .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc))
            } else {
                None
            }
        }
    }

    #[cfg(test)]
    fn find_first_weekday_in_month(
        year: i32,
        month: u32,
        target_weekday: Weekday,
    ) -> Option<DateTime<Utc>> {
        let first_of_month = NaiveDate::from_ymd_opt(year, month, 1)?;
        let first_datetime = first_of_month.and_hms_opt(0, 0, 0)?;
        let first_utc = DateTime::from_naive_utc_and_offset(first_datetime, Utc);

        // Find the first occurrence of the target weekday
        for day_offset in 0..7 {
            let candidate = first_utc + Duration::days(day_offset);
            if candidate.weekday() == target_weekday {
                return Some(candidate);
            }
        }

        None
    }

    fn last_day_of_month(year: i32, month: u32) -> Option<u32> {
        // Get the first day of the next month, then subtract one day
        let (next_year, next_month) = if month == 12 {
            (year + 1, 1)
        } else {
            (year, month + 1)
        };

        NaiveDate::from_ymd_opt(next_year, next_month, 1)
            .map(|d| d.pred_opt())
            .flatten()
            .map(|d| d.day())
    }

    #[allow(dead_code)]
    fn advance_date_for_frequency(
        freq: &Frequency,
        date: DateTime<Utc>,
        interval: u32,
    ) -> AppResult<DateTime<Utc>> {
        match freq {
            Frequency::Daily => Ok(date + Duration::days(interval as i64)),
            Frequency::Weekly => Ok(date + Duration::weeks(interval as i64)),
            Frequency::Monthly => {
                let new_month = date.month() + interval;
                let (year_offset, month) = if new_month > 12 {
                    ((new_month - 1) / 12, (new_month - 1) % 12 + 1)
                } else {
                    (0, new_month)
                };

                date.with_year(date.year() + year_offset as i32)
                    .and_then(|d| d.with_month(month))
                    .ok_or_else(|| AppError::validation("Invalid date calculation"))
            }
            Frequency::Yearly => date
                .with_year(date.year() + interval as i32)
                .ok_or_else(|| AppError::validation("Invalid year calculation")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::rrule_parser::RRuleParser;
    use chrono::TimeZone;

    #[test]
    fn test_generate_daily_instances() {
        let rule = RRuleParser::parse("FREQ=DAILY;COUNT=5").unwrap();
        let config = GenerationConfig {
            horizon_days: 10,
            max_instances: 100,
            start_date: Some(Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap()),
        };

        let instances =
            InstanceGenerator::generate_instances("template_1", "Daily Task", &rule, &config)
                .unwrap();

        assert_eq!(instances.len(), 5);
        assert_eq!(instances[0].instance_date.day(), 2); // Next day after start
        assert_eq!(instances[1].instance_date.day(), 3);
        assert_eq!(instances[4].instance_date.day(), 6);
    }

    #[test]
    fn test_generate_weekly_instances() {
        let rule = RRuleParser::parse("FREQ=WEEKLY;BYDAY=MO,WE,FR").unwrap();
        let config = GenerationConfig {
            horizon_days: 14,
            max_instances: 10,
            start_date: Some(Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap()), // Wednesday
        };

        let instances =
            InstanceGenerator::generate_instances("template_1", "Weekly Task", &rule, &config)
                .unwrap();

        assert!(instances.len() >= 3); // Should have at least Mon, Wed, Fri of first week

        // Check that all instances fall on the correct weekdays
        for instance in &instances {
            let weekday = instance.instance_date.weekday();
            assert!(weekday == Weekday::Mon || weekday == Weekday::Wed || weekday == Weekday::Fri);
        }
    }

    #[test]
    fn test_generate_monthly_by_day() {
        let rule = RRuleParser::parse("FREQ=MONTHLY;BYMONTHDAY=15").unwrap();
        let config = GenerationConfig {
            horizon_days: 90,
            max_instances: 5,
            start_date: Some(Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap()),
        };

        let instances =
            InstanceGenerator::generate_instances("template_1", "Monthly Task", &rule, &config)
                .unwrap();

        assert!(instances.len() >= 3); // Should have at least 3 months worth

        // Check that all instances fall on the 15th
        for instance in &instances {
            assert_eq!(instance.instance_date.day(), 15);
        }
    }

    #[test]
    fn test_generate_monthly_positional_byday() {
        let rule = RRuleParser::parse("FREQ=MONTHLY;BYDAY=1MO,-1FR").unwrap();
        let config = GenerationConfig {
            horizon_days: 90,
            max_instances: 10,
            start_date: Some(Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap()),
        };

        let instances = InstanceGenerator::generate_instances(
            "template_1",
            "Monthly Positional",
            &rule,
            &config,
        )
        .unwrap();

        assert!(instances.len() >= 4);

        let first = instances[0].instance_date.date_naive();
        let second = instances[1].instance_date.date_naive();
        assert_eq!(first, NaiveDate::from_ymd_opt(2025, 1, 6).unwrap());
        assert_eq!(second, NaiveDate::from_ymd_opt(2025, 1, 31).unwrap());

        // Ensure subsequent month also follows pattern
        let feb_dates: Vec<_> = instances
            .iter()
            .map(|instance| instance.instance_date.date_naive())
            .filter(|date| date.month() == 2)
            .collect();

        assert!(feb_dates.contains(&NaiveDate::from_ymd_opt(2025, 2, 3).unwrap()));
        assert!(feb_dates.contains(&NaiveDate::from_ymd_opt(2025, 2, 28).unwrap()));
    }

    #[test]
    fn test_generate_with_until_limit() {
        let rule = RRuleParser::parse("FREQ=DAILY;UNTIL=20250105T000000Z").unwrap();
        let config = GenerationConfig {
            horizon_days: 30,
            max_instances: 100,
            start_date: Some(Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap()),
        };

        let instances =
            InstanceGenerator::generate_instances("template_1", "Limited Task", &rule, &config)
                .unwrap();

        // Should stop at January 5th
        assert!(instances.len() <= 5);
        for instance in &instances {
            assert!(instance.instance_date <= Utc.with_ymd_and_hms(2025, 1, 5, 0, 0, 0).unwrap());
        }
    }

    #[test]
    fn test_calculate_next_occurrence_daily() {
        let rule = RRuleParser::parse("FREQ=DAILY;INTERVAL=2").unwrap();
        let from_date = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();

        let next = InstanceGenerator::calculate_next_occurrence(&rule, from_date)
            .unwrap()
            .unwrap();
        assert_eq!(next.day(), 3); // 2 days later
    }

    #[test]
    fn test_calculate_next_occurrence_weekly() {
        let rule = RRuleParser::parse("FREQ=WEEKLY;BYDAY=FR").unwrap();
        let from_date = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(); // Wednesday

        let next = InstanceGenerator::calculate_next_occurrence(&rule, from_date)
            .unwrap()
            .unwrap();
        assert_eq!(next.weekday(), Weekday::Fri);
        assert_eq!(next.day(), 3); // First Friday after Wednesday Jan 1
    }

    #[test]
    fn test_last_day_of_month() {
        assert_eq!(InstanceGenerator::last_day_of_month(2025, 1), Some(31)); // January
        assert_eq!(InstanceGenerator::last_day_of_month(2025, 2), Some(28)); // February (non-leap)
        assert_eq!(InstanceGenerator::last_day_of_month(2024, 2), Some(29)); // February (leap)
        assert_eq!(InstanceGenerator::last_day_of_month(2025, 4), Some(30)); // April
    }

    #[test]
    fn test_create_date_with_negative_month_day() {
        // Last day of January 2025 (-1)
        let date = InstanceGenerator::create_date_with_month_day(2025, 1, -1).unwrap();
        assert_eq!(date.day(), 31);

        // Second to last day of February 2025 (-2)
        let date = InstanceGenerator::create_date_with_month_day(2025, 2, -2).unwrap();
        assert_eq!(date.day(), 27); // 28 - 1
    }

    #[test]
    fn test_find_first_weekday_in_month() {
        // First Monday of January 2025
        let first_monday =
            InstanceGenerator::find_first_weekday_in_month(2025, 1, Weekday::Mon).unwrap();
        assert_eq!(first_monday.day(), 6); // January 6, 2025 is the first Monday

        // First Friday of January 2025
        let first_friday =
            InstanceGenerator::find_first_weekday_in_month(2025, 1, Weekday::Fri).unwrap();
        assert_eq!(first_friday.day(), 3); // January 3, 2025 is the first Friday
    }
}
