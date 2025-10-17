# Tasks Document

- [x] 1. Extend planning database schema and migrations
  - File: src-tauri/src/db/schema.sql; src-tauri/src/db/migrations.rs (new migration entry)
  - Add `planning_sessions`, `planning_options`, `planning_time_blocks`, `schedule_preferences` tables with indices and foreign keys
  - Wire the new migration into the runner and provide data seeding defaults for preferences snapshot
  - Purpose: Persist planning sessions, options, time blocks, and personalization data locally
  - _Leverage: existing task tables and migration helpers in src-tauri/src/db/migrations.rs_
  - _Requirements: 1, 2, 4_
  - _Prompt: Role: Rust backend engineer specializing in SQLite migrations | Task: Add the planning-related tables and migration wiring following requirements 1, 2, and 4, mirroring existing migration helpers and ensuring referential integrity | Restrictions: Keep migrations idempotent, follow timestamped migration naming, ensure foreign key cascades align with task deletions | Success: Schema compiles, migration applies cleanly, and new tables are ready for planning services_

- [x] 2. Create planning models and repository layer
  - File: src-tauri/src/models/planning.rs (new); src-tauri/src/db/repositories/planning_repository.rs (new); update mod.rs exports
  - Define Rust structs mirroring the new tables and implement CRUD helpers for sessions, options, time blocks, and preferences snapshotting
  - Purpose: Provide typed accessors for planning persistence used by services
  - _Leverage: src-tauri/src/models/task.rs, src-tauri/src/db/repositories/task_repository.rs_
  - _Requirements: 1, 2, 4_
  - _Prompt: Role: Rust data layer developer | Task: Introduce planning models and repository functions aligned with requirements 1, 2, and 4, reusing patterns from task models/repositories | Restrictions: Use From/Into conversions for serde JSON columns, respect transaction boundaries, expose only required methods for services | Success: Repository supports session creation, option/time block reads & writes, and preference storage with tests ready to consume_

- [x] 3. Implement schedule optimizer service and utilities
  - File: src-tauri/src/services/schedule_optimizer.rs (new); src-tauri/src/services/mod.rs (register); src-tauri/src/services/schedule_utils.rs (new helper)
  - Generate ranked plan options, detect conflicts, and score blocks using provided constraints and preferences
  - Purpose: Supply deterministic scheduling candidates and conflict detection for PlanningService
  - _Leverage: src-tauri/src/services/task_service.rs, src-tauri/src/utils/semantic.rs_
  - _Requirements: 1, 2, 3_
  - _Prompt: Role: Rust algorithms engineer with focus on scheduling | Task: Build the ScheduleOptimizer module per requirements 1, 2, and 3, factoring scoring helpers into schedule_utils.rs and following existing service registration patterns | Restrictions: Keep interfaces pure for unit testing, expose deterministic outputs when seeded, surface conflict metadata for UI consumers | Success: Optimizer returns multiple ranked options with conflict data and integrates cleanly into the service layer_

- [x] 4. Implement behavior learning service with preference feedback
  - File: src-tauri/src/services/behavior_learning.rs (new); update src-tauri/src/services/mod.rs
  - Snapshot user preferences before planning, ingest feedback after task completion, and adjust weights based on execution history
  - Purpose: Adapt scheduling to user performance and expose preferences for UI editing
  - _Leverage: src-tauri/src/services/task_service.rs, src-tauri/src/services/cache_service.rs_
  - _Requirements: 3, 4_
  - _Prompt: Role: Rust ML infrastructure engineer | Task: Implement BehaviorLearningService satisfying requirements 3 and 4, reusing task metrics and cache utilities to manage preference data | Restrictions: Keep preference updates transactional, support offline caching, and expose clear error types for command layer | Success: Preferences snapshot/ingest APIs operate correctly and feed into planning sessions_

