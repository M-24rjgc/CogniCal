# Phase 3: Intelligent Analysis & Insights Dashboard - Tasks

## Frontend

- [x] 1. Implement analytics data layer (React)
  - File: src/hooks/useAnalytics.ts (new)
  - Extend React Query + Zustand to fetch overview, history, and export mutations
  - Wire analyticsStore for filters, onboarding completion, export states
  - _Leverage: src/services/tauriApi.ts, src/stores/taskStore.ts, src/stores/uiStore.ts_
  - _Requirements: US-3.1, AC-3.1.1, AC-3.1.3, AC-3.4.2_
  - _Prompt: Role: Senior React Data Engineer specializing in React Query and Zustand | Task: Implement the analytics data layer for Phase 3, integrating new Tauri commands (overview/history/export) via src/services/tauriApi.ts and coordinating state with analyticsStore for filters/onboarding and uiStore for toasts; ensure zero-state and error handling align with AC-3.4.2 | Restrictions: Do not block rendering on export; keep hooks pure and avoid direct DOM; reuse existing formatting utilities | \_Leverage: src/services/tauriApi.ts, src/stores/uiStore.ts, src/utils/date.ts_ | _Requirements: US-3.1, US-3.14, AC-3.1.1, AC-3.4.2 | Success: useAnalytics hook returns typed data/loading/error/export handlers, supports range filters, caches results, surfaces toasts for failures_
- [x] 2. Build analytics UI components & zero state
  - File: src/components/analytics/\*\* (new folder), src/pages/Dashboard.tsx (update)
  - Create AnalyticsOverview, trend/time allocation charts, insight cards, zero-state banner, phase indicator
  - Replace placeholder dashboard copy with data-driven layout integrating Settings CTA and navigation links
  - _Leverage: src/components/ui/card.tsx, src/components/tasks/TaskTable.tsx, src/utils/taskLabels.ts_
  - _Requirements: US-3.1, US-3.2, US-3.3, US-3.13, AC-3.1.\*, AC-3.4.1_
  - _Prompt: Role: Frontend Architect skilled in data visualization and shadcn/ui | Task: Implement analytics UI components delivering trend charts, time allocation visuals, efficiency insight cards, zero-state onboarding, and update Dashboard to Phase 3 copy; integrate with useAnalytics hook, support Recharts, and ensure accessibility | Restrictions: Keep components presentational with typed props; zero state must be dismissible; use CSS variables for theming | \_Leverage: src/components/ui/button.tsx, src/components/ui/card.tsx, src/utils/format.ts_ | _Requirements: US-3.1, US-3.2, US-3.3, US-3.13, AC-3.1.\*, AC-3.4.1 | Success: Dashboard renders real metrics when data exists, shows guided zero state otherwise, and passes accessibility checks_

- [x] 3. Upgrade Settings page & global phase indicator
  - File: src/pages/Settings.tsx, src/stores/settingsStore.ts (new), src/components/layout/Sidebar.tsx
  - Implement settings form for DeepSeek API key, workday hours, theme; persist via Tauri commands; update sidebar/footer to Phase 3 label with summary
  - Display status badges when API key missing, link from dashboard insights to settings
  - _Leverage: src/providers/toast-provider.tsx, src/services/tauriApi.ts, src/components/ui/form.tsx_
  - _Requirements: US-3.10, US-3.13, AC-3.4.1, AC-3.4.2_
  - _Prompt: Role: React Developer focused on form UX and state management | Task: Implement Phase 3 settings experience including DeepSeek key, work hours, theme toggles; wire settingsStore + Tauri commands; update layout to show "Phase 3" status and integrate cross-navigation from dashboard | Restrictions: Securely mask API key, avoid storing raw key in Zustand without gating; reuse react-hook-form + zod | \_Leverage: src/components/ui/form.tsx, src/components/ui/input.tsx, src/providers/toast-provider.tsx_ | _Requirements: US-3.10, US-3.13, AC-3.4.1, AC-3.4.2 | Success: Settings form validates, persists, surfaces toasts, and global phase indicator updates_

- [x] 4. Update Tasks & Calendar pages for cross-navigation
  - File: src/pages/Tasks.tsx, src/pages/Calendar.tsx, src/components/tasks/TaskTable.tsx
  - Add deep links from tasks/planning to analytics insights, ensure time line shows applied plans with statuses, replace placeholder text
  - _Leverage: src/hooks/useTasks.ts, src/hooks/usePlanning.ts, src/components/tasks/TaskPlanningPanel.tsx_
  - _Requirements: US-3.8, US-3.9, US-3.14, AC-3.4.2_
  - _Prompt: Role: Product Engineer bridging task planning and analytics | Task: Enhance Tasks and Calendar screens with new cross-navigation (e.g., "View analytics" links), show plan status timeline, remove placeholder copy, and ensure state sync with analytics store | Restrictions: Do not duplicate analytics logic; must handle empty states gracefully | \_Leverage: src/hooks/useTasks.ts, src/hooks/usePlanning.ts_ | _Requirements: US-3.8, US-3.9, US-3.14, AC-3.4.2 | Success: Users can navigate between tasks/plans and analytics seamlessly; placeholders gone_

- [x] 5. Integrate zero-state onboarding & sample data
  - File: src/stores/analyticsStore.ts (new), src/pages/Dashboard.tsx, src/services/tauriApi.ts
  - Track onboarding completion, provide sample data injection when Tauri unavailable, ensure banner logic matches requirements
  - _Leverage: src/stores/uiStore.ts, src/utils/date.ts, src/services/tauriApi.ts_
  - _Requirements: US-3.13, AC-3.4.1_
  - _Prompt: Role: Frontend Platform Engineer | Task: Add onboarding state machine and sample data support to analytics flow, using Zustand store and tauriApi mocks; banner should respond to real data availability and user actions | Restrictions: Keep sample data flagged as demo; ensure persistence respects privacy | \_Leverage: src/services/tauriApi.ts, src/stores/uiStore.ts_ | _Requirements: US-3.13, AC-3.4.1 | Success: First-run experience guides user and hides once completed or real data exists_

