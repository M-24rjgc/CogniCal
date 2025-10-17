import { ArrowUpRight, Download, Timer, TrendingUp, Unplug } from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '../ui/card';
import { Button } from '../ui/button';
import { Skeleton } from '../ui/skeleton';
import { type AnalyticsSummary, type ZeroStateMeta } from '../../types/analytics';
import { type AnalyticsExportStatus } from '../../stores/analyticsStore';
import { cn } from '../../lib/utils';

interface SummaryCardsProps {
  summary: AnalyticsSummary | null;
  meta: { generatedAt: string; isDemo: boolean } | null;
  zeroState: ZeroStateMeta | null;
  isLoading: boolean;
  onExport: () => void;
  exportStatus: AnalyticsExportStatus;
  isExporting: boolean;
  rangeLabel: string;
}

export function SummaryCards({
  summary,
  meta,
  zeroState,
  isLoading,
  onExport,
  exportStatus,
  isExporting,
  rangeLabel,
}: SummaryCardsProps) {
  const metrics = createMetricList(summary, rangeLabel);

  return (
    <div className="space-y-4">
      {/* 指标卡片 - 4列网格 */}
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        {metrics.map((metric) => (
          <Card key={metric.id} className={cn('relative overflow-hidden', metric.accent)}>
            <CardHeader className="space-y-2">
              <div className="flex items-center justify-between text-xs uppercase tracking-[0.28em] text-muted-foreground">
                <span>{metric.label}</span>
                {metric.icon}
              </div>
              <CardTitle className="text-3xl font-semibold">
                {isLoading ? <Skeleton className="h-9 w-24" /> : metric.value}
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-1 text-sm text-muted-foreground">
              {isLoading ? <Skeleton className="h-4 w-3/4" /> : <span>{metric.description}</span>}
              {metric.deltaLabel ? (
                <div className="flex items-center gap-1 text-xs text-emerald-600 dark:text-emerald-400">
                  <TrendingUp className="h-3 w-3" />
                  <span>{metric.deltaLabel}</span>
                </div>
              ) : null}
            </CardContent>
          </Card>
        ))}
      </div>

      {/* 操作卡片 - 单独一行 */}
      <Card>
        <div className="flex flex-col gap-4 p-6 sm:flex-row sm:items-center sm:justify-between">
          <div className="space-y-1">
            <CardTitle className="text-lg">快速操作</CardTitle>
            <p className="text-sm text-muted-foreground">
              最新生成时间：{meta ? new Date(meta.generatedAt).toLocaleString('zh-CN') : '—'}
            </p>
            {zeroState?.isEmpty ? (
              <p className="flex items-center gap-2 text-xs text-amber-600 dark:text-amber-400">
                <Unplug className="h-3.5 w-3.5" /> 尚未导入真实任务数据，建议先完成任务与规划。
              </p>
            ) : null}
          </div>
          <div className="flex items-center gap-3">
            <Button
              type="button"
              variant="outline"
              size="sm"
              disabled={isExporting}
              onClick={onExport}
              className="inline-flex items-center gap-2"
            >
              {isExporting ? (
                <span className="h-3 w-3 animate-spin rounded-full border-2 border-primary border-t-transparent" />
              ) : (
                <Download className="h-4 w-4" />
              )}{' '}
              导出报告
            </Button>
            <span className="text-xs text-muted-foreground">
              {exportStatus === 'success'
                ? '✓ 已生成'
                : exportStatus === 'loading'
                  ? '⏳ 生成中'
                  : '待生成'}
            </span>
          </div>
        </div>
      </Card>
    </div>
  );
}

function createMetricList(summary: AnalyticsSummary | null, rangeLabel: string) {
  const valueOrPlaceholder = (
    value: number | null | undefined,
    formatter?: (value: number) => string,
  ) => {
    if (value === null || value === undefined || Number.isNaN(value)) return '—';
    return formatter ? formatter(value) : value.toLocaleString('zh-CN');
  };

  return [
    {
      id: 'completion-rate',
      label: '完成率',
      value: valueOrPlaceholder(summary?.completionRate, (value) => `${Math.round(value * 100)}%`),
      description: `${rangeLabel}内任务平均完成率`,
      deltaLabel: summary
        ? `趋势变化 ${summary.trendDelta >= 0 ? '+' : ''}${summary.trendDelta.toFixed(1)}%`
        : null,
      icon: <ArrowUpRight className="h-4 w-4 text-primary" />,
      accent: 'bg-gradient-to-br from-primary/5 via-background to-background',
    },
    {
      id: 'total-completed',
      label: '完成任务',
      value: valueOrPlaceholder(summary?.totalCompleted),
      description: `${rangeLabel}内的完成数量`,
      deltaLabel: null,
      icon: <TrendingUp className="h-4 w-4 text-emerald-500" />,
      accent: 'bg-gradient-to-br from-emerald-500/5 via-background to-background',
    },
    {
      id: 'focus-minutes',
      label: '专注时长',
      value: valueOrPlaceholder(summary?.focusMinutes, (value) => `${Math.round(value / 60)} 小时`),
      description: `${rangeLabel}内记录的专注时间`,
      deltaLabel: null,
      icon: <Timer className="h-4 w-4 text-sky-500" />,
      accent: 'bg-gradient-to-br from-sky-500/5 via-background to-background',
    },
    {
      id: 'overdue-tasks',
      label: '逾期任务',
      value: valueOrPlaceholder(summary?.overdueTasks),
      description: '需重点关注的逾期数',
      deltaLabel: null,
      icon: <ArrowUpRight className="h-4 w-4 rotate-45 text-rose-500" />,
      accent: 'bg-gradient-to-br from-rose-500/5 via-background to-background',
    },
  ];
}
