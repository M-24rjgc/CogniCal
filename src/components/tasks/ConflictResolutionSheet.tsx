import { AlertTriangle, CheckCircle2, Clock, Wrench } from 'lucide-react';
import { useMemo } from 'react';
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetFooter,
  SheetHeader,
  SheetTitle,
} from '../ui/sheet';
import { Badge } from '../ui/badge';
import { Button } from '../ui/button';
import {
  type PlanningOptionView,
  type PlanningSessionView,
  type ScheduleConflict,
  type TimeBlockOverride,
} from '../../types/planning';
import { pushToast } from '../../stores/uiStore';

interface ConflictResolutionSheetProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  session: PlanningSessionView | null;
  optionId: string | null;
  onResolve: (input: TimeBlockOverride[]) => Promise<void>;
  isResolving?: boolean;
}

export function ConflictResolutionSheet({
  open,
  onOpenChange,
  session,
  optionId,
  onResolve,
  isResolving = false,
}: ConflictResolutionSheetProps) {
  const option = useMemo(() => {
    if (!session || !optionId) return null;
    return session.options.find((entry) => entry.option.id === optionId) ?? null;
  }, [session, optionId]);

  const conflicts = option?.conflicts ?? [];

  const handleAutoResolve = async () => {
    if (!session || !option) return;
    const adjustments = buildAutoAdjustments(option);
    if (!adjustments.length) {
      pushToast({
        title: '暂无可自动调整项',
        description: '当前冲突需要手动处理，请检查具体冲突详情。',
        variant: 'warning',
      });
      return;
    }

    await onResolve(adjustments);
  };

  const handleMarkResolved = async () => {
    if (!session || !option) return;
    await onResolve([]);
  };

  return (
    <Sheet open={open} onOpenChange={onOpenChange}>
      <SheetContent className="flex w-full flex-col gap-4 space-y-0 sm:max-w-xl">
        <SheetHeader>
          <SheetTitle className="flex items-center gap-2 text-lg">
            <AlertTriangle className="h-5 w-5 text-rose-500" />
            冲突处理助手
          </SheetTitle>
          <SheetDescription>
            分析当前方案的冲突并尝试自动优化，也可手动记录调整建议。
          </SheetDescription>
        </SheetHeader>

        <section className="flex flex-col gap-3 rounded-2xl border border-border/70 bg-muted/40 p-4">
          <span className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
            当前方案
          </span>
          <p className="text-sm text-foreground">{option ? option.option.summary : '未选择方案'}</p>
          {session ? (
            <p className="text-xs text-muted-foreground">
              会话 #{session.session.id.slice(0, 8)} · 生成时间{' '}
              {formatDate(session.session.generatedAt)}
            </p>
          ) : null}
        </section>

        <div className="h-80 overflow-y-auto rounded-2xl border border-border/70 bg-background/60 p-4">
          <div className="grid gap-3 text-sm">
            {conflicts.length ? (
              conflicts.map((conflict, index) => (
                <ConflictCard
                  key={`${conflict.conflictType}-${index}`}
                  conflict={conflict}
                  option={option}
                />
              ))
            ) : (
              <div className="flex h-48 flex-col items-center justify-center gap-3 rounded-2xl border border-dashed border-emerald-500/40 bg-emerald-500/5 text-sm text-emerald-600">
                <CheckmarkMessage />
                <p>当前方案未检出冲突，已准备就绪。</p>
              </div>
            )}
          </div>
        </div>

        <SheetFooter className="flex flex-col gap-2 sm:flex-row sm:justify-between">
          <Button
            type="button"
            variant="secondary"
            onClick={() => onOpenChange(false)}
            disabled={isResolving}
          >
            关闭
          </Button>
          <div className="flex flex-col gap-2 sm:flex-row">
            <Button
              type="button"
              variant="outline"
              onClick={handleAutoResolve}
              disabled={isResolving || !conflicts.length}
            >
              <Wrench className="mr-2 h-4 w-4" /> 自动调整
            </Button>
            <Button
              type="button"
              onClick={handleMarkResolved}
              disabled={isResolving || !conflicts.length}
            >
              标记为已处理
            </Button>
          </div>
        </SheetFooter>
      </SheetContent>
    </Sheet>
  );
}

interface ConflictCardProps {
  conflict: ScheduleConflict;
  option: PlanningOptionView | null;
}

function ConflictCard({ conflict, option }: ConflictCardProps) {
  const relatedBlock = option?.blocks.find((block) => block.id === conflict.relatedBlockId);

  return (
    <article className="flex flex-col gap-3 rounded-2xl border border-rose-500/40 bg-rose-500/5 p-3">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <Badge variant="destructive" className="flex items-center gap-2 text-xs">
          <AlertTriangle className="h-3.5 w-3.5" /> {severityLabel(conflict.severity)} 冲突
        </Badge>
        <span className="rounded-full bg-rose-500/10 px-3 py-1 text-[11px] text-rose-500">
          {conflict.conflictType}
        </span>
      </div>
      <p className="text-sm text-foreground">{conflict.message}</p>
      {relatedBlock ? (
        <div className="flex flex-col gap-1 rounded-2xl border border-border/70 bg-background/70 p-3 text-xs text-muted-foreground">
          <span className="font-semibold text-foreground">关联时间块</span>
          <span className="flex items-center gap-2 text-muted-foreground/90">
            <Clock className="h-3.5 w-3.5" />{' '}
            {formatTimeRange(relatedBlock.startAt, relatedBlock.endAt)}
          </span>
        </div>
      ) : null}
      {conflict.relatedEventId ? (
        <div className="rounded-2xl border border-border/70 bg-background/70 p-3 text-xs text-muted-foreground">
          <span className="font-semibold text-foreground">相关事件</span>
          <p className="mt-1 text-muted-foreground/90">日程事件：{conflict.relatedEventId}</p>
        </div>
      ) : null}
    </article>
  );
}

function CheckmarkMessage() {
  return (
    <div className="flex flex-col items-center gap-2 text-emerald-600">
      <CheckCircle2 className="h-8 w-8" />
      <span className="text-sm font-medium">冲突已解决</span>
    </div>
  );
}

function buildAutoAdjustments(option: PlanningOptionView): TimeBlockOverride[] {
  const adjustmentsMap = new Map<string, TimeBlockOverride>();

  for (const conflict of option.conflicts) {
    const blockId = conflict.relatedBlockId;
    if (!blockId) continue;
    const block = option.blocks.find((entry) => entry.id === blockId);
    if (!block) continue;

    const start = new Date(block.startAt);
    const end = new Date(block.endAt);
    if (Number.isNaN(start.getTime()) || Number.isNaN(end.getTime())) continue;

    start.setMinutes(start.getMinutes() + 15);
    end.setMinutes(end.getMinutes() + 15);

    adjustmentsMap.set(blockId, {
      blockId,
      startAt: start.toISOString(),
      endAt: end.toISOString(),
      flexibility: block.flexibility ?? 'medium',
    });
  }

  return Array.from(adjustmentsMap.values());
}

function severityLabel(value: string) {
  switch (value) {
    case 'high':
      return '高';
    case 'medium':
      return '中';
    case 'low':
    default:
      return '低';
  }
}

function formatTimeRange(start: string, end: string) {
  const formatter = new Intl.DateTimeFormat('zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
  });
  return `${formatter.format(new Date(start))} - ${formatter.format(new Date(end))}`;
}

function formatDate(value: string) {
  const formatter = new Intl.DateTimeFormat('zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
    month: '2-digit',
    day: '2-digit',
  });
  return formatter.format(new Date(value));
}
