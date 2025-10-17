import { Eye, MoreHorizontal, Pencil, Sparkles, Trash2 } from 'lucide-react';
import { useMemo } from 'react';
import { type Task, type TaskAISource } from '../../types/task';
import { Badge } from '../ui/badge';
import { Button } from '../ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '../ui/dropdown-menu';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '../ui/table';

interface TaskTableProps {
  tasks: Task[];
  isLoading?: boolean;
  isMutating?: boolean;
  selectedTaskId?: string | null;
  onSelect?: (task: Task | null) => void;
  onViewDetails?: (task: Task) => void;
  onEditTask?: (task: Task) => void;
  onDeleteTask?: (task: Task) => void;
  onPlanTask?: (task: Task) => void;
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

const DATE_FORMATTER = new Intl.DateTimeFormat('zh-CN', {
  month: 'short',
  day: '2-digit',
  hour: '2-digit',
  minute: '2-digit',
});

export function TaskTable({
  tasks,
  isLoading = false,
  isMutating = false,
  selectedTaskId,
  onSelect,
  onViewDetails,
  onEditTask,
  onDeleteTask,
  onPlanTask,
}: TaskTableProps) {
  const hasData = tasks.length > 0;

  const sortedTasks = useMemo(() => {
    return [...tasks].sort((a, b) => {
      const aTime = Date.parse(a.updatedAt ?? '');
      const bTime = Date.parse(b.updatedAt ?? '');
      if (Number.isNaN(aTime) || Number.isNaN(bTime)) return 0;
      return bTime - aTime;
    });
  }, [tasks]);

  return (
    <div className="overflow-hidden rounded-2xl border border-border/60 bg-card shadow-sm">
      <Table>
        <TableHeader>
          <TableRow className="bg-muted/30">
            <TableHead className="w-[30%]">任务</TableHead>
            <TableHead className="w-[12%]">状态</TableHead>
            <TableHead className="w-[10%]">优先级</TableHead>
            <TableHead className="w-[16%]">计划时间</TableHead>
            <TableHead className="w-[14%]">标签</TableHead>
            <TableHead className="w-[20%]">AI 洞察</TableHead>
            <TableHead className="w-[14%]">更新于</TableHead>
            <TableHead className="w-[5%] text-right" aria-label="操作">
              操作
            </TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {isLoading ? (
            <LoadingRows />
          ) : hasData ? (
            sortedTasks.map((task) => (
              <TaskRow
                key={task.id}
                task={task}
                isSelected={task.id === selectedTaskId}
                isMutating={isMutating}
                onSelect={onSelect}
                onViewDetails={onViewDetails}
                onEditTask={onEditTask}
                onDeleteTask={onDeleteTask}
                onPlanTask={onPlanTask}
              />
            ))
          ) : (
            <TableRow>
              <TableCell colSpan={8} className="py-12 text-center text-sm text-muted-foreground">
                当前没有任务，点击右上方的「新建任务」按钮开始吧。
              </TableCell>
            </TableRow>
          )}
        </TableBody>
      </Table>
    </div>
  );
}

interface TaskRowProps
  extends Pick<
    TaskTableProps,
    'onSelect' | 'onViewDetails' | 'onEditTask' | 'onDeleteTask' | 'onPlanTask'
  > {
  task: Task;
  isSelected: boolean;
  isMutating: boolean;
}

function TaskRow({
  task,
  isSelected,
  isMutating,
  onSelect,
  onViewDetails,
  onEditTask,
  onDeleteTask,
  onPlanTask,
}: TaskRowProps) {
  const handleSelect = () => {
    onSelect?.(task);
  };

  const handleView = () => {
    onViewDetails?.(task);
  };

  const handleEdit = () => {
    onEditTask?.(task);
  };

  const handleDelete = () => {
    onDeleteTask?.(task);
  };

  const handlePlan = () => {
    onPlanTask?.(task);
  };

  const dueAtDisplay = task.dueAt ? DATE_FORMATTER.format(new Date(task.dueAt)) : '未设置';
  const startAtDisplay = task.startAt ? DATE_FORMATTER.format(new Date(task.startAt)) : null;

  return (
    <TableRow
      data-state={isSelected ? 'selected' : undefined}
      className="cursor-pointer"
      onClick={handleSelect}
      onDoubleClick={handleView}
    >
      <TableCell>
        <div className="flex flex-col gap-1">
          <span className="text-sm font-medium text-foreground">{task.title}</span>
          {task.description ? (
            <p className="line-clamp-1 text-xs text-muted-foreground">{task.description}</p>
          ) : null}
        </div>
      </TableCell>
      <TableCell>
        <Badge className={statusBadgeClass(task.status)}>{STATUS_LABELS[task.status]}</Badge>
      </TableCell>
      <TableCell>
        <Badge className={priorityBadgeClass(task.priority)} variant="secondary">
          {PRIORITY_LABELS[task.priority]}
        </Badge>
      </TableCell>
      <TableCell>
        <div className="flex flex-col text-xs text-muted-foreground">
          {startAtDisplay ? <span>开始 · {startAtDisplay}</span> : null}
          <span className="text-foreground">截止 · {dueAtDisplay}</span>
        </div>
      </TableCell>
      <TableCell>
        {task.tags?.length ? (
          <div className="flex flex-wrap gap-1">
            {task.tags.map((tag) => (
              <Badge key={tag} variant="muted" className="bg-muted/80 text-xs">
                #{tag}
              </Badge>
            ))}
          </div>
        ) : (
          <span className="text-xs text-muted-foreground">未添加</span>
        )}
      </TableCell>
      <TableCell>
        {task.ai ? (
          <div className="flex flex-col gap-1.5 text-xs text-muted-foreground">
            <div className="flex flex-wrap items-center gap-2">
              {typeof task.ai.complexityScore === 'number' ? (
                <Badge className={aiComplexityBadgeClass(task.ai.complexityScore)}>
                  复杂度 · {task.ai.complexityScore.toFixed(1)}
                </Badge>
              ) : (
                <Badge variant="secondary" className="bg-muted text-muted-foreground">
                  复杂度未知
                </Badge>
              )}
              {task.ai.source ? (
                <Badge className={aiSourceBadgeClass(task.ai.source)}>
                  {task.ai.source === 'live' ? '实时' : '缓存'}
                </Badge>
              ) : null}
              {typeof task.ai.confidence === 'number' ? (
                <Badge variant="outline" className="border-primary/40 text-primary">
                  置信 {Math.round(task.ai.confidence * 100)}%
                </Badge>
              ) : null}
            </div>
            {task.ai.summary ? (
              <p className="line-clamp-2 leading-relaxed text-muted-foreground">
                {task.ai.summary}
              </p>
            ) : task.ai.nextAction ? (
              <p className="line-clamp-2 leading-relaxed text-muted-foreground">
                下一步：{task.ai.nextAction}
              </p>
            ) : (
              <span className="text-muted-foreground/80">暂无摘要</span>
            )}
            {task.ai.generatedAt ? (
              <span className="text-[10px] text-muted-foreground/70">
                更新于 {DATE_FORMATTER.format(new Date(task.ai.generatedAt))}
              </span>
            ) : null}
          </div>
        ) : (
          <span className="text-xs text-muted-foreground">尚未生成 AI 洞察</span>
        )}
      </TableCell>
      <TableCell>
        <span className="text-xs text-muted-foreground">
          {task.updatedAt ? DATE_FORMATTER.format(new Date(task.updatedAt)) : '未知'}
        </span>
      </TableCell>
      <TableCell className="text-right">
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button
              variant="ghost"
              size="icon"
              className="h-8 w-8"
              onClick={(event) => event.stopPropagation()}
            >
              <MoreHorizontal className="h-4 w-4" />
              <span className="sr-only">更多操作</span>
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent
            align="end"
            className="w-44"
            onCloseAutoFocus={(event) => event.preventDefault()}
          >
            <DropdownMenuItem
              onSelect={(event) => {
                event.preventDefault();
                handleView();
              }}
            >
              <Eye className="mr-2 h-4 w-4" /> 查看详情
            </DropdownMenuItem>
            <DropdownMenuItem
              onSelect={(event) => {
                event.preventDefault();
                handlePlan();
              }}
            >
              <Sparkles className="mr-2 h-4 w-4 text-primary" /> 加入规划
            </DropdownMenuItem>
            <DropdownMenuItem
              disabled={isMutating}
              onSelect={(event) => {
                event.preventDefault();
                handleEdit();
              }}
            >
              <Pencil className="mr-2 h-4 w-4" /> 编辑任务
            </DropdownMenuItem>
            <DropdownMenuItem
              className="text-destructive focus:text-destructive"
              disabled={isMutating}
              onSelect={(event) => {
                event.preventDefault();
                handleDelete();
              }}
            >
              <Trash2 className="mr-2 h-4 w-4" /> 删除
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </TableCell>
    </TableRow>
  );
}

function LoadingRows() {
  return (
    <>
      {Array.from({ length: 3 }).map((_, index) => (
        <TableRow key={index}>
          <TableCell colSpan={8}>
            <div className="flex animate-pulse flex-col gap-3">
              <div className="h-5 w-2/3 rounded bg-muted/70" />
              <div className="grid grid-cols-4 gap-3">
                <span className="h-4 rounded bg-muted/60" />
                <span className="h-4 rounded bg-muted/60" />
                <span className="h-4 rounded bg-muted/60" />
                <span className="h-4 rounded bg-muted/60" />
              </div>
            </div>
          </TableCell>
        </TableRow>
      ))}
    </>
  );
}

function statusBadgeClass(status: Task['status']) {
  switch (status) {
    case 'backlog':
      return 'bg-muted text-muted-foreground';
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
    default:
      return '';
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
