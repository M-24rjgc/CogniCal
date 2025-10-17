# Phase 3: Intelligent Analysis & Insights Dashboard - Design

## Overview

This design describes how Phase 3 upgrades CogniCal into a baseline-usable product by delivering:

- A production-ready analytics dashboard with real-time metrics, historical trends, AI-powered insights, and exportable reports.
- Zero-state onboarding, cross-page workflow coherence, and resilient offline behavior so that the end-to-end task → planning → insights loop runs without placeholders.
- The minimal preferences surface (DeepSeek API key, theme, working hours) necessary for users to activate AI features within the desktop app.
- Architectural foundations (Rust analytics service, Tauri commands, React modules) that align with steering documents and remain extensible for future agents and plugins.

## Steering Document Alignment

### Technical Standards (tech.md)

- **Frontend stack**: Reuse React + TypeScript + Tailwind + shadcn/ui; add Recharts-based visualizations per tech.md guidance. All charts are encapsulated in typed components that accept data-only props, ensuring presentational/pure logic separation.
- **Backend stack**: Implement a new `analytics_service` in Rust using SQLite aggregations, caching via existing `cache_service`, and expose Tauri commands `analytics_overview_fetch`, `analytics_report_export`, and `settings_update`. This follows the documented pattern of command handlers delegating to services.
- **IPC contract**: Commands return camelCase JSON payloads consistent with `tauriApi.ts` conventions. Error mapping leverages existing `AppError` conversions.
- **Performance**: Metrics queries use parameterized SQL with indexed fields (`tasks.status`, `planning_time_blocks.start_at`). Heavy calculations (daily snapshots) run in background cron triggered via Rust task to avoid UI blocking.

### Project Structure (structure.md)

- **Backend modules**: Add `src-tauri/src/services/analytics_service.rs`, `src-tauri/src/services/settings_service.rs`, and `src-tauri/src/commands/analytics.rs`, wired through `commands/mod.rs` and `services/mod.rs` per module structure guidelines. Database migrations live in `src-tauri/src/db/migrations.rs`.
- **Frontend feature folders**: Add `src/components/analytics/` (charts, insight cards), `src/components/settings/` (forms), and `src/hooks/useAnalytics.ts`. Dashboard implementation remains under `src/pages/Dashboard.tsx` but delegates to analytics components to keep files focused.
- **Stores & services**: Introduce `src/stores/analyticsStore.ts` and `src/stores/settingsStore.ts` aligning with feature-first structure. Extend `src/services/tauriApi.ts` for analytics/settings commands.
- **Testing layout**: Place new unit tests in `src/__tests__/analyticsStore.test.ts`, integration tests in `tests/integration/analytics_flow.rs`, and Playwright scenarios under `e2e/analytics.e2e.ts` following structure.md guidance.

## Code Reuse Analysis

### Existing Components to Leverage

- **`TaskTable`, `TaskPlanningPanel`, `TaskDetailsDrawer`**: Provide navigation hooks to analytics insights (e.g., deep links from cards to tasks/plans).
- **`useTaskStore` & `usePlanning` hooks**: Source real-time task and planning block data required for chart inputs when generating combined views.
- **`cache_service` & `AiService`**: Supply cached AI recommendations and CoT summaries for insight cards; analytics service will read existing CoT metadata fields.
- **`ToastProvider` & `useUIStore`**: Continue surfacing success/error toasts for analytics refresh, export, and settings persistence.

### Integration Points

- **Tauri Commands**: Extend `tauriApi.ts` with `ANALYTICS_OVERVIEW_FETCH`, `ANALYTICS_HISTORY_FETCH`, `ANALYTICS_EXPORT`, `SETTINGS_GET`, `SETTINGS_UPDATE`, mirroring existing invoke wrappers and offline mocks.
- **Database**: Reuse `tasks`, `planning_sessions`, and `planning_time_blocks` tables for aggregations; add new tables `analytics_snapshots` (daily rollups) and `app_settings` (user preferences) with migrations and indexes.
- **React Query / Zustand**: Use React Query for analytics data fetching + caching, while Zustand manages persisted UI state (e.g., selected insight filters, onboarding completion flags).

## Architecture

The solution splits responsibilities across clear layers:

