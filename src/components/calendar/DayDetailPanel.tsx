import { Clock3, ListChecks, Sparkles, X } from 'lucide-react';
import { Button } from '../ui/button';
import { Badge } from '../ui/badge';
import type { Task } from '../../types/task';
import type { PlanningOptionView } from '../../types/planning';

interface DayDetailPanelProps {
  date: Date;
  tasks: Task[];
  blocks: PlanningOptionView['blocks'];
  taskTitles: Record<string, string>;
  onClose: () => void;
  onTaskClick?: (task: Task) => void;
  onBlockClick?: (block: PlanningOptionView['blocks'][number]) => void;
}

const TIME_FORMATTER = new Intl.DateTimeFormat('zh-CN', {
  hour: '2-digit',
  minute: '2-digit',
});

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

export function DayDetailPanel({
  date,
  tasks,
  blocks,
  taskTitles,
  onClose,
  onTaskClick,
  onBlockClick,
}: DayDetailPanelProps) {
  const dateLabel = new Intl.DateTimeFormat('zh-CN', {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
    weekday: 'long',
  }).format(date);

  const sortedBlocks = [...blocks].sort(
    (a, b) => new Date(a.startAt).getTime() - new Date(b.startAt).getTime(),
  );

  return (
    <div className="flex flex-col gap-4 rounded-3xl border border-border/60 bg-card/90 p-6 shadow-lg">
      {/* 头部 */}
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-semibold text-foreground">{dateLabel}</h3>
          <p className="text-sm text-muted-foreground">
            {tasks.length} 个任务 · {blocks.length} 个时间块
          </p>
        </div>
        <Button variant="ghost" size="icon" onClick={onClose}>
          <X className="h-4 w-4" />
        </Button>
      </div>

      {/* 内容区域 */}
      <div className="grid gap-4 max-h-[600px] overflow-y-auto">
        {/* 规划时间块 */}
        {blocks.length > 0 && (
          <section className="space-y-3">
            <div className="flex items-center gap-2 text-sm font-semibold text-foreground">
              <Sparkles className="h-4 w-4 text-primary" />
              <span>规划时间块</span>
            </div>
            <div className="space-y-2">
              {sortedBlocks.map((block) => {
                const taskTitle = taskTitles[block.taskId] ?? `任务 ${block.taskId}`;
                const start = new Date(block.startAt);
                const end = new Date(block.endAt);
                const duration = Math.round((end.getTime() - start.getTime()) / 60000);

                return (
                  <div
                    key={block.id}
                    className="rounded-2xl border border-primary/40 bg-primary/5 p-3 cursor-pointer hover:bg-primary/10 transition"
                    onClick={() => onBlockClick?.(block)}
                  >
                    <div className="flex items-start justify-between gap-2">
                      <div className="flex-1 space-y-1">
                        <div className="font-medium text-foreground">{taskTitle}</div>
                        <div className="flex items-center gap-2 text-xs text-muted-foreground">
                          <Clock3 className="h-3 w-3" />
                          <span>
                            {TIME_FORMATTER.format(start)} - {TIME_FORMATTER.format(end)}
                          </span>
                          <span className="text-muted-foreground/70">({duration} 分钟)</span>
                        </div>
                      </div>
                      <div className="flex flex-col gap-1">
                        {typeof block.confidence === 'number' && (
                          <Badge variant="outline" className="text-[10px]">
                            置信 {Math.round(block.confidence * 100)}%
                          </Badge>
                        )}
                        {block.flexibility && (
                          <Badge variant="secondary" className="text-[10px]">
                            {block.flexibility}
                          </Badge>
                        )}
                      </div>
                    </div>
                  </div>
                );
              })}
            </div>
          </section>
        )}

        {/* 任务列表 */}
        {tasks.length > 0 && (
          <section className="space-y-3">
            <div className="flex items-center gap-2 text-sm font-semibold text-foreground">
              <ListChecks className="h-4 w-4 text-sky-600" />
              <span>任务列表</span>
            </div>
            <div className="space-y-2">
              {tasks.map((task) => (
                <div
                  key={task.id}
                  className="rounded-2xl border border-border/60 bg-background/80 p-3 cursor-pointer hover:border-primary/40 hover:bg-primary/5 transition"
                  onClick={() => onTaskClick?.(task)}
                >
                  <div className="flex items-start justify-between gap-2">
                    <div className="flex-1 space-y-1">
                      <div className="font-medium text-foreground">{task.title}</div>
                      {task.description && (
                        <p className="text-xs text-muted-foreground line-clamp-2">
                          {task.description}
                        </p>
                      )}
                      {task.dueAt && (
                        <div className="flex items-center gap-2 text-xs text-muted-foreground">
                          <Clock3 className="h-3 w-3" />
                          <span>截止 {TIME_FORMATTER.format(new Date(task.dueAt))}</span>
                        </div>
                      )}
                    </div>
                    <div className="flex flex-col gap-1">
                      <Badge className={statusBadgeClass(task.status)} variant="secondary">
                        {STATUS_LABELS[task.status]}
                      </Badge>
                      <Badge className={priorityBadgeClass(task.priority)} variant="outline">
                        {PRIORITY_LABELS[task.priority]}
                      </Badge>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </section>
        )}

        {/* 空状态 */}
        {tasks.length === 0 && blocks.length === 0 && (
          <div className="flex flex-col items-center justify-center gap-2 rounded-2xl border border-dashed border-border/60 bg-muted/30 p-8 text-center text-sm text-muted-foreground">
            <span>这一天还没有任务或规划</span>
            <span className="text-xs">点击日期可以添加新任务</span>
          </div>
        )}
      </div>
    </div>
  );
}

function statusBadgeClass(status: Task['status']) {
  switch (status) {
    case 'done':
      return 'bg-emerald-500/15 text-emerald-600 dark:text-emerald-400';
    case 'in_progress':
      return 'bg-amber-500/15 text-amber-600 dark:text-amber-400';
    case 'blocked':
      return 'bg-rose-500/15 text-rose-600 dark:text-rose-400';
    case 'todo':
      return 'bg-sky-500/15 text-sky-600 dark:text-sky-400';
    default:
      return 'bg-muted text-muted-foreground';
  }
}

function priorityBadgeClass(priority: Task['priority']) {
  switch (priority) {
    case 'urgent':
      return 'border-rose-500/40 text-rose-600 dark:text-rose-400';
    case 'high':
      return 'border-orange-500/40 text-orange-600 dark:text-orange-400';
    case 'medium':
      return 'border-primary/40 text-primary';
    default:
      return 'border-muted text-muted-foreground';
  }
}
