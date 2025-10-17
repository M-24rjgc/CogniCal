# Design Document

## Overview

Phase 1 为 CogniCal 引入“智能任务解析 + CoT 推理引擎”，在 Phase 0 的基础任务管理框架上补足自然语言解析、AI 增强字段、推理可视化与缓存回退能力。本设计将扩展前端任务创建体验、后端 Tauri 命令层与服务层，并更新本地 SQLite Schema，以确保在离线优先、隐私优先的架构中安全复用 DeepSeek API。

## Steering Document Alignment

### Technical Standards (tech.md)

- **前端技术栈**：延续 React + TypeScript + Tailwind + shadcn/ui，复用现有表单、对话框与 Zustand store，新增组件遵循可组合、类型安全原则。
- **后端架构**：在 Tauri 2 + Rust 服务层中实现 DeepSeek 客户端、CoT 引擎与缓存服务，遵循命令 → 服务 → 数据访问的分层结构。
- **AI 集成约束**：采用 reqwest 访问 DeepSeek API，封装 Prompt 模板，保留思维链数据以便可视化；控制网络超时、重试与 Token 消耗。

### Project Structure (structure.md)

- **Rust 目录**：在 `src-tauri/src/services/` 下新增 `ai_service.rs`、`cot_engine.rs`、`cache_service.rs`，并在 `commands/ai.rs` 中暴露解析命令；扩展 `db/models/task.rs` 与 `db/migrations.rs`。
- **前端目录**：在 `src/components/tasks/` 下新增 `TaskAiParsePanel.tsx`、`TaskCotViewer.tsx`，复用 `TaskFormDialog.tsx`；在 `hooks/useTaskForm.ts` 中串联 AI 解析；在 `services/tauriApi.ts` 与 `types/task.ts` 中扩展数据结构。

## Code Reuse Analysis

### Existing Components to Leverage

- **`TaskFormDialog` / `TaskForm`**：扩展现有表单以支持 AI 解析按钮、结果回填与状态提示。
- **`useTaskForm` Hook**：新增调用 AI 解析逻辑、字段映射与错误提示。
- **`taskStore` (Zustand)**：复用加载态、错误态管理，将 AI 字段写入 `Task` 实体并保留历史行为。
- **`tauriApi` 服务**：在现有命令封装中增加 `parseTask` 方法，同时共享错误映射与离线 mock 机制。
- **`commands/task.rs` 与 `TaskService`**：复用任务持久化接口，为 AI 解析后的字段写入提供合规入口。

### Integration Points

- **Tauri Commands**：新增 `tasks_parse_ai` 命令；`tasks_create` / `tasks_update` 在持久化前会接受 AI 增强字段。
- **SQLite 数据库**：更新 `tasks` 表以持久化 `planned_start_time`、`estimated_hours`、`task_type`、`tags`、`complexity_score`、`ai_suggested_start_time`、`focus_mode`, `efficiency_prediction` 等字段；建立 `ai_parse_cache` 表保存最近推理结果。
- **缓存与日志**：在 `utils/logger.rs` 中新增日志目标 `app::ai`，使 Phase 2 可读取缓存命中率、延迟指标。

## Architecture

整体架构分为前端交互、Tauri 命令层、服务层与数据层：

```mermaid
graph TD
  UI[TaskFormDialog
+TaskAiParsePanel
+TaskCotViewer] -->|invoke| TC[Tauri Command
(tasks_parse_ai)]
  TC -->|call| AS[AiService]
  AS -->|prompt| CE[CotEngine]
  AS -->|cache lookup| CS[CacheService]
  AS -->|persist| DB[(SQLite tasks
+ ai_parse_cache)]
  AS -->|log| LOG[Tracing Logger]
  TC -->|result DTO| UI
```

- **前端**：表单触发 AI 解析，展示加载、成功、失败与缓存命中状态；CoT Viewer 展示思维链步骤。
- **命令层**：纯粹进行输入验证、错误映射与异步任务调度。
- **服务层**：
  - `AiService`：封装 DeepSeek 调用、CoT 引擎、缓存策略，提供结构化结果。
  - `CotEngine`：拼装 Prompt、解析思维链、生成增强字段。
  - `CacheService`：按语义指纹缓存结果，提供 TTL（7 天）与相似度检查。