```mermaid
graph TD
    subgraph UI Layer (React)
        Dashboard[DashboardPage]
        Charts[Analytics Components]
        Insights[InsightFeed]
        Settings[SettingsForm]
    end

    subgraph State Layer
        AStore[analyticsStore (Zustand)]
        SStore[settingsStore (Zustand)]
        Query[React Query]
    end

    subgraph IPC (Tauri Commands)
        CmdAnalytics[analytics_overview_fetch]
        CmdHistory[analytics_history_fetch]
        CmdExport[analytics_report_export]
        CmdSettings[settings_get / settings_update]
    end

    subgraph Backend (Rust)
        AnalyticsSvc[AnalyticsService]
        SettingsSvc[SettingsService]
        CacheSvc[CacheService]
        TaskSvc[TaskService]
        PlanningSvc[PlanningService]
        Repo[AnalyticsRepository + SettingsRepository]
        DB[(SQLite)]
    end

    Dashboard -->|useAnalytics hook| Query
    Charts --> AStore
    Insights --> Query
    Settings --> SStore
    Query --> CmdAnalytics
    Query --> CmdHistory
    Query --> CmdSettings
    AStore --> CmdExport
    CmdAnalytics --> AnalyticsSvc
    CmdHistory --> AnalyticsSvc
    CmdExport --> AnalyticsSvc
    CmdSettings --> SettingsSvc
    AnalyticsSvc --> Repo
    SettingsSvc --> Repo
    Repo --> DB
    AnalyticsSvc --> CacheSvc
    AnalyticsSvc --> TaskSvc
    AnalyticsSvc --> PlanningSvc
```

### Modular Design Principles

- **Single File Responsibility**: Each chart (`ProductivityTrendChart.tsx`, `TimeAllocationPie.tsx`, etc.) only renders data passed via props; data shaping stays inside hooks/services.
- **Component Isolation**: Dashboard composes cards via `AnalyticsOverview` container, preventing monolithic JSX.
- **Service Separation**: Rust analytics service handles aggregation; command handlers only marshal requests; React components rely on typed hooks.
- **Utility Modularity**: Shared helpers (date ranges, percentile calculations) live in `src/utils/analytics.ts` with pure functions and tests.

## Components and Interfaces

### Frontend Components

1. **`AnalyticsOverview`**
   - **Purpose**: Fetch overview metrics, orchestrate cards, handle zero-state walkthrough.
   - **Interfaces**: Props `onRequestExport`, internal context for child components.
   - **Dependencies**: `useAnalytics` hook, `analyticsStore` for filter state.
   - **Reuses**: `Card`, `Skeleton`, `Button` from shadcn/ui.

2. **`ProductivityTrendChart`**
   - **Purpose**: Display completion rates vs. productivity score over selectable time ranges.
   - **Interfaces**: `data: TrendPoint[]`, `range: '7d' | '30d' | '90d'`.
   - **Dependencies**: Recharts `LineChart`, `TimeRangeSelector` control.

3. **`TimeAllocationPie`**
   - **Purpose**: Show time spent by task type and priority.
   - **Dependencies**: Recharts `PieChart`, `Tooltip`; uses `formatDuration` from `utils/time`.

4. **`EfficiencyInsightList`**
   - **Purpose**: Render AI recommendations with CoT summary, action CTA linking to tasks/plans.
   - **Dependencies**: `Link` from React Router, `notifySuccessToast` for quick actions.

5. **`ZeroStateBanner`**
   - **Purpose**: Provide first-run guidance, sample data injection option, & CTA to create tasks.
   - **Dependencies**: `useSettings`, `useTasks` for counts.

6. **`SettingsForm`**
   - **Purpose**: Collect DeepSeek API key, work hours, theme preferences with validation.
   - **Interfaces**: Controlled by `react-hook-form` + `zod` schema; persists via `settingsStore`.

7. **`PhaseIndicator`**
   - **Purpose**: Update sidebar/footer to show "Phase 3 - Intelligent Analysis & Insights" and summary of phases.
   - **Dependencies**: Reads from `analyticsStore` for completion stats.

### Hooks & Stores

- **`useAnalytics` hook**: Wraps React Query `useQuery` for overview data, `useMutation` for export. Accepts filter state from Zustand store, returns typed results with loading/error states.
- **`analyticsStore` (Zustand)**: Holds UI filter preferences (date range, grouping, selected insight). Provides actions `setRange`, `toggleInsight`, `completeOnboarding`.
- **`settingsStore`**: Persists preferences, stores API key in memory with optional secure storage (Windows Credential Locker via Tauri plugin in future phases). Exposes `loadSettings`, `updateSettings`.

### Backend Services

1. **`AnalyticsService`**
   - **Functions**:
     - `get_overview(range: DateRange) -> AnalyticsOverview`: aggregates completion rates, time allocations, efficiency metrics.
     - `get_history(range: DateRange) -> Vec<AnalyticsSnapshot>`: returns daily snapshots for charts.
     - `export_report(range, format) -> ReportFile`: generates Markdown or PNG (via headless render + storing path).
   - **Dependencies**: `TaskService` (task data), `PlanningService` (block schedules), `CacheService` (memoization for range queries).
   - **Implementation**: Parameterized SQL queries using window functions where available; fallback manual aggregation in Rust iterators.

2. **`SettingsService`**
   - **Functions**: `get()`, `upsert(SettingsPayload)`, `clear_sensitive()`. Stores encrypted API key (base64 + OS keyring in future; Phase 3 uses reversible XOR + environment-specific path with caution, documented).

3. **Command Handlers (`commands/analytics.rs`, `commands/settings.rs`)**
   - Map Tauri invoke payloads to services, convert errors using `CommandError`.

## Data Models

### Database (SQLite) Migrations

