# Technical Architecture

## Technology Stack

### Frontend (UI Layer)

- **Framework**: React 18+ with TypeScript
- **Build Tool**: Vite 5+
- **UI Library**:
  - Tailwind CSS 3+ for styling
  - shadcn/ui for component library
  - Radix UI for accessible primitives
- **State Management**:
  - Zustand for global state
  - React Query (TanStack Query) for server/async state
- **Routing**: React Router v6
- **Date/Time**: Day.js or date-fns
- **Visualization**:
  - Recharts for charts and analytics
  - React Flow for task dependency graphs

### Backend (Tauri Layer)

- **Framework**: Tauri 2.x
- **Language**: Rust (stable)
- **Database**:
  - SQLite (via rusqlite or diesel-rs)
  - Local file-based storage
- **IPC**: Tauri Commands for Frontend-Backend communication
- **System Integration**:
  - Tauri Tray API for system tray
  - Tauri Notification API for desktop notifications
  - Tauri Dialog API for file system access
  - Tauri Window API for window management

### AI Integration

- **Primary LLM**: DeepSeek API (REST API)
- **HTTP Client**: reqwest (Rust) or axios (TypeScript)
- **CoT Engine**: Custom implementation in Rust
- **MCTS Algorithm**: Custom Rust implementation for task decomposition
- **Prompt Engineering**: Structured prompts with template system

### Development Tools

- **Package Manager**:
  - pnpm (frontend)
  - Cargo (Rust)
- **Testing**:
  - Vitest for unit/integration tests
  - Playwright for E2E tests
  - cargo test for Rust tests
- **Linting/Formatting**:
  - ESLint + Prettier (TypeScript/React)
  - Clippy + rustfmt (Rust)
- **Version Control**: Git with conventional commits

## System Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        Desktop UI Layer                      │
│  (React + TypeScript + Tailwind CSS + shadcn/ui)           │
│                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │ Task Manager │  │   Calendar   │  │  Analytics   │     │
│  │  Component   │  │  Component   │  │  Dashboard   │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
└─────────────────────────────────────────────────────────────┘
                            ↕ (Tauri IPC)
