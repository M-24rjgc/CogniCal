use cognical_app_lib::models::memory::{
    MemoryExportFormat, MemoryExportOptions, MemorySearchQuery,
};
use cognical_app_lib::services::memory_service::MemoryService;
use chrono::{Duration, Utc};
use std::fs;
use tempfile::tempdir;

async fn setup_test_memory_service() -> (MemoryService, tempfile::TempDir) {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let memory_dir = temp_dir.path().join("memory");
    
    let service = MemoryService::new(memory_dir).expect("Failed to create memory service");
    (service, temp_dir)
}

#[tokio::test]
async fn test_store_conversation_success() {
    let (service, _temp_dir) = setup_test_memory_service().await;
    
    let conversation_id = "test_conv_1";
    let user_message = "How do I create recurring tasks?";
    let ai_response = "You can create recurring tasks by setting up recurrence rules...";
    let topics = vec!["recurring tasks".to_string(), "task management".to_string()];
    
    let result = service
        .store_conversation(conversation_id, user_message, ai_response, topics)
        .await;
    
    assert!(result.is_ok());
    let doc_id = result.unwrap();
    assert!(!doc_id.is_empty());
    
    // Verify the document was stored
    let stats = service.get_memory_stats().expect("Failed to get stats");
    assert_eq!(stats.total_documents, 1);
    assert_eq!(stats.total_topics, 2);
}

#[tokio::test]
async fn test_search_memory_basic() {
    let (service, _temp_dir) = setup_test_memory_service().await;
    
    // Store some test conversations
    let _ = service
        .store_conversation(
            "conv1",
            "How to create tasks?",
            "You can create tasks by...",
            vec!["task management".to_string()],
        )
        .await;
    
    let _ = service
        .store_conversation(
            "conv2",
            "What about recurring tasks?",
            "Recurring tasks can be set up...",
            vec!["recurring tasks".to_string()],
        )
        .await;
    
    // Search for relevant content
    let result = service.search_memory("recurring tasks", 5).await;
    assert!(result.is_ok());
    
    let context = result.unwrap();
    assert!(!context.relevant_documents.is_empty());
    // The document about recurring tasks should be first (highest relevance)
    assert!(context.relevant_documents[0].metadata.summary.contains("recurring"));
}

#[tokio::test]
async fn test_search_memory_with_query() {
    let (service, _temp_dir) = setup_test_memory_service().await;
    
    // Store test conversations with different dates
    let now = Utc::now();
    let old_date = now - Duration::days(10);
    
    let _ = service
        .store_conversation(
            "recent_conv",
            "Recent task question",
            "Recent answer about tasks",
            vec!["task management".to_string()],
        )
        .await;
    
    // Search with date range
    let search_query = MemorySearchQuery {
        query: "task".to_string(),
        limit: 10,
        min_relevance_score: Some(0.1),
        date_range: Some((old_date, now + Duration::hours(1))),
        topics: Some(vec!["task management".to_string()]),
    };
    
    let result = service.search_memory_with_query(&search_query).await;
    assert!(result.is_ok());
    
    let context = result.unwrap();
    assert!(!context.relevant_documents.is_empty());
}

#[tokio::test]
async fn test_get_recent_context() {
    let (service, _temp_dir) = setup_test_memory_service().await;
    
    // Store multiple conversations
    for i in 1..=3 {
        let _ = service
            .store_conversation(
                &format!("conv{}", i),
                &format!("Question {}", i),
                &format!("Answer {}", i),
                vec!["general".to_string()],
            )
            .await;
    }
    
    let result = service.get_recent_context(30, 5).await;
    assert!(result.is_ok());
    
    let recent_docs = result.unwrap();
    assert_eq!(recent_docs.len(), 3);
    
    // Should be sorted by most recent first
    for i in 0..recent_docs.len() - 1 {
        assert!(recent_docs[i].created_at >= recent_docs[i + 1].created_at);
    }
}

#[tokio::test]
async fn test_semantic_search_with_token_limit() {
    let (service, _temp_dir) = setup_test_memory_service().await;
    
    // Store a conversation with substantial content
    let long_response = "This is a very long response about task management. ".repeat(50);
    let _ = service
        .store_conversation(
            "long_conv",
            "Tell me about task management",
            &long_response,
            vec!["task management".to_string()],
        )
        .await;
    
    // Search with token limit
    let result = service.semantic_search("task management", 5, Some(100)).await;
    assert!(result.is_ok());
    
    let context = result.unwrap();
    assert!(!context.relevant_documents.is_empty());
    // Context should be limited by token count
    assert!(context.total_context_length <= 400); // 100 tokens * 4 chars per token
}

