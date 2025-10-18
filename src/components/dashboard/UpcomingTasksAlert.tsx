import { useCallback, useMemo, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import {
  AlarmClock,
  AlertTriangle,
  CalendarClock,
  ChevronDown,
  ChevronUp,
  Clock,
  Flame,
  RefreshCw,
} from 'lucide-react';
import ModuleContainer from './ModuleContainer';
import { Button } from '../ui/button';
import { Badge } from '../ui/badge';
import { Skeleton } from '../ui/skeleton';
import { useTasks } from '../../hooks/useTasks';
import { listTasks, isAppError } from '../../services/tauriApi';
import type { TaskPriority, TaskStatus } from '../../types/task';

const DEFAULT_THRESHOLD_HOURS = 24;
const EXPANDED_THRESHOLD_HOURS = 72;
const COLLAPSED_ITEMS = 3;
const EXPANDED_ITEMS = 6;

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
          <Skeleton className="h-4 w-1/2" />
          <Skeleton className="mt-3 h-7 w-20" />
        </div>
      ))}
    </div>
    <div className="space-y-2">
      {Array.from({ length: 3 }).map((_, index) => (
        <Skeleton key={`alert-${index}`} className="h-16 w-full rounded-2xl" />
      ))}
    </div>
  </div>
);

const emptyState = (
  <div className="flex flex-col items-start gap-3 rounded-2xl border border-dashed border-border/60 bg-muted/20 p-6 text-sm text-muted-foreground">
    <div className="flex items-center gap-2">
      <AlarmClock className="h-4 w-4" />
      <span>未来 24 小时内没有即将到期的任务，请继续保持节奏。</span>
    </div>
    <Button asChild variant="outline" size="sm">
      <a href="/tasks">浏览任务列表</a>
    </Button>
  </div>
);

const resolveError = (error: unknown): Error | null => {
  if (!error) return null;
  if (error instanceof Error) return error;
  if (isAppError(error)) {
    return new Error(error.message);
  }
  return new Error('加载到期提醒时出现问题');
};

const addHours = (date: Date, hours: number) => {
  const result = new Date(date);
  result.setTime(result.getTime() + hours * 60 * 60 * 1000);
  return result;
};

const getMinutesToDue = (iso?: string | null) => {
  if (!iso) return null;
  const parsed = Date.parse(iso);
  if (Number.isNaN(parsed)) return null;
  return Math.round((parsed - Date.now()) / 60000);
};

const formatRelative = (minutes: number | null) => {
  if (minutes === null) return '未设置截止时间';
  if (minutes <= 0) {
    const value = Math.abs(minutes);
    if (value < 60) return `${value} 分钟前到期`;
    const hours = Math.round(value / 60);
    return `${hours} 小时前到期`;
  }
  if (minutes < 60) return `${minutes} 分钟后到期`;
  const hours = Math.round(minutes / 60);
  if (hours < 24) return `${hours} 小时后到期`;
  const days = Math.round(hours / 24);
  return `${days} 天后到期`;
};

