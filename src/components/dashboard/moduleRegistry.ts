import { lazy } from 'react';
import type { ComponentType } from 'react';
import type { DashboardModuleId } from '../../types/dashboard';

export interface DashboardModuleRegistration {
  component: ComponentType;
  lazy?: boolean;
}

export const dashboardModuleRegistry: Partial<
  Record<DashboardModuleId, DashboardModuleRegistration>
> = {
  'today-tasks': {
    component: lazy(() => import('./TodayTasksOverview')),
    lazy: true,
  },
  'upcoming-alerts': {
    component: lazy(() => import('./UpcomingTasksAlert')),
    lazy: true,
  },
  'productivity-lite': {
    component: lazy(() => import('./ProductivityScoreCardLite')),
    lazy: true,
  },
  'analytics-overview': {
    component: lazy(() => import('../analytics/AnalyticsOverview')),
    lazy: true,
  },
  'wellness-summary': {
    component: lazy(() =>
      import('../wellness/WeeklySummaryPanel').then((module) => ({
        default: module.WeeklySummaryPanel,
      })),
    ),
    lazy: true,
  },
  'workload-forecast': {
    component: lazy(() =>
      import('./modules/WorkloadForecastModule').then((module) => ({
        default: module.WorkloadForecastModule,
      })),
    ),
    lazy: true,
  },
};
