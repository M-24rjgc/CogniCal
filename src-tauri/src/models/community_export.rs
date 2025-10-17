use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommunityExport {
    pub id: i64,
    pub generated_at: String,
    pub payload_path: String,
    pub metrics_summary: Value,
    #[serde(default)]
    pub includes_feedback: bool,
    pub checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommunityExportCreate {
    pub payload_path: String,
    pub metrics_summary: Value,
    #[serde(default)]
    pub includes_feedback: bool,
    pub checksum: String,
}

impl CommunityExportCreate {
    pub fn new(
        payload_path: String,
        metrics_summary: Value,
        includes_feedback: bool,
        checksum: String,
    ) -> Self {
        Self {
            payload_path,
            metrics_summary,
            includes_feedback,
            checksum,
        }
    }
}