const UpcomingTasksAlert = () => {
  const navigate = useNavigate();
  const { fetchTasks, setFilters } = useTasks({ autoFetch: false });
  const [isExpanded, setIsExpanded] = useState(false);

  const thresholdHours = isExpanded ? EXPANDED_THRESHOLD_HOURS : DEFAULT_THRESHOLD_HOURS;

  const { data, isLoading, error, refetch, isFetching } = useQuery({
    queryKey: ['dashboard', 'upcoming-alerts', thresholdHours],
    queryFn: async () => {
      const now = new Date();
      const dueAfter = now.toISOString();
      const dueBefore = addHours(now, thresholdHours).toISOString();
      const response = await listTasks({
        dueAfter,
        dueBefore,
        includeArchived: false,
        statuses: ['todo', 'in_progress', 'blocked'],
        sortBy: 'dueAt',
        sortOrder: 'asc',
        page: 1,
        pageSize: 50,
      });
      return { items: response.items, dueAfter, dueBefore };
    },
    staleTime: 60 * 1000,
    gcTime: 5 * 60 * 1000,
    refetchInterval: 2 * 60 * 1000,
  });

  const tasks = useMemo(() => data?.items ?? [], [data]);

  const metrics = useMemo(() => {
    let urgent = 0;
    let critical = 0;
    tasks.forEach((task) => {
      if (task.priority === 'urgent') {
        urgent += 1;
      }
      const minutes = getMinutesToDue(task.dueAt);
      if (minutes !== null && minutes <= 120) {
        critical += 1;
      }
    });
    return {
      total: tasks.length,
      urgent,
      critical,
    };
  }, [tasks]);

  const displayedTasks = useMemo(() => {
    const limit = isExpanded ? EXPANDED_ITEMS : COLLAPSED_ITEMS;
    return tasks.slice(0, limit);
  }, [isExpanded, tasks]);

  const moduleError = resolveError(error);

  const handleToggle = useCallback(() => {
    setIsExpanded((prev) => !prev);
  }, []);

  const handleRefresh = useCallback(() => {
    void refetch();
  }, [refetch]);

  const handleViewAll = useCallback(() => {
    const dueAfter = data?.dueAfter ?? new Date().toISOString();
    const dueBefore = data?.dueBefore ?? addHours(new Date(), thresholdHours).toISOString();
    const nextFilters = {
      dueAfter,
      dueBefore,
      statuses: ['todo', 'in_progress', 'blocked'] as TaskStatus[],
      includeArchived: false,
      sortBy: 'dueAt' as const,
      sortOrder: 'asc' as const,
      page: 1,
    };
    setFilters(nextFilters);
    void fetchTasks(nextFilters);
    navigate('/tasks');
  }, [data, fetchTasks, navigate, setFilters, thresholdHours]);

  const empty = metrics.total === 0;

  return (
    <ModuleContainer
      moduleId="upcoming-alerts"
      title="即将到期提醒"
      description="跟踪未来 24 小时内可能失控的任务，及时调整节奏。"
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
                <Clock className="h-3.5 w-3.5 text-primary" />
                未来 {thresholdHours} 小时
              </span>
              <span>{metrics.total} 项</span>
            </div>
            <div className="mt-2 text-2xl font-semibold text-foreground">{metrics.total}</div>
            <p className="mt-1 text-xs text-muted-foreground">需要排定的即将到期任务</p>
          </div>
          <div className="rounded-2xl border border-border/60 bg-background/80 p-4">
            <div className="flex items-center justify-between text-xs text-muted-foreground">
              <span className="inline-flex items-center gap-1">
                <Flame className="h-3.5 w-3.5 text-destructive" />
                紧急优先级
              </span>
              <span>{metrics.urgent} 项</span>
            </div>
            <div className="mt-2 text-2xl font-semibold text-destructive">{metrics.urgent}</div>
            <p className="mt-1 text-xs text-muted-foreground">紧急优先级任务需要立即关注</p>
          </div>
          <div className="rounded-2xl border border-border/60 bg-background/80 p-4">
            <div className="flex items-center justify-between text-xs text-muted-foreground">
              <span className="inline-flex items-center gap-1">
                <AlertTriangle className="h-3.5 w-3.5 text-amber-500" />
                两小时内
              </span>
              <span>{metrics.critical} 项</span>
            </div>
            <div className="mt-2 text-2xl font-semibold text-foreground">{metrics.critical}</div>
            <p className="mt-1 text-xs text-muted-foreground">距离截止不到两小时的关键任务</p>
          </div>
        </div>

        {metrics.total === 0 ? null : (
          <div className="space-y-2">
            {displayedTasks.map((task) => {
              const minutes = getMinutesToDue(task.dueAt);
              const isCritical = minutes !== null && minutes <= 120;
              const isOverdue = minutes !== null && minutes <= 0;
              return (
                <button
                  key={task.id}
                  type="button"
                  onClick={handleViewAll}
                  className={`w-full rounded-2xl border border-border/60 bg-background/80 p-4 text-left transition hover:border-primary/50 hover:bg-primary/10 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary ${isCritical ? 'border-amber-400/60' : ''}`}
                >
                  <div className="flex flex-col gap-2">
                    <div className="flex flex-wrap items-center justify-between gap-2">
                      <div className="flex items-center gap-2">
                        <Badge
                          variant={task.priority === 'urgent' ? 'destructive' : 'outline'}
                          className="text-xs"
                        >
                          {PRIORITY_LABELS[task.priority ?? 'medium'] ?? task.priority}
                        </Badge>
                        <span className="text-sm font-medium text-foreground">{task.title}</span>
                      </div>
                      <span
                        className={`text-xs font-semibold ${isOverdue ? 'text-destructive' : 'text-muted-foreground'}`}
                      >
                        {formatRelative(minutes)}
                      </span>
                    </div>
                    {isExpanded && task.description ? (
                      <p className="text-xs text-muted-foreground truncate">{task.description}</p>
                    ) : null}
                    <div className="flex flex-wrap items-center gap-3 text-xs text-muted-foreground">
                      <span className="inline-flex items-center gap-1">
                        <CalendarClock className="h-3.5 w-3.5" />
                        {task.dueAt
                          ? new Date(task.dueAt).toLocaleTimeString([], {
                              hour: '2-digit',
                              minute: '2-digit',
                            })
                          : '未设置截止时间'}
                      </span>
                      {isCritical ? (
                        <span className="inline-flex items-center gap-1 font-semibold text-amber-600">
                          <AlarmClock className="h-3 w-3" />
                          立即准备
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
            查看到期任务详情
          </Button>
          <Button variant="ghost" size="sm" onClick={handleToggle}>
            {isExpanded ? (
              <>
                收起至 24 小时
                <ChevronUp className="ml-1.5 h-4 w-4" />
              </>
            ) : (
              <>
                展开至 72 小时
                <ChevronDown className="ml-1.5 h-4 w-4" />
              </>
            )}
          </Button>
        </div>
      </div>
    </ModuleContainer>
  );
};

export default UpcomingTasksAlert;
