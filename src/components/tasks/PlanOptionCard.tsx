import {
  AlertTriangle,
  CheckCircle2,
  Clock,
  Sparkles,
  CalendarClock,
  Split,
  RefreshCw,
} from 'lucide-react';
import { type PlanningOptionView } from '../../types/planning';
import { cn } from '../../lib/utils';
import { Badge } from '../ui/badge';
import { Button } from '../ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '../ui/dropdown-menu';

interface PlanOptionCardProps {
  option: PlanningOptionView;
  isSelected?: boolean;
  disabled?: boolean;
  onSelect?: (optionId: string) => void;
  onApply?: (option: PlanningOptionView) => void;
  onShowConflicts?: (option: PlanningOptionView) => void;
  onAdjustTime?: (option: PlanningOptionView) => void;
  onSplitTask?: (option: PlanningOptionView) => void;
  onReplaceTask?: (option: PlanningOptionView) => void;
  onReject?: (option: PlanningOptionView) => void;
  taskTitles?: Record<string, string>;
}

const severityStyle: Record<string, string> = {
  low: 'bg-emerald-500/10 text-emerald-600 dark:text-emerald-400',
  medium: 'bg-amber-500/10 text-amber-600 dark:text-amber-400',
  high: 'bg-rose-500/10 text-rose-600 dark:text-rose-400',
};

const weekdayNames = ['周一', '周二', '周三', '周四', '周五', '周六', '周日'];

