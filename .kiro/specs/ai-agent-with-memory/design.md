# Design Document: AI Agent with Memory and Tool Calling

## Overview

This design document outlines the architecture for transforming the existing AI chat feature into an intelligent agent system with long-term memory and tool-calling capabilities. The system will integrate kb-mcp-server (a txtai-powered semantic memory backend) and implement a flexible tool framework that allows the AI to interact with task management and calendar features.

### Key Design Goals

1. **Semantic Memory**: Enable the AI to remember and retrieve relevant conversation context using semantic search
2. **Tool Integration**: Allow the AI to perform actions (create tasks, schedule events) through natural conversation
3. **Backward Compatibility**: Maintain existing chat functionality while adding new capabilities
4. **Graceful Degradation**: System should work even when memory or tools are unavailable
5. **Performance**: Keep response times under 2 seconds for typical interactions

## Architecture

### High-Level Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                        Frontend (React)                      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │  Chat UI     │  │  Task View   │  │ Calendar View│      │
│  └──────┬───────┘  └──────────────┘  └──────────────┘      │
│         │                                                     │
│  ┌──────▼──────────────────────────────────────────┐        │
│  │          Chat Store (Zustand)                   │        │
│  │  - Messages                                     │        │
│  │  - Tool Call Status                             │        │
│  └──────┬──────────────────────────────────────────┘        │
└─────────┼──────────────────────────────────────────────────┘
          │ Tauri IPC
┌─────────▼──────────────────────────────────────────────────┐
│                    Backend (Rust/Tauri)                     │
│                                                              │
│  ┌────────────────────────────────────────────────────┐    │
│  │              AI Agent Service                      │    │
│  │  ┌──────────────┐  ┌──────────────┐              │    │
│  │  │ Chat Handler │  │ Tool Executor│              │    │
│  │  └──────┬───────┘  └──────┬───────┘              │    │
│  │         │                  │                       │    │
│  │  ┌──────▼──────────────────▼───────┐             │    │
│  │  │    Context Builder               │             │    │
│  │  │  - Memory Context                │             │    │
│  │  │  - Tool Schemas                  │             │    │
│  │  │  - System Prompts                │             │    │
│  │  └──────┬───────────────────────────┘             │    │
│  └─────────┼─────────────────────────────────────────┘    │
│            │                                                │
│  ┌─────────▼──────────┐  ┌──────────────────────────┐    │
│  │  Memory Service    │  │   Tool Registry          │    │
│  │  - Store           │  │   - Task Tools           │    │
│  │  - Retrieve        │  │   - Calendar Tools       │    │
│  │  - Search          │  │   - Schema Validation    │    │
│  └─────────┬──────────┘  └──────────┬───────────────┘    │
│            │                         │                     │
│  ┌─────────▼──────────┐  ┌──────────▼───────────────┐    │
│  │  MCP Client        │  │   Tool Implementations   │    │
│  │  (kb-mcp-server)   │  │   - TaskService          │    │
│  └─────────┬──────────┘  │   - CalendarService      │    │
│            │              └──────────────────────────┘    │
└────────────┼─────────────────────────────────────────────┘
             │
    ┌────────▼────────┐
    │  MCP Server     │
    │  (Python)       │
    │  ┌───────────┐  │
    │  │  txtai    │  │
    │  │  Knowledge│  │
    │  │  Base     │  │
    │  └───────────┘  │
    └─────────────────┘