┌─────────────────────────────────────────────────────────────┐
│                     Tauri Core (Rust)                        │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐  │
│  │              Command Handlers Layer                   │  │
│  │  - Task CRUD Commands                                │  │
│  │  - Calendar Operations Commands                      │  │
│  │  - AI Integration Commands                           │  │
│  │  - Analytics Commands                                │  │
│  └──────────────────────────────────────────────────────┘  │
│                            ↕                                 │
│  ┌──────────────────────────────────────────────────────┐  │
│  │                 Business Logic Layer                  │  │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────────────┐   │  │
│  │  │   Task   │  │ Calendar │  │  CoT Reasoning   │   │  │
│  │  │  Engine  │  │  Engine  │  │     Engine       │   │  │
│  │  └──────────┘  └──────────┘  └──────────────────┘   │  │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────────────┐   │  │
│  │  │   MCTS   │  │  Habit   │  │   Analytics      │   │  │
│  │  │  Solver  │  │  Tracker │  │     Engine       │   │  │
│  │  └──────────┘  └──────────┘  └──────────────────┘   │  │
│  └──────────────────────────────────────────────────────┘  │
│                            ↕                                 │
│  ┌──────────────────────────────────────────────────────┐  │
│  │                   Data Access Layer                   │  │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────────────┐   │  │
│  │  │  SQLite  │  │  Cache   │  │   File System    │   │  │
│  │  │   DAL    │  │  Manager │  │     Manager      │   │  │
│  │  └──────────┘  └──────────┘  └──────────────────┘   │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                            ↕ (HTTPS)
┌─────────────────────────────────────────────────────────────┐
│                    External AI Services                      │
│  ┌──────────────┐  ┌──────────────┐                        │
│  │  DeepSeek    │  │   (Future:   │                        │
│  │     API      │  │ Local LLM)   │                        │
│  └──────────────┘  └──────────────┘                        │
└─────────────────────────────────────────────────────────────┘
```

### Component Interactions

1. **User Input Flow**:

   - User interacts with React UI
   - UI dispatches Tauri command via IPC
   - Rust command handler processes request
   - Business logic layer executes operations
   - Results returned to UI via IPC response

2. **AI Integration Flow**:

   - User submits natural language input
   - Frontend sends to Rust AI command handler
   - Rust constructs prompt and calls DeepSeek API
   - CoT Reasoning Engine processes AI response
   - Structured task data saved to SQLite
   - UI updated with new task

3. **Data Persistence Flow**:
   - All operations go through Data Access Layer
   - SQLite for structured data (tasks, calendar, analytics)
   - File system for exports and user preferences
   - Cache manager for performance optimization

## Data Architecture

### Database Schema (SQLite)

#### Tasks Table

```sql
CREATE TABLE tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    priority TEXT NOT NULL CHECK(priority IN ('high', 'medium', 'low')),
    status TEXT NOT NULL DEFAULT 'pending' CHECK(status IN ('pending', 'in_progress', 'completed', 'cancelled')),
    task_type TEXT NOT NULL CHECK(task_type IN ('work', 'study', 'life', 'other')),

    -- Time fields
    planned_start_time TEXT NOT NULL,
    deadline TEXT NOT NULL,
    estimated_hours REAL NOT NULL,
    actual_hours REAL,

    -- AI-enhanced fields
    complexity_score INTEGER CHECK(complexity_score BETWEEN 0 AND 10),
    ai_suggested_start_time TEXT,
    focus_mode_required BOOLEAN DEFAULT 0,
    predicted_efficiency REAL,

    -- Metadata
    tags TEXT, -- JSON array
    parent_task_id INTEGER,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at TEXT,

    FOREIGN KEY (parent_task_id) REFERENCES tasks(id) ON DELETE CASCADE
);
```

#### Task Dependencies Table

```sql
CREATE TABLE task_dependencies (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id INTEGER NOT NULL,
    depends_on_task_id INTEGER NOT NULL,
    dependency_type TEXT NOT NULL CHECK(dependency_type IN ('blocking', 'related', 'prerequisite')),
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,
    FOREIGN KEY (depends_on_task_id) REFERENCES tasks(id) ON DELETE CASCADE,
    UNIQUE(task_id, depends_on_task_id)
);
```

#### Calendar Events Table

```sql
CREATE TABLE calendar_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id INTEGER,
    title TEXT NOT NULL,
    start_time TEXT NOT NULL,
    end_time TEXT NOT NULL,
    event_type TEXT NOT NULL CHECK(event_type IN ('task', 'focus', 'break', 'meeting', 'personal')),
    is_all_day BOOLEAN DEFAULT 0,
    reminder_minutes INTEGER,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE SET NULL
);
```

#### User Habits Table

```sql
CREATE TABLE user_habits (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    habit_name TEXT NOT NULL,
    category TEXT NOT NULL CHECK(category IN ('work', 'health', 'learning', 'personal')),
    frequency TEXT NOT NULL CHECK(frequency IN ('daily', 'weekly', 'monthly')),
    target_count INTEGER NOT NULL,
    current_streak INTEGER DEFAULT 0,
    longest_streak INTEGER DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    is_active BOOLEAN DEFAULT 1
);
```

#### Habit Tracking Table

```sql
CREATE TABLE habit_tracking (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    habit_id INTEGER NOT NULL,
    completion_date TEXT NOT NULL,
    completion_status BOOLEAN DEFAULT 0,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (habit_id) REFERENCES user_habits(id) ON DELETE CASCADE,
    UNIQUE(habit_id, completion_date)
);
```

#### Analytics Table

```sql
CREATE TABLE analytics_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    snapshot_date TEXT NOT NULL UNIQUE,
    total_tasks_completed INTEGER DEFAULT 0,
    total_hours_worked REAL DEFAULT 0,
    productivity_score REAL,
    efficiency_rating REAL,
    work_life_balance_score REAL,
    peak_productivity_hours TEXT, -- JSON array
    task_completion_rate REAL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

#### AI Interaction Logs Table

