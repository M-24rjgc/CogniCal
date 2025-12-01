use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::Instant;

use chrono::{DateTime, Utc};
use regex::Regex;
use serde_yaml;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::memory::{
    ContextSufficiency, ConversationSummary, ExportInfo, IndexStatistics, JsonExport,
    MemoryContext, MemoryDocument, MemoryExportFormat, MemoryExportOptions, MemoryIndex,
    MemoryMetadata, MemorySearchQuery, MemoryStats, MemoryUsage, MemoryValidationReport,
};

/// Search result cache for frequently accessed queries
#[derive(Clone)]
struct SearchCache {
    cache: Arc<RwLock<HashMap<String, (MemoryContext, DateTime<Utc>)>>>,
}

impl SearchCache {
    fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn get(&self, query: &str) -> Option<MemoryContext> {
        let cache = self.cache.read().unwrap();

        if let Some((context, timestamp)) = cache.get(query) {
            // Cache is valid for 2 minutes
            let age = Utc::now().signed_duration_since(*timestamp);
            if age.num_minutes() < 2 {
                return Some(context.clone());
            }
        }

        None
    }

    fn set(&self, query: String, context: MemoryContext) {
        let mut cache = self.cache.write().unwrap();

        // Limit cache size to 100 entries
        if cache.len() >= 100 {
            // Remove oldest entry
            if let Some(oldest_key) = cache
                .iter()
                .min_by_key(|(_, (_, timestamp))| timestamp)
                .map(|(key, _)| key.clone())
            {
                cache.remove(&oldest_key);
            }
        }

        cache.insert(query, (context, Utc::now()));
    }

    fn clear(&self) {
        let mut cache = self.cache.write().unwrap();
        cache.clear();
    }
}

/// Optimized inverted index for fast text search with incremental updates
#[derive(Clone)]
struct InvertedIndex {
    word_to_docs: Arc<RwLock<HashMap<String, HashSet<String>>>>,
    /// Track document updates to enable incremental reindexing
    pending_updates: Arc<RwLock<HashSet<String>>>,
    /// Last rebuild timestamp
    last_rebuild: Arc<RwLock<DateTime<Utc>>>,
}

impl InvertedIndex {
    fn new() -> Self {
        Self {
            word_to_docs: Arc::new(RwLock::new(HashMap::new())),
            pending_updates: Arc::new(RwLock::new(HashSet::new())),
            last_rebuild: Arc::new(RwLock::new(Utc::now())),
        }
    }

    /// Add a document to the index incrementally
    fn add_document(&self, doc_id: &str, content: &str) {
        let mut index = self.word_to_docs.write().unwrap();
        let words = Self::tokenize(content);

        for word in words {
            index
                .entry(word)
                .or_insert_with(HashSet::new)
                .insert(doc_id.to_string());
        }

        // Mark as updated
        let mut pending = self.pending_updates.write().unwrap();
        pending.insert(doc_id.to_string());
    }

    /// Update a document in place without full rebuild
    fn update_document(&self, doc_id: &str, new_content: &str, _old_content: &str) {
        // For now, remove and re-add (optimized version could diff the content)
        self.remove_document(doc_id);
        self.add_document(doc_id, new_content);
    }

    /// Remove a document from the index
    fn remove_document(&self, doc_id: &str) {
        let mut index = self.word_to_docs.write().unwrap();

        for doc_set in index.values_mut() {
            doc_set.remove(doc_id);
        }

        // Remove empty word entries
        let empty_words: Vec<String> = index
            .iter()
            .filter(|(_, docs)| docs.is_empty())
            .map(|(word, _)| word.clone())
            .collect();

        for word in empty_words {
            index.remove(&word);
        }

        // Mark as updated
        let mut pending = self.pending_updates.write().unwrap();
        pending.remove(doc_id);
    }

    /// Get pending updates count (for monitoring)
    fn get_pending_updates_count(&self) -> usize {
        let pending = self.pending_updates.read().unwrap();
        pending.len()
    }

    /// Force rebuild of entire index (use sparingly)
    fn force_rebuild(&self, documents: &HashMap<String, MemoryDocument>) {
        let mut index = self.word_to_docs.write().unwrap();
        index.clear();

        for (doc_id, document) in documents {
            let words = Self::tokenize(&document.content);
            for word in words {
                index
                    .entry(word)
                    .or_insert_with(HashSet::new)
                    .insert(doc_id.clone());
            }
        }

        // Clear pending updates after rebuild
        let mut pending = self.pending_updates.write().unwrap();
        pending.clear();

        let mut last_rebuild = self.last_rebuild.write().unwrap();
        *last_rebuild = Utc::now();
    }

    /// Check if index needs rebuilding (for maintenance)
    fn needs_rebuild(&self) -> bool {
        let pending = self.pending_updates.read().unwrap();
        let last_rebuild = self.last_rebuild.read().unwrap();

        // Rebuild if too many pending updates or index is very old
        pending.len() > 100 || (Utc::now() - *last_rebuild).num_hours() > 24
    }

    fn search(&self, query: &str) -> HashSet<String> {
        let index = self.word_to_docs.read().unwrap();
        let query_words = Self::tokenize(query);

        if query_words.is_empty() {
            return HashSet::new();
        }

        // Find documents containing all query words (AND search)
        let mut result: Option<HashSet<String>> = None;

        for word in query_words {
            if let Some(docs) = index.get(&word) {
                result = Some(match result {
                    None => docs.clone(),
                    Some(existing) => existing.intersection(docs).cloned().collect(),
                });
            } else {
                // If any word is not found, no documents match
                return HashSet::new();
            }
        }

        result.unwrap_or_default()
    }

    fn tokenize(text: &str) -> Vec<String> {
        text.to_lowercase()
            .split_whitespace()
            .filter(|word| word.len() > 2) // Ignore very short words
            .map(|word| {
                // Remove punctuation
                word.chars()
                    .filter(|c| c.is_alphanumeric())
                    .collect::<String>()
            })
            .filter(|word| !word.is_empty())
            .collect()
    }

    fn clear(&self) {
        let mut index = self.word_to_docs.write().unwrap();
        index.clear();
    }
}

