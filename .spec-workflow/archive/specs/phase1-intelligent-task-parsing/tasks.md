# Tasks Document

- [x] 1. Extend task domain models for AI metadata
- File: src/types/task.ts
- File: src/utils/validators.ts
- Update Task and TaskPayload interfaces with AI增强字段、计划开始时间、预估工时、任务类型、标签，并同步校验与默认值，确保兼容 existing store。
- _Leverage: src/types/task.ts, src/utils/validators.ts, requirements.md_
- _Requirements: Requirement 1, Requirement 3_
- _Prompt: Implement the task for spec phase1-intelligent-task-parsing, first run spec-workflow-guide to get the workflow guide then implement the task: Role: TypeScript Domain Modeler | Task: Extend task-related TypeScript types and validators to cover AI-enhanced metadata defined in requirements while keeping backward compatibility | Restrictions: Limit changes to the listed files, do not break existing exports, preserve existing validation helpers | Success: All new fields are typed, parsed, and validated; existing unit tests compile; no TypeScript errors introduced | Instructions: Before coding set this task to [-] in tasks.md and set it to [x] once the work and reviews are complete._

- [x] 2. Add AI parse command wrappers in Tauri API layer
  - File: src/services/tauriApi.ts
  - File: src/utils/validators.ts
  - Introduce tasks_parse_ai invoke wrapper、错误映射与 mock fallback，同时接收并返回 AI metadata。
  - _Leverage: src/services/tauriApi.ts, TaskListResponse utilities, requirements.md_
  - _Requirements: Requirement 1, Requirement 4_
  - _Prompt: Implement the task for spec phase1-intelligent-task-parsing, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Frontend Infrastructure Engineer | Task: Add a dedicated parseTask API wrapper with proper validation, error handling, and mock behavior so the UI can request AI parsing | Restrictions: Modify only the listed files, reuse existing error mapping helpers, keep TAURI_UNAVAILABLE flow working | Success: New parseTask function exported, type-safe payload/response, unit mocks continue to operate | Instructions: Before coding set this task to [-] in tasks.md and set it to [x] after completion and verification._

- [x] 3. Enhance useTaskForm hook with AI workflow state
  - File: src/hooks/useTaskForm.ts
  - File: src/components/tasks/TaskFormDialog.tsx
  - Manage AI解析状态、缺失字段提示、结果回填与缓存标记；协调表单与 store 提交逻辑。
  - _Leverage: src/hooks/useTaskForm.ts, src/components/tasks/TaskFormDialog.tsx_
  - _Requirements: Requirement 1, Requirement 2, Requirement 4_
  - _Prompt: Implement the task for spec phase1-intelligent-task-parsing, first run spec-workflow-guide to get the workflow guide then implement the task: Role: React Hook Specialist | Task: Extend useTaskForm to coordinate invoking parseTask, tracking loading/error state, surfacing missing fields, and applying AI results while preserving manual overrides | Restrictions: Keep hook API backward-compatible for callers, avoid side effects outside hook scope, respect existing validation schemas | Success: Hook exposes AI state and callbacks, form dialog renders status chips/tooltips, manual edits remain intact | Instructions: Before coding set this task to [-] in tasks.md and when done revert to [x] after tests and review._

- [x] 4. Implement TaskAiParsePanel component
  - File: src/components/tasks/TaskAiParsePanel.tsx
  - File: src/components/tasks/TaskFormDialog.tsx
  - 提供自然语言输入区、触发按钮、进度/错误反馈、应用与反馈操作，并连接 useTaskForm。
  - _Leverage: shadcn/ui button/input components, ToastProvider, useTaskForm_
  - _Requirements: Requirement 1, Requirement 2_
  - _Prompt: Implement the task for spec phase1-intelligent-task-parsing, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Frontend Component Engineer | Task: Build the TaskAiParsePanel UI that consumes hook callbacks to trigger AI parsing, show live/cache status, and apply results into the form | Restrictions: Follow existing component styling conventions, ensure accessibility labels, do not introduce global state | Success: Panel renders within TaskFormDialog, handles loading/error states, fires callbacks correctly | Instructions: Before coding mark this task as [-] in tasks.md and set to [x] when implementation passes review._

- [x] 5. Create TaskCotViewer and integrate CoT insights
  - File: src/components/tasks/TaskCotViewer.tsx
  - File: src/components/tasks/TaskDetailsDrawer.tsx
  - File: src/components/tasks/TaskFormDialog.tsx
  - 展示思维链步骤、摘要、复制/反馈交互，并在表单与详情抽屉中挂载查看入口。
  - _Leverage: Dialog, ScrollArea, Tooltip components, design.md architecture section_
  - _Requirements: Requirement 2, Requirement 3_
  - _Prompt: Implement the task for spec phase1-intelligent-task-parsing, first run spec-workflow-guide to get the workflow guide then implement the task: Role: UX-focused React Engineer | Task: Build a reusable CoT viewer component and embed it where users inspect AI reasoning, including copy and feedback affordances | Restrictions: Keep component self-contained, reuse shared UI primitives, avoid duplicating styles | Success: Viewer displays ordered steps and summary, accessible trigger buttons exist in form & details, feedback events bubble up | Instructions: Before coding set this task to [-] in tasks.md and when done update it to [x] after validation._