- **数据层**：`DbPool` 执行迁移、更新任务字段、读写缓存表。

### Modular Design Principles

- **Single File Responsibility**：AI 解析 UI、推理展示、缓存提示分别独立组件；Rust 服务各自处理 API 调用、推理解析、缓存管理。
- **Component Isolation**：`TaskAiParsePanel` 只处理请求与展示，不直接修改表单状态；状态更新由 `useTaskForm` 协调。
- **Service Layer Separation**：命令层无业务逻辑，业务集中在 `AiService`；数据库操作集中在 `repositories`。
- **Utility Modularity**：将 Prompt 模板与语义指纹算法放在 `src-tauri/src/utils/ai_prompt.rs`、`semantic.rs` 等独立文件。

## Components and Interfaces

### 前端组件

#### `TaskAiParsePanel`

- **Purpose**：提供自然语言输入、触发 AI 解析、展示状态与结果摘要。
- **Interfaces**：
  - `onApply(result: ParsedTaskPayload)` 回调写回表单。
  - `onFeedback(feedback: { helpful: boolean; message?: string })` 记录用户反馈。
- **Dependencies**：`useTaskForm`、`parseTask` API、`Button`、`Textarea`。
- **Reuses**：`ToastProvider` 反馈状态。

#### `TaskCotViewer`

- **Purpose**：展示链式推理步骤与最终结论，支持复制。
- **Interfaces**：`steps: CotStep[]`、`summary: string`。
- **Dependencies**：`Dialog`、`ScrollArea`、`Button`。
- **Reuses**：`cn` 工具、`Tooltip`。

#### `useTaskForm` 更新

- **Purpose**：新增 `triggerAiParse`、`applyAiResult` 方法；管理解析状态、错误、缓存标记。
- **Interfaces**：
  - `aiState: { status: 'idle'|'loading'|'success'|'error'; source: 'live'|'cache' }`
  - `triggerAiParse(input: string): Promise<void>`
- **Dependencies**：`tauriApi.parseTask`、`form` 实例。
- **Reuses**：原有表单验证。

### Tauri 命令 & 服务

#### `tasks_parse_ai` 命令

- **Purpose**：处理前端解析请求，返回结构化字段与思维链。
- **Interfaces**：`payload: { input: String, context?: TaskContext }`，返回 `ParsedTaskResponse`。
- **Dependencies**：`AiService`、`AppState`。
- **Reuses**：`CommandResult`、`CommandError`。

#### `AiService`

- **Purpose**：协调 DeepSeek API 调用、CoT 解析、缓存读写。
- **Interfaces**：`parse_task(input: &AiParseInput) -> Result<AiParseOutput, AppError>`。
- **Dependencies**：`CotEngine`、`CacheService`、`TaskRepository`（用于字段默认值）。
- **Reuses**：`DbPool`、`AppError`。

#### `CotEngine`

- **Purpose**：生成 Prompt、解析思维链、计算复杂度与建议时间。
- **Interfaces**：`run(&TaskContext, &str) -> Result<CotResult, CotError>`。
- **Dependencies**：`serde_json`、`chrono`、`utils::datetime`。

#### `CacheService`

- **Purpose**：维护相似度索引、TTL、缓存命中率日志。
- **Interfaces**：
  - `get(&SemanticKey) -> Option<AiParseOutput>`
  - `put(entry: CacheEntry)`
- **Dependencies**：`DbPool`、`semantic_hash` 工具。

## Data Models

### TypeScript 扩展

```
interface Task {
  ...existing fields
  plannedStartAt?: string;
  estimatedMinutes?: number;
  taskType?: 'work' | 'study' | 'life' | 'other';
  tags: string[];
  ai?: {
    complexityScore?: number;
    suggestedStartAt?: string;
    focusMode?: { pomodoros: number; recommendedSlots: string[] };
    efficiencyPrediction?: {
      expectedHours: number;
      confidence: number;
    };
    cotSteps?: CotStep[];
    source: 'live' | 'cache';
    generatedAt: string;
  };
}

interface ParsedTaskResponse {
  payload: Partial<TaskPayload>;
  ai: Required<Task['ai']>;
  missingFields: Array<keyof TaskPayload>;
}
```

### Rust 数据结构

