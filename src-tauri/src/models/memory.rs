use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryDocument {
    pub id: String,
    pub file_path: PathBuf,
    pub metadata: MemoryMetadata,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetadata {
    pub date: String,
    pub topics: Vec<String>,
    pub participants: Vec<String>,
    pub summary: String,
    pub relevance_score: f32,
    pub conversation_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryContext {
    pub relevant_documents: Vec<MemoryDocument>,
    pub total_context_length: usize,
    pub search_query: String,
    /// Enhanced conversation context for better AI understanding
    pub conversation_context: Option<Vec<ConversationSummary>>,
    /// Context quality metrics for AI agent
    pub context_quality: ContextQuality,
    /// Token usage estimate for this context
    pub estimated_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextQuality {
    pub relevance_score: f32,
    pub diversity_score: f32,
    pub recency_score: f32,
    pub context_sufficiency: ContextSufficiency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContextSufficiency {
    Insufficient,  // Not enough context for good responses
    Adequate,      // Enough context for basic responses
    Rich,          // Good amount of relevant context
    Comprehensive, // Excellent context coverage
}

#[derive(Debug, Clone)]
pub struct MemoryIndex {
    pub documents: HashMap<String, MemoryDocument>,
    pub topic_index: HashMap<String, Vec<String>>,
    pub date_index: std::collections::BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationSummary {
    pub conversation_id: String,
    pub user_message: String,
    pub ai_response: String,
    pub topics: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySearchQuery {
    pub query: String,
    pub limit: usize,
    pub min_relevance_score: Option<f32>,
    pub date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    pub topics: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryExportOptions {
    pub output_path: PathBuf,
    pub include_metadata: bool,
    pub date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    pub format: MemoryExportFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryExportFormat {
    Archive,
    Json,
    Markdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub total_documents: usize,
    pub total_topics: usize,
    pub total_size_bytes: usize,
    pub oldest_date: Option<String>,
    pub newest_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryValidationReport {
    pub total_checked: usize,
    pub valid_documents: usize,
    pub invalid_documents: Vec<String>,
    pub missing_files: Vec<String>,
    pub corrupted_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportInfo {
    pub export_date: DateTime<Utc>,
    pub total_documents: usize,
    pub date_range: (Option<DateTime<Utc>>, Option<DateTime<Utc>>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonExport {
    pub export_date: DateTime<Utc>,
    pub documents: Vec<MemoryDocument>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryUsage {
    pub total_files: usize,
    pub total_size_bytes: u64,
    pub archive_size_bytes: u64,
    pub size_by_month: HashMap<String, u64>,
}

impl MemoryIndex {
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
            topic_index: HashMap::new(),
            date_index: std::collections::BTreeMap::new(),
        }
    }

    pub fn add_document(&mut self, document: MemoryDocument) {
        let doc_id = document.id.clone();

        // Add to topic index
        for topic in &document.metadata.topics {
            self.topic_index
                .entry(topic.clone())
                .or_insert_with(Vec::new)
                .push(doc_id.clone());
        }

        // Add to date index
        self.date_index
            .entry(document.metadata.date.clone())
            .or_insert_with(Vec::new)
            .push(doc_id.clone());

        // Add to main documents
        self.documents.insert(doc_id, document);
    }

    pub fn remove_document(&mut self, doc_id: &str) {
        if let Some(document) = self.documents.remove(doc_id) {
            // Remove from topic index
            for topic in &document.metadata.topics {
                if let Some(doc_ids) = self.topic_index.get_mut(topic) {
                    doc_ids.retain(|id| id != doc_id);
                    if doc_ids.is_empty() {
                        self.topic_index.remove(topic);
                    }
                }
            }

            // Remove from date index
            if let Some(doc_ids) = self.date_index.get_mut(&document.metadata.date) {
                doc_ids.retain(|id| id != doc_id);
                if doc_ids.is_empty() {
                    self.date_index.remove(&document.metadata.date);
                }
            }
        }
    }

    pub fn get_documents_by_topic(&self, topic: &str) -> Vec<&MemoryDocument> {
        self.topic_index
            .get(topic)
            .map(|doc_ids| {
                doc_ids
                    .iter()
                    .filter_map(|id| self.documents.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_documents_by_date(&self, date: &str) -> Vec<&MemoryDocument> {
        self.date_index
            .get(date)
            .map(|doc_ids| {
                doc_ids
                    .iter()
                    .filter_map(|id| self.documents.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl Default for MemoryIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the search index state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStatistics {
    pub total_documents: usize,
    pub total_topics: usize,
    pub total_date_entries: usize,
    pub pending_index_updates: usize,
    pub last_rebuild_time: DateTime<Utc>,
    pub index_needs_rebuild: bool,
}