- [x] 5. Orchestrate planning workflow in PlanningService
  - File: src-tauri/src/services/planning_service.rs (new); update src-tauri/src/services/mod.rs
  - Coordinate repositories, optimizer, behavior learning, and AI explanations to produce planning sessions, apply options, and resolve conflicts
  - Purpose: Centralize planning business logic with transactional guarantees
  - _Leverage: src-tauri/src/services/ai_service.rs, src-tauri/src/services/task_service.rs, ScheduleOptimizer, BehaviorLearningService_
  - _Requirements: 1, 2, 3, 4_
  - _Prompt: Role: Rust service orchestrator | Task: Implement PlanningService covering requirements 1-4, coordinating optimizer, AI summaries, repositories, and conflict resolution | Restrictions: Maintain transactional integrity, log decision paths for debugging, emit structured errors for the command layer | Success: Service exposes generate, apply, resolve APIs used by Tauri commands and passes integration tests_

- [x] 6. Expose planning commands in Tauri layer
  - File: src-tauri/src/commands/planning.rs (new); src-tauri/src/commands/mod.rs; src-tauri/src/main.rs registration
  - Add `planning_generate`, `planning_apply`, `planning_resolve_conflict`, and `planning_preferences_*` commands invoking PlanningService and BehaviorLearningService
  - Purpose: Provide frontend access to planning capabilities via invoke API
  - _Leverage: src-tauri/src/commands/task.rs, src-tauri/src/utils/logger.rs_
  - _Requirements: 1, 2, 3, 4_
  - _Prompt: Role: Tauri command engineer | Task: Wire planning commands per requirements 1-4, matching invoke patterns and structured responses from existing command modules | Restrictions: Use CommandResult wrapper, map domain errors to AppError codes, emit planning events through AppHandle | Success: Commands build, expose correct payloads, and forward events to the frontend_

- [x] 7. Define planning TypeScript types and tauriApi bindings
  - File: src/types/planning.ts (new); src/services/tauriApi.ts (extend); src/utils/taskLabels.ts (ensure labels map); vite-env declarations if needed
  - Model planning session, option, conflict, preference DTOs and add invoke helpers for new commands
  - Purpose: Enable strongly-typed planning data flow on frontend
  - _Leverage: src/types/task.ts, src/services/tauriApi.ts existing patterns_
  - _Requirements: 1, 2, 3, 4_
  - _Prompt: Role: TypeScript platform engineer | Task: Create planning types and tauriApi helpers aligned with requirements 1-4, reusing existing DTO patterns and error mappers | Restrictions: Avoid breaking existing exports, ensure JSON parsing safety, keep enums string literal based | Success: New types compile, tauriApi exposes typed helpers, and lint/tests succeed_

- [x] 8. Build planning store and hooks for session management
  - File: src/stores/planningStore.ts (new); src/hooks/usePlanning.ts (new); update providers/toast-provider if needed
  - Manage generate/apply/resolve flows, loading states, event subscriptions, and preference CRUD
  - Purpose: Centralize planning state and provide reusable hooks to UI components
  - _Leverage: src/stores/taskStore.ts, src/hooks/useTasks.ts, src/providers/toast-provider.tsx_
  - _Requirements: 1, 2, 3, 4_
  - _Prompt: Role: React state management specialist | Task: Implement planning store and hook covering requirements 1-4, mirroring taskStore patterns and handling command events | Restrictions: Keep Zustand slices serializable, surface optimistic updates carefully, reuse toast/error helpers | Success: Store exposes typed actions/selectors, integrates with tauri events, and unit tests pass_

- [x] 9. Implement planning UI components and integrate into task workflows
  - File: src/components/tasks/TaskPlanningPanel.tsx (new or extend); src/components/tasks/PlanOptionCard.tsx (new); src/components/tasks/ConflictResolutionSheet.tsx (new); src/components/tasks/PersonalizationDialog.tsx (new); update TaskDetailsDrawer.tsx and TaskTable.tsx triggers
  - Present plan generation trigger, option cards with CoT summaries, conflict sheet with resolution actions, and personalization dialog for preferences
  - Purpose: Deliver end-to-end planning interaction to users within tasks experience
  - _Leverage: src/components/tasks/TaskTable.tsx, src/components/tasks/TaskDetailsDrawer.tsx, src/components/ui/\* primitives_
  - _Requirements: 1, 2, 3, 4_
  - _Prompt: Role: Senior React engineer focusing on UX flows | Task: Build planning UI per requirements 1-4, reusing existing design system components and ensuring accessibility | Restrictions: Maintain responsive layouts, lazy-load heavy panels, hook into planningStore without duplicating state | Success: Users can generate plans, inspect CoT, resolve conflicts, and adjust preferences directly in the UI_