```sql
CREATE TABLE ai_interaction_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    interaction_type TEXT NOT NULL CHECK(interaction_type IN ('task_creation', 'task_decomposition', 'scheduling', 'recommendation')),
    user_input TEXT NOT NULL,
    ai_response TEXT NOT NULL,
    cot_reasoning TEXT, -- JSON array of reasoning steps
    tokens_used INTEGER,
    response_time_ms INTEGER,
    was_helpful BOOLEAN,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

### Data Flow Patterns

1. **Create Task Flow**:

   ```
   User Input (NL)
   → Frontend Validation
   → Tauri Command (create_task)
   → AI Service (DeepSeek API)
   → CoT Processing
   → SQLite Insert
   → Calendar Auto-Schedule
   → UI Update
   ```

2. **Smart Scheduling Flow**:

   ```
   Trigger (New Task / Time Change)
   → Load All Pending Tasks
   → CoT Reasoning (Conflict Detection)
   → MCTS Optimization
   → Generate 3 Scheduling Options
   → User Selection
   → Update Calendar Events
   → UI Refresh
   ```

3. **Analytics Generation Flow**:
   ```
   Daily Cron Job
   → Query Completed Tasks
   → Calculate Metrics
   → AI Insight Generation
   → Save Analytics Snapshot
   → Cache Dashboard Data
   ```

## AI/CoT Implementation Details

### CoT Reasoning Engine Architecture

```rust
// Core CoT reasoning structure
pub struct CotReasoning {
    steps: Vec<ReasoningStep>,
    confidence: f32,
    final_conclusion: String,
}

pub struct ReasoningStep {
    step_number: usize,
    thought: String,
    intermediate_result: Option<String>,
}

// Example: Task complexity analysis
pub async fn analyze_task_complexity(task_description: &str) -> Result<ComplexityAnalysis> {
    let prompt = build_cot_prompt(
        "task_complexity",
        task_description,
        &[
            "1. Identify required skills and knowledge",
            "2. Estimate number of sub-steps",
            "3. Assess external dependencies",
            "4. Calculate complexity score (0-10)",
        ]
    );

    let ai_response = call_deepseek_api(prompt).await?;
    let parsed_cot = parse_cot_response(&ai_response)?;

    Ok(ComplexityAnalysis {
        score: parsed_cot.extract_score(),
        reasoning: parsed_cot.steps,
        recommendations: parsed_cot.extract_recommendations(),
    })
}
```

### MCTS Task Decomposition

```rust
// Monte Carlo Tree Search for task breakdown
pub struct MctsNode {
    task: TaskNode,
    children: Vec<MctsNode>,
    visits: u32,
    total_reward: f32,
}

pub async fn decompose_task_mcts(
    task: &Task,
    max_iterations: u32,
) -> Result<Vec<DecompositionStrategy>> {
    let root = MctsNode::new(task);

    for _ in 0..max_iterations {
        // 1. Selection: UCB1 algorithm
        let leaf = select_leaf(&root);

        // 2. Expansion: AI generates sub-tasks
        let ai_subtasks = call_deepseek_api(
            build_decomposition_prompt(&leaf.task)
        ).await?;

        // 3. Simulation: Evaluate feasibility
        let reward = simulate_strategy(&ai_subtasks)?;

        // 4. Backpropagation
        backpropagate(&leaf, reward);
    }

    // Return top 3 strategies
    let strategies = extract_top_strategies(&root, 3);
    Ok(strategies)
}
```

### Dual-Agent System

```rust
// Task Planning Agent
pub struct TaskPlanningAgent {
    cot_engine: CotEngine,
    scheduler: SmartScheduler,
    mcts_solver: MctsSolver,
}

impl TaskPlanningAgent {
    pub async fn plan_tasks(&self, tasks: Vec<Task>) -> Result<PlanningResult> {
        // Long-term goal decomposition
        let milestones = self.decompose_goals(&tasks).await?;

        // Resource allocation optimization
        let allocation = self.optimize_resources(&tasks).await?;

        // Risk assessment
        let risks = self.assess_risks(&tasks).await?;

        Ok(PlanningResult { milestones, allocation, risks })
    }
}

// Life Assistant Agent
pub struct LifeAssistantAgent {
    cot_engine: CotEngine,
    habit_tracker: HabitTracker,
}

