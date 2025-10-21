/// Streaming support for AI responses
/// 
/// This module provides infrastructure for streaming AI responses to the UI.
/// Currently implements a buffered approach that can be extended to true streaming.

use serde::{Deserialize, Serialize};

/// A chunk of streaming response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    /// The text content of this chunk
    pub content: String,
    /// Whether this is the final chunk
    pub is_final: bool,
    /// Chunk sequence number
    pub sequence: usize,
    /// Optional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<StreamMetadata>,
}

/// Metadata for a stream chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamMetadata {
    /// Tokens generated so far
    pub tokens_generated: usize,
    /// Estimated completion percentage (0-100)
    pub completion_percent: u8,
}

/// Stream configuration
#[derive(Debug, Clone)]
pub struct StreamConfig {
    /// Minimum chunk size in characters
    pub min_chunk_size: usize,
    /// Maximum time to buffer before sending (ms)
    pub max_buffer_time_ms: u64,
    /// Whether to enable streaming
    pub enabled: bool,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            min_chunk_size: 50,
            max_buffer_time_ms: 100,
            enabled: false, // Disabled by default for now
        }
    }
}

/// Stream buffer for accumulating response chunks
pub struct StreamBuffer {
    buffer: String,
    sequence: usize,
    config: StreamConfig,
}

impl StreamBuffer {
    /// Create a new stream buffer
    pub fn new(config: StreamConfig) -> Self {
        Self {
            buffer: String::new(),
            sequence: 0,
            config,
        }
    }

    /// Add content to the buffer
    pub fn push(&mut self, content: &str) {
        self.buffer.push_str(content);
    }

    /// Check if buffer should be flushed
    pub fn should_flush(&self) -> bool {
        self.buffer.len() >= self.config.min_chunk_size
    }

    /// Flush the buffer and return a chunk
    pub fn flush(&mut self, is_final: bool) -> Option<StreamChunk> {
        if self.buffer.is_empty() && !is_final {
            return None;
        }

        let chunk = StreamChunk {
            content: self.buffer.clone(),
            is_final,
            sequence: self.sequence,
            metadata: None,
        };

        self.buffer.clear();
        self.sequence += 1;

        Some(chunk)
    }

    /// Get the current buffer content without flushing
    pub fn peek(&self) -> &str {
        &self.buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_buffer_basic() {
        let config = StreamConfig {
            min_chunk_size: 10,
            ..Default::default()
        };
        let mut buffer = StreamBuffer::new(config);

        buffer.push("Hello");
        assert!(!buffer.should_flush());

        buffer.push(" World!");
        assert!(buffer.should_flush());

        let chunk = buffer.flush(false).unwrap();
        assert_eq!(chunk.content, "Hello World!");
        assert_eq!(chunk.sequence, 0);
        assert!(!chunk.is_final);
    }

    #[test]
    fn test_stream_buffer_final() {
        let config = StreamConfig::default();
        let mut buffer = StreamBuffer::new(config);

        buffer.push("Final");
        let chunk = buffer.flush(true).unwrap();
        assert!(chunk.is_final);
    }
}