## Backend (Rust)

- [x] 6. Implement analytics service & commands
  - File: src-tauri/src/services/analytics_service.rs (new), src-tauri/src/commands/analytics.rs (new), src-tauri/src/commands/mod.rs, src-tauri/src/services/mod.rs
  - Aggregate metrics from tasks, planning_time_blocks; compute trends, time allocation, efficiency metrics; expose overview/history/export; integrate cache
  - _Leverage: src-tauri/src/services/task_service.rs, src-tauri/src/services/planning_service.rs, src-tauri/src/services/cache_service.rs_
  - _Requirements: AC-3.1.*, AC-3.2.*, AC-3.3.1_
  - _Prompt: Role: Senior Rust Engineer with data analytics focus | Task: Build analytics service performing SQLite aggregations for overview/history/export per requirements, using TaskService and PlanningService; add new Tauri commands and cache layer | Restrictions: Use parameterized queries, avoid long-running locks, ensure JSON serialization matches frontend types | \_Leverage: src-tauri/src/services/task_service.rs, src-tauri/src/services/cache_service.rs_ | _Requirements: AC-3.1.*, AC-3.2.*, AC-3.3.1 | Success: Commands return accurate metrics across ranges and pass integration tests_

- [x] 7. Add settings service & secure storage
  - File: src-tauri/src/services/settings_service.rs (new), src-tauri/src/commands/settings.rs (new), src-tauri/src/commands/mod.rs, src-tauri/src/services/mod.rs
  - Persist DeepSeek API key (basic encryption), work hours, theme; provide get/update commands; respect privacy requirements
  - _Leverage: src-tauri/src/db/migrations.rs, src-tauri/src/services/cache_service.rs_
  - _Requirements: US-3.10, AC-3.4.1, AC-3.3.1_
  - _Prompt: Role: Rust Backend Engineer focusing on secure storage | Task: Implement settings persistence service with minimal encryption for API key, expose get/update commands, and ensure cached settings invalidation | Restrictions: Keep encryption reversible locally; log nothing sensitive; follow AppError conventions | \_Leverage: src-tauri/src/db/migrations.rs_ | _Requirements: US-3.10, AC-3.4.1, AC-3.3.1 | Success: Settings read/write works, API key never logged, tests cover round-trip_

- [x] 8. Extend migrations & repositories
  - File: src-tauri/src/db/migrations.rs, src-tauri/src/db/schema.sql, src-tauri/src/db/repositories (new analytics_repo.rs, settings_repo.rs)
  - Add analytics_snapshots and app_settings tables, indexes, and helper queries; schedule snapshot generation job
  - _Leverage: src-tauri/src/db/mod.rs, src-tauri/src/services/behavior_learning.rs_
  - _Requirements: AC-3.3.1, AC-3.3.3_
  - _Prompt: Role: Database Engineer with SQLite expertise | Task: Update schema for analytics snapshots and app settings, create repository helpers, ensure migrations run idempotently, and provide hooks for daily snapshot job | Restrictions: Maintain backward compatibility; no destructive migrations | \_Leverage: src-tauri/src/db/mod.rs_ | _Requirements: AC-3.3.1, AC-3.3.3 | Success: Migrations apply cleanly, repositories expose typed methods, snapshot job executes_

## Cross-cutting & QA

- [x] 9. Update tauriApi & TypeScript types
  - File: src/services/tauriApi.ts, src/types/analytics.ts (new), src/types/settings.ts (new)
  - Add invoke wrappers, offline mock data, Zod validation for analytics/settings payloads; ensure TypeScript types align with Rust structs
  - _Leverage: src/utils/validators.ts, src/types/task.ts_
  - _Requirements: AC-3.3.2, AC-3.3.3_
  - _Prompt: Role: TypeScript Platform Engineer | Task: Extend tauriApi with analytics/settings commands, add type-safe schemas and offline mocks per requirements, coordinate with new analytics/settings types | Restrictions: Avoid breaking existing exports; maintain consistent error mapping | \_Leverage: src/utils/validators.ts_ | _Requirements: AC-3.3.2, AC-3.3.3 | Success: Commands callable from hooks with type safety; offline mode returns demo data flagged_

- [x] 10. Implement tests & smoke checklist
  - File: src/**tests**/analyticsStore.test.ts (new), tests/integration/analytics_flow.rs, e2e/analytics.e2e.ts, docs/SMOKE-CHECKLIST.md (new)
  - Add unit/integration/e2e tests covering analytics flow, onboarding, settings persistence; document release smoke steps
  - _Leverage: src/**tests**/TaskPlanningPanel.test.tsx, tests/integration/planning_flow.rs, e2e/smoke.e2e.ts_
  - _Requirements: AC-3.3.2, AC-3.4.2, Success Metrics_
  - _Prompt: Role: QA Lead for desktop analytics features | Task: Build automated coverage across unit/integration/e2e for analytics and settings features, plus author Phase 3 smoke checklist | Restrictions: Tests must run in CI; reuse existing utilities | \_Leverage: tests/integration/planning_flow.rs, e2e/smoke.e2e.ts_ | _Requirements: AC-3.3.2, AC-3.4.2, Success Metrics | Success: Tests pass reliably, checklist stored, coverage includes key workflows_