#[tokio::test]
async fn test_export_memory_archive() {
    let (service, temp_dir) = setup_test_memory_service().await;
    
    // Store test conversations
    let _ = service
        .store_conversation(
            "export_test",
            "Test export question",
            "Test export answer",
            vec!["export".to_string()],
        )
        .await;
    
    // Export as archive
    let export_path = temp_dir.path().join("export");
    let export_options = MemoryExportOptions {
        output_path: export_path.clone(),
        include_metadata: true,
        date_range: None,
        format: MemoryExportFormat::Archive,
    };
    
    let result = service.export_memory_archive(&export_options).await;
    assert!(result.is_ok());
    
    // Verify export files exist
    assert!(export_path.exists());
    assert!(export_path.join("metadata.json").exists());
    assert!(export_path.join("export_info.json").exists());
}

#[tokio::test]
async fn test_export_as_json() {
    let (service, temp_dir) = setup_test_memory_service().await;
    
    let _ = service
        .store_conversation(
            "json_test",
            "JSON export test",
            "JSON export response",
            vec!["export".to_string()],
        )
        .await;
    
    let export_path = temp_dir.path().join("json_export");
    let export_options = MemoryExportOptions {
        output_path: export_path.clone(),
        include_metadata: false,
        date_range: None,
        format: MemoryExportFormat::Json,
    };
    
    let result = service.export_memory_archive(&export_options).await;
    assert!(result.is_ok());
    
    let json_file = export_path.join("memory_export.json");
    assert!(json_file.exists());
    
    // Verify JSON content
    let json_content = fs::read_to_string(json_file).expect("Failed to read JSON file");
    assert!(json_content.contains("json_test"));
}

#[tokio::test]
async fn test_export_as_markdown() {
    let (service, temp_dir) = setup_test_memory_service().await;
    
    let _ = service
        .store_conversation(
            "md_test",
            "Markdown export test",
            "Markdown export response",
            vec!["export".to_string()],
        )
        .await;
    
    let export_path = temp_dir.path().join("md_export");
    let export_options = MemoryExportOptions {
        output_path: export_path.clone(),
        include_metadata: false,
        date_range: None,
        format: MemoryExportFormat::Markdown,
    };
    
    let result = service.export_memory_archive(&export_options).await;
    assert!(result.is_ok());
    
    let md_file = export_path.join("memory_export.md");
    assert!(md_file.exists());
    
    // Verify Markdown content
    let md_content = fs::read_to_string(md_file).expect("Failed to read Markdown file");
    assert!(md_content.contains("# Memory Export"));
    assert!(md_content.contains("md_test"));
}

#[tokio::test]
async fn test_cleanup_old_memories() {
    let (service, _temp_dir) = setup_test_memory_service().await;
    
    // Store some conversations
    for i in 1..=3 {
        let _ = service
            .store_conversation(
                &format!("cleanup_test_{}", i),
                &format!("Question {}", i),
                &format!("Answer {}", i),
                vec!["cleanup".to_string()],
            )
            .await;
    }
    
    // Verify documents exist
    let stats_before = service.get_memory_stats().expect("Failed to get stats");
    assert_eq!(stats_before.total_documents, 3);
    
    // Clean up memories older than 0 days (should clean all)
    let result = service.cleanup_old_memories(0).await;
    assert!(result.is_ok());
    
    let cleaned_count = result.unwrap();
    assert_eq!(cleaned_count, 3);
    
    // Verify documents are gone
    let stats_after = service.get_memory_stats().expect("Failed to get stats");
    assert_eq!(stats_after.total_documents, 0);
}

