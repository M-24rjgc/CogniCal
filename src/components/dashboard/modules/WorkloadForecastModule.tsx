import { useMemo } from 'react';
import { RefreshCw } from 'lucide-react';
import ModuleContainer from '../ModuleContainer';
import { Button } from '../../ui/button';
import { Skeleton } from '../../ui/skeleton';
import { WorkloadForecastBanner } from '../../analytics/WorkloadForecastBanner';
import { useLatestWorkloadForecasts } from '../../../hooks/useWorkloadForecast';
import { isAppError } from '../../../services/tauriApi';

const loadingFallback = (
  <div className="space-y-3">
    <Skeleton className="h-5 w-32" />
    <Skeleton className="h-24 w-full" />
    <Skeleton className="h-4 w-1/2" />
  </div>
);

const emptyState = (
  <div className="flex flex-col items-start gap-2 rounded-2xl border border-dashed border-border/60 bg-muted/20 p-6 text-sm text-muted-foreground">
    <span>未来 72 小时内未检测到工作量预警，保持当前节奏即可。</span>
  </div>
);

const resolveError = (error: unknown): Error | null => {
  if (!error) return null;
  if (error instanceof Error) return error;
  if (isAppError(error)) {
    return new Error(error.message);
  }
  return new Error('加载工作量预测时出现问题');
};

export function WorkloadForecastModule() {
  const { data, isLoading, error, refetch, isFetching } = useLatestWorkloadForecasts();

  const forecasts = useMemo(() => data ?? [], [data]);
  const moduleError = resolveError(error);
  const hasAlerts = useMemo(
    () =>
      forecasts.some(
        (forecast) => forecast.riskLevel === 'warning' || forecast.riskLevel === 'critical',
      ),
    [forecasts],
  );
  const isEmpty = !isLoading && !moduleError && !hasAlerts;

  return (
    <ModuleContainer
      moduleId="workload-forecast"
      title="工作量预警"
      description="监控未来时间段的任务容量利用情况，提前识别风险并调整计划。"
      isLoading={isLoading}
      loadingFallback={loadingFallback}
      isEmpty={isEmpty}
      emptyState={emptyState}
      error={moduleError}
      onRetry={() => {
        void refetch();
      }}
      actions={
        <Button
          type="button"
          variant="ghost"
          size="sm"
          onClick={() => {
            void refetch();
          }}
          disabled={isFetching}
        >
          <RefreshCw className={`mr-1.5 h-3.5 w-3.5 ${isFetching ? 'animate-spin' : ''}`} />
          刷新
        </Button>
      }
    >
      <WorkloadForecastBanner forecasts={forecasts} />
    </ModuleContainer>
  );
}

export default WorkloadForecastModule;