```

### Component Interaction Flow

```
User Message → Chat UI → Chat Store → Tauri Command
                                           ↓
                                    AI Agent Service
                                           ↓
                              ┌────────────┴────────────┐
                              ↓                         ↓
                        Memory Service            Tool Registry
                              ↓                         ↓
                        MCP Client                Tool Schemas
                              ↓                         ↓
                        Retrieve Context          Build Prompt
                              ↓                         ↓
                              └────────────┬────────────┘
                                           ↓
                                    DeepSeek API
                                    (with tools)
                                           ↓
                              ┌────────────┴────────────┐
                              ↓                         ↓
                        Text Response            Tool Calls
                              ↓                         ↓
                        Return to UI            Tool Executor
                                                        ↓
                                                Execute Tools
                                                        ↓
                                                Return Results
                                                        ↓
                                                DeepSeek API
                                                (with results)
                                                        ↓
                                                Final Response
                                                        ↓
                                                   Store Memory
                                                        ↓
                                                   Return to UI
```

## Components and Interfaces

### 1. Memory Service (Rust)

**Location**: `src-tauri/src/services/memory_service.rs`

**Responsibilities**:
- Manage connection to MCP server process
- Store conversation turns in the knowledge base
- Retrieve relevant context using semantic search
- Handle memory persistence and archival

**Key Structures**:

```rust
pub struct MemoryService {
    mcp_client: Arc<RwLock<Option<McpClient>>>,
    db_pool: DbPool,
    config: MemoryConfig,
}

pub struct MemoryConfig {
    pub kb_path: PathBuf,
    pub max_context_entries: usize,
    pub search_limit: usize,
    pub enable_graph: bool,
}

pub struct MemoryEntry {
    pub id: String,
    pub conversation_id: String,
    pub user_message: String,
    pub assistant_message: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

pub struct MemoryContext {
    pub entries: Vec<MemoryEntry>,
    pub relevance_scores: Vec<f32>,
    pub total_tokens: usize,
}
```

**Key Methods**:

```rust
impl MemoryService {
    pub fn new(db_pool: DbPool, config: MemoryConfig) -> AppResult<Self>;
    
    pub async fn initialize(&self) -> AppResult<()>;
    
    pub async fn store_conversation(
        &self,
        conversation_id: &str,
        user_message: &str,
        assistant_message: &str,
        metadata: Option<HashMap<String, String>>,
    ) -> AppResult<String>;
    
    pub async fn retrieve_context(
        &self,
        query: &str,
        limit: usize,
    ) -> AppResult<MemoryContext>;
    
    pub async fn search_conversations(
        &self,
        query: &str,
        filters: Option<HashMap<String, String>>,
    ) -> AppResult<Vec<MemoryEntry>>;
    
    pub async fn export_knowledge_base(&self, path: &Path) -> AppResult<()>;
    
    pub async fn clear_conversation(&self, conversation_id: &str) -> AppResult<()>;
    
    pub fn is_available(&self) -> bool;
}
```

### 2. MCP Client (Rust)

**Location**: `src-tauri/src/services/mcp_client.rs`

**Responsibilities**:
- Spawn and manage kb-mcp-server process
- Communicate with MCP server via stdio
- Handle MCP protocol messages
- Manage connection lifecycle

**Key Structures**:

```rust
pub struct McpClient {
    process: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    request_id: AtomicU64,
}

pub struct McpRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    pub params: serde_json::Value,
}

pub struct McpResponse {
    pub jsonrpc: String,
    pub id: u64,
    pub result: Option<serde_json::Value>,
    pub error: Option<McpError>,
}
```

**Key Methods**:

```rust
impl McpClient {
    pub fn spawn(kb_path: &Path) -> AppResult<Self>;
    
    pub async fn call(
        &mut self,
        method: &str,
        params: serde_json::Value,
    ) -> AppResult<serde_json::Value>;
    
    pub async fn search(
        &mut self,
        query: &str,
        limit: usize,
    ) -> AppResult<Vec<SearchResult>>;
    
    pub async fn add_document(
        &mut self,
        text: &str,
        metadata: HashMap<String, String>,
    ) -> AppResult<String>;
    
