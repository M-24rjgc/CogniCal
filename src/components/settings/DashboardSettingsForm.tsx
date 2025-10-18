import { useCallback, useMemo } from 'react';
import { AlertCircle, Loader2, RefreshCw, RotateCcw, Sparkles } from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '../ui/card';
import { Button } from '../ui/button';
import { Badge } from '../ui/badge';
import { Skeleton } from '../ui/skeleton';
import { useDashboardConfig } from '../../hooks/useDashboardConfig';
import { DASHBOARD_MODULE_DEFINITIONS } from '../../utils/dashboardConfig';
import type { DashboardModule, DashboardModuleId } from '../../types/dashboard';

const MODULE_DESCRIPTIONS: Record<DashboardModuleId, string> = {
  'quick-actions': '在仪表盘顶部提供常用操作入口，方便快速创建任务或跳转关键页面。',
  'today-tasks': '展示今日任务完成度与重点事项，帮助保持专注与优先级。',
  'upcoming-alerts': '提醒即将到期的任务，提前处理高风险事项，减少临近压力。',
  'productivity-lite': '以轻量卡片呈现生产力评分，让你快速了解效率走势。',
  'analytics-overview': '加载完整分析概览，包括趋势、类型占比与效率洞察。',
  'wellness-summary': '集合健康提示与专注建议，提醒保持良好工作节奏。',
  'workload-forecast': '预估未来工作量波动，为排期与资源协调提供参考。',
};

const getModuleDescription = (module: DashboardModule) =>
  MODULE_DESCRIPTIONS[module.id] ?? '启用后将于仪表盘中呈现对应信息模块。';

const formatTimestamp = (timestamp: string | null): string => {
  if (!timestamp) {
    return '尚未保存过配置';
  }
  const parsed = new Date(timestamp);
  if (Number.isNaN(parsed.getTime())) {
    return '尚未保存过配置';
  }
  return parsed.toLocaleString('zh-CN');
};

const renderLoadingRows = () => (
  <div className="space-y-3">
    {Array.from({ length: 4 }).map((_, index) => (
      <div
        key={index}
        className="flex items-center justify-between rounded-2xl border border-border/60 bg-background/70 p-4"
      >
        <div className="space-y-2">
          <Skeleton className="h-3.5 w-40" />
          <Skeleton className="h-2.5 w-56" />
        </div>
        <Skeleton className="h-6 w-11" />
      </div>
    ))}
  </div>
);

const DashboardSettingsForm = () => {
  const {
    config,
    enabledModules,
    isLoading,
    isSaving,
    error,
    refresh,
    reset,
    setModuleEnabled,
    clearError,
  } = useDashboardConfig();

  const modules = useMemo(() => [...DASHBOARD_MODULE_DEFINITIONS], []);
  const enabledCount = enabledModules.length;
  const totalCount = modules.length;

  const handleRefresh = useCallback(async () => {
    try {
      await refresh();
    } catch {
      // 错误提示由 hook 内部处理
    }
  }, [refresh]);

  const handleReset = useCallback(async () => {
    try {
      await reset();
    } catch {
      // 错误提示由 hook 内部处理
    }
  }, [reset]);

  const handleToggle = useCallback(
    async (moduleId: DashboardModuleId, checked: boolean) => {
      try {
        await setModuleEnabled(moduleId, checked);
      } catch {
        // hook 已处理错误提示
      }
    },
    [setModuleEnabled],
  );

  const lastUpdatedLabel = formatTimestamp(config.lastUpdatedAt ?? null);
  const isBusy = isSaving;

  return (
    <Card className="rounded-3xl border-border/70 bg-card/80 shadow-sm">
      <CardHeader className="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
        <div className="space-y-1">
          <CardTitle className="text-base">仪表盘模块配置</CardTitle>
          <p className="text-sm text-muted-foreground">
            按需启用或停用模块，保存后仪表盘布局将即时刷新，可在桌面端同步生效。
          </p>
        </div>
        <div className="flex flex-wrap gap-2">
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={() => {
              clearError();
              void handleRefresh();
            }}
            disabled={isLoading}
            className="inline-flex items-center gap-1"
          >
            {isLoading ? (
              <Loader2 className="h-3.5 w-3.5 animate-spin" />
            ) : (
              <RefreshCw className="h-3.5 w-3.5" />
            )}
            重新获取
          </Button>
          <Button
            type="button"
            variant="secondary"
            size="sm"
            onClick={() => {
              clearError();
              void handleReset();
            }}
            disabled={isBusy || isLoading}
            className="inline-flex items-center gap-1"
          >
            {isBusy ? (
              <Loader2 className="h-3.5 w-3.5 animate-spin" />
            ) : (
              <RotateCcw className="h-3.5 w-3.5" />
            )}
            恢复默认
          </Button>
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        {error ? (
          <div className="flex flex-wrap items-center justify-between gap-3 rounded-2xl border border-destructive/40 bg-destructive/10 px-3 py-2 text-xs text-destructive">
            <div className="flex items-center gap-2">
              <AlertCircle className="h-4 w-4" />
              <span>{error.message}</span>
            </div>
            <Button
              type="button"
              size="sm"
              variant="ghost"
              onClick={() => {
                clearError();
                void handleRefresh();
              }}
            >
              <RefreshCw className="mr-1.5 h-3.5 w-3.5" /> 重试
            </Button>
          </div>
        ) : null}

        {isLoading && !error ? renderLoadingRows() : null}

        {!isLoading ? (
          <div className="space-y-3">
            {modules.map((module) => {
              const isEnabled = Boolean(config.modules[module.id]);
              const moduleDescription = getModuleDescription(module);
              return (
                <div
                  key={module.id}
                  className="flex flex-col gap-3 rounded-2xl border border-border/60 bg-background/80 p-4 sm:flex-row sm:items-center sm:justify-between"
                >
                  <div className="space-y-2">
                    <div className="flex flex-wrap items-center gap-2">
                      <span className="text-sm font-medium text-foreground">{module.title}</span>
                      {module.enabledByDefault ? <Badge variant="secondary">默认启用</Badge> : null}
                      {module.lazy ? <Badge variant="outline">懒加载</Badge> : null}
                    </div>
                    <p className="text-xs text-muted-foreground">{moduleDescription}</p>
                  </div>
                  <label className="relative inline-flex h-6 w-11 cursor-pointer items-center">
                    <input
                      aria-label={`${module.title} 开关`}
                      type="checkbox"
                      className="peer sr-only"
                      checked={isEnabled}
                      onChange={(event) => {
                        const nextChecked = event.target.checked;
                        void handleToggle(module.id, nextChecked);
                      }}
                      disabled={isBusy || isLoading}
                    />
                    <span className="h-full w-full rounded-full bg-muted transition peer-checked:bg-primary peer-disabled:opacity-60" />
                    <span className="absolute left-0.5 top-1/2 h-5 w-5 -translate-y-1/2 rounded-full bg-background shadow transition peer-checked:translate-x-5 peer-disabled:bg-muted-foreground/30" />
                  </label>
                </div>
              );
            })}
          </div>
        ) : null}

        <div className="space-y-2 rounded-2xl border border-primary/40 bg-primary/5 px-4 py-3">
          <div className="flex items-center gap-2 text-primary">
            <Sparkles className="h-4 w-4" />
            <span className="text-sm font-medium">实时预览提示</span>
          </div>
          <p className="text-xs text-primary">
            当前已启用 {enabledCount} / {totalCount} 个模块，更改后前往仪表盘即可看到最新布局。
          </p>
          <p className="text-xs text-muted-foreground">最近一次配置保存：{lastUpdatedLabel}</p>
        </div>
      </CardContent>
    </Card>
  );
};

export default DashboardSettingsForm;
