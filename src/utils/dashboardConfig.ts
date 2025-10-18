import {
  DASHBOARD_MODULE_IDS,
  type DashboardConfig,
  type DashboardConfigInput,
  type DashboardModule,
  type DashboardModuleId,
  type DashboardModuleVisibilityMap,
} from '../types/dashboard';

const MODULE_DEFINITIONS: readonly DashboardModule[] = [
  {
    id: 'quick-actions',
    title: 'Quick Actions',
    enabledByDefault: true,
    order: 0,
  },
  {
    id: 'today-tasks',
    title: 'Today Tasks Overview',
    enabledByDefault: true,
    order: 1,
  },
  {
    id: 'upcoming-alerts',
    title: 'Upcoming Tasks Alert',
    enabledByDefault: true,
    order: 2,
  },
  {
    id: 'productivity-lite',
    title: 'Productivity Score Lite',
    enabledByDefault: true,
    order: 3,
    lazy: true,
  },
  {
    id: 'analytics-overview',
    title: 'Analytics Overview',
    enabledByDefault: false,
    order: 4,
    lazy: true,
  },
  {
    id: 'wellness-summary',
    title: 'Wellness Summary',
    enabledByDefault: false,
    order: 5,
    lazy: true,
  },
  {
    id: 'workload-forecast',
    title: 'Workload Forecast Banner',
    enabledByDefault: false,
    order: 6,
    lazy: true,
  },
] as const;

const DEFAULT_MODULE_VISIBILITY: DashboardModuleVisibilityMap = DASHBOARD_MODULE_IDS.reduce(
  (acc, id) => {
    const definition = MODULE_DEFINITIONS.find((module) => module.id === id);
    acc[id] = definition ? !!definition.enabledByDefault : false;
    return acc;
  },
  {} as DashboardModuleVisibilityMap,
);

export const DASHBOARD_MODULE_DEFINITIONS = MODULE_DEFINITIONS;

export const DEFAULT_DASHBOARD_CONFIG: DashboardConfig = {
  modules: { ...DEFAULT_MODULE_VISIBILITY },
  lastUpdatedAt: null,
};

export function normalizeDashboardConfig(
  config?: DashboardConfig | DashboardConfigInput | null,
): DashboardConfig {
  const modules: DashboardModuleVisibilityMap = { ...DEFAULT_MODULE_VISIBILITY };
  const providedModules = config?.modules;

  if (providedModules) {
    for (const id of DASHBOARD_MODULE_IDS) {
      const override = providedModules[id];
      if (typeof override === 'boolean') {
        modules[id] = override;
      }
    }
  }

  const lastUpdatedAt = config?.lastUpdatedAt ?? null;

  return {
    modules,
    lastUpdatedAt,
  };
}

export function isDashboardModuleEnabled(config: DashboardConfig, id: DashboardModuleId): boolean {
  return config.modules[id] ?? DEFAULT_MODULE_VISIBILITY[id];
}

export function getEnabledDashboardModules(config: DashboardConfig): DashboardModuleId[] {
  return DASHBOARD_MODULE_IDS.filter((id) => isDashboardModuleEnabled(config, id));
}

export function sortModulesByOrder(modules: DashboardModule[]): DashboardModule[] {
  return [...modules].sort((a, b) => a.order - b.order);
}
