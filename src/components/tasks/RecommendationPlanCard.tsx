import { memo } from 'react';
import {
  AlertTriangle,
  CalendarClock,
  Clock,
  Sparkles,
  BadgeCheck,
  ThumbsDown,
} from 'lucide-react';
import type { RecommendationPlan } from '../../types/recommendations';
import { Badge } from '../ui/badge';
import { Button } from '../ui/button';
import { cn } from '../../lib/utils';

interface RecommendationPlanCardProps {
  plan: RecommendationPlan;
  disabled?: boolean;
  isProcessing?: boolean;
  onAccept?: (plan: RecommendationPlan) => void;
  onReject?: (plan: RecommendationPlan) => void;
  expiresAt?: string;
  taskTitles?: Record<string, string>;
}

const severityStyle: Record<string, string> = {
  low: 'bg-emerald-500/10 text-emerald-600 dark:text-emerald-400',
  medium: 'bg-amber-500/10 text-amber-600 dark:text-amber-400',
  high: 'bg-rose-500/10 text-rose-600 dark:text-rose-400',
};

const weekdayNames = ['周一', '周二', '周三', '周四', '周五', '周六', '周日'];

export const RecommendationPlanCard = memo(function RecommendationPlanCard({
  plan,
  disabled = false,
  isProcessing = false,
  onAccept,
  onReject,
  expiresAt,
  taskTitles,
}: RecommendationPlanCardProps) {
  const conflictCount = plan.conflicts.length;

  const handleAccept = () => {
    if (disabled) return;
    onAccept?.(plan);
  };

  const handleReject = () => {
    if (disabled) return;
    onReject?.(plan);
  };

  return (
    <article
      className={cn(
        'flex flex-col gap-4 rounded-3xl border-2 border-border/60 bg-card/80 p-5 text-sm shadow-sm transition hover:border-primary/40 hover:shadow-md focus:outline-none focus:ring-2 focus:ring-primary',
        disabled ? 'pointer-events-none opacity-60' : null,
      )}
    >
      <header className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex flex-col gap-1">
          <span className="text-xs font-medium uppercase tracking-wide text-muted-foreground">
            推荐方案
          </span>
          <h3 className="text-base font-semibold text-foreground">{plan.name}</h3>
          <p className="text-xs text-muted-foreground/90">{plan.description}</p>
        </div>
        <div className="flex flex-col items-end gap-2 text-xs text-muted-foreground">
          <Badge variant="secondary" className="flex items-center gap-1 bg-primary/10 text-primary">
            <Sparkles className="h-3.5 w-3.5" /> 置信度{' '}
            {plan.confidenceScore ? Math.round(plan.confidenceScore * 100) : 72}%
          </Badge>
          <Badge variant="outline" className="flex items-center gap-1 text-xs">
            <CalendarClock className="h-3.5 w-3.5" /> 时间块 {plan.timeBlocks.length}
          </Badge>
          <Badge
            variant={conflictCount ? 'destructive' : 'outline'}
            className="flex items-center gap-1 text-xs"
          >
            <AlertTriangle className="h-3.5 w-3.5" /> 冲突 {conflictCount}
          </Badge>
        </div>
      </header>

      <section className="grid gap-3">
        {plan.timeBlocks.length ? (
          <div className="grid gap-2 rounded-2xl border border-border/70 bg-background/90 p-3">
            <span className="text-xs font-semibold uppercase tracking-wide text-muted-foreground/80">
              推荐时间安排
            </span>
            <ol className="grid gap-2 text-xs text-foreground">
              {plan.timeBlocks.map((block, index) => {
                const start = new Date(block.startTime);
                const end = new Date(block.endTime);
                const weekdayName =
                  weekdayNames[start.getDay() === 0 ? 6 : start.getDay() - 1] ?? '';
                const taskTitle = block.taskId ? taskTitles?.[block.taskId] : undefined;
                const fallbackTitle = block.title?.trim().length
                  ? block.title
                  : `推荐任务 ${index + 1}`;
                const truncatedTaskId =
                  block.taskId && block.taskId.length > 12
                    ? `${block.taskId.slice(0, 12)}…`
                    : block.taskId;
                return (
                  <li
                    key={`${block.taskId ?? 'block'}-${index}`}
                    className="flex flex-wrap items-center justify-between gap-2 rounded-xl border border-border/60 bg-muted/40 p-3"
                  >
                    <div className="flex flex-col gap-1">
                      <span className="text-sm font-medium text-foreground">
                        {taskTitle ?? fallbackTitle}
                      </span>
                      <span className="text-xs text-muted-foreground">
                        {weekdayName} · {formatTimeRange(start, end)}
                      </span>
                      {taskTitle ? null : truncatedTaskId ? (
                        <span className="text-[11px] text-muted-foreground/70">
                          任务 ID · {truncatedTaskId}
                        </span>
                      ) : null}
                    </div>
                    <div className="flex flex-col items-end gap-1 text-xs text-muted-foreground/90">
                      {block.priority ? <span>优先级：{block.priority}</span> : null}
                      {typeof block.estimatedDuration === 'number' ? (
                        <span>建议时长：{block.estimatedDuration} 分钟</span>
                      ) : null}
                    </div>
                  </li>
                );
              })}
            </ol>
          </div>
        ) : (
          <div className="rounded-2xl border border-dashed border-border/60 bg-muted/70 p-4 text-xs text-muted-foreground">
            暂无详细时间块，请参考描述自主安排。
          </div>
        )}

        {plan.conflicts.length ? (
          <div className="grid gap-2 rounded-2xl border border-amber-500/40 bg-amber-500/5 p-3">
            <span className="flex items-center gap-2 text-xs font-semibold uppercase tracking-wide text-amber-600">
              <AlertTriangle className="h-3.5 w-3.5" /> 潜在冲突
            </span>
            <ul className="grid gap-1.5 text-xs text-amber-700 dark:text-amber-400">
              {plan.conflicts.slice(0, 3).map((conflict, index) => (
                <li
                  key={`${conflict.conflictType}-${index}`}
                  className="rounded-lg bg-amber-500/10 p-2"
                >
                  <Badge
                    variant="secondary"
                    className={cn('mb-1', severityStyle[conflict.severity] ?? severityStyle.medium)}
                  >
                    严重度 · {severityLabel(conflict.severity)}
                  </Badge>
                  <p>{conflict.message}</p>
                </li>
              ))}
              {plan.conflicts.length > 3 ? (
                <li className="text-[11px] text-amber-600/80">
                  ... 还有 {plan.conflicts.length - 3} 个冲突
                </li>
              ) : null}
            </ul>
          </div>
        ) : null}
      </section>

      <footer className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex flex-wrap gap-2">
          <Badge variant="outline" className="flex items-center gap-1 text-xs">
            <Clock className="h-3.5 w-3.5" /> 有效期 {formatRelative(expiresAt)}
          </Badge>
        </div>
        <div className="flex flex-wrap gap-2">
          <Button
            type="button"
            variant="ghost"
            size="sm"
            onClick={handleReject}
            disabled={disabled || isProcessing}
            className="gap-1"
          >
            <ThumbsDown className="h-3.5 w-3.5" /> 拒绝
          </Button>
          <Button
            type="button"
            size="sm"
            onClick={handleAccept}
            disabled={disabled || isProcessing}
            className="gap-1"
          >
            <BadgeCheck className="h-3.5 w-3.5" /> 接受推荐
          </Button>
        </div>
      </footer>
    </article>
  );
});

function formatTimeRange(start: Date, end: Date) {
  const formatter = new Intl.DateTimeFormat('zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
  });
  return `${formatter.format(start)} - ${formatter.format(end)}`;
}

function severityLabel(severity: string) {
  switch (severity) {
    case 'high':
      return '高';
    case 'medium':
      return '中';
    case 'low':
    default:
      return '低';
  }
}

function formatRelative(expiresAt?: string) {
  if (!expiresAt) {
    return '未知';
  }
  const expires = new Date(expiresAt);
  if (Number.isNaN(expires.getTime())) {
    return '未知';
  }
  const now = Date.now();
  const diffMs = expires.getTime() - now;
  if (diffMs <= 0) return '已过期';
  const diffMinutes = Math.round(diffMs / 60000);
  if (diffMinutes < 60) {
    return `${diffMinutes} 分钟后`;
  }
  const diffHours = Math.round(diffMinutes / 60);
  if (diffHours < 24) {
    return `${diffHours} 小时后`;
  }
  const diffDays = Math.round(diffHours / 24);
  return `${diffDays} 天后`;
}
