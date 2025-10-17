import { CalendarClock, Clock, Edit3, Gauge, Sparkles, Trash2 } from 'lucide-react';
import { type ReactNode } from 'react';
import { type Task, type TaskAISource } from '../../types/task';
import { Badge } from '../ui/badge';
import { Button } from '../ui/button';
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetFooter,
  SheetHeader,
  SheetTitle,
} from '../ui/sheet';

interface TaskDetailsDrawerProps {
  task: Task | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onEdit?: (task: Task) => void;
  onDelete?: (task: Task) => void;
  onPlanTask?: (task: Task) => void;
  isMutating?: boolean;
}

export function TaskDetailsDrawer({
  task,
  open,
  onOpenChange,
  onEdit,
  onDelete,
  onPlanTask,
  isMutating = false,
}: TaskDetailsDrawerProps) {
  return (
    <Sheet open={open} onOpenChange={onOpenChange}>
      <SheetContent side="right" className="flex flex-col gap-0 p-0">
        {task ? (
          <>
            <SheetHeader>
              <div className="flex items-start justify-between gap-4">
                <div className="space-y-2">
                  <SheetTitle className="text-xl">{task.title}</SheetTitle>
                  <SheetDescription>
                    {STATUS_LABELS[task.status]} · {PRIORITY_LABELS[task.priority]}
                  </SheetDescription>
                  <div className="flex flex-wrap gap-2">
                    <Badge className={statusBadgeClass(task.status)}>
                      {STATUS_LABELS[task.status]}
                    </Badge>
                    <Badge className={priorityBadgeClass(task.priority)} variant="secondary">
                      {PRIORITY_LABELS[task.priority]}
                    </Badge>
                    {task.isRecurring ? <Badge variant="muted">循环任务</Badge> : null}
                  </div>
                </div>
                <div className="flex items-center gap-2">
                  <Button
                    type="button"
                    variant="secondary"
                    size="sm"
                    onClick={() => {
                      if (!task) return;
                      onPlanTask?.(task);
                    }}
                    disabled={isMutating}
                  >
                    <Sparkles className="mr-2 h-4 w-4" /> 加入规划
                  </Button>
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    onClick={() => task && onEdit?.(task)}
                    disabled={isMutating}
                  >
                    <Edit3 className="mr-2 h-4 w-4" /> 编辑
                  </Button>
                  <Button
                    type="button"
                    variant="ghost"
                    className="text-destructive hover:text-destructive"
                    size="sm"
                    onClick={() => task && onDelete?.(task)}
                    disabled={isMutating}
                  >
                    <Trash2 className="mr-2 h-4 w-4" /> 删除
                  </Button>
                </div>
              </div>
            </SheetHeader>

            <div className="flex-1 overflow-y-auto px-6 pb-6">
              <section className="space-y-3">
                <h4 className="text-sm font-semibold text-muted-foreground">描述</h4>
                <p className="rounded-2xl border border-border/60 bg-muted/40 px-4 py-3 text-sm leading-relaxed text-foreground">
                  {task.description ? task.description : '暂无更详细的描述信息。'}
                </p>
              </section>

              <section className="mt-6 grid gap-4 rounded-2xl border border-border/60 bg-muted/30 p-4 text-sm sm:grid-cols-2">
                <TimeBlock
                  icon={<CalendarClock className="h-4 w-4 text-primary" />}
                  label="计划时间"
                  value={formatDateRange(task.startAt, task.dueAt)}
                />
                <TimeBlock
                  icon={<Clock className="h-4 w-4 text-primary" />}
                  label="预估时长"
                  value={task.estimatedMinutes ? `${task.estimatedMinutes} 分钟` : '未填写'}
                />
                <InfoBlock label="标签">
                  {task.tags?.length ? (
                    <div className="flex flex-wrap gap-2">
                      {task.tags.map((tag) => (
                        <Badge key={tag} variant="muted" className="bg-muted/80">
                          #{tag}
                        </Badge>
                      ))}
                    </div>
                  ) : (
                    <span className="text-muted-foreground">未添加</span>
                  )}
                </InfoBlock>
                <InfoBlock label="AI 洞察">
                  {task.ai ? (
                    <div className="space-y-4 text-sm">
                      <div className="flex flex-wrap items-center gap-2 text-xs">
                        {typeof task.ai.complexityScore === 'number' ? (
                          <Badge className={aiComplexityBadgeClass(task.ai.complexityScore)}>
                            <Gauge className="mr-1 h-3 w-3" /> 复杂度{' '}
                            {task.ai.complexityScore.toFixed(1)}
                          </Badge>
                        ) : null}
                        {typeof task.ai.confidence === 'number' ? (
                          <Badge variant="outline" className="border-primary/40 text-primary">
                            置信 {Math.round(task.ai.confidence * 100)}%
                          </Badge>
                        ) : null}
                        {task.ai.source ? (
                          <Badge className={aiSourceBadgeClass(task.ai.source)}>
                            {AI_SOURCE_LABELS[task.ai.source]}
                          </Badge>
                        ) : null}
                        {task.ai.generatedAt ? (
                          <span className="rounded-full bg-muted/80 px-2 py-0.5 text-[10px] text-muted-foreground">
                            生成于 {formatDate(task.ai.generatedAt)}
                          </span>
                        ) : null}
                      </div>

                      {task.ai.summary ? (
                        <p className="rounded-xl bg-background/60 p-3 text-sm text-foreground shadow-inner">
                          {task.ai.summary}
                        </p>
                      ) : null}

                      {task.ai.nextAction ? (
                        <p className="rounded-xl border border-dashed border-primary/40 bg-primary/5 p-3 text-xs text-primary">
                          下一步：{task.ai.nextAction}
                        </p>
                      ) : null}

                      <div className="grid gap-3 text-xs text-muted-foreground/90">
                        {task.ai.suggestedStartAt ? (
                          <div className="rounded-lg border border-border/60 bg-background/70 p-3">
                            建议开始时间：{formatDate(task.ai.suggestedStartAt)}
                          </div>
                        ) : null}

                        {task.ai.focusMode ? (
                          <div className="space-y-2 rounded-lg border border-border/60 bg-background/70 p-3">
                            <div className="flex items-center gap-2 text-foreground">
                              <Sparkles className="h-3.5 w-3.5 text-primary" />
                              <span>专注模式建议 · {task.ai.focusMode.pomodoros} 个番茄钟</span>
                            </div>
                            {task.ai.focusMode.recommendedSlots?.length ? (
                              <div className="space-y-1 text-[11px]">
                                {task.ai.focusMode.recommendedSlots.map((slot, index) => (
                                  <div key={`${slot}-${index}`}>
                                    第 {index + 1} 段：{formatDate(slot)}
                                  </div>
                                ))}
                              </div>
                            ) : null}
                          </div>
                        ) : null}

                        {task.ai.efficiencyPrediction ? (
                          <div className="rounded-lg border border-border/60 bg-background/70 p-3">
                            预计耗时：{task.ai.efficiencyPrediction.expectedHours.toFixed(1)} 小时 ·
                            可信度
                            {Math.round(task.ai.efficiencyPrediction.confidence * 100)}%
                          </div>
                        ) : null}
                      </div>

                      {task.ai.cotSummary || (task.ai.cotSteps && task.ai.cotSteps.length) ? (
                        <div className="rounded-xl border border-dashed border-primary/30 bg-primary/5 p-3 text-xs text-primary">
                          推理详情暂不展示，可在后续版本中回顾完整思维链。
                        </div>
                      ) : null}
                    </div>
                  ) : (
                    <span className="text-muted-foreground">暂无 AI 建议</span>
                  )}
                </InfoBlock>
              </section>

              <section className="mt-6 grid gap-3 rounded-2xl border border-border/60 bg-muted/20 p-4 text-xs text-muted-foreground">
                <p>创建于：{formatDate(task.createdAt)}</p>
                <p>更新于：{formatDate(task.updatedAt)}</p>
                {task.completedAt ? <p>完成于：{formatDate(task.completedAt)}</p> : null}
              </section>
            </div>

            <SheetFooter>
              <Button type="button" variant="ghost" onClick={() => onOpenChange(false)}>
                关闭
              </Button>
              <Button type="button" onClick={() => task && onEdit?.(task)}>
                编辑任务
              </Button>
            </SheetFooter>
          </>
        ) : (
          <div className="flex h-full items-center justify-center px-6 text-sm text-muted-foreground">
            请选择左侧的任务查看详情。
          </div>
        )}
      </SheetContent>
    </Sheet>
  );
}