#[derive(Clone)]
pub struct MemoryService {
    memory_dir: PathBuf,
    search_index: Arc<RwLock<MemoryIndex>>,
    search_cache: SearchCache,
    inverted_index: InvertedIndex,
}

impl MemoryService {
    pub fn new(memory_dir: PathBuf) -> AppResult<Self> {
        // Ensure memory directory exists
        if !memory_dir.exists() {
            fs::create_dir_all(&memory_dir).map_err(|e| {
                AppError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to create memory directory: {}", e),
                ))
            })?;
        }

        let service = Self {
            memory_dir,
            search_index: Arc::new(RwLock::new(MemoryIndex::new())),
            search_cache: SearchCache::new(),
            inverted_index: InvertedIndex::new(),
        };

        // Load existing memory documents into index
        service.rebuild_index()?;

        Ok(service)
    }

    /// Store a conversation as a memory document
    pub async fn store_conversation(
        &self,
        conversation_id: &str,
        user_message: &str,
        ai_response: &str,
        topics: Vec<String>,
    ) -> AppResult<String> {
        let now = Utc::now();
        let doc_id = Uuid::new_v4().to_string();

        // Extract topics if none provided
        let extracted_topics = if topics.is_empty() {
            self.extract_topics(&format!("{}\n{}", user_message, ai_response))
        } else {
            topics
        };

        // Generate summary
        let summary = self.generate_summary(user_message, ai_response)?;

        // Create metadata
        let metadata = MemoryMetadata {
            date: now.format("%Y-%m-%d").to_string(),
            topics: extracted_topics,
            participants: vec!["user".to_string(), "assistant".to_string()],
            summary,
            relevance_score: 1.0, // Initial score, will be updated based on usage
            conversation_id: conversation_id.to_string(),
        };

        // Create document content
        let content = self.create_document_content(&metadata, user_message, ai_response)?;

        // Determine file path
        let file_path = self.get_document_path(&now, &doc_id)?;

        // Ensure directory exists
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Write document to file
        fs::write(&file_path, content.clone())?;

        // Create memory document
        let document = MemoryDocument {
            id: doc_id.clone(),
            file_path: file_path.clone(),
            metadata,
            content,
            created_at: now,
        };

        // Add to index
        {
            let mut index = self.search_index.write().unwrap();
            index.add_document(document.clone());
        }

        // Add to inverted index for fast search
        self.inverted_index.add_document(&doc_id, &document.content);

        // Check if index needs rebuilding after adding document
        if self.inverted_index.needs_rebuild() {
            info!("Index has too many pending updates, triggering rebuild");
            let documents = {
                let index = self.search_index.read().unwrap();
                index.documents.clone()
            };
            self.inverted_index.force_rebuild(&documents);
        }

        // Clear search cache since new document was added
        self.search_cache.clear();

        info!("Stored memory document: {}", doc_id);
        Ok(doc_id)
    }

    /// Search memory documents for relevant context
    pub async fn search_memory(&self, query: &str, limit: usize) -> AppResult<MemoryContext> {
        let search_query = MemorySearchQuery {
            query: query.to_string(),
            limit,
            min_relevance_score: Some(0.1),
            date_range: None,
            topics: None,
        };

        self.search_memory_with_query(&search_query).await
    }

    /// Search memory with detailed query parameters and enhanced conversation context
    pub async fn search_memory_with_query(
        &self,
        search_query: &MemorySearchQuery,
    ) -> AppResult<MemoryContext> {
        // Check cache first for exact query matches
        let cache_key = format!(
            "{}:{}:{:?}:{:?}",
            search_query.query,
            search_query.limit,
            search_query.min_relevance_score,
            search_query.topics
        );

        if let Some(cached_context) = self.search_cache.get(&cache_key) {
            return Ok(cached_context);
        }

        let start_time = Instant::now();

        // Collect documents to search while holding the lock
        let documents_to_search: Vec<MemoryDocument>;
        let candidate_doc_ids: HashSet<String>;

        {
            let index = self.search_index.read().unwrap();

            // Use inverted index for fast initial filtering
            candidate_doc_ids = self.inverted_index.search(&search_query.query);

            // Only search through candidate documents from inverted index
            documents_to_search = if candidate_doc_ids.is_empty() {
                // If no candidates from inverted index, fall back to full search
                index.documents.values().cloned().collect()
            } else {
                candidate_doc_ids
                    .iter()
                    .filter_map(|id| index.documents.get(id).cloned())
                    .collect()
            };
        } // Lock is released here

        // Now search through documents without holding the lock
        let mut relevant_docs = Vec::new();
        let mut total_context_length = 0;
        let mut total_relevance = 0.0;
        let mut topics_diversity: HashSet<String> = HashSet::new();

        for document in &documents_to_search {
            let relevance_score = self.calculate_relevance_score(document, &search_query.query);

            // Apply filters
            if let Some(min_score) = search_query.min_relevance_score {
                if relevance_score < min_score {
                    continue;
                }
            }

            if let Some(ref topics) = search_query.topics {
                if !document.metadata.topics.iter().any(|t| topics.contains(t)) {
                    continue;
                }
            }

            if let Some((start, end)) = search_query.date_range {
                if document.created_at < start || document.created_at > end {
                    continue;
                }
            }

            let mut doc_with_score = document.clone();
            doc_with_score.metadata.relevance_score = relevance_score;
            relevant_docs.push(doc_with_score.clone());

            total_context_length += doc_with_score.content.len();
            total_relevance += relevance_score;
            topics_diversity.extend(doc_with_score.metadata.topics.iter().cloned());
        }

        // Sort by relevance score (descending)
        relevant_docs.sort_by(|a, b| {
            b.metadata
                .relevance_score
                .partial_cmp(&a.metadata.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Limit results and calculate total context length
        relevant_docs.truncate(search_query.limit);
        for doc in &relevant_docs {
            total_context_length += doc.content.len();
        }

        // Calculate metrics before moving relevant_docs
        let avg_relevance = if relevant_docs.is_empty() {
            0.0
        } else {
            total_relevance / relevant_docs.len() as f32
        };
        let doc_count = relevant_docs.len();

        // Enhance with conversation context (this is the await point)
        let conversation_context = self
            .build_conversation_context(
                &relevant_docs,
                &search_query.query,
                3, // Max 3 related conversations
            )
            .await?;
        let diversity_score = self.calculate_diversity_score(&topics_diversity);
        let recency_score = self.calculate_recency_score(&relevant_docs);
        let estimated_tokens = self.estimate_context_tokens(&relevant_docs, &conversation_context);

        let context = MemoryContext {
            relevant_documents: relevant_docs,
            total_context_length,
            search_query: search_query.query.clone(),
            conversation_context: Some(conversation_context),
            context_quality: crate::models::memory::ContextQuality {
                relevance_score: avg_relevance,
                diversity_score,
                recency_score,
                context_sufficiency: self.determine_context_sufficiency(
                    avg_relevance,
                    diversity_score,
                    recency_score,
                    doc_count,
                ),
            },
            estimated_tokens,
        };

        // Cache the result
        self.search_cache.set(cache_key, context.clone());

        debug!("Memory search completed in {:?}", start_time.elapsed());
        Ok(context)
    }

    /// Build conversation context from related documents
    async fn build_conversation_context(
        &self,
        relevant_docs: &[MemoryDocument],
        _current_query: &str,
        max_conversations: usize,
    ) -> AppResult<Vec<ConversationSummary>> {
        use std::collections::HashSet;

        let mut conversation_summaries = Vec::new();
        let mut seen_conversations = HashSet::new();

        // Extract conversation IDs from relevant documents
        for doc in relevant_docs.iter().take(max_conversations * 2) {
            // Look at more docs to find conversations
            let conv_id = &doc.metadata.conversation_id;

            if !seen_conversations.contains(conv_id) {
                seen_conversations.insert(conv_id.clone());

                // Find all documents in this conversation
                let related_docs: Vec<_> = relevant_docs
                    .iter()
                    .filter(|d| d.metadata.conversation_id == *conv_id)
                    .collect();

                if related_docs.len() > 1 {
                    // Only include actual conversations (multiple exchanges)
                    let summary = self.create_conversation_summary(conv_id, &related_docs)?;
                    conversation_summaries.push(summary);

                    if conversation_summaries.len() >= max_conversations {
                        break;
                    }
                }
            }
        }

        Ok(conversation_summaries)
    }

    /// Create a summary of a conversation from multiple documents
    fn create_conversation_summary(
        &self,
        conversation_id: &str,
        documents: &[&MemoryDocument],
    ) -> AppResult<ConversationSummary> {
        let mut user_messages = Vec::new();
        let mut ai_responses = Vec::new();
        let mut topics = HashSet::new();
        let mut timestamp = Utc::now();

        for doc in documents {
            // Extract user message and AI response from content
            if let Some((user_msg, ai_resp)) = self.parse_conversation_exchange(&doc.content) {
                user_messages.push(user_msg);
                ai_responses.push(ai_resp);
            }

            topics.extend(doc.metadata.topics.iter());
            if doc.created_at < timestamp {
                timestamp = doc.created_at;
            }
        }

        // Create a condensed summary
        let _summary_text = self.create_conversation_summary_text(&user_messages, &ai_responses);

        Ok(ConversationSummary {
            conversation_id: conversation_id.to_string(),
            user_message: user_messages.join(" | "),
            ai_response: ai_responses.join(" | "),
            topics: topics.into_iter().map(|s| s.clone()).collect(),
            timestamp,
        })
    }

    /// Parse conversation exchange from document content
    fn parse_conversation_exchange(&self, content: &str) -> Option<(String, String)> {
        // Simple parsing logic - could be enhanced based on actual content format
        let lines: Vec<&str> = content.lines().collect();
        if lines.len() >= 2 {
            let user_msg = lines[0].to_string();
            let ai_resp = lines[1..].join("\n").to_string();
            Some((user_msg, ai_resp))
        } else {
            None
        }
    }

    /// Create a text summary of a conversation
    fn create_conversation_summary_text(
        &self,
        user_messages: &[String],
        _ai_responses: &[String],
    ) -> String {
        if user_messages.is_empty() {
            return "No conversation data".to_string();
        }

        let msg_count = user_messages.len();
        let avg_msg_length = user_messages.iter().map(|s| s.len()).sum::<usize>() / msg_count;

        format!(
            "Conversation: {} exchanges, avg message length: {} chars, topics: varied",
            msg_count, avg_msg_length
        )
    }

    /// Calculate diversity score based on topic variety
    fn calculate_diversity_score(&self, topics: &HashSet<String>) -> f32 {
        if topics.is_empty() {
            0.0
        } else {
            (topics.len() as f32 / 10.0).min(1.0) // Normalize to 0-1, capped at 10 topics
        }
    }

    /// Calculate recency score based on document timestamps
    fn calculate_recency_score(&self, documents: &[MemoryDocument]) -> f32 {
        if documents.is_empty() {
            return 0.0;
        }

        let now = Utc::now();
        let mut recency_scores = Vec::new();

        for doc in documents {
            let age_days = now.signed_duration_since(doc.created_at).num_days();
            let recency_score = if age_days <= 1 {
                1.0
            } else if age_days <= 7 {
                0.8
            } else if age_days <= 30 {
                0.5
            } else if age_days <= 90 {
                0.2
            } else {
                0.1
            };
            recency_scores.push(recency_score);
        }

        // Return average recency score
        recency_scores.into_iter().sum::<f32>() / documents.len() as f32
    }

    /// Estimate token count for the context
    fn estimate_context_tokens(
        &self,
        documents: &[MemoryDocument],
        conversations: &[ConversationSummary],
    ) -> u32 {
        let mut total_chars = 0;

        for doc in documents {
            total_chars += doc.content.len();
        }

        for conv in conversations {
            total_chars += conv.user_message.len() + conv.ai_response.len();
        }

        // Rough estimate: 1 token ≈ 4 characters
        (total_chars / 4) as u32
    }

    /// Determine context sufficiency level
    fn determine_context_sufficiency(
        &self,
        relevance: f32,
        diversity: f32,
        recency: f32,
        doc_count: usize,
    ) -> ContextSufficiency {
        let overall_score = (relevance + diversity + recency) / 3.0;

        if doc_count < 2 || overall_score < 0.3 {
            ContextSufficiency::Insufficient
        } else if doc_count < 5 || overall_score < 0.6 {
            ContextSufficiency::Adequate
        } else if doc_count < 8 || overall_score < 0.8 {
            ContextSufficiency::Rich
        } else {
            ContextSufficiency::Comprehensive
        }
    }

    /// Get recent memory context
    pub async fn get_recent_context(
        &self,
        days: u32,
        limit: usize,
    ) -> AppResult<Vec<MemoryDocument>> {
        let cutoff_date = Utc::now() - chrono::Duration::days(days as i64);
        let index = self.search_index.read().unwrap();

        let mut recent_docs: Vec<MemoryDocument> = index
            .documents
            .values()
            .filter(|doc| doc.created_at >= cutoff_date)
            .cloned()
            .collect();

        // Sort by creation date (most recent first)
        recent_docs.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        recent_docs.truncate(limit);

        Ok(recent_docs)
    }

    /// Export memory archive with options
    pub async fn export_memory_archive(&self, options: &MemoryExportOptions) -> AppResult<()> {
        // Clone documents to avoid holding lock across await
        let documents_to_export: Vec<MemoryDocument> = {
            let index = self.search_index.read().unwrap();

            // Filter documents by date range if specified
            let docs: Vec<MemoryDocument> = if let Some((start, end)) = options.date_range {
                index
                    .documents
                    .values()
                    .filter(|doc| doc.created_at >= start && doc.created_at <= end)
                    .cloned()
                    .collect()
            } else {
                index.documents.values().cloned().collect()
            };

            docs
        }; // Lock is released here

        // Create output directory
        fs::create_dir_all(&options.output_path)?;

        match options.format {
            MemoryExportFormat::Archive => {
                self.export_as_archive(
                    &documents_to_export,
                    &options.output_path,
                    options.include_metadata,
                )
                .await?;
            }
            MemoryExportFormat::Json => {
                self.export_as_json(&documents_to_export, &options.output_path)
                    .await?;
            }
            MemoryExportFormat::Markdown => {
                self.export_as_markdown(&documents_to_export, &options.output_path)
                    .await?;
            }
        }

        info!(
            "Exported {} memory documents to {:?}",
            documents_to_export.len(),
            options.output_path
        );
        Ok(())
    }

    /// Export as file archive (preserving directory structure)
    async fn export_as_archive(
        &self,
        documents: &[MemoryDocument],
        output_path: &Path,
        include_metadata: bool,
    ) -> AppResult<()> {
        // Copy all memory documents preserving structure
        for document in documents {
            let relative_path = document
                .file_path
                .strip_prefix(&self.memory_dir)
                .map_err(|_| AppError::Other("Invalid file path".to_string()))?;

            let dest_path = output_path.join(relative_path);

            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::copy(&document.file_path, &dest_path)?;
        }

        if include_metadata {
            // Create metadata index
            let metadata_path = output_path.join("metadata.json");
            let metadata: Vec<&MemoryMetadata> =
                documents.iter().map(|doc| &doc.metadata).collect();
            let metadata_json = serde_json::to_string_pretty(&metadata)?;
            fs::write(metadata_path, metadata_json)?;

            // Create export info
            let export_info = ExportInfo {
                export_date: Utc::now(),
                total_documents: documents.len(),
                date_range: documents.iter().map(|doc| doc.created_at).fold(
                    (None, None),
                    |acc, date| {
                        let min = acc.0.map_or(Some(date), |min| Some(min.min(date)));
                        let max = acc.1.map_or(Some(date), |max| Some(max.max(date)));
                        (min, max)
                    },
                ),
            };

            let info_path = output_path.join("export_info.json");
            let info_json = serde_json::to_string_pretty(&export_info)?;
            fs::write(info_path, info_json)?;
        }

        Ok(())
    }

    /// Export as single JSON file
    async fn export_as_json(
        &self,
        documents: &[MemoryDocument],
        output_path: &Path,
    ) -> AppResult<()> {
        let export_data = JsonExport {
            export_date: Utc::now(),
            documents: documents.to_vec(),
        };

        let json_path = output_path.join("memory_export.json");
        let json_data = serde_json::to_string_pretty(&export_data)?;
        fs::write(json_path, json_data)?;

        Ok(())
    }

    /// Export as single Markdown file
    async fn export_as_markdown(
        &self,
        documents: &[MemoryDocument],
        output_path: &Path,
    ) -> AppResult<()> {
        let mut markdown_content = String::new();
        markdown_content.push_str("# Memory Export\n\n");
        markdown_content.push_str(&format!(
            "Export Date: {}\n",
            Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        ));
        markdown_content.push_str(&format!("Total Documents: {}\n\n", documents.len()));

        // Sort documents by date
        let mut sorted_docs = documents.to_vec();
        sorted_docs.sort_by(|a, b| a.created_at.cmp(&b.created_at));

        for document in sorted_docs {
            markdown_content.push_str(&format!(
                "## {} ({})\n\n",
                document.metadata.summary, document.metadata.date
            ));
            markdown_content.push_str(&format!(
                "**Topics:** {}\n",
                document.metadata.topics.join(", ")
            ));
            markdown_content.push_str(&format!(
                "**Conversation ID:** {}\n\n",
                document.metadata.conversation_id
            ));
            markdown_content.push_str(&document.content);
            markdown_content.push_str("\n\n---\n\n");
        }

        let markdown_path = output_path.join("memory_export.md");
        fs::write(markdown_path, markdown_content)?;

        Ok(())
    }

    /// Archive old memories (move to archive directory)
    pub async fn archive_old_memories(&self, older_than_days: u32) -> AppResult<usize> {
        let cutoff_date = Utc::now() - chrono::Duration::days(older_than_days as i64);
        let archive_dir = self.memory_dir.join("archive");

        // Create archive directory
        fs::create_dir_all(&archive_dir)?;

        let index = self.search_index.read().unwrap();
        let mut archived_count = 0;

        let docs_to_archive: Vec<MemoryDocument> = index
            .documents
            .values()
            .filter(|doc| doc.created_at < cutoff_date)
            .cloned()
            .collect();

        drop(index);

        for document in docs_to_archive {
            // Create archive path
            let relative_path = document
                .file_path
                .strip_prefix(&self.memory_dir)
                .map_err(|_| AppError::Other("Invalid file path".to_string()))?;
            let archive_path = archive_dir.join(relative_path);

            // Ensure archive directory exists
            if let Some(parent) = archive_path.parent() {
                fs::create_dir_all(parent)?;
            }

            // Move file to archive
            if let Err(e) = fs::rename(&document.file_path, &archive_path) {
                warn!(
                    "Failed to archive memory file {:?}: {}",
                    document.file_path, e
                );
            } else {
                archived_count += 1;
            }

            // Remove from index
            let mut index = self.search_index.write().unwrap();
            index.remove_document(&document.id);
        }

        info!("Archived {} old memory documents", archived_count);
        Ok(archived_count)
    }

    /// Restore archived memories
    pub async fn restore_archived_memories(
        &self,
        archive_date_filter: Option<&str>,
    ) -> AppResult<usize> {
        let archive_dir = self.memory_dir.join("archive");

        if !archive_dir.exists() {
            return Ok(0);
        }

        let mut restored_count = 0;
        self.restore_from_directory(&archive_dir, archive_date_filter, &mut restored_count)?;

        // Rebuild index to include restored documents
        self.rebuild_index()?;

        info!("Restored {} archived memory documents", restored_count);
        Ok(restored_count)
    }

    /// Recursively restore files from archive directory
    fn restore_from_directory(
        &self,
        dir: &Path,
        date_filter: Option<&str>,
        restored_count: &mut usize,
    ) -> AppResult<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                self.restore_from_directory(&path, date_filter, restored_count)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("md") {
                // Check date filter if specified
                if let Some(filter_date) = date_filter {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok((metadata, _)) = self.parse_document_content(&content) {
                            if metadata.date != filter_date {
                                continue;
                            }
                        }
                    }
                }

                // Calculate restore path
                let relative_path = path
                    .strip_prefix(&self.memory_dir.join("archive"))
                    .map_err(|_| AppError::Other("Invalid archive path".to_string()))?;
                let restore_path = self.memory_dir.join(relative_path);

                // Ensure restore directory exists
                if let Some(parent) = restore_path.parent() {
                    fs::create_dir_all(parent)?;
                }

                // Move file back to main memory directory
                if let Err(e) = fs::rename(&path, &restore_path) {
                    warn!("Failed to restore memory file {:?}: {}", path, e);
                } else {
                    *restored_count += 1;
                }
            }
        }

        Ok(())
    }

    /// Get memory usage statistics
    pub async fn get_memory_usage(&self) -> AppResult<MemoryUsage> {
        let index = self.search_index.read().unwrap();

        let total_files = index.documents.len();
        let total_size = self.calculate_directory_size(&self.memory_dir)?;
        let archive_size = self
            .calculate_directory_size(&self.memory_dir.join("archive"))
            .unwrap_or(0);

        // Calculate size by date
        let mut size_by_month = HashMap::new();
        for document in index.documents.values() {
            let month_key = document.created_at.format("%Y-%m").to_string();
            let file_size = fs::metadata(&document.file_path)
                .map(|m| m.len())
                .unwrap_or(0);
            *size_by_month.entry(month_key).or_insert(0) += file_size;
        }

        Ok(MemoryUsage {
            total_files,
            total_size_bytes: total_size,
            archive_size_bytes: archive_size,
            size_by_month,
        })
    }

    /// Calculate total size of a directory
    fn calculate_directory_size(&self, dir: &Path) -> AppResult<u64> {
        if !dir.exists() {
            return Ok(0);
        }

        let mut total_size = 0;
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                total_size += self.calculate_directory_size(&path)?;
            } else {
                total_size += fs::metadata(&path)?.len();
            }
        }

        Ok(total_size)
    }

    /// Clean up old memory documents
    pub async fn cleanup_old_memories(&self, older_than_days: u32) -> AppResult<usize> {
        let cutoff_date = Utc::now() - chrono::Duration::days(older_than_days as i64);
        let mut index = self.search_index.write().unwrap();
        let mut removed_count = 0;

        let docs_to_remove: Vec<String> = index
            .documents
            .values()
            .filter(|doc| doc.created_at < cutoff_date)
            .map(|doc| doc.id.clone())
            .collect();

        for doc_id in docs_to_remove {
            if let Some(document) = index.documents.get(&doc_id) {
                // Remove file
                if let Err(e) = fs::remove_file(&document.file_path) {
                    warn!(
                        "Failed to remove memory file {:?}: {}",
                        document.file_path, e
                    );
                } else {
                    removed_count += 1;
                }
            }

            // Remove from index and inverted index
            index.remove_document(&doc_id);
            self.inverted_index.remove_document(&doc_id);
        }

        // Clear search cache after cleanup
        self.search_cache.clear();

        info!("Cleaned up {} old memory documents", removed_count);
        Ok(removed_count)
    }

    /// Rebuild the search index from existing files
    pub fn rebuild_index(&self) -> AppResult<()> {
        let mut index = self.search_index.write().unwrap();
        *index = MemoryIndex::new();

        // Clear inverted index and search cache
        self.inverted_index.clear();
        self.search_cache.clear();

        self.scan_directory(&self.memory_dir, &mut index)?;

        info!(
            "Rebuilt memory index with {} documents",
            index.documents.len()
        );
        Ok(())
    }

    /// Recursively scan directory for memory documents
    fn scan_directory(&self, dir: &Path, index: &mut MemoryIndex) -> AppResult<()> {
        if !dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                self.scan_directory(&path, index)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("md") {
                if let Ok(document) = self.load_document_from_file(&path) {
                    // Add to inverted index for fast search
                    self.inverted_index
                        .add_document(&document.id, &document.content);
                    index.add_document(document);
                }
            }
        }

        Ok(())
    }

    /// Load a memory document from a file
    fn load_document_from_file(&self, file_path: &Path) -> AppResult<MemoryDocument> {
        let content = fs::read_to_string(file_path)?;
        let (metadata, body) = self.parse_document_content(&content)?;

        let doc_id = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(MemoryDocument {
            id: doc_id,
            file_path: file_path.to_path_buf(),
            metadata,
            content: body,
            created_at: Utc::now(), // Will be overridden by metadata date if available
        })
    }

    /// Parse document content to extract metadata and body
    fn parse_document_content(&self, content: &str) -> AppResult<(MemoryMetadata, String)> {
        // Look for YAML frontmatter
        let frontmatter_regex = Regex::new(r"^---\n(.*?)\n---\n(.*)$").unwrap();

        if let Some(captures) = frontmatter_regex.captures(content) {
            let yaml_content = captures.get(1).unwrap().as_str();
            let body = captures.get(2).unwrap().as_str();

            let metadata: MemoryMetadata = serde_yaml::from_str(yaml_content)
                .map_err(|e| AppError::Other(format!("Failed to parse metadata: {}", e)))?;

            Ok((metadata, body.to_string()))
        } else {
            Err(AppError::Other("Invalid document format".to_string()))
        }
    }

    /// Generate a summary from user message and AI response
    fn generate_summary(&self, user_message: &str, _ai_response: &str) -> AppResult<String> {
        // Simple summary generation - take first sentence of user message
        let user_first_sentence = user_message
            .split('.')
            .next()
            .unwrap_or(user_message)
            .trim();

        // Limit summary length, ensuring we don't break UTF-8 characters
        if user_first_sentence.len() > 100 {
            let mut end = 97;
            while end > 0 && !user_first_sentence.is_char_boundary(end) {
                end -= 1;
            }
            Ok(format!("{}...", &user_first_sentence[..end]))
        } else {
            Ok(user_first_sentence.to_string())
        }
    }

    /// Extract topics from conversation content
    fn extract_topics(&self, content: &str) -> Vec<String> {
        let mut topics = Vec::new();
        let content_lower = content.to_lowercase();

        // Simple keyword-based topic extraction
        let topic_keywords = [
            (
                "task management",
                vec!["task", "todo", "schedule", "deadline"],
            ),
            (
                "recurring tasks",
                vec!["recurring", "repeat", "daily", "weekly", "monthly"],
            ),
            (
                "project planning",
                vec!["project", "plan", "milestone", "goal"],
            ),
            (
                "dependencies",
                vec!["dependency", "depends", "prerequisite", "blocker"],
            ),
            ("ai assistance", vec!["ai", "assistant", "help", "suggest"]),
            (
                "productivity",
                vec!["productivity", "efficiency", "workflow"],
            ),
            (
                "calendar",
                vec!["calendar", "event", "meeting", "appointment"],
            ),
        ];

        for (topic, keywords) in &topic_keywords {
            if keywords
                .iter()
                .any(|keyword| content_lower.contains(keyword))
            {
                topics.push(topic.to_string());
            }
        }

        if topics.is_empty() {
            topics.push("general".to_string());
        }

        topics
    }

    /// Calculate relevance score for a document given a query
    fn calculate_relevance_score(&self, document: &MemoryDocument, query: &str) -> f32 {
        let query_lower = query.to_lowercase();
        let content_lower = document.content.to_lowercase();
        let summary_lower = document.metadata.summary.to_lowercase();

        let mut score = 0.0;

        // Check for exact matches in summary (high weight)
        if summary_lower.contains(&query_lower) {
            score += 0.8;
        }

        // Check for word matches in content with TF-IDF-like scoring
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();
        let content_words: Vec<&str> = content_lower.split_whitespace().collect();

        let matching_words = query_words
            .iter()
            .filter(|word| content_words.contains(word))
            .count();

        if !query_words.is_empty() {
            let word_match_score = matching_words as f32 / query_words.len() as f32;
            score += word_match_score * 0.6;

            // Bonus for multiple word matches
            if matching_words > 1 {
                score += 0.2;
            }
        }

        // Check topic relevance with partial matching
        for topic in &document.metadata.topics {
            let topic_lower = topic.to_lowercase();
            if query_lower.contains(&topic_lower) || topic_lower.contains(&query_lower) {
                score += 0.3;
            }
        }

        // Boost recent documents with decay
        let days_old = (Utc::now() - document.created_at).num_days();
        let recency_boost = match days_old {
            0..=1 => 0.2,
            2..=7 => 0.1,
            8..=30 => 0.05,
            _ => 0.0,
        };
        score += recency_boost;

        // Boost documents with higher base relevance scores
        score += document.metadata.relevance_score * 0.1;

        score.min(1.0)
    }

    /// Advanced search with semantic similarity (simplified version)
    pub async fn semantic_search(
        &self,
        query: &str,
        limit: usize,
        context_limit_tokens: Option<usize>,
    ) -> AppResult<MemoryContext> {
        let search_query = MemorySearchQuery {
            query: query.to_string(),
            limit,
            min_relevance_score: Some(0.2),
            date_range: None,
            topics: None,
        };

        let mut context = self.search_memory_with_query(&search_query).await?;

        // Apply token limit if specified
        if let Some(token_limit) = context_limit_tokens {
            self.limit_context_by_tokens(&mut context, token_limit);
        }

        Ok(context)
    }

    /// Limit memory context by token count with safe string handling
    fn limit_context_by_tokens(&self, context: &mut MemoryContext, token_limit: usize) {
        let mut total_tokens = 0;
        let mut limited_docs = Vec::new();

        for doc in &context.relevant_documents {
            // Rough token estimation: 1 token ≈ 4 characters
            let doc_tokens = doc.content.len() / 4;

            if total_tokens + doc_tokens <= token_limit {
                total_tokens += doc_tokens;
                limited_docs.push(doc.clone());
            } else {
                // If adding this document would exceed the limit, try to add a truncated version
                let remaining_tokens = token_limit - total_tokens;
                if remaining_tokens > 0 {
                    let remaining_chars = remaining_tokens * 4;
                    if remaining_chars > 10 {
                        // Only add if meaningful amount of content
                        let mut truncated_doc = doc.clone();
                        truncated_doc.content =
                            self.safe_truncate_content(&doc.content, remaining_chars);
                        limited_docs.push(truncated_doc);
                    }
                }
                break;
            }
        }

        context.relevant_documents = limited_docs;
        context.total_context_length = total_tokens * 4; // Convert back to characters
    }

    /// Safely truncate content at character boundary
    fn safe_truncate_content(&self, content: &str, max_chars: usize) -> String {
        if content.len() <= max_chars {
            return content.to_string();
        }

        // Find the nearest character boundary at or before max_chars
        let mut end = max_chars;
        while end > 0 && !content.is_char_boundary(end) {
            end -= 1;
        }

        if end == 0 {
            // Fallback: just return empty string if we can't find a boundary
            String::new()
        } else {
            format!("{}...", &content[..end])
        }
    }

    /// Search by conversation ID
    pub async fn search_by_conversation_id(
        &self,
        conversation_id: &str,
    ) -> AppResult<Vec<MemoryDocument>> {
        let index = self.search_index.read().unwrap();
        let documents: Vec<MemoryDocument> = index
            .documents
            .values()
            .filter(|doc| doc.metadata.conversation_id == conversation_id)
            .cloned()
            .collect();

        Ok(documents)
    }

    /// Get contextually related documents
    pub async fn get_related_documents(
        &self,
        doc_id: &str,
        limit: usize,
    ) -> AppResult<Vec<MemoryDocument>> {
        let index = self.search_index.read().unwrap();

        if let Some(source_doc) = index.documents.get(doc_id) {
            let mut related_docs = Vec::new();

            // Find documents with overlapping topics
            for (other_id, other_doc) in &index.documents {
                if other_id == doc_id {
                    continue;
                }

                let topic_overlap = source_doc
                    .metadata
                    .topics
                    .iter()
                    .filter(|topic| other_doc.metadata.topics.contains(topic))
                    .count();

                if topic_overlap > 0 {
                    let mut doc_with_score = other_doc.clone();
                    doc_with_score.metadata.relevance_score =
                        topic_overlap as f32 / source_doc.metadata.topics.len() as f32;
                    related_docs.push(doc_with_score);
                }
            }

            // Sort by relevance and limit
            related_docs.sort_by(|a, b| {
                b.metadata
                    .relevance_score
                    .partial_cmp(&a.metadata.relevance_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            related_docs.truncate(limit);

            Ok(related_docs)
        } else {
            Err(AppError::NotFound)
        }
    }

    /// Get memory context for AI conversation
    pub async fn get_conversation_context(
        &self,
        query: &str,
        max_context_tokens: usize,
    ) -> AppResult<String> {
        let context = self
            .semantic_search(query, 5, Some(max_context_tokens))
            .await?;

        if context.relevant_documents.is_empty() {
            return Ok(String::new());
        }

        let mut context_text = String::new();
        context_text.push_str("## Relevant Memory Context\n\n");

        for (i, doc) in context.relevant_documents.iter().enumerate() {
            context_text.push_str(&format!(
                "### Memory {}: {} (Score: {:.2})\n",
                i + 1,
                doc.metadata.summary,
                doc.metadata.relevance_score
            ));

            context_text.push_str(&format!("**Topics:** {}\n", doc.metadata.topics.join(", ")));
            context_text.push_str(&format!("**Date:** {}\n\n", doc.metadata.date));

            // Include a snippet of the content using safe truncation
            let content_snippet = self.safe_truncate_content(&doc.content, 300);

            context_text.push_str(&content_snippet);
            context_text.push_str("\n\n---\n\n");
        }

        Ok(context_text)
    }

    /// Create document content with YAML frontmatter
    fn create_document_content(
        &self,
        metadata: &MemoryMetadata,
        user_message: &str,
        ai_response: &str,
    ) -> AppResult<String> {
        let yaml_metadata = serde_yaml::to_string(metadata)
            .map_err(|e| AppError::Other(format!("Failed to serialize metadata: {}", e)))?;

        let content = format!(
            "---\n{}---\n\n# Conversation Summary: {}\n\n## User Message\n{}\n\n## AI Response\n{}\n\n## Topics\n{}\n",
            yaml_metadata,
            metadata.summary,
            user_message,
            ai_response,
            metadata.topics.join(", ")
        );

        Ok(content)
    }

    /// Get the file path for a memory document
    fn get_document_path(&self, timestamp: &DateTime<Utc>, doc_id: &str) -> AppResult<PathBuf> {
        let year = timestamp.format("%Y").to_string();
        let month = timestamp.format("%m").to_string();

        let path = self
            .memory_dir
            .join(year)
            .join(month)
            .join(format!("{}.md", doc_id));

        Ok(path)
    }

    /// Get memory statistics
    pub fn get_memory_stats(&self) -> AppResult<MemoryStats> {
        let index = self.search_index.read().unwrap();

        let total_documents = index.documents.len();
        let total_topics = index.topic_index.len();
        let total_size = index
            .documents
            .values()
            .map(|doc| doc.content.len())
            .sum::<usize>();

        // Calculate date range
        let dates: Vec<&String> = index.date_index.keys().collect();
        let oldest_date = dates.first().map(|s| s.as_str());
        let newest_date = dates.last().map(|s| s.as_str());

        Ok(MemoryStats {
            total_documents,
            total_topics,
            total_size_bytes: total_size,
            oldest_date: oldest_date.map(|s| s.to_string()),
            newest_date: newest_date.map(|s| s.to_string()),
        })
    }

    /// Get all available topics
    pub fn get_all_topics(&self) -> Vec<String> {
        let index = self.search_index.read().unwrap();
        index.topic_index.keys().cloned().collect()
    }

    /// Get search performance metrics
    pub fn get_search_performance_metrics(&self) -> HashMap<String, usize> {
        let cache = self.search_cache.cache.read().unwrap();
        let inverted_index = self.inverted_index.word_to_docs.read().unwrap();

        let mut metrics = HashMap::new();
        metrics.insert("cache_size".to_string(), cache.len());
        metrics.insert("indexed_words".to_string(), inverted_index.len());
        metrics.insert(
            "total_word_document_mappings".to_string(),
            inverted_index.values().map(|set| set.len()).sum(),
        );

        metrics
    }

    /// Clear all caches (useful for testing or memory management)
    pub fn clear_all_caches(&self) {
        self.search_cache.clear();
        info!("Cleared all memory service caches");
    }

    /// Warm up cache with common queries
    pub async fn warmup_cache(&self, common_queries: Vec<String>) -> AppResult<()> {
        let query_count = common_queries.len();
        for query in common_queries {
            let _ = self.search_memory(&query, 5).await;
        }
        info!("Warmed up cache with {} queries", query_count);
        Ok(())
    }

    /// Get documents by date range
    pub async fn get_documents_by_date_range(
        &self,
        start_date: &str,
        end_date: &str,
    ) -> AppResult<Vec<MemoryDocument>> {
        let index = self.search_index.read().unwrap();
        let mut documents = Vec::new();

        for (date, doc_ids) in &index.date_index {
            if date.as_str() >= start_date && date.as_str() <= end_date {
                for doc_id in doc_ids {
                    if let Some(doc) = index.documents.get(doc_id) {
                        documents.push(doc.clone());
                    }
                }
            }
        }

        // Sort by creation date
        documents.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        Ok(documents)
    }

    /// Update document content incrementally (optimized version)
    pub async fn update_document_content(&self, doc_id: &str, new_content: &str) -> AppResult<()> {
        let mut index = self.search_index.write().unwrap();

        if let Some(document) = index.documents.get_mut(doc_id) {
            // Store old content for index update
            let old_content = document.content.clone();

            // Update document content
            document.content = new_content.to_string();

            // Write updated content to file
            fs::write(&document.file_path, new_content)?;

            // Incrementally update inverted index instead of full rebuild
            self.inverted_index
                .update_document(doc_id, new_content, &old_content);

            // Clear relevant cache entries
            self.search_cache.clear();

            info!("Updated document content: {}", doc_id);
            Ok(())
        } else {
            Err(AppError::NotFound)
        }
    }

    /// Update document metadata
    pub async fn update_document_metadata(
        &self,
        doc_id: &str,
        new_metadata: MemoryMetadata,
    ) -> AppResult<()> {
        let mut index = self.search_index.write().unwrap();

        if let Some(document) = index.documents.get_mut(doc_id) {
            // Store old content for index update
            let old_content = document.content.clone();
            let updated_doc = document.clone();

            // Drop the mutable reference before we modify the index
            drop(index);

            // Update metadata
            let mut updated_doc = updated_doc;
            updated_doc.metadata = new_metadata.clone();

            // Recreate document content
            let content_str = self.create_document_content(
                &new_metadata,
                "Updated document", // Placeholder - in real implementation, extract from existing content
                "",
            )?;

            updated_doc.content = content_str.clone();

            let new_content = updated_doc.content.clone();

            // Write updated content to file
            fs::write(&updated_doc.file_path, content_str)?;

            // Now update the index with the new document
            {
                let mut index = self.search_index.write().unwrap();
                // Remove old document from index and add updated one
                index.remove_document(doc_id);
                index.add_document(updated_doc);
            }

            // Incrementally update inverted index
            self.inverted_index
                .update_document(doc_id, &new_content, &old_content);

            // Clear search cache since document was updated
            self.search_cache.clear();

            Ok(())
        } else {
            Err(AppError::NotFound)
        }
    }

    /// Force rebuild of entire search index (maintenance operation)
    pub async fn rebuild_search_index(&self) -> AppResult<()> {
        info!("Starting full search index rebuild");

        let documents = {
            let index = self.search_index.read().unwrap();
            index.documents.clone()
        };

        // Clear and rebuild inverted index
        self.inverted_index.force_rebuild(&documents);

        info!("Search index rebuild completed");
        Ok(())
    }

    /// Get index statistics for monitoring
    pub async fn get_index_statistics(&self) -> AppResult<IndexStatistics> {
        let index = self.search_index.read().unwrap();
        let pending_updates = self.inverted_index.get_pending_updates_count();
        let last_rebuild = {
            let rebuild_time = self.inverted_index.last_rebuild.read().unwrap();
            *rebuild_time
        };

        Ok(IndexStatistics {
            total_documents: index.documents.len(),
            total_topics: index.topic_index.len(),
            total_date_entries: index.date_index.len(),
            pending_index_updates: pending_updates,
            last_rebuild_time: last_rebuild,
            index_needs_rebuild: self.inverted_index.needs_rebuild(),
        })
    }

    /// Validate memory document integrity
    pub async fn validate_memory_documents(&self) -> AppResult<MemoryValidationReport> {
        let index = self.search_index.read().unwrap();
        let mut report = MemoryValidationReport {
            total_checked: 0,
            valid_documents: 0,
            invalid_documents: Vec::new(),
            missing_files: Vec::new(),
            corrupted_files: Vec::new(),
        };

        for (doc_id, document) in &index.documents {
            report.total_checked += 1;

            // Check if file exists
            if !document.file_path.exists() {
                report.missing_files.push(doc_id.clone());
                continue;
            }

            // Try to read and parse the file
            match fs::read_to_string(&document.file_path) {
                Ok(content) => match self.parse_document_content(&content) {
                    Ok(_) => report.valid_documents += 1,
                    Err(_) => report.corrupted_files.push(doc_id.clone()),
                },
                Err(_) => report.corrupted_files.push(doc_id.clone()),
            }
        }

        Ok(report)
    }

    /// Repair corrupted memory documents
    pub async fn repair_memory_documents(&self) -> AppResult<usize> {
        let validation_report = self.validate_memory_documents().await?;
        let mut repaired_count = 0;

        // Remove references to missing files
        {
            let mut index = self.search_index.write().unwrap();
            for doc_id in &validation_report.missing_files {
                index.remove_document(doc_id);
                repaired_count += 1;
            }
        }

        // Try to repair corrupted files by removing them from index
        {
            let mut index = self.search_index.write().unwrap();
            for doc_id in &validation_report.corrupted_files {
                if let Some(document) = index.documents.get(doc_id) {
                    // Try to remove the corrupted file
                    let _ = fs::remove_file(&document.file_path);
                }
                index.remove_document(doc_id);
                repaired_count += 1;
            }
        }

        Ok(repaired_count)
    }
}