- [x] 10. Sync calendar view with planning time blocks
  - File: src/pages/Calendar.tsx; src/components/tasks/TaskTable.tsx (selection context); src/styles/globals.css (scoped styles if needed)
  - Display newly applied time blocks, highlight conflicts, and keep calendar synchronized with planningStore events
  - Purpose: Visualize scheduled time blocks and changes resulting from planning
  - _Leverage: existing calendar data adapters in src/pages/Calendar.tsx_
  - _Requirements: 2, 3_
  - _Prompt: Role: Frontend engineer specializing in calendaring UIs | Task: Integrate planning time blocks into the calendar per requirements 2 and 3, reusing existing calendar utilities and maintaining performance | Restrictions: Avoid duplicate renders, respect timezone handling, ensure conflict badges align with design | Success: Calendar reflects applied plans, updates reactively, and highlights conflicts_

- [x] 11. Write Rust unit and integration tests for planning pipeline
  - File: src-tauri/src/services/schedule_optimizer.rs (tests module); src-tauri/src/services/behavior_learning.rs (tests); tests/integration/planning_flow.rs (new)
  - Cover optimizer scoring/conflict detection, behavior preference updates, and end-to-end generate/apply/resolve flow using in-memory SQLite
  - Purpose: Guarantee backend planning logic correctness and regression safety
  - _Leverage: existing test helpers in src-tauri/src/services/task_service.rs tests, rusqlite in-memory setup_
  - _Requirements: 1, 2, 3, 4_
  - _Prompt: Role: Rust test engineer | Task: Author unit and integration tests fulfilling requirements 1-4, using in-memory DB and mocked AI responses to verify planning workflows | Restrictions: Keep tests deterministic, isolate external API calls, assert transactional rollbacks on failure | Success: Tests pass, cover success/failure cases, and protect core planning logic_

- [x] 12. Add frontend unit tests for planning store and components
  - File: src/**tests**/planningStore.test.ts (new); src/**tests**/TaskPlanningPanel.test.tsx (new); update vitest config if necessary
  - Validate store reducers, command interactions, UI rendering of options/conflicts, and preference editing flows
  - Purpose: Ensure frontend planning logic behaves as expected
  - _Leverage: src/**tests**/taskStore.test.ts, src/**tests**/smoke.test.ts patterns_
  - _Requirements: 1, 2, 3, 4_
  - _Prompt: Role: React testing engineer | Task: Implement Vitest/RTL tests addressing requirements 1-4, mocking tauriApi calls and verifying user stories | Restrictions: Avoid brittle snapshot tests, emphasize behavior assertions, reuse shared test utilities | Success: Tests cover critical states, pass reliably, and document expected UX behavior_

- [x] 13. Extend Playwright E2E coverage for planning journey
  - File: e2e/planning.e2e.ts (new); update playwright.config.ts if routes needed
  - Automate user flow: select tasks → generate plans → review CoT → apply option → handle conflict → confirm calendar update
  - Purpose: Validate planning feature end-to-end in desktop runtime
  - _Leverage: e2e/smoke.e2e.ts setup, existing selectors in smoke.spec.ts_
  - _Requirements: 1, 2, 3, 4_
  - _Prompt: Role: QA automation engineer | Task: Create Playwright E2E test covering requirements 1-4, using stable selectors and waiting on tauri events | Restrictions: Keep test resilient to timing variance, capture screenshots on failure, reuse login/setup helpers | Success: E2E passes locally/CI and proves the planning journey works as expected_