- [x] 6. Update task store and list surfaces for AI metadata
  - File: src/stores/taskStore.ts
  - File: src/components/tasks/TaskTable.tsx
  - File: src/components/tasks/TaskDetailsDrawer.tsx
  - 同步 store 状态与 UI 列表以展示复杂度、标签、缓存来源、过滤能力。
  - _Leverage: Zustand patterns in taskStore, TaskTable columns, requirements.md Requirement 3_
  - _Requirements: Requirement 2, Requirement 3_
  - _Prompt: Implement the task for spec phase1-intelligent-task-parsing, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Frontend State Engineer | Task: Extend task store and table/detail views to persist and render AI metadata, supporting filters and badges per design | Restrictions: Maintain immutability in state updates, keep pagination intact, avoid performance regressions | Success: Store keeps new fields, UI surfaces badges/tooltips, filters operate within 200ms | Instructions: Before coding flip this task to [-] in tasks.md and return it to [x] once complete._

- [x] 7. Add tasks_parse_ai Tauri command plumbing
  - File: src-tauri/src/commands/ai.rs
  - File: src-tauri/src/commands/mod.rs
  - File: src-tauri/src/lib.rs
  - 实现新的命令入口、参数校验、错误映射，并将命令注册进 Builder。
  - _Leverage: existing task commands, AppState management, design.md architecture diagram_
  - _Requirements: Requirement 1, Requirement 4_
  - _Prompt: Implement the task for spec phase1-intelligent-task-parsing, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Tauri Command Engineer | Task: Create the tasks_parse_ai command that invokes AiService, handles AppError mapping, and registers within the command set | Restrictions: Keep command async boundaries consistent, reuse CommandResult types, avoid duplicating logging | Success: Command compiles, returns structured ParsedTaskResponse, integrated into invoke handler | Instructions: Before coding mark this task as [-] in tasks.md and set back to [x] after finishing implementation and tests._

- [x] 8. Implement AiService, CotEngine, and CacheService
  - File: src-tauri/src/services/ai_service.rs
  - File: src-tauri/src/services/cot_engine.rs
  - File: src-tauri/src/services/cache_service.rs
  - 构建 DeepSeek 调用、Prompt 生成、思维链解析、缓存命中/TTL/日志、回退策略。
  - _Leverage: reqwest client utilities, tracing logger, design.md service breakdown_
  - _Requirements: Requirement 1, Requirement 2, Requirement 4_
  - _Prompt: Implement the task for spec phase1-intelligent-task-parsing, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust AI Service Developer | Task: Implement service modules that orchestrate DeepSeek requests, CoT processing, and cache lookups per design | Restrictions: Respect SRP across the three files, handle errors via AppError, ensure blocking work runs via async runtime | Success: Services compile, unit tests for core logic pass, cache hits logged with metrics | Instructions: Before coding toggle this task to [-] in tasks.md and back to [x] when done._

- [x] 9. Update database schema and repositories for new fields
  - File: src-tauri/src/db/schema.sql
  - File: src-tauri/src/db/migrations.rs
  - File: src-tauri/src/models/task.rs
  - 添加任务字段列、AI缓存表、序列化/反序列化逻辑，确保迁移幂等且启动自检。
  - _Leverage: existing migration helpers, TaskRecord mapping, requirements.md Requirement 3_
  - _Requirements: Requirement 3, Requirement 4_
  - _Prompt: Implement the task for spec phase1-intelligent-task-parsing, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Persistence Engineer | Task: Extend DB schema and repositories to store AI metadata and cache entries with proper defaults and constraints | Restrictions: Maintain backward compatibility by setting defaults, ensure migrations run once, update TaskRecord conversions | Success: Migrations succeed on fresh and existing DBs, repository returns enriched records, cache table accessible | Instructions: Before coding set this task to [-] in tasks.md and mark [x] when verified._

- [x] 10. Add logging, configuration, and tests for AI workflow
  - File: src-tauri/src/utils/logger.rs
  - File: src-tauri/tests/integration/ai_parse.rs
  - File: src/**tests**/taskStore.test.ts
  - 配置 tracing target、覆盖正向/离线/缓存回退的集成测试与前端单元测试。
  - _Leverage: existing tracing setup, Vitest suites, Playwright fixtures_
  - _Requirements: Requirement 2, Requirement 4, Non-Functional Requirements_
  - _Prompt: Implement the task for spec phase1-intelligent-task-parsing, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Full-stack Test & Observability Engineer | Task: Instrument logging for AI flows and implement automated tests covering parsing success, failure fallback, and UI state | Restrictions: Keep test runtime reasonable, reuse existing test utilities, do not introduce flaky assertions | Success: Logs show cache hit/miss metrics, integration and unit tests pass locally and in CI | Instructions: Before coding set this task status to [-] in tasks.md and upon completion revert to [x]._