    pub async fn shutdown(&mut self) -> AppResult<()>;
}
```

### 3. Tool Registry (Rust)

**Location**: `src-tauri/src/services/tool_registry.rs`

**Responsibilities**:
- Register available tools with schemas
- Validate tool parameters
- Provide tool definitions for AI context
- Route tool calls to implementations

**Key Structures**:

```rust
pub struct ToolRegistry {
    tools: HashMap<String, ToolDefinition>,
}

pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value, // JSON Schema
    pub handler: ToolHandler,
}

pub type ToolHandler = Arc<dyn Fn(serde_json::Value) -> BoxFuture<'static, AppResult<serde_json::Value>> + Send + Sync>;

pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

pub struct ToolResult {
    pub tool_call_id: String,
    pub result: serde_json::Value,
    pub error: Option<String>,
}
```

**Key Methods**:

```rust
impl ToolRegistry {
    pub fn new() -> Self;
    
    pub fn register_tool(
        &mut self,
        name: String,
        description: String,
        parameters: serde_json::Value,
        handler: ToolHandler,
    ) -> AppResult<()>;
    
    pub fn get_tool_schemas(&self) -> Vec<serde_json::Value>;
    
    pub fn validate_tool_call(&self, tool_call: &ToolCall) -> AppResult<()>;
    
    pub async fn execute_tool(
        &self,
        tool_call: ToolCall,
    ) -> AppResult<ToolResult>;
    
    pub fn has_tool(&self, name: &str) -> bool;
}
```

### 4. AI Agent Service (Rust)

**Location**: `src-tauri/src/services/ai_agent_service.rs`

**Responsibilities**:
- Orchestrate chat interactions with memory and tools
- Build context from memory and tool schemas
- Handle multi-turn tool calling
- Manage conversation state

**Key Structures**:

```rust
pub struct AiAgentService {
    ai_service: Arc<AiService>,
    memory_service: Arc<MemoryService>,
    tool_registry: Arc<ToolRegistry>,
    conversation_manager: Arc<RwLock<ConversationManager>>,
}

pub struct AgentContext {
    pub conversation_id: String,
    pub memory_context: Option<MemoryContext>,
    pub available_tools: Vec<serde_json::Value>,
    pub system_prompt: String,
}

pub struct AgentResponse {
    pub message: String,
    pub tool_calls: Vec<ToolCall>,
    pub memory_stored: bool,
    pub metadata: AgentMetadata,
}

pub struct AgentMetadata {
    pub tokens_used: HashMap<String, u64>,
    pub latency_ms: u128,
    pub memory_entries_used: usize,
    pub tools_executed: Vec<String>,
}
```

**Key Methods**:

```rust
impl AiAgentService {
    pub fn new(
        ai_service: Arc<AiService>,
        memory_service: Arc<MemoryService>,
        tool_registry: Arc<ToolRegistry>,
    ) -> Self;
    
    pub async fn chat(
        &self,
        conversation_id: &str,
        message: &str,
    ) -> AppResult<AgentResponse>;
    
    async fn build_context(
        &self,
        conversation_id: &str,
        message: &str,
    ) -> AppResult<AgentContext>;
    
    async fn execute_tool_calls(
        &self,
        tool_calls: Vec<ToolCall>,
    ) -> AppResult<Vec<ToolResult>>;
    
    async fn store_conversation(
        &self,
        conversation_id: &str,
        user_message: &str,
        assistant_message: &str,
        metadata: AgentMetadata,
    ) -> AppResult<()>;
}
```

### 5. Tool Implementations

**Task Management Tools** (`src-tauri/src/tools/task_tools.rs`):

```rust
pub async fn create_task_tool(
    app_state: Arc<AppState>,
    args: serde_json::Value,
) -> AppResult<serde_json::Value>;

pub async fn update_task_tool(
    app_state: Arc<AppState>,
    args: serde_json::Value,
) -> AppResult<serde_json::Value>;

pub async fn list_tasks_tool(
    app_state: Arc<AppState>,
    args: serde_json::Value,
) -> AppResult<serde_json::Value>;

