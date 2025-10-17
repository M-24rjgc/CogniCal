import { Loader2, RefreshCw } from 'lucide-react';
import { Link } from 'react-router-dom';
import { Button } from '../ui/button';
import { Card } from '../ui/card';
import { useAnalytics } from '../../hooks/useAnalytics';
import { useProductivityScore } from '../../hooks/useProductivityScore';
import { SummaryCards } from './SummaryCards';
import { ProductivityScoreCard } from './ProductivityScoreCard';
import { ProductivityTrendChart } from './ProductivityTrendChart';
import { TimeAllocationChart } from './TimeAllocationChart';
import { EfficiencyInsights } from './EfficiencyInsights';
import { ZeroStateBanner } from './ZeroStateBanner';

const RANGE_OPTIONS = {
  '7d': '近 7 天',
  '30d': '近 30 天',
  '90d': '近 90 天',
} as const;

type RangeKey = keyof typeof RANGE_OPTIONS;

type GroupingKey = 'day' | 'week';

export function AnalyticsOverview() {
  const {
    overview,
    history,
    isLoading,
    isHistoryLoading,
    isRefetching,
    range,
    grouping,
    setRange,
    setGrouping,
    refreshOverview,
    exportReport,
    isExporting,
    lastRefreshedAt,
    isDemoData,
    exportStatus,
    isOnboardingComplete,
    completeOnboarding,
    markOnboardingIncomplete,
    loadSampleData,
  } = useAnalytics();

  const {
    score: currentScore,
    isLoading: isScoreLoading,
    error: scoreError,
    refreshScore,
  } = useProductivityScore();

  const showZeroState =
    !isLoading && overview?.zeroState?.isEmpty && !isOnboardingComplete && !isRefetching;

  const handleRangeChange = (nextRange: RangeKey) => {
    if (nextRange === range) return;
    setRange(nextRange);
  };

  const handleGroupingChange = (nextGrouping: GroupingKey) => {
    if (nextGrouping === grouping) return;
    setGrouping(nextGrouping);
  };

  return (
    <section className="flex flex-col gap-6">
      <header className="flex flex-col gap-4 rounded-3xl border border-border/60 bg-background/90 p-6 shadow-sm">
        <div className="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
          <div className="space-y-2">
            <div className="flex flex-wrap items-center gap-3">
              <div className="inline-flex items-center gap-2 rounded-full bg-gradient-to-r from-primary/10 to-primary/5 px-4 py-1.5">
                <span className="text-xs font-semibold uppercase tracking-[0.2em] text-primary">
                  智能分析与洞察
                </span>
              </div>
              {isDemoData ? (
                <span className="rounded-full bg-amber-500/15 px-3 py-1 text-[11px] font-semibold text-amber-600 dark:text-amber-400">
                  示例数据
                </span>
              ) : null}
            </div>
            <h1 className="text-3xl font-semibold text-foreground sm:text-4xl">分析仪表盘</h1>
            <p className="max-w-2xl text-sm text-muted-foreground">
              汇总任务完成率、专注时间与效率建议，帮助你在智能规划与任务执行之间形成闭环。
            </p>
          </div>
          <div className="flex flex-col items-start gap-3 text-xs text-muted-foreground sm:flex-row sm:items-center sm:text-right">
            <div className="flex items-center gap-2 rounded-full bg-muted/60 px-3 py-1">
              <span className="font-medium text-foreground/80">刷新时间</span>
              <span>
                {lastRefreshedAt ? new Date(lastRefreshedAt).toLocaleString('zh-CN') : '—'}
              </span>
            </div>
            <div className="flex items-center gap-2 text-xs">
              <Link
                to="/settings"
                className="inline-flex items-center gap-1 rounded-full border border-border/60 px-3 py-1 text-muted-foreground transition hover:bg-muted/70 hover:text-foreground"
              >
                配置 AI 服务
              </Link>
              <Button
                type="button"
                variant="outline"
                size="sm"
                disabled={isLoading}
                onClick={() => void refreshOverview()}
                className="inline-flex items-center gap-1"
              >
                {isRefetching ? (
                  <Loader2 className="h-3.5 w-3.5 animate-spin" />
                ) : (
                  <RefreshCw className="h-3.5 w-3.5" />
                )}{' '}
                探测更新
              </Button>
            </div>
          </div>
        </div>

        <div className="flex flex-wrap gap-3">
          <div className="flex items-center gap-1 rounded-full border border-border/60 bg-background/50 p-1 text-xs">
            {(Object.keys(RANGE_OPTIONS) as RangeKey[]).map((option) => (
              <button
                key={option}
                type="button"
                className={`rounded-full px-3 py-1 font-medium transition ${
                  option === range
                    ? 'bg-primary text-primary-foreground shadow'
                    : 'text-muted-foreground hover:text-foreground'
                }`}
                onClick={() => handleRangeChange(option)}
              >
                {RANGE_OPTIONS[option]}
              </button>
            ))}
          </div>

          <div className="flex items-center gap-1 rounded-full border border-border/60 bg-background/50 p-1 text-xs">
            {(['day', 'week'] as GroupingKey[]).map((option) => (
              <button
                key={option}
                type="button"
                className={`rounded-full px-3 py-1 font-medium transition ${
                  option === grouping
                    ? 'bg-secondary text-secondary-foreground shadow'
                    : 'text-muted-foreground hover:text-foreground'
                }`}
                onClick={() => handleGroupingChange(option)}
              >
                {option === 'day' ? '按日分组' : '按周分组'}
              </button>
            ))}
          </div>
        </div>
      </header>

      {showZeroState && overview ? (
        <ZeroStateBanner
          zeroState={overview.zeroState}
          onCreateTasksPath="/tasks"
          onLoadSampleData={() => loadSampleData()}
          onDismiss={() => completeOnboarding()}
          onRemindLater={() => markOnboardingIncomplete()}
        />
      ) : null}

      <SummaryCards
        summary={overview?.summary ?? null}
        meta={overview?.meta ?? null}
        zeroState={overview?.zeroState ?? null}
        isLoading={isLoading}
        onExport={() => exportReport()}
        exportStatus={exportStatus}
        isExporting={isExporting}
        rangeLabel={RANGE_OPTIONS[(range as RangeKey) ?? '7d']}
      />

      {/* Productivity Score Card */}
      <ProductivityScoreCard
        score={currentScore ?? null}
        isLoading={isScoreLoading}
        error={scoreError}
        onRefresh={refreshScore}
      />

      {/* Charts Grid - 生产力趋势图 */}
      <div className="grid gap-6">
        <ProductivityTrendChart
          analyticsData={history?.points ?? []}
          grouping={grouping as GroupingKey}
          isLoading={isLoading || isHistoryLoading}
          rangeLabel={RANGE_OPTIONS[(range as RangeKey) ?? '7d']}
        />
      </div>

      {/* Time Allocation Chart - 时间分配 */}
      <div className="grid gap-6">
        <TimeAllocationChart allocation={overview?.timeAllocation ?? null} isLoading={isLoading} />
      </div>

      {/* Efficiency Insights - 效率洞察和重点提醒 */}
      <EfficiencyInsights
        efficiency={overview?.efficiency ?? null}
        insights={overview?.insights ?? []}
        isLoading={isLoading}
      />

      <Card className="rounded-3xl border-dashed border-primary/40 bg-primary/5 p-5 text-sm text-primary">
        <p>
          💡 提示：在{' '}
          <Link to="/settings" className="underline underline-offset-4">
            设置
          </Link>{' '}
          中配置 AI 服务后，可获得更精准的智能洞察与个性化建议。
        </p>
      </Card>
    </section>
  );
}

export default AnalyticsOverview;
