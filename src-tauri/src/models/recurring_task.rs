use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::services::rrule_parser::RecurrenceRule;

/// Recurring task template that defines how task instances are generated
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecurringTaskTemplate {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub recurrence_rule: RecurrenceRule,
    pub priority: String,
    pub tags: Vec<String>,
    pub estimated_minutes: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: bool,
}

impl RecurringTaskTemplate {
    /// Create a new recurring task template
    pub fn new(title: String, recurrence_rule: RecurrenceRule) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title,
            description: None,
            recurrence_rule,
            priority: "medium".to_string(),
            tags: Vec::new(),
            estimated_minutes: None,
            created_at: now,
            updated_at: now,
            is_active: true,
        }
    }

    /// Set the description for this template
    pub fn with_description(mut self, description: Option<String>) -> Self {
        self.description = description;
        self
    }

    /// Set the priority for this template
    pub fn with_priority(mut self, priority: String) -> Self {
        self.priority = priority;
        self
    }

    /// Set the tags for this template
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Set the estimated minutes for this template
    pub fn with_estimated_minutes(mut self, estimated_minutes: Option<i64>) -> Self {
        self.estimated_minutes = estimated_minutes;
        self
    }

    /// Activate this template
    pub fn activate(&mut self) {
        self.is_active = true;
        self.updated_at = Utc::now();
    }

    /// Deactivate this template
    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.updated_at = Utc::now();
    }

    /// Update the template with new values
    pub fn update(&mut self, update: RecurringTaskTemplateUpdate) {
        if let Some(title) = update.title {
            self.title = title;
        }
        if let Some(description) = update.description {
            self.description = description;
        }
        if let Some(recurrence_rule) = update.recurrence_rule {
            self.recurrence_rule = recurrence_rule;
        }
        if let Some(priority) = update.priority {
            self.priority = priority;
        }
        if let Some(tags) = update.tags {
            self.tags = tags;
        }
        if let Some(estimated_minutes) = update.estimated_minutes {
            self.estimated_minutes = estimated_minutes;
        }
        if let Some(is_active) = update.is_active {
            self.is_active = is_active;
        }
        self.updated_at = Utc::now();
    }

    /// Clone this template with a new ID
    pub fn clone_template(&self) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title: format!("{} (Copy)", self.title),
            description: self.description.clone(),
            recurrence_rule: self.recurrence_rule.clone(),
            priority: self.priority.clone(),
            tags: self.tags.clone(),
            estimated_minutes: self.estimated_minutes,
            created_at: now,
            updated_at: now,
            is_active: true,
        }
    }
}

/// Input for creating a new recurring task template
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecurringTaskTemplateCreate {
    pub title: String,
    pub description: Option<String>,
    pub recurrence_rule_string: String, // RRULE string
    pub priority: Option<String>,
    pub tags: Option<Vec<String>>,
    pub estimated_minutes: Option<i64>,
}

/// Input for updating a recurring task template
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecurringTaskTemplateUpdate {
    pub title: Option<String>,
    pub description: Option<Option<String>>,
    pub recurrence_rule: Option<RecurrenceRule>,
    pub priority: Option<String>,
    pub tags: Option<Vec<String>>,
    pub estimated_minutes: Option<Option<i64>>,
    pub is_active: Option<bool>,
}

/// Task instance generated from a recurring template
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskInstance {
    pub id: String,
    pub template_id: String,
    pub instance_date: DateTime<Utc>,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub due_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub is_exception: bool, // Modified from template
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TaskInstance {
    /// Create a new task instance from a template
    pub fn from_template(
        template: &RecurringTaskTemplate,
        instance_date: DateTime<Utc>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            template_id: template.id.clone(),
            instance_date,
            title: template.title.clone(),
            description: template.description.clone(),
            status: "todo".to_string(),
            priority: template.priority.clone(),
            due_at: None,
            completed_at: None,
            is_exception: false,
            created_at: now,
            updated_at: now,
        }
    }

    /// Mark this instance as an exception (modified from template)
    pub fn mark_as_exception(&mut self) {
        self.is_exception = true;
        self.updated_at = Utc::now();
    }

    /// Update this instance
    pub fn update(&mut self, update: TaskInstanceUpdate) {
        if let Some(title) = update.title {
            self.title = title;
            self.mark_as_exception();
        }
        if let Some(description) = update.description {
            self.description = description;
            self.mark_as_exception();
        }
        if let Some(status) = update.status {
            self.status = status;
        }
        if let Some(priority) = update.priority {
            self.priority = priority;
            self.mark_as_exception();
        }
        if let Some(due_at) = update.due_at {
            self.due_at = due_at;
            self.mark_as_exception();
        }
        if let Some(completed_at) = update.completed_at {
            self.completed_at = completed_at;
            if completed_at.is_some() {
                self.status = "completed".to_string();
            }
        }
        self.updated_at = Utc::now();
    }
}

/// Input for updating a task instance
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskInstanceUpdate {
    pub title: Option<String>,
    pub description: Option<Option<String>>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub due_at: Option<Option<DateTime<Utc>>>,
    pub completed_at: Option<Option<DateTime<Utc>>>,
}

/// Filter for querying recurring task templates
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecurringTaskTemplateFilter {
    pub is_active: Option<bool>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub title_contains: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// Filter for querying task instances
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskInstanceFilter {
    pub template_id: Option<String>,
    pub status: Option<String>,
    pub instance_date_after: Option<DateTime<Utc>>,
    pub instance_date_before: Option<DateTime<Utc>>,
    pub due_after: Option<DateTime<Utc>>,
    pub due_before: Option<DateTime<Utc>>,
    pub is_exception: Option<bool>,
}