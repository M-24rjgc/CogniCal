import { useCallback, useMemo, useState } from 'react';
import { RefreshCw, ChevronDown, ChevronUp, Sparkles } from 'lucide-react';
import ModuleContainer from './ModuleContainer';
import { Button } from '../ui/button';
import { Badge } from '../ui/badge';
import { Skeleton } from '../ui/skeleton';
import {
  useLatestProductivityScore,
  getScoreLevel,
  getScoreColor,
  getScoreBgColor,
  formatScoreExplanation,
} from '../../hooks/useProductivityScore';
import { isAppError } from '../../services/tauriApi';

const loadingFallback = (
  <div className="space-y-4">
    <div className="flex items-center justify-between">
      <Skeleton className="h-5 w-24" />
      <Skeleton className="h-6 w-16" />
    </div>
    <div className="flex items-baseline gap-3">
      <Skeleton className="h-12 w-20" />
      <Skeleton className="h-6 w-12" />
    </div>
    <div className="space-y-2">
      {Array.from({ length: 3 }).map((_, index) => (
        <div key={index} className="space-y-1">
          <Skeleton className="h-4 w-32" />
          <Skeleton className="h-2 w-full" />
        </div>
      ))}
    </div>
  </div>
);

const emptyState = (
  <div className="flex flex-col items-start gap-3 rounded-2xl border border-dashed border-border/60 bg-muted/20 p-6 text-sm text-muted-foreground">
    <div className="flex items-center gap-2">
      <Sparkles className="h-4 w-4" />
      <span>尚未生成生产力评分，完成几项任务后即可看到智能洞察。</span>
    </div>
    <Button asChild variant="outline" size="sm">
      <a href="/tasks">前往任务中心</a>
    </Button>
  </div>
);

const resolveError = (error: unknown): Error | null => {
  if (!error) return null;
  if (error instanceof Error) return error;
  if (isAppError(error)) {
    return new Error(error.message);
  }
  return new Error('加载生产力评分时出现问题');
};

const levelBadgeCopy: Record<ReturnType<typeof getScoreLevel>, string> = {
  excellent: '表现优异',
  good: '表现良好',
  moderate: '需持续跟进',
  'needs-improvement': '亟待调整',
};

const dimensionLabels: Record<string, string> = {
  completionRate: '完成率',
  onTimeRatio: '准时率',
  focusConsistency: '专注一致性',
  restBalance: '休息平衡',
  efficiencyRating: '效率评级',
};

const resolveLevelBadgeVariant = (level: ReturnType<typeof getScoreLevel>) => {
  switch (level) {
    case 'excellent':
      return 'default' as const;
    case 'good':
      return 'secondary' as const;
    case 'moderate':
      return 'outline' as const;
    case 'needs-improvement':
      return 'destructive' as const;
    default:
      return 'outline' as const;
  }
};