```
pub struct AiParseInput {
    pub raw_input: String,
    pub timezone: String,
    pub user_preferences: Option<UserPreferences>,
}

pub struct AiParseOutput {
    pub title: String,
    pub description: String,
    pub priority: String,
    pub planned_start_time: Option<DateTime<Utc>>,
    pub due_at: Option<DateTime<Utc>>,
    pub estimated_hours: Option<f32>,
    pub task_type: Option<String>,
    pub tags: Vec<String>,
    pub complexity_score: i32,
    pub ai_suggested_start_time: Option<DateTime<Utc>>,
    pub focus_mode: Option<FocusModeRecommendation>,
    pub efficiency_prediction: Option<EfficiencyPrediction>,
    pub cot_steps: Vec<CotStep>,
    pub cot_summary: String,
    pub source: AiSource,
}
```

### 数据库 Schema 更新

```
ALTER TABLE tasks ADD COLUMN planned_start_time TEXT;
ALTER TABLE tasks ADD COLUMN estimated_hours REAL;
ALTER TABLE tasks ADD COLUMN task_type TEXT;
ALTER TABLE tasks ADD COLUMN tags TEXT; -- JSON array
ALTER TABLE tasks ADD COLUMN complexity_score INTEGER;
ALTER TABLE tasks ADD COLUMN ai_suggested_start_time TEXT;
ALTER TABLE tasks ADD COLUMN focus_mode TEXT; -- JSON
ALTER TABLE tasks ADD COLUMN efficiency_prediction TEXT; -- JSON

CREATE TABLE ai_parse_cache (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  semantic_hash TEXT NOT NULL UNIQUE,
  raw_input TEXT NOT NULL,
  output_json TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  expires_at TEXT NOT NULL,
  usage_count INTEGER NOT NULL DEFAULT 0
);
```

## Error Handling

### Error Scenarios

1. **DeepSeek API 失败（网络/配额）**
   - **Handling**：捕获后落回缓存（若可用）或返回 `AppError::Other("AI 解析失败")`；前端展示错误并允许手动填写。
   - **User Impact**：提示“AI 服务暂不可用，可稍后重试或改用手动输入”。

2. **缓存解析数据过期或损坏**
   - **Handling**：`CacheService` 校验 `expires_at` 与 JSON 解析，损坏时自动删除并记录日志；命中失败后回退至实时解析。
   - **User Impact**：无感知；若实时解析也失败，则同 Error1。

3. **数据库迁移失败**
   - **Handling**：迁移在应用启动时执行，失败记录在日志并阻止应用继续启动，提示用户检查文件系统权限。
   - **User Impact**：无法启动应用，提供错误弹窗与日志路径。

4. **前端字段回填冲突**（用户已手动填写与 AI 结果冲突）
   - **Handling**：`useTaskForm` 在回填前比对差异，使用对话框提示“是否覆盖”；支持逐项应用。
   - **User Impact**：明确选择覆盖或保留原值。

## Testing Strategy

### Unit Testing

- **前端**：
  - `useTaskForm` 针对 `triggerAiParse`、`applyAiResult` 的状态转换、错误处理。
  - `TaskAiParsePanel` 渲染状态（loading/success/error）、按钮禁用逻辑。
- **后端**：
  - `CotEngine` 对 Prompt 解析、复杂度评分、时间计算的纯函数测试。
  - `CacheService` 对命中、TTL、损坏数据清理的逻辑测试。

### Integration Testing

- **Tauri 命令**：
  - `tasks_parse_ai` 通过 mock DeepSeek HTTP server 验证正向流程、离线回退、缓存命中。
  - 任务 CRUD 搭配新增字段的序列化、反序列化与数据库写入。
- **前端-后端链路**：
  - 使用 `vitest` + `@tauri-apps/api/mocks` 模拟命令返回，确保表单与列表正确展示 AI 字段。

### End-to-End Testing

- **Playwright 场景**：
  1. 用户输入自然语言 → AI 解析成功 → 字段回填 → 保存任务 → 列表中展示复杂度与 AI 提示。
  2. AI 解析失败 → 显示错误 → 用户手动补全 → 保存任务。
  3. 缓存命中 → 显示“来自缓存”标记 → 用户查看 CoT 步骤。
- **桌面打包测试**：在 Windows/macOS 下验证离线模式回退、API Key 管理、任务导出包含新字段。
