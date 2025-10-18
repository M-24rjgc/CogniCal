use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

pub const DASHBOARD_MODULE_DEFAULTS: [(&str, bool); 7] = [
    ("quick-actions", true),
    ("today-tasks", true),
    ("upcoming-alerts", true),
    ("productivity-lite", true),
    ("analytics-overview", false),
    ("wellness-summary", false),
    ("workload-forecast", false),
];

fn default_dashboard_modules() -> BTreeMap<String, bool> {
    let mut modules = BTreeMap::new();
    for (module, enabled) in DASHBOARD_MODULE_DEFAULTS.iter() {
        modules.insert((*module).to_string(), *enabled);
    }
    modules
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DashboardConfig {
    #[serde(default)]
    pub modules: BTreeMap<String, bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_updated_at: Option<String>,
}

impl DashboardConfig {
    pub fn normalize(mut self) -> Self {
        let mut modules = default_dashboard_modules();
        for (module, enabled) in self.modules.into_iter() {
            let normalized = module.to_lowercase();
            modules.insert(normalized, enabled);
        }
        self.modules = modules;
        self
    }

    pub fn is_known_module(id: &str) -> bool {
        DASHBOARD_MODULE_DEFAULTS
            .iter()
            .any(|(module, _)| module.eq_ignore_ascii_case(id))
    }
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            modules: default_dashboard_modules(),
            last_updated_at: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deepseek_api_key: Option<String>,
    pub workday_start_minute: i16,
    pub workday_end_minute: i16,
    pub theme: String,
    pub updated_at: String,
    /// Privacy setting: Opt out of AI feedback collection
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_feedback_opt_out: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dashboard_config: Option<DashboardConfig>,
}