impl LifeAssistantAgent {
    pub async fn analyze_wellbeing(&self, user_data: &UserData) -> Result<WellbeingReport> {
        // Work-life balance analysis
        let balance = self.calculate_work_life_balance(user_data).await?;

        // Habit tracking
        let habits = self.track_habits(user_data).await?;

        // Recommendations
        let recommendations = self.generate_recommendations(&balance, &habits).await?;

        Ok(WellbeingReport { balance, habits, recommendations })
    }
}
```

## Security & Privacy

### Data Protection

- **Local-Only Storage**: All sensitive data in SQLite (local file)
- **No Cloud Dependency**: Application fully functional offline
- **Encryption** (Future): Optional database encryption with user-provided key
- **API Key Security**: DeepSeek API key stored in OS keychain (Tauri Plugin)

### Network Security

- **HTTPS Only**: All external API calls use TLS
- **API Rate Limiting**: Client-side throttling to prevent abuse
- **Request Validation**: Input sanitization before API calls
- **Error Masking**: No sensitive data in error logs

## Performance Considerations

### Optimization Strategies

1. **Frontend Performance**:

   - React.lazy for code splitting
   - Virtual scrolling for large task lists (react-window)
   - Debounced search and filters
   - Optimistic UI updates

2. **Backend Performance**:

   - SQLite connection pooling
   - Indexed database queries
   - Async Rust for non-blocking operations
   - LRU cache for frequently accessed data

3. **AI Performance**:
   - Request batching where possible
   - Streaming responses for real-time feedback
   - Fallback to cached reasoning for offline mode
   - Token optimization in prompts

### Performance Targets

- **App Launch**: < 2 seconds
- **Task Creation (No AI)**: < 100ms
- **Task Creation (With AI)**: < 3 seconds
- **Calendar Rendering**: < 200ms
- **Analytics Dashboard Load**: < 500ms
- **Memory Footprint**: < 200MB (idle)

## Testing Strategy

### Test Pyramid

1. **Unit Tests** (70%):

   - Rust: `cargo test` for all business logic
   - TypeScript: Vitest for React components and utilities
   - Target: > 80% code coverage

2. **Integration Tests** (20%):

   - Tauri command integration tests
   - Database operations tests
   - AI service mock tests

3. **E2E Tests** (10%):
   - Playwright for critical user flows
   - Task creation to completion workflow
   - Calendar scheduling scenarios

### Test Coverage Goals

- Core business logic: 90%+
- UI components: 70%+
- Integration points: 85%+

## Deployment & Distribution

### Build Process

```bash
# Development build
pnpm tauri dev

# Production build (all platforms)
pnpm tauri build --target all

# Platform-specific builds
pnpm tauri build --target x86_64-pc-windows-msvc  # Windows
pnpm tauri build --target x86_64-apple-darwin      # macOS Intel
pnpm tauri build --target aarch64-apple-darwin     # macOS Apple Silicon
pnpm tauri build --target x86_64-unknown-linux-gnu # Linux
```

### Distribution Channels

- **GitHub Releases**: Primary distribution method
- **Direct Download**: From project website
- **Auto-Update**: Tauri updater for seamless updates

### Version Management

- **Semantic Versioning**: MAJOR.MINOR.PATCH
- **Changelog**: Conventional commits for auto-generation
- **Release Cadence**: Monthly minor releases, weekly patch releases

## Scalability Considerations

### Current Phase (MVP)

- Single-user desktop application
- Local SQLite database (10,000+ tasks supported)
- Single-threaded AI requests

### Future Scalability

- **Multi-user Support**: P2P sync via local network
- **Database Migration**: Consider IndexedDB for web version
- **Distributed AI**: Support local LLM inference
- **Plugin Ecosystem**: Modular architecture for extensions

## Monitoring & Observability

### Local Logging

- **Rust Logs**: `tracing` crate for structured logging
- **Frontend Logs**: Console in development, file in production
- **Log Rotation**: 7-day retention, max 100MB per file

### Error Tracking

- **Crash Reports**: Optional anonymous crash reporting
- **User Feedback**: In-app feedback form
- **Error Recovery**: Graceful degradation on failures

### Performance Monitoring

- **Metrics Collection**: Local performance metrics
- **Analytics**: Anonymized usage statistics (opt-in)
- **Benchmarking**: Automated performance regression tests
