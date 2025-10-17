# Design Document - Phase 0 Foundation

## Overview

Phase 0 Foundation 的目标是为 CogniCal 提供一个生产可用的 Tauri 2 桌面应用骨架，集成 React 18 + TypeScript 前端和 Rust 后端，交付稳定的任务管理 CRUD 功能。本设计文档将对项目初始化、前后端分层、数据库建模、Tauri Commands、UI 组件与状态管理、错误处理、测试策略等进行详细设计，确保满足 `requirements.md` 中定义的 10 项需求并为后续阶段的智能化能力奠定基础。

## Steering Document Alignment

### Technical Standards (tech.md)

- 遵循 `tech.md` 中推荐的技术栈：Tauri 2 + Rust 后端、React 18 + TypeScript + Vite 前端、Zustand 状态管理、Tailwind CSS + shadcn/ui UI 体系、SQLite 数据库。
- 按照 `tech.md` 提供的数据架构和命名规范实现 `tasks`、`users` 基础表，并为未来的任务依赖、AI 日志等拓展字段留出扩展位。
- 结合 `tech.md` 数据流范式，在设计中明确前端 → Tauri Command → Service → DB 的调用链，保证后续 AI 能力接入时只需扩展 Service 层。

### Project Structure (structure.md)

- 目录结构严格遵循 `structure.md`，前端位于 `src/`，后端位于 `src-tauri/src/`，并按照 commands / services / db / utils 分层。
- 前端组件按照 `structure.md` 约定拆分：`components/`、`pages/`、`hooks/`、`stores/`、`types/`、`services/`、`utils/`，保证单一职责与可复用性。
- 预置 `tests/`、`docs/`、`.vscode/` 等目录，为 Phase 0 完成后续持续集成、文档与测试奠定基础。

## Code Reuse Analysis

项目为全新启动，当前仓库尚无可复用代码，但将依赖成熟工具链与模板：

### Existing Components to Leverage

- **Tauri CLI (`pnpm dlx create-tauri-app`)**: 快速生成 Tauri 2 + React + Vite 项目骨架，减少样板代码。
- **shadcn/ui CLI (`pnpm dlx shadcn-ui@latest init`)**: 生成标准化 Tailwind + Radix 组件，保证 UI 一致性。
- **Lucide React Icons**: 直接复用图标组件库构建导航、按钮等图标。
- **Zustand Devtools Middleware**: 便于调试全局状态。

### Integration Points

- **DeepSeek API**: Phase 0 暂不调用，仅在 `tauri.conf.json` 与 `src-tauri/src/utils/` 中预留接口，Phase 1 可直接接入。
- **SQLite**: 通过 `rusqlite` 在后端建立数据库连接，配合 `tauri::api::path::app_data_dir` 获取用户本地数据目录。
- **Tailwind + PostCSS + Prettier**: 和 Vite 集成，确保 CSS 与代码风格一致。

## Architecture

整体架构采用前后端分层 + 命令式 IPC 调用：

```mermaid
graph TD
    subgraph React UI (src/)
        A[AppShell]
        B[Pages (Dashboard/Tasks/Settings)]
        C[Components (TaskList/TaskForm)]
        D[Zustand Stores]
        E[Tauri Service Wrapper]
    end

    subgraph Tauri Commands (src-tauri/src/commands)
        F[task.rs]
        G[app.rs (init)]
    end

    subgraph Services (src-tauri/src/services)
        H[TaskService]
        I[BootstrapService]
    end

    subgraph Database (src-tauri/src/db)
        J[connection.rs]
        K[schema.sql]
        L[repositories]
    end

    A --> B --> C
    B --> D
    D --> E
    E --> F
    F --> H
    H --> J
    J --> K
```

### Modular Design Principles

- **Single File Responsibility**: 每个 React 组件只关注展示/交互，Zustand store 专注状态管理，Rust service 专注业务逻辑。
- **Component Isolation**: `TaskList`, `TaskTable`, `TaskFormDialog` 等组件解耦，复用 shadcn/ui 的 Button、Dialog、Form 控件。
- **Service Layer Separation**: `TaskService` 管理任务 CRUD；`BootstrapService` 负责项目初始化和数据库迁移；未来可新增 `NotificationService`、`TrayService`。
- **Utility Modularity**: `datetime.rs`, `validation.rs`, `error.rs` 等工具统一封装，前端 `validators.ts`, `formatters.ts` 与之对应。

## Components and Interfaces

### Component 1: AppShell (React)

- **Purpose:** 提供 sidebar + header + main content 的主布局，负责路由切换、主题切换。
- **Interfaces:**
  - Props: none
  - Hooks: `useTheme()`, `useSidebar()`
- **Dependencies:** `@radix-ui/react-slot`, `lucide-react`, `ThemeProvider`, `Sidebar` 组件。
- **Reuses:** shadcn/ui 的 Layout, Button, DropdownMenu; Tailwind utility classes。

### Component 2: TaskListPage (React)

- **Purpose:** 展示任务列表，提供过滤、排序、快速操作入口。
- **Interfaces:**
  - Reads store: `taskStore.tasks`, `taskStore.loading`
  - Actions: `taskStore.fetchTasks()`, `taskStore.deleteTask(id)`
- **Dependencies:** `TaskTable`, `TaskToolbar`, `TaskFormDialog`, `useTasks()` hook。
- **Reuses:** `TauriApiService.getAllTasks()`，shadcn/ui Table, Badge 组件。

### Component 3: TaskStore (Zustand)

- **Purpose:** 管理任务状态、加载状态、错误状态，集中封装前端 CRUD。
- **Interfaces:**
  - State: `tasks: Task[]`, `selectedTaskId`, `loading`, `error`
  - Actions: `fetchTasks`, `createTask`, `updateTask`, `deleteTask`, `selectTask`