pub async fn search_tasks_tool(
    app_state: Arc<AppState>,
    args: serde_json::Value,
) -> AppResult<serde_json::Value>;

pub async fn delete_task_tool(
    app_state: Arc<AppState>,
    args: serde_json::Value,
) -> AppResult<serde_json::Value>;
```

**Calendar Tools** (`src-tauri/src/tools/calendar_tools.rs`):

```rust
pub async fn get_calendar_events_tool(
    app_state: Arc<AppState>,
    args: serde_json::Value,
) -> AppResult<serde_json::Value>;

pub async fn create_calendar_event_tool(
    app_state: Arc<AppState>,
    args: serde_json::Value,
) -> AppResult<serde_json::Value>;

pub async fn update_calendar_event_tool(
    app_state: Arc<AppState>,
    args: serde_json::Value,
) -> AppResult<serde_json::Value>;
```

### 6. Frontend Updates

**Chat Store Enhancement** (`src/stores/chatStore.ts`):

```typescript
interface ChatStoreState {
  messages: ChatMessage[];
  toolCalls: ToolCallStatus[];
  conversationId: string;
  isLoading: boolean;
  error: AppError | null;
  
  sendMessage: (content: string) => Promise<void>;
  clearMessages: () => void;
  exportConversation: () => Promise<void>;
}

interface ToolCallStatus {
  id: string;
  toolName: string;
  status: 'pending' | 'executing' | 'completed' | 'failed';
  result?: any;
  error?: string;
}
```

**Enhanced Chat UI** (`src/pages/Chat.tsx`):
- Display tool execution status
- Show memory context indicators
- Add export conversation button
- Display token usage and performance metrics

## Data Models

### Memory Entry Schema (txtai)

```json
{
  "id": "uuid",
  "conversation_id": "string",
  "user_message": "string",
  "assistant_message": "string",
  "timestamp": "ISO8601",
  "metadata": {
    "user_id": "string",
    "session_id": "string",
    "tools_used": ["string"],
    "sentiment": "string",
    "topics": ["string"]
  }
}
```

### Tool Schema Format (OpenAI Function Calling)

```json
{
  "type": "function",
  "function": {
    "name": "create_task",
    "description": "Create a new task with the specified details",
    "parameters": {
      "type": "object",
      "properties": {
        "title": {
          "type": "string",
          "description": "The title of the task"
        },
        "description": {
          "type": "string",
          "description": "Detailed description of the task"
        },
        "priority": {
          "type": "string",
          "enum": ["low", "medium", "high"],
          "description": "Priority level of the task"
        },
        "due_date": {
          "type": "string",
          "format": "date",
          "description": "Due date in YYYY-MM-DD format"
        },
        "tags": {
          "type": "array",
          "items": {"type": "string"},
          "description": "Tags to categorize the task"
        }
      },
      "required": ["title"]
    }
  }
}
```

## Error Handling

### Error Categories

1. **Memory Errors**:
   - MCP server unavailable → Fallback to stateless mode
   - Search timeout → Return empty context
   - Storage failure → Log error, continue without storing

2. **Tool Errors**:
   - Invalid parameters → Return validation error to AI
   - Execution failure → Return error message to AI
   - Timeout → Cancel and report to AI

3. **AI Errors**:
   - API failure → Return error to user
   - Invalid response → Retry with simplified context
   - Rate limit → Queue and retry with backoff

### Error Response Format

```rust
pub enum AgentError {
    MemoryUnavailable(String),
    ToolExecutionFailed { tool_name: String, reason: String },
    AiProviderError(AppError),
    InvalidToolCall { tool_name: String, validation_error: String },
    ContextTooLarge { tokens: usize, limit: usize },
}
```

## Testing Strategy

### Unit Tests

1. **Memory Service Tests**:
   - Test MCP client communication
   - Test semantic search accuracy
   - Test conversation storage and retrieval
   - Test error handling when MCP unavailable

2. **Tool Registry Tests**:
   - Test tool registration
   - Test parameter validation
   - Test tool execution
   - Test error propagation

3. **AI Agent Service Tests**:
   - Test context building
   - Test tool call parsing
   - Test multi-turn conversations
   - Test memory integration

### Integration Tests

1. **End-to-End Chat Flow**:
   - User message → Memory retrieval → AI response → Memory storage
   - User message → Tool call → Tool execution → AI response with result

2. **Tool Integration Tests**:
   - Create task via chat
   - List tasks via chat
   - Schedule event via chat
   - Handle tool errors gracefully

3. **Memory Integration Tests**:
   - Store and retrieve conversations
   - Semantic search accuracy
   - Context relevance
   - Performance under load

### Performance Tests

1. **Latency Benchmarks**:
   - Memory retrieval < 500ms
   - Tool execution < 200ms
   - End-to-end response < 2s

2. **Scalability Tests**:
   - 10,000 conversation turns
   - 100 concurrent tool calls
   - Memory search with large knowledge base

## Configuration

### Environment Variables

```bash
# MCP Server Configuration
COGNICAL_KB_PATH=/path/to/knowledge_base
COGNICAL_MCP_ENABLED=true
COGNICAL_MCP_MAX_CONTEXT=5

