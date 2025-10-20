import { useCallback, useMemo, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import {
  AlertTriangle,
  CalendarCheck,
  CalendarClock,
  CheckCircle2,
  ChevronDown,
  ChevronUp,
  ListTodo,
  RefreshCw,
} from 'lucide-react';
import ModuleContainer from './ModuleContainer';
import { Button } from '../ui/button';
import { Badge } from '../ui/badge';
import { Skeleton } from '../ui/skeleton';
import { useTasks } from '../../hooks/useTasks';
import { listTasks, isAppError } from '../../services/tauriApi';
import type { TaskPriority, TaskStatus } from '../../types/task';
import {
  formatTaskRelative,
  formatTaskTimeRange,
  getTaskTiming,
  type TaskTimingInfo,
} from '../../utils/taskTime';

const MAX_COLLAPSED_ITEMS = 3;
const MAX_EXPANDED_ITEMS = 6;

const STATUS_LABELS: Record<TaskStatus, string> = {
  backlog: '待整理',
  todo: '待开始',
  in_progress: '进行中',
  blocked: '受阻',
  done: '已完成',
  archived: '已归档',
};

const PRIORITY_LABELS: Record<TaskPriority, string> = {
  low: '低',
  medium: '中',
  high: '高',
  urgent: '紧急',
};

const loadingFallback = (
  <div className="space-y-4">
    <div className="grid gap-3 sm:grid-cols-3">
      {Array.from({ length: 3 }).map((_, index) => (
        <div key={index} className="rounded-2xl border border-border/60 bg-muted/20 p-4">
          <Skeleton className="h-4 w-2/3" />
          <Skeleton className="mt-3 h-7 w-16" />
        </div>
      ))}
    </div>
    <div className="space-y-2">
      {Array.from({ length: 3 }).map((_, index) => (
        <Skeleton key={`task-${index}`} className="h-16 w-full rounded-2xl" />
      ))}
    </div>
  </div>
);

const emptyState = (
  <div className="flex flex-col items-start gap-3 rounded-2xl border border-dashed border-border/60 bg-muted/20 p-6 text-sm text-muted-foreground">
    <div className="flex items-center gap-2 text-muted-foreground">
      <CheckCircle2 className="h-4 w-4" />
      <span>今日暂无已安排的任务，保持这种状态很棒！</span>
    </div>
    <Button asChild variant="outline" size="sm">
      <a href="/tasks">前往任务列表</a>
    </Button>
  </div>
);

const resolveError = (error: unknown): Error | null => {
  if (!error) return null;
  if (error instanceof Error) return error;
  if (isAppError(error)) {
    return new Error(error.message);
  }
  return new Error('加载今日任务时出现问题');
};

const TodayTasksOverview = () => {
  const navigate = useNavigate();
  const { fetchTasks, setFilters } = useTasks({ autoFetch: false });
  const [isExpanded, setIsExpanded] = useState(false);

  const { startIso, endIso } = useMemo(() => {
    const now = new Date();
    const start = new Date(now);
    start.setHours(0, 0, 0, 0);
    const end = new Date(now);
    end.setHours(23, 59, 59, 999);
    return {
      startIso: start.toISOString(),
      endIso: end.toISOString(),
    };
  }, []);

  const {
    data: todayTasks,
    isLoading,
    error,
    refetch,
    isFetching,
  } = useQuery({
    queryKey: ['dashboard', 'today-tasks', startIso, endIso],
    queryFn: async () => {
      const response = await listTasks({
        windowStart: startIso,
        windowEnd: endIso,
        includeArchived: false,
        statuses: ['todo', 'in_progress', 'blocked', 'done'],
        sortBy: 'dueAt',
        sortOrder: 'asc',
        page: 1,
        pageSize: 100,
      });
      return response.items;
    },
    staleTime: 60 * 1000,
    gcTime: 5 * 60 * 1000,
  });

  const taskEntries = useMemo(() => {
    const now = new Date();
    return (todayTasks ?? []).map((task) => ({ task, timing: getTaskTiming(task, now) }));
  }, [todayTasks]);

  const metrics = useMemo(() => {
    const pending = taskEntries.filter(
      ({ task }) => task.status !== 'done' && task.status !== 'archived',
    );
    const completed = taskEntries.filter(({ task }) => task.status === 'done').length;
    const overdue = pending.filter(({ timing }) => timing.isOverdue).length;
    const completion =
      taskEntries.length === 0 ? 0 : Math.round((completed / taskEntries.length) * 100);
    return {
      total: taskEntries.length,
      completed,
      pendingCount: pending.length,
      overdue,
      completion,
      pending,
    };
  }, [taskEntries]);

  const orderedPendingEntries = useMemo(() => {
    const score = ({ timing }: { timing: TaskTimingInfo }) => {
      if (timing.minutesUntilEnd !== null && timing.minutesUntilEnd < 0) {
        return timing.minutesUntilEnd;
      }
      if (timing.minutesUntilStart !== null && timing.minutesUntilStart >= 0) {
        return timing.minutesUntilStart;
      }
      if (timing.minutesUntilEnd !== null) {
        return timing.minutesUntilEnd;
      }
      return Number.MAX_SAFE_INTEGER;
    };

    return [...metrics.pending].sort((a, b) => score(a) - score(b));
  }, [metrics.pending]);

  const displayedEntries = useMemo(() => {
    const limit = isExpanded ? MAX_EXPANDED_ITEMS : MAX_COLLAPSED_ITEMS;
    return orderedPendingEntries.slice(0, limit);
  }, [isExpanded, orderedPendingEntries]);

  const moduleError = resolveError(error);

  const handleToggle = useCallback(() => {
    setIsExpanded((prev) => !prev);
  }, []);

  const handleViewAll = useCallback(() => {
    const nextFilters = {
      windowStart: startIso,
      windowEnd: endIso,
      statuses: ['todo', 'in_progress', 'blocked'] as TaskStatus[],
      includeArchived: false,
      sortBy: 'dueAt' as const,
      sortOrder: 'asc' as const,
      page: 1,
    };
    setFilters(nextFilters);
    void fetchTasks(nextFilters);
    navigate('/tasks');
  }, [endIso, fetchTasks, navigate, setFilters, startIso]);

  const handleRefresh = useCallback(() => {
    void refetch();
  }, [refetch]);

  const empty = metrics.total === 0;

  return (
    <ModuleContainer
      moduleId="today-tasks"
      title="今日任务概览"
      description="一目了然掌握今日分配的任务与完成进度。"
      isLoading={isLoading}
      loadingFallback={loadingFallback}
      isEmpty={empty}
      emptyState={emptyState}
      error={moduleError}
      onRetry={handleRefresh}
      actions={
        <Button variant="ghost" size="sm" onClick={handleRefresh} disabled={isFetching}>
          <RefreshCw className={`mr-1.5 h-3.5 w-3.5 ${isFetching ? 'animate-spin' : ''}`} />
          刷新
        </Button>
      }
    >
      <div className="space-y-4">
        <div className="grid gap-3 sm:grid-cols-3">
          <div className="rounded-2xl border border-border/60 bg-background/80 p-4">
            <div className="flex items-center justify-between text-xs text-muted-foreground">
              <span className="inline-flex items-center gap-1">
                <ListTodo className="h-3.5 w-3.5 text-primary" />
                待处理
              </span>
              <span>{metrics.total} 项</span>
            </div>
            <div className="mt-2 text-2xl font-semibold text-foreground">
              {metrics.pendingCount}
            </div>
            <p className="mt-1 text-xs text-muted-foreground">需要优先关注的任务</p>
          </div>
          <div className="rounded-2xl border border-border/60 bg-background/80 p-4">
            <div className="flex items-center justify-between text-xs text-muted-foreground">
              <span className="inline-flex items-center gap-1">
                <CheckCircle2 className="h-3.5 w-3.5 text-green-500" />
                已完成
              </span>
              <span>{metrics.completion}%</span>
            </div>
            <div className="mt-2 text-2xl font-semibold text-foreground">{metrics.completed}</div>
            <div className="mt-2 h-1.5 w-full overflow-hidden rounded-full bg-muted">
              <div
                className="h-full rounded-full bg-green-500 transition-all"
                style={{ width: `${metrics.completion}%` }}
              />
            </div>
            <p className="mt-1 text-xs text-muted-foreground">今日完成度</p>
          </div>
          <div className="rounded-2xl border border-border/60 bg-background/80 p-4">
            <div className="flex items-center justify-between text-xs text-muted-foreground">
              <span className="inline-flex items-center gap-1">
                <AlertTriangle className="h-3.5 w-3.5 text-destructive" />
                逾期风险
              </span>
              <span>{metrics.pendingCount} 待办</span>
            </div>
            <div className="mt-2 text-2xl font-semibold text-destructive">{metrics.overdue}</div>
            <p className="mt-1 text-xs text-muted-foreground">立即跟进即将或已逾期事项</p>
          </div>
        </div>

        {metrics.pendingCount === 0 ? (
          <div className="flex items-center gap-2 rounded-2xl border border-border/60 bg-muted/20 p-4 text-sm text-muted-foreground">
            <CheckCircle2 className="h-4 w-4 text-green-500" />
            <span>今日任务全部处理完毕，抽出时间回顾或规划后续吧。</span>
          </div>
        ) : (
          <div className="space-y-2">
            {displayedEntries.map(({ task, timing }) => {
              const { startText, endText } = formatTaskTimeRange(timing);
              const relative = formatTaskRelative(timing);
              const overdue = timing.isOverdue;
              const priority = task.priority ?? 'medium';
              return (
                <button
                  key={task.id}
                  type="button"
                  onClick={handleViewAll}
                  className="w-full rounded-2xl border border-border/60 bg-background/80 p-4 text-left transition hover:border-primary/50 hover:bg-primary/10 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary"
                >
                  <div className="flex flex-col gap-2">
                    <div className="flex flex-wrap items-center justify-between gap-2">
                      <div className="flex items-center gap-2">
                        <Badge variant="outline" className="text-xs">
                          {STATUS_LABELS[task.status] ?? task.status}
                        </Badge>
                        <span className="text-sm font-medium text-foreground">{task.title}</span>
                      </div>
                      <span
                        className={`text-xs font-medium ${overdue ? 'text-destructive' : 'text-muted-foreground'}`}
                      >
                        {relative}
                      </span>
                    </div>
                    {isExpanded && task.description ? (
                      <p className="text-xs text-muted-foreground truncate">{task.description}</p>
                    ) : null}
                    <div className="flex flex-wrap items-center gap-3 text-xs text-muted-foreground">
                      <span className="inline-flex items-center gap-1">
                        <CalendarClock className="h-3.5 w-3.5" />
                        开始 {startText}
                      </span>
                      <span className="inline-flex items-center gap-1">
                        <CalendarCheck className="h-3.5 w-3.5" />
                        截止 {endText}
                      </span>
                      {task.priority ? (
                        <span className="inline-flex items-center gap-1">
                          优先级
                          <strong>{PRIORITY_LABELS[priority] ?? priority}</strong>
                        </span>
                      ) : null}
                    </div>
                  </div>
                </button>
              );
            })}
          </div>
        )}

        <div className="flex flex-wrap items-center justify-between gap-2">
          <Button
            variant="ghost"
            className="px-0 text-sm font-medium text-primary hover:bg-transparent hover:underline"
            onClick={handleViewAll}
          >
            查看今日任务详情
          </Button>
          <Button variant="ghost" size="sm" onClick={handleToggle}>
            {isExpanded ? (
              <>
                收起详细信息
                <ChevronUp className="ml-1.5 h-4 w-4" />
              </>
            ) : (
              <>
                展开详细信息
                <ChevronDown className="ml-1.5 h-4 w-4" />
              </>
            )}
          </Button>
        </div>
      </div>
    </ModuleContainer>
  );
};

export default TodayTasksOverview;
