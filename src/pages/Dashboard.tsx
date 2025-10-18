import { Suspense, useMemo } from 'react';
import { CalendarDays, Loader2, Plus, Sparkles } from 'lucide-react';
import { useNavigate } from 'react-router-dom';
import QuickActionsBar, { type QuickActionsBarItem } from '../components/dashboard/QuickActionsBar';
import { dashboardModuleRegistry } from '../components/dashboard/moduleRegistry';
import { DASHBOARD_MODULE_DEFINITIONS, sortModulesByOrder } from '../utils/dashboardConfig';
import type { DashboardModuleId } from '../types/dashboard';
import { useDashboardConfig } from '../hooks/useDashboardConfig';
import { WellnessNudgeToast } from '../components/wellness/WellnessNudgeToast';
import { usePendingNudge } from '../hooks/useWellness';

export default function DashboardPage() {
  const navigate = useNavigate();
  const { config } = useDashboardConfig();
  const { data: pendingNudge } = usePendingNudge();

  const activeModules = useMemo(
    () =>
      sortModulesByOrder(
        DASHBOARD_MODULE_DEFINITIONS.filter((definition) => config.modules[definition.id]),
      ),
    [config],
  );

  const quickActionsEnabled = activeModules.some((definition) => definition.id === 'quick-actions');
  const gridModules = activeModules.filter((definition) => definition.id !== 'quick-actions');

  const moduleLayoutClasses: Partial<Record<DashboardModuleId, string>> = {
    'analytics-overview': 'xl:col-span-2',
    'workload-forecast': 'xl:col-span-2',
  };

  const quickActions = useMemo<QuickActionsBarItem[]>(
    () => [
      {
        id: 'create-task',
        label: '创建任务',
        description: '快速添加需要关注的任务条目',
        tooltip: '打开任务创建对话框',
        icon: Plus,
        shortcut: 'C',
        onSelect: () => {
          navigate('/tasks', { state: { intent: 'create-task' } });
        },
      },
      {
        id: 'open-calendar',
        label: '查看日历',
        description: '检查今日安排与空闲可用时间',
        tooltip: '跳转至日历视图',
        icon: CalendarDays,
        shortcut: 'K',
        to: '/calendar',
      },
      {
        id: 'smart-plan',
        label: '生成智能规划',
        description: '挑选任务并生成多种执行方案',
        tooltip: '前往智能规划中心',
        icon: Sparkles,
        shortcut: 'G',
        onSelect: () => {
          navigate('/tasks', { state: { intent: 'open-planning' } });
        },
      },
    ],
    [navigate],
  );

  return (
    <div className="flex-1 overflow-y-auto">
      <div
        className="mx-auto flex w-full max-w-7xl flex-col gap-6 p-6 pb-14"
        data-onboarding="dashboard-overview"
      >
        {quickActionsEnabled ? (
          <QuickActionsBar
            actions={quickActions}
            title="快捷操作"
            subtitle="集中处理高频任务与调度操作，提高响应效率。"
            badgeText="默认布局"
          />
        ) : null}

        {gridModules.length > 0 ? (
          <div className="grid gap-6 xl:grid-cols-2">
            {gridModules.map((definition) => {
              const registration = dashboardModuleRegistry[definition.id];
              if (!registration) {
                return null;
              }

              const ModuleComponent = registration.component;
              const wrapperClass = moduleLayoutClasses[definition.id] ?? '';
              const fallback = (
                <div className="flex min-h-[160px] items-center justify-center rounded-3xl border border-border/60 bg-muted/40">
                  <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
                </div>
              );

              return (
                <div key={definition.id} className={wrapperClass}>
                  {registration.lazy ? (
                    <Suspense fallback={fallback}>
                      <ModuleComponent />
                    </Suspense>
                  ) : (
                    <ModuleComponent />
                  )}
                </div>
              );
            })}
          </div>
        ) : (
          <div className="rounded-3xl border border-dashed border-border/60 bg-muted/20 p-6 text-sm text-muted-foreground">
            当前未启用任何仪表盘模块，可在设置 &gt; 仪表盘配置中重新选择需要展示的内容。
          </div>
        )}
      </div>
      <WellnessNudgeToast nudge={pendingNudge ?? null} />
    </div>
  );
}