# Tool Configuration
COGNICAL_TOOLS_ENABLED=true
COGNICAL_TOOLS_TIMEOUT_MS=5000

# Memory Configuration
COGNICAL_MEMORY_SEARCH_LIMIT=10
COGNICAL_MEMORY_ENABLE_GRAPH=true
```

### Database Schema

```sql
-- Memory configuration table
CREATE TABLE IF NOT EXISTS memory_config (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Conversation metadata table
CREATE TABLE IF NOT EXISTS conversations (
    id TEXT PRIMARY KEY,
    user_id TEXT,
    started_at TEXT NOT NULL,
    last_message_at TEXT NOT NULL,
    message_count INTEGER DEFAULT 0,
    archived BOOLEAN DEFAULT FALSE
);
```

## Deployment Considerations

### MCP Server Setup

1. **Installation**:
   ```bash
   pip install kb-mcp-server
   ```

2. **Knowledge Base Initialization**:
   ```bash
   kb-build --input ./docs --config config.yml --export kb.tar.gz
   ```

3. **Server Configuration**:
   - Embed kb-mcp-server binary with application
   - Auto-start on application launch
   - Graceful shutdown on application exit

### Resource Requirements

- **Memory**: +200MB for MCP server and knowledge base
- **Disk**: ~50MB per 10,000 conversation turns
- **CPU**: Minimal overhead, spikes during semantic search

### Migration Path

1. **Phase 1**: Deploy memory service (read-only mode)
2. **Phase 2**: Enable conversation storage
3. **Phase 3**: Deploy tool registry and basic tools
4. **Phase 4**: Enable full agent capabilities

## Security Considerations

1. **Tool Execution**:
   - Validate all tool parameters
   - Implement rate limiting
   - Log all tool executions
   - Require user confirmation for destructive actions

2. **Memory Storage**:
   - Encrypt sensitive conversation data
   - Implement data retention policies
   - Allow users to delete their data
   - Comply with privacy regulations

3. **MCP Communication**:
   - Use local stdio communication (no network exposure)
   - Validate all MCP responses
   - Implement timeout protections

## Future Enhancements

1. **Advanced Memory Features**:
   - User preference learning
   - Automatic topic extraction
   - Conversation summarization
   - Cross-conversation insights

2. **Extended Tool Capabilities**:
   - File operations
   - Email integration
   - External API calls
   - Custom user-defined tools

3. **Multi-Agent Collaboration**:
   - Specialized agents for different domains
   - Agent-to-agent communication
   - Hierarchical task delegation

4. **Personalization**:
   - User-specific system prompts
   - Adaptive response styles
   - Context-aware suggestions