- **Dependencies:** `tauriApi.ts` 封装的 IPC 调用，`taskSchema` 验证函数。
- **Reuses:** 将来 Phase 1 任务分解时可直接扩展 `tasks` 状态字段。

### Component 4: TaskCommandHandlers (Rust)

- **Purpose:** 暴露给前端的 Tauri Commands，实现任务 CRUD。
- **Interfaces:**
  - `#[tauri::command] async fn create_task(payload: TaskPayload) -> Result<Task>`
  - `get_all_tasks`, `get_task_by_id`, `update_task`, `delete_task`
- **Dependencies:** `TaskService`, `AppState`（包含 DB connection pool）
- **Reuses:** `TaskRepository` 数据访问模块；错误类型 `AppError`。

### Component 5: TaskService (Rust)

- **Purpose:** 承载业务逻辑，处理验证、DTO ↔ 实体转换、事务。
- **Interfaces:**
  - `pub async fn create(&self, payload: CreateTaskInput) -> AppResult<TaskEntity>`
  - 同步的 `get_all`, `get_by_id`, `update`, `delete`
- **Dependencies:** `TaskRepository`, `Validator`, `chrono` 时间处理。
- **Reuses:** 可在 Phase 1 添加 AI 建议字段的写入逻辑。

### Component 6: BootstrapService (Rust)

- **Purpose:** 第一次启动或版本升级时执行数据库初始化、迁移脚本。
- **Interfaces:** `pub fn init(app_handle: &AppHandle) -> AppResult<()>`
- **Dependencies:** `DbConnection`, `include_str!("schema.sql")`
- **Reuses:** Phase 1 可扩展为执行 Alembic/Tauri migration。

## Data Models

### Task (SQLite & Rust)

```
CREATE TABLE tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL CHECK(length(title) <= 200),
    description TEXT NOT NULL CHECK(length(description) <= 2000),
    priority TEXT NOT NULL CHECK(priority IN ('high','medium','low')),
    status TEXT NOT NULL DEFAULT 'pending' CHECK(status IN ('pending','in_progress','completed','cancelled')),
    task_type TEXT NOT NULL CHECK(task_type IN ('work','study','life','other')),
    planned_start_time TEXT NOT NULL,
    deadline TEXT NOT NULL,
    estimated_hours REAL NOT NULL,
    actual_hours REAL,
    tags TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

- Rust `TaskEntity` 对应字段使用 `chrono::DateTime<Utc>` / `Option<f64>` / `serde_json::Value`。
- TypeScript `Task` 类型匹配字段并使用 `string` 表示 ISO 时间。

### User (SQLite & Rust)

```
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    time_zone TEXT NOT NULL,
    locale TEXT NOT NULL,
    preferences TEXT, -- JSON store for theme, focus settings
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

- 单用户模式：默认插入一条主用户记录。
- TypeScript `UserPreferences` 提供类型定义供设置页面使用。

## Error Handling

### Error Scenario 1: 数据库连接失败

- **Handling:**
  - `BootstrapService` 捕获连接异常，返回 `AppError::DatabaseUnavailable`。
  - 前端在 `TaskStore.fetchTasks` 捕获错误并显示带重试按钮的 toast。
  - 记录错误到 `logs/app.log`。
- **User Impact:** 用户看到“数据库初始化失败，请重试或检查磁盘权限”的提示，可点击“重试”。

### Error Scenario 2: 表单验证失败

- **Handling:**
  - 前端使用 `zod` schema 即时验证；若失败，阻止提交并高亮错误字段。
  - 如果绕过前端验证，Rust 命令通过 `validator` 再次验证并返回结构化错误（字段 + 消息）。
- **User Impact:** 在 TaskFormDialog 中显示字段级错误信息，提交按钮禁用直至问题解决。

### Error Scenario 3: Tauri Command Panic

- **Handling:**
  - 使用 `tauri::async_runtime::spawn` + `anyhow::Result`，所有命令返回 `Result`，统一转换为 `AppError`。
  - 设置 `tauri.conf.json` 中的 `error-dialogs` 为 true，开发模式显示详细调试信息。
- **User Impact:** 前端弹出错误提示，日志捕获详细堆栈，避免应用崩溃。

## Testing Strategy

### Unit Testing

- **前端**: 使用 Vitest + React Testing Library 对 `TaskStore`、`TaskForm` 验证表单逻辑、状态更新。
- **后端**: 使用 `cargo test` 对 `TaskService`、`TaskRepository` 进行单元测试，采用临时内存数据库。
- **工具方法**: 对 `datetime.rs`, `validators.rs` 等工具函数编写快照测试。

### Integration Testing

- **前后端 IPC**: 使用 `@tauri-apps/api` mock + Vitest，模拟 `invoke` 调用验证命令参数格式。
- **数据库集成**: 在 `tests/integration/` 中使用 `sqlite` 临时文件测试 `TaskService` 的 CRUD 流程。
- **UI 集成**: 使用 Storybook (Phase 0 optional) 或直接在 Vitest 中对 `TaskListPage` 进行集成测试。

### End-to-End Testing

- **工具**: Playwright + Tauri 测试 runner。
- **场景**:
  1. 用户首次启动应用，看到空任务列表。
  2. 创建任务 → 任务出现在列表中。
  3. 编辑任务 → 字段更新反映在列表。
  4. 删除任务 → 列表中移除。
- **环境**: CI 中使用 GitHub Actions + `tauri-apps/tauri-action` 打包并运行 E2E。

---

通过该设计，Phase 0 将交付一个可运行、可测试、易扩展的桌面应用基础架构，满足所有功能与非功能需求，并为 Phase 1 引入 AI 能力、CoT 推理、双智能体系统提供清晰的扩展点和技术路径。
