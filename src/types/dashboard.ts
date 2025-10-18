export type DashboardModuleId =
  | 'quick-actions'
  | 'today-tasks'
  | 'upcoming-alerts'
  | 'productivity-lite'
  | 'analytics-overview'
  | 'wellness-summary'
  | 'workload-forecast';

export const DASHBOARD_MODULE_IDS: readonly DashboardModuleId[] = [
  'quick-actions',
  'today-tasks',
  'upcoming-alerts',
  'productivity-lite',
  'analytics-overview',
  'wellness-summary',
  'workload-forecast',
] as const;

export interface DashboardModule {
  id: DashboardModuleId;
  title: string;
  /**
   * Indicates whether the module should be visible in the default simplified layout.
   */
  enabledByDefault: boolean;
  /**
   * Lower numbers appear earlier in the dashboard layout.
   */
  order: number;
  /**
   * Lazy modules are loaded on demand when enabled by configuration.
   */
  lazy?: boolean;
  description?: string;
}

export type DashboardModuleVisibilityMap = Record<DashboardModuleId, boolean>;

export interface DashboardConfig {
  modules: DashboardModuleVisibilityMap;
  /** ISO timestamp of last modification, or null when never persisted. */
  lastUpdatedAt: string | null;
}

export interface DashboardConfigInput {
  modules?: Partial<DashboardModuleVisibilityMap>;
  lastUpdatedAt?: string | null;
}

export type DashboardConfigPatch = DashboardConfigInput;

export type QuickActionId = 'create-task' | 'open-calendar' | 'smart-plan';

export type QuickActionAction = 'openModal' | 'navigate';

export interface QuickAction {
  id: QuickActionId;
  label: string;
  tooltip: string;
  action: QuickActionAction;
  target: string;
  icon?: string;
}