export function PlanOptionCard({
  option,
  isSelected = false,
  disabled = false,
  onSelect,
  onApply,
  onShowConflicts,
  onAdjustTime,
  onSplitTask,
  onReplaceTask,
  onReject,
  taskTitles,
}: PlanOptionCardProps) {
  const conflictCount = option.conflicts.length;
  const hasConflicts = conflictCount > 0;

  const handleSelect = () => {
    onSelect?.(option.option.id);
  };

  const handleApply = (event: React.MouseEvent<HTMLButtonElement>) => {
    event.stopPropagation();
    if (disabled) return;
    onApply?.(option);
  };

  const handleShowConflicts = (event: React.MouseEvent<HTMLButtonElement>) => {
    event.stopPropagation();
    if (disabled) return;
    onShowConflicts?.(option);
  };

  const handleAdjustTime = (event: React.MouseEvent) => {
    event.stopPropagation();
    if (disabled) return;
    onAdjustTime?.(option);
  };

  const handleSplitTask = (event: React.MouseEvent) => {
    event.stopPropagation();
    if (disabled) return;
    onSplitTask?.(option);
  };

  const handleReplaceTask = (event: React.MouseEvent) => {
    event.stopPropagation();
    if (disabled) return;
    onReplaceTask?.(option);
  };

  const handleReject = (event: React.MouseEvent) => {
    event.stopPropagation();
    if (disabled) return;
    onReject?.(option);
  };

  return (
    <article
      className={cn(
        'group flex flex-col gap-4 rounded-3xl border-2 border-border/60 bg-card/80 p-5 text-sm shadow-sm transition hover:border-primary/40 hover:shadow-md focus:outline-none focus:ring-2 focus:ring-primary',
        isSelected ? 'border-primary/60 ring-2 ring-primary' : null,
        disabled ? 'pointer-events-none opacity-50' : null,
      )}
    >
      <header className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex flex-col gap-1">
          <span className="text-xs font-medium uppercase tracking-wide text-muted-foreground">
            方案 #{option.option.rank}
          </span>
          <h3 className="text-base font-semibold text-foreground">{option.option.summary}</h3>
          <div className="flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
            <Badge variant="secondary" className="bg-primary/10 text-primary">
              评分 {option.option.score?.toFixed(2) ?? '未知'}
            </Badge>
            {option.option.isFallback ? (
              <Badge variant="outline" className="border-amber-400/60 text-amber-600">
                备选方案
              </Badge>
            ) : (
              <Badge variant="outline" className="border-emerald-400/60 text-emerald-600">
                推荐方案
              </Badge>
            )}
            <span className="rounded-full bg-muted/60 px-2 py-0.5 text-[11px]">
              共 {option.blocks.length} 个时间块
            </span>
          </div>
        </div>
        <div className="flex flex-col items-end gap-2 text-xs text-muted-foreground">
          <Button
            type="button"
            size="sm"
            variant={isSelected ? 'default' : 'secondary'}
            onClick={handleSelect}
            disabled={disabled}
          >
            {isSelected ? '当前选中' : '设为当前'}
          </Button>
          <Badge variant="secondary" className="flex items-center gap-1 bg-muted text-xs">
            <Clock className="h-3.5 w-3.5" /> {formatDate(option.option.createdAt)}
          </Badge>
          {hasConflicts ? (
            <Badge variant="destructive" className="flex items-center gap-1 text-xs">
              <AlertTriangle className="h-3.5 w-3.5" /> 冲突 {conflictCount} 项
            </Badge>
          ) : (
            <Badge variant="outline" className="flex items-center gap-1 text-xs">
              <CheckCircle2 className="h-3.5 w-3.5 text-emerald-500" /> 冲突已处理
            </Badge>
          )}
        </div>
      </header>

      <section className="grid gap-3 text-sm text-muted-foreground">
        <div className="grid gap-2 rounded-2xl border border-border/70 bg-background/90 p-3">
          <span className="text-xs font-semibold uppercase tracking-wide text-muted-foreground/80">
            时间块安排
          </span>
          <ol className="grid gap-2 text-xs text-foreground">
            {option.blocks.map((block, index) => {
              const start = new Date(block.startAt);
              const end = new Date(block.endAt);
              const taskTitle = taskTitles?.[block.taskId];
              const fallbackTitle = `任务 ${index + 1}`;
              const truncatedTaskId =
                block.taskId.length > 12 ? `${block.taskId.slice(0, 12)}…` : block.taskId;
              return (
                <li
                  key={block.id}
                  className="flex flex-wrap items-center justify-between gap-2 rounded-xl border border-border/60 bg-muted/40 p-3"
                >
                  <div className="flex flex-col gap-1">
                    <span className="text-sm font-medium">{taskTitle ?? fallbackTitle}</span>
                    <span className="text-xs text-muted-foreground">
                      {weekdayNames[start.getDay() === 0 ? 6 : start.getDay() - 1]} ·
                      {formatTimeRange(start, end)}
                    </span>
                    {taskTitle ? null : (
                      <span className="text-[11px] text-muted-foreground/70">
                        任务 ID · {truncatedTaskId}
                      </span>
                    )}
                  </div>
                  <div className="flex flex-col items-end gap-1 text-xs text-muted-foreground/90">
                    <span>灵活性：{block.flexibility ?? '中等'}</span>
                    <span>
                      置信度：{block.confidence ? Math.round(block.confidence * 100) : 70}%
                    </span>
                  </div>
                </li>
              );
            })}
          </ol>
        </div>

        {option.option.cotSteps?.length ? (
          <div className="grid gap-2 rounded-2xl border border-dashed border-primary/40 bg-primary/5 p-3">
            <span className="flex items-center gap-2 text-xs font-semibold uppercase tracking-wide text-primary">
              <Sparkles className="h-3.5 w-3.5" /> 规划思路
            </span>
            <ol className="grid gap-2 text-xs text-primary/90">
              {option.option.cotSteps.slice(0, 3).map((step, index) => (
                <li key={`${step.step}-${index}`} className="rounded-lg bg-primary/10 p-2">
                  <span className="font-medium">步骤 {step.step ?? index + 1}：</span>{' '}
                  {step.thought}{' '}
                  {step.result ? <em className="text-primary/70">→ {step.result}</em> : null}
                </li>
              ))}
              {option.option.cotSteps.length > 3 ? (
                <li className="text-[11px] text-primary/60">
                  ... 共 {option.option.cotSteps.length} 条推理步骤
                </li>
              ) : null}
            </ol>
          </div>
        ) : null}

        {option.option.riskNotes?.notes?.length ? (
          <div className="grid gap-2 rounded-2xl border border-amber-500/40 bg-amber-500/5 p-3">
            <span className="flex items-center gap-2 text-xs font-semibold uppercase tracking-wide text-amber-600">
              <AlertTriangle className="h-3.5 w-3.5" /> 风险提示
            </span>
            <ul className="grid gap-1.5 text-xs text-amber-700">
              {option.option.riskNotes.notes.map((note, index) => (
                <li key={`${note}-${index}`} className="rounded-lg bg-amber-500/10 p-2">
                  {note}
                </li>
              ))}
            </ul>
          </div>
        ) : null}

        {hasConflicts ? (
          <div className="grid gap-2 rounded-2xl border border-rose-500/40 bg-rose-500/5 p-3">
            <span className="text-xs font-semibold uppercase tracking-wide text-rose-600">
              冲突详情
            </span>
            <ul className="grid gap-2 text-xs text-rose-600">
              {option.conflicts.slice(0, 3).map((conflict, index) => (
                <li
                  key={`${conflict.conflictType}-${index}`}
                  className="rounded-lg bg-rose-500/10 p-2"
                >
                  <Badge
                    variant="secondary"
                    className={cn('mb-1', severityStyle[conflict.severity])}
                  >
                    严重度 · {severityLabel(conflict.severity)}
                  </Badge>
                  <p>{conflict.message}</p>
                </li>
              ))}
              {option.conflicts.length > 3 ? (
                <li className="text-[11px] text-rose-500/70">
                  ... 还有 {option.conflicts.length - 3} 个冲突
                </li>
              ) : null}
            </ul>
          </div>
        ) : null}
      </section>

      <footer className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex flex-wrap gap-2">
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={handleShowConflicts}
            disabled={!hasConflicts || disabled}
          >
            查看冲突
          </Button>
          {hasConflicts && (onAdjustTime || onSplitTask || onReplaceTask) ? (
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  disabled={disabled}
                  className="gap-1"
                >
                  <AlertTriangle className="h-3.5 w-3.5" />
                  解决冲突
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="start" className="w-48">
                {onAdjustTime ? (
                  <DropdownMenuItem onClick={handleAdjustTime} className="gap-2">
                    <CalendarClock className="h-4 w-4" />
                    <div className="flex flex-col gap-0.5">
                      <span className="font-medium">调整时间</span>
                      <span className="text-xs text-muted-foreground">修改任务的计划时间段</span>
                    </div>
                  </DropdownMenuItem>
                ) : null}
                {onSplitTask ? (
                  <DropdownMenuItem onClick={handleSplitTask} className="gap-2">
                    <Split className="h-4 w-4" />
                    <div className="flex flex-col gap-0.5">
                      <span className="font-medium">拆分任务</span>
                      <span className="text-xs text-muted-foreground">将任务拆分为多个子任务</span>
                    </div>
                  </DropdownMenuItem>
                ) : null}
                {onReplaceTask ? (
                  <DropdownMenuItem onClick={handleReplaceTask} className="gap-2">
                    <RefreshCw className="h-4 w-4" />
                    <div className="flex flex-col gap-0.5">
                      <span className="font-medium">替换任务</span>
                      <span className="text-xs text-muted-foreground">用其他任务替换此任务</span>
                    </div>
                  </DropdownMenuItem>
                ) : null}
              </DropdownMenuContent>
            </DropdownMenu>
          ) : null}
        </div>
        <div className="flex flex-wrap gap-2">
          {onReject ? (
            <Button
              type="button"
              variant="ghost"
              size="sm"
              onClick={handleReject}
              disabled={disabled}
            >
              拒绝
            </Button>
          ) : null}
          <Button type="button" size="sm" onClick={handleApply} disabled={disabled}>
            应用方案
          </Button>
        </div>
      </footer>
    </article>
  );
}

function formatDate(value: string | undefined) {
  if (!value) return '未知时间';
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return '未知时间';
  return new Intl.DateTimeFormat('zh-CN', {
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  }).format(date);
}

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