1. **`analytics_snapshots`**

   ```sql
   CREATE TABLE IF NOT EXISTS analytics_snapshots (
       snapshot_date TEXT PRIMARY KEY,
       total_tasks_completed INTEGER NOT NULL,
       completion_rate REAL NOT NULL,
       overdue_tasks INTEGER NOT NULL,
       total_focus_minutes INTEGER NOT NULL,
       productivity_score REAL NOT NULL,
       efficiency_rating REAL NOT NULL,
       time_spent_work REAL NOT NULL,
       time_spent_study REAL NOT NULL,
       time_spent_life REAL NOT NULL,
       time_spent_other REAL NOT NULL,
       on_time_ratio REAL NOT NULL,
       created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
   );
   CREATE INDEX IF NOT EXISTS idx_snapshots_created_at ON analytics_snapshots(created_at);
   ```

2. **`app_settings`**
   ```sql
   CREATE TABLE IF NOT EXISTS app_settings (
       key TEXT PRIMARY KEY,
       value TEXT NOT NULL,
       updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
   );
   ```
   Stored keys include `deepseek_api_key`, `workday_start_minute`, `workday_end_minute`, `theme`.

### Rust Structs

```rust
pub struct AnalyticsOverview {
    pub range: DateRange,
    pub summary: SummaryMetrics,
    pub trend: Vec<TrendPoint>,
    pub time_allocation: TimeAllocation,
    pub efficiency: EfficiencyMetrics,
    pub insights: Vec<InsightCard>,
    pub zero_state: ZeroStateMeta,
}

pub struct SettingsPayload {
    pub deepseek_api_key: Option<String>,
    pub workday_start_minute: i16,
    pub workday_end_minute: i16,
    pub theme: String,
}
```

### TypeScript Types

```ts
export type AnalyticsOverview = {
  range: DateRange;
  summary: {
    totalCompleted: number;
    completionRate: number;
    trendDelta: number;
    workloadPrediction: number;
  };
  trend: TrendPoint[];
  timeAllocation: {
    byType: Array<{ type: TaskType; minutes: number }>;
    byPriority: Array<{ priority: TaskPriority; minutes: number }>;
  };
  efficiency: {
    estimateAccuracy: number;
    onTimeRate: number;
    complexityCorrelation: number;
    suggestions: EfficiencySuggestion[];
  };
  insights: InsightCard[];
  zeroState: {
    isEmpty: boolean;
    recommendedActions: string[];
  };
};

export type AppSettings = {
  deepseekApiKey?: string;
  workdayStartMinute: number;
  workdayEndMinute: number;
  theme: 'system' | 'light' | 'dark';
};
```

## Error Handling

### Error Scenarios

1. **Analytics query failure (SQLite offline/corrupt)**
   - **Handling**: Rust service returns `AppError::Database`; command converts to `CommandError`. Frontend uses fallback cached snapshot (last successful query) and displays banner with retry + support instructions.
   - **User Impact**: Dashboard shows "数据暂不可用" card with retry button; rest of app remains functional.

2. **DeepSeek API key missing when generating AI insights**
   - **Handling**: Analytics service checks `SettingsService` for key; if absent, only deterministic insights render, and `zero_state.recommendedActions` includes "前往设置填写 API Key".
   - **User Impact**: Insight list shows partial data + CTA linking to Settings; toast warns "未配置 AI Key".

3. **Export failure (filesystem permission)**
   - **Handling**: Command catches `AppError::Io`, prompts user to choose alternative path via Tauri dialog; analytics store sets export status `failed` with detail.
   - **User Impact**: Toast displays failure reason; export dialog surfaces new location option.

4. **Offline / Tauri unavailable**
   - **Handling**: `tauriApi` offline mock returns synthesized metrics (with watermark) to maintain onboarding flow; UI labels data as "示例数据" until reconnection.
   - **User Impact**: Users can still explore charts but see ribbons indicating demo data.

## Testing Strategy

### Unit Testing

- **Rust**: `analytics_service` aggregation functions with in-memory SQLite; ensure edge cases (zero tasks, mixed statuses).
- **TypeScript**: Utility functions in `utils/analytics.ts`, Zustand stores reducers, React component snapshot tests (using `@testing-library/react`).

### Integration Testing

- **Rust Integration**: `tests/integration/analytics_flow.rs` seeds tasks/plans, calls commands to validate JSON shape and caching behavior.
- **React**: `AnalyticsOverview.test.tsx` renders with mocked query Client to validate zero-state, loading skeleton, and filter interactions.

### End-to-End Testing

- **Playwright Scenario**: `e2e/analytics.e2e.ts` automates: create sample tasks → generate plan → verify dashboard charts, export report, navigate to settings to update API key.
- **Smoke Checklist Automation**: Extend existing smoke tests to cover navigation, offline mode toggle, and zero-state onboarding flow.

---

This design provides the blueprint for implementing Phase 3 so that CogniCal delivers a fully functional analytics experience, replaces placeholders, and meets the baseline usability expectations outlined in the approved requirements.