const ProductivityScoreCardLite = () => {
  const { latestScore, isLoading, error, refetch } = useLatestProductivityScore();
  const [isExpanded, setIsExpanded] = useState(false);

  const score = latestScore ?? null;
  const level = score ? getScoreLevel(score.compositeScore) : null;
  const scoreColor = score ? getScoreColor(score.compositeScore) : '';
  const scoreBg = score ? getScoreBgColor(score.compositeScore) : '';
  const moduleError = resolveError(error);

  const explanation = useMemo(() => {
    if (!score) return null;
    return formatScoreExplanation(score);
  }, [score]);

  const dimensionEntries = useMemo(() => {
    if (!score) return [];
    return Object.entries(score.dimensionScores).map(([key, value]) => ({
      key,
      label: dimensionLabels[key] ?? key,
      value,
    }));
  }, [score]);

  const handleToggle = useCallback(() => {
    setIsExpanded((prev) => !prev);
  }, []);

  const handleRefresh = useCallback(() => {
    void refetch();
  }, [refetch]);

  const empty = !isLoading && !moduleError && !score;

  return (
    <ModuleContainer
      moduleId="productivity-lite"
      title="生产力评分"
      description="AI 汇总你的近期任务表现，识别优势与潜在风险。"
      isLoading={isLoading}
      loadingFallback={loadingFallback}
      isEmpty={empty}
      emptyState={emptyState}
      error={moduleError}
      onRetry={handleRefresh}
      actions={
        <Button variant="ghost" size="sm" onClick={handleRefresh}>
          <RefreshCw className="mr-1.5 h-3.5 w-3.5" />
          刷新
        </Button>
      }
    >
      {!score ? null : (
        <div className="space-y-4">
          <div className="flex flex-wrap items-center justify-between gap-3">
            <div className="flex items-center gap-3">
              <div
                className={`rounded-full px-3 py-1 text-sm font-semibold ${scoreBg} ${scoreColor}`}
              >
                {score.compositeScore.toFixed(1)} / 100
              </div>
              {level ? (
                <Badge variant={resolveLevelBadgeVariant(level)} className="text-xs">
                  {levelBadgeCopy[level]}
                </Badge>
              ) : null}
            </div>
            <div className="text-xs text-muted-foreground">
              数据更新于{' '}
              {new Date(score.snapshotDate).toLocaleDateString(undefined, {
                month: 'short',
                day: 'numeric',
              })}
            </div>
          </div>

          {explanation?.summary ? (
            <p className="text-sm text-muted-foreground leading-relaxed">{explanation.summary}</p>
          ) : null}

          <div className="space-y-3">
            <h4 className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
              核心维度
            </h4>
            <div className="space-y-2">
              {dimensionEntries.slice(0, isExpanded ? dimensionEntries.length : 3).map((item) => {
                const clamped = Math.max(0, Math.min(100, item.value));
                return (
                  <div key={item.key} className="space-y-1">
                    <div className="flex items-center justify-between text-xs">
                      <span className="text-muted-foreground">{item.label}</span>
                      <span className="font-semibold text-foreground">{clamped.toFixed(1)}%</span>
                    </div>
                    <div className="h-1.5 w-full overflow-hidden rounded-full bg-muted">
                      <div
                        className="h-full rounded-full bg-primary"
                        style={{ width: `${clamped}%` }}
                      />
                    </div>
                  </div>
                );
              })}
            </div>
          </div>

          {isExpanded && explanation ? (
            <div className="space-y-3 rounded-2xl border border-border/60 bg-muted/20 p-4">
              <div>
                <h5 className="text-xs font-semibold text-muted-foreground">优势表现</h5>
                {explanation.insights.length === 0 ? (
                  <p className="mt-1 text-xs text-muted-foreground">暂无优势备注。</p>
                ) : (
                  <ul className="mt-2 space-y-1 text-xs text-muted-foreground">
                    {explanation.insights.slice(0, 4).map((insight, index) => (
                      <li key={`insight-${index}`}>• {insight}</li>
                    ))}
                  </ul>
                )}
              </div>
              <div>
                <h5 className="text-xs font-semibold text-muted-foreground">改进建议</h5>
                {explanation.recommendations.length === 0 ? (
                  <p className="mt-1 text-xs text-muted-foreground">暂无具体建议。</p>
                ) : (
                  <ul className="mt-2 space-y-1 text-xs text-muted-foreground">
                    {explanation.recommendations.slice(0, 4).map((recommendation, index) => (
                      <li key={`recommendation-${index}`}>• {recommendation}</li>
                    ))}
                  </ul>
                )}
              </div>
            </div>
          ) : null}

          <div className="flex justify-end">
            <Button variant="ghost" size="sm" onClick={handleToggle}>
              {isExpanded ? (
                <>
                  收起详细洞察
                  <ChevronUp className="ml-1.5 h-4 w-4" />
                </>
              ) : (
                <>
                  展开详细洞察
                  <ChevronDown className="ml-1.5 h-4 w-4" />
                </>
              )}
            </Button>
          </div>
        </div>
      )}
    </ModuleContainer>
  );
};

export default ProductivityScoreCardLite;