function formatDate(value?: string | null) {
  if (!value) return '暂无';
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return '未知';
  return new Intl.DateTimeFormat('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  }).format(date);
}

function formatDateRange(startAt?: string | null, dueAt?: string | null) {
  if (!startAt && !dueAt) {
    return '未设定时间';
  }
  const start = startAt ? formatDate(startAt) : '未设置开始时间';
  const due = dueAt ? formatDate(dueAt) : '未设置截止时间';
  return `${start} → ${due}`;
}

function statusBadgeClass(status: Task['status']) {
  switch (status) {
    case 'todo':
      return 'bg-sky-500/15 text-sky-600 dark:text-sky-400';
    case 'in_progress':
      return 'bg-amber-500/15 text-amber-600 dark:text-amber-400';
    case 'blocked':
      return 'bg-rose-500/15 text-rose-600 dark:text-rose-400';
    case 'done':
      return 'bg-emerald-500/15 text-emerald-600 dark:text-emerald-400';
    case 'archived':
      return 'bg-muted text-muted-foreground';
    case 'backlog':
    default:
      return 'bg-muted text-muted-foreground';
  }
}

function priorityBadgeClass(priority: Task['priority']) {
  switch (priority) {
    case 'urgent':
      return 'bg-rose-500/15 text-rose-600 dark:text-rose-400';
    case 'high':
      return 'bg-orange-500/15 text-orange-600 dark:text-orange-400';
    case 'medium':
      return 'bg-primary/15 text-primary';
    case 'low':
    default:
      return 'bg-muted text-muted-foreground';
  }
}

function aiSourceBadgeClass(source: TaskAISource) {
  switch (source) {
    case 'live':
      return 'bg-sky-500/15 text-sky-600 dark:text-sky-400';
    case 'cache':
      return 'bg-violet-500/15 text-violet-600 dark:text-violet-400';
    default:
      return 'bg-muted text-muted-foreground';
  }
}

function aiComplexityBadgeClass(score: number) {
  if (score >= 7) {
    return 'bg-rose-500/15 text-rose-600 dark:text-rose-400';
  }
  if (score >= 4) {
    return 'bg-amber-500/15 text-amber-600 dark:text-amber-400';
  }
  return 'bg-emerald-500/15 text-emerald-600 dark:text-emerald-400';
}

const STATUS_LABELS: Record<Task['status'], string> = {
  backlog: '待整理',
  todo: '待开始',
  in_progress: '进行中',
  blocked: '受阻',
  done: '已完成',
  archived: '已归档',
};

const PRIORITY_LABELS: Record<Task['priority'], string> = {
  low: '低',
  medium: '中',
  high: '高',
  urgent: '紧急',
};

const AI_SOURCE_LABELS: Record<TaskAISource, string> = {
  live: 'AI · 实时',
  cache: 'AI · 缓存',
};

function InfoBlock({ label, children }: { label: string; children: ReactNode }) {
  return (
    <div className="space-y-2">
      <span className="text-xs font-semibold uppercase tracking-wide text-muted-foreground/80">
        {label}
      </span>
      <div className="text-sm text-foreground">{children}</div>
    </div>
  );
}

function TimeBlock({ icon, label, value }: { icon: ReactNode; label: string; value: string }) {
  return (
    <div className="flex items-start gap-3 rounded-xl border border-border/50 bg-background/80 p-3">
      <div className="mt-0.5 rounded-full bg-primary/10 p-2 text-primary">{icon}</div>
      <div className="space-y-1">
        <p className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
          {label}
        </p>
        <p className="text-sm text-foreground">{value}</p>
      </div>
    </div>
  );
}