#[tokio::test]
async fn test_get_related_documents() {
    let (service, _temp_dir) = setup_test_memory_service().await;
    
    // Store conversations with overlapping topics
    let doc1_id = service
        .store_conversation(
            "related_test_1",
            "Question about tasks",
            "Answer about tasks",
            vec!["task management".to_string(), "productivity".to_string()],
        )
        .await
        .expect("Failed to store conversation");
    
    let _ = service
        .store_conversation(
            "related_test_2",
            "Question about productivity",
            "Answer about productivity",
            vec!["productivity".to_string(), "efficiency".to_string()],
        )
        .await;
    
    let _ = service
        .store_conversation(
            "unrelated_test",
            "Question about weather",
            "Answer about weather",
            vec!["weather".to_string()],
        )
        .await;
    
    // Get related documents
    let result = service.get_related_documents(&doc1_id, 5).await;
    assert!(result.is_ok());
    
    let related_docs = result.unwrap();
    assert_eq!(related_docs.len(), 1); // Only one document shares topics
    assert!(related_docs[0].metadata.topics.contains(&"productivity".to_string()));
}

#[tokio::test]
async fn test_get_conversation_context() {
    let (service, _temp_dir) = setup_test_memory_service().await;
    
    let _ = service
        .store_conversation(
            "context_test",
            "How do I manage my tasks effectively?",
            "To manage tasks effectively, you should prioritize them...",
            vec!["task management".to_string(), "productivity".to_string()],
        )
        .await;
    
    let result = service.get_conversation_context("task management", 1000).await;
    assert!(result.is_ok());
    
    let context_text = result.unwrap();
    assert!(!context_text.is_empty());
    assert!(context_text.contains("## Relevant Memory Context"));
    assert!(context_text.contains("task management"));
}

#[tokio::test]
async fn test_validate_memory_documents() {
    let (service, _temp_dir) = setup_test_memory_service().await;
    
    // Store a valid document
    let _ = service
        .store_conversation(
            "validation_test",
            "Test validation",
            "Test response",
            vec!["validation".to_string()],
        )
        .await;
    
    let result = service.validate_memory_documents().await;
    assert!(result.is_ok());
    
    let report = result.unwrap();
    assert_eq!(report.total_checked, 1);
    assert_eq!(report.valid_documents, 1);
    assert!(report.missing_files.is_empty());
    assert!(report.corrupted_files.is_empty());
}

#[tokio::test]
async fn test_memory_stats() {
    let (service, _temp_dir) = setup_test_memory_service().await;
    
    // Initially empty
    let stats = service.get_memory_stats().expect("Failed to get stats");
    assert_eq!(stats.total_documents, 0);
    assert_eq!(stats.total_topics, 0);
    
    // Store some documents
    for i in 1..=3 {
        let _ = service
            .store_conversation(
                &format!("stats_test_{}", i),
                &format!("Question {}", i),
                &format!("Answer {}", i),
                vec!["stats".to_string(), format!("topic{}", i)],
            )
            .await;
    }
    
    let stats = service.get_memory_stats().expect("Failed to get stats");
    assert_eq!(stats.total_documents, 3);
    assert!(stats.total_topics >= 3); // At least 3 unique topics
    assert!(stats.total_size_bytes > 0);
}

#[tokio::test]
async fn test_relevance_scoring() {
    let (service, _temp_dir) = setup_test_memory_service().await;
    
    // Store documents with different relevance to query
    let _ = service
        .store_conversation(
            "high_relevance",
            "How to create recurring tasks daily?",
            "To create daily recurring tasks...",
            vec!["recurring tasks".to_string()],
        )
        .await;
    
    let _ = service
        .store_conversation(
            "medium_relevance",
            "Task management tips",
            "Here are some task management tips...",
            vec!["task management".to_string()],
        )
        .await;
    
    let _ = service
        .store_conversation(
            "low_relevance",
            "Weather forecast",
            "Today's weather will be...",
            vec!["weather".to_string()],
        )
        .await;
    
    let result = service.search_memory("recurring tasks", 10).await;
    assert!(result.is_ok());
    
    let context = result.unwrap();
    assert!(!context.relevant_documents.is_empty());
    
    // The document about recurring tasks should have the highest relevance
    let highest_score = context.relevant_documents[0].metadata.relevance_score;
    assert!(highest_score > 0.5); // Should be reasonably high
    
    // Documents should be sorted by relevance (descending)
    for i in 0..context.relevant_documents.len() - 1 {
        assert!(
            context.relevant_documents[i].metadata.relevance_score
                >= context.relevant_documents[i + 1].metadata.relevance_score
        );
    }
}