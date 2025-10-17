import { AlertTriangle, Calendar, TrendingUp } from 'lucide-react';
import { Alert, AlertDescription, AlertTitle } from '../ui/alert';
import { Badge } from '../ui/badge';
import { Card, CardContent, CardHeader, CardTitle } from '../ui/card';
import type { WorkloadForecast, ContributingTask } from '../../hooks/useWorkloadForecast';

interface WorkloadForecastBannerProps {
  forecasts: WorkloadForecast[];
}

const riskLevelConfig = {
  ok: {
    color: 'text-green-600 dark:text-green-400',
    bgColor: 'bg-green-50 dark:bg-green-950/30',
    borderColor: 'border-green-200 dark:border-green-800',
    label: '正常',
    icon: TrendingUp,
  },
  warning: {
    color: 'text-yellow-600 dark:text-yellow-400',
    bgColor: 'bg-yellow-50 dark:bg-yellow-950/30',
    borderColor: 'border-yellow-200 dark:border-yellow-800',
    label: '预警',
    icon: AlertTriangle,
  },
  critical: {
    color: 'text-red-600 dark:text-red-400',
    bgColor: 'bg-red-50 dark:bg-red-950/30',
    borderColor: 'border-red-200 dark:border-red-800',
    label: '严重',
    icon: AlertTriangle,
  },
};

const horizonLabels: Record<string, string> = {
  '7d': '未来7天',
  '14d': '未来14天',
  '30d': '未来30天',
};

export function WorkloadForecastBanner({ forecasts }: WorkloadForecastBannerProps) {
  // Only show warnings and critical forecasts
  const alertForecasts = forecasts.filter(
    (f) => f.riskLevel === 'warning' || f.riskLevel === 'critical',
  );

  if (alertForecasts.length === 0) {
    return null;
  }

  return (
    <div className="space-y-3">
      {alertForecasts.map((forecast) => {
        const config = riskLevelConfig[forecast.riskLevel as keyof typeof riskLevelConfig];
        const Icon = config.icon;
        const utilizationPercent = Math.round(
          (forecast.totalHours / forecast.capacityThreshold) * 100,
        );

        return (
          <Alert key={forecast.horizon} className={`${config.bgColor} ${config.borderColor}`}>
            <Icon className={`h-4 w-4 ${config.color}`} />
            <AlertTitle className={config.color}>
              {config.label} - {horizonLabels[forecast.horizon] || forecast.horizon}
            </AlertTitle>
            <AlertDescription className="mt-2 space-y-2">
              <div className="flex items-center gap-4 text-sm">
                <div className="flex items-center gap-1">
                  <Calendar className="h-3.5 w-3.5" />
                  <span>
                    工作量: <strong>{forecast.totalHours.toFixed(1)}h</strong> /{' '}
                    {forecast.capacityThreshold}h
                  </span>
                </div>
                <Badge
                  variant={forecast.riskLevel === 'critical' ? 'destructive' : 'secondary'}
                  className="font-mono"
                >
                  {utilizationPercent}%
                </Badge>
                <div className="text-xs text-muted-foreground">
                  置信度: {Math.round(forecast.confidence * 100)}%
                </div>
              </div>

              {forecast.recommendations.length > 0 && (
                <div className="mt-3 rounded-md bg-background/50 p-3">
                  <p className="text-xs font-medium mb-1.5">建议:</p>
                  <ul className="space-y-1 text-xs">
                    {forecast.recommendations.map((rec: string, idx: number) => (
                      <li key={idx} className="flex items-start gap-1.5">
                        <span className="text-muted-foreground">•</span>
                        <span>{rec}</span>
                      </li>
                    ))}
                  </ul>
                </div>
              )}

              {forecast.contributingTasks.length > 0 && (
                <details className="mt-2">
                  <summary className="cursor-pointer text-xs font-medium text-muted-foreground hover:text-foreground">
                    查看 {forecast.contributingTasks.length} 个相关任务
                  </summary>
                  <div className="mt-2 space-y-1">
                    {forecast.contributingTasks.slice(0, 5).map((task: ContributingTask) => (
                      <div
                        key={task.taskId}
                        className="flex items-center justify-between text-xs p-1.5 rounded hover:bg-background/80"
                      >
                        <span className="truncate flex-1">{task.title}</span>
                        <Badge variant="outline" className="ml-2 font-mono text-[10px]">
                          {task.estimatedHours.toFixed(1)}h
                        </Badge>
                      </div>
                    ))}
                    {forecast.contributingTasks.length > 5 && (
                      <p className="text-xs text-muted-foreground text-center py-1">
                        还有 {forecast.contributingTasks.length - 5} 个任务...
                      </p>
                    )}
                  </div>
                </details>
              )}
            </AlertDescription>
          </Alert>
        );
      })}
    </div>
  );
}

export function WorkloadForecastCard({ forecasts }: WorkloadForecastBannerProps) {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <TrendingUp className="h-5 w-5" />
          工作量预测
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        {forecasts.map((forecast) => {
          const config = riskLevelConfig[forecast.riskLevel as keyof typeof riskLevelConfig];
          const utilizationPercent = Math.round(
            (forecast.totalHours / forecast.capacityThreshold) * 100,
          );

          return (
            <div key={forecast.horizon} className="space-y-2">
              <div className="flex items-center justify-between">
                <span className="text-sm font-medium">
                  {horizonLabels[forecast.horizon] || forecast.horizon}
                </span>
                <Badge variant={forecast.riskLevel === 'critical' ? 'destructive' : 'secondary'}>
                  {config.label}
                </Badge>
              </div>

              <div className="space-y-1">
                <div className="flex items-center justify-between text-sm">
                  <span className="text-muted-foreground">工作量</span>
                  <span className="font-mono">
                    {forecast.totalHours.toFixed(1)}h / {forecast.capacityThreshold}h
                  </span>
                </div>

                <div className="w-full h-2 bg-secondary rounded-full overflow-hidden">
                  <div
                    className={`h-full transition-all ${
                      forecast.riskLevel === 'critical'
                        ? 'bg-red-500'
                        : forecast.riskLevel === 'warning'
                          ? 'bg-yellow-500'
                          : 'bg-green-500'
                    }`}
                    style={{ width: `${Math.min(utilizationPercent, 100)}%` }}
                  />
                </div>

                <div className="flex items-center justify-between text-xs text-muted-foreground">
                  <span>利用率: {utilizationPercent}%</span>
                  <span>置信度: {Math.round(forecast.confidence * 100)}%</span>
                </div>
              </div>

              {forecast.contributingTasks.length > 0 && (
                <div className="text-xs text-muted-foreground">
                  {forecast.contributingTasks.length} 个待完成任务
                </div>
              )}
            </div>
          );
        })}
      </CardContent>
    </Card>
  );
}
