import { useMemo, useState } from 'react';
import { Link } from 'react-router-dom';
import { AlertTriangle, BarChart3, Calendar as CalendarIcon, Layers3 } from 'lucide-react';
import { usePlanning } from '../hooks/usePlanning';
import { useTasks } from '../hooks/useTasks';
import { Badge } from '../components/ui/badge';
import { Button } from '../components/ui/button';
import type { PlanningOptionView } from '../types/planning';
import type { Task } from '../types/task';
import { useAnalyticsStore } from '../stores/analyticsStore';
import { useSettingsStore } from '../stores/settingsStore';
import { HelpPopover } from '../components/help/HelpPopover';
import { CalendarView } from '../components/calendar/CalendarView';
import { DayDetailPanel } from '../components/calendar/DayDetailPanel';
import { TaskDetailsDrawer } from '../components/tasks/TaskDetailsDrawer';
import { extractDateKey, formatDateKey } from '../utils/date';

const TIME_FORMATTER = new Intl.DateTimeFormat('zh-CN', {
  hour: '2-digit',
  minute: '2-digit',
});

export default function CalendarPage() {
  const { session, selectedOption } = usePlanning({
    autoAttachEvents: true,
    autoLoadPreferences: false,
  });
  const { tasks, deleteTask, selectTask, selectedTaskId } = useTasks({
    autoFetch: true,
  });

  const [selectedDate, setSelectedDate] = useState<Date | null>(null);
  const [isDetailsDrawerOpen, setIsDetailsDrawerOpen] = useState(false);

  const taskTitleMap = useMemo(() => {
    const map: Record<string, string> = {};
    for (const task of tasks) {
      map[task.id] = task.title;
    }
    return map;
  }, [tasks]);

  const planningBlocks = selectedOption?.blocks ?? [];

  const selectedTask = useMemo(
    () => tasks.find((task) => task.id === selectedTaskId) ?? null,
    [tasks, selectedTaskId],
  );

  const selectedDateTasks = useMemo(() => {
    if (!selectedDate) return [];
    const selectedKey = formatDateKey(selectedDate);
    return tasks.filter((task) => {
      const dueKey = task.dueAt ? extractDateKey(task.dueAt) : null;
      const startKey = task.startAt ? extractDateKey(task.startAt) : null;
      return dueKey === selectedKey || startKey === selectedKey;
    });
  }, [selectedDate, tasks]);

  const selectedDateBlocks = useMemo(() => {
    if (!selectedDate) return [];
    const selectedKey = formatDateKey(selectedDate);
    const blocks = selectedOption?.blocks ?? [];
    return blocks.filter((block) => extractDateKey(block.startAt) === selectedKey);
  }, [selectedDate, selectedOption]);

  const conflicts = selectedOption?.conflicts ?? session?.conflicts ?? [];
  const conflictCount = conflicts.length;

  const selectedLabel = selectedOption?.option.summary ?? '当前方案';
  const statusSummary = session?.session.status === 'applied' ? '已生效' : '待应用';
  const updatedAt = session ? new Date(session.session.updatedAt) : null;

  const analyticsLastRefreshed = useAnalyticsStore((state) => state.lastRefreshedAt);
  const analyticsIsDemo = useAnalyticsStore((state) => state.isDemoData);
  const hasDeepseekKey = useSettingsStore((state) => state.settings?.hasDeepseekKey ?? false);

  const analyticsSummaryCopy = analyticsLastRefreshed
    ? `最新仪表盘数据更新于 ${new Date(analyticsLastRefreshed).toLocaleString('zh-CN')}。`
    : hasDeepseekKey
      ? '完成规划后，可前往仪表盘查看专注时间与冲突洞察。'
      : '配置 DeepSeek API Key 才能启用智能分析与冲突洞察。';

  const handleDateClick = (date: Date) => {
    setSelectedDate((prev) => {
      if (!prev) {
        return date;
      }
      return formatDateKey(prev) === formatDateKey(date) ? null : date;
    });
  };

  const handleTaskClick = (task: Task) => {
    selectTask(task.id);
    setIsDetailsDrawerOpen(true);
  };

  const handleBlockClick = (block: PlanningOptionView['blocks'][number]) => {
    const task = tasks.find((t) => t.id === block.taskId);
    if (task) {
      selectTask(task.id);
      setIsDetailsDrawerOpen(true);
    }
  };

  const handleDeleteTask = async (task: Task) => {
    const confirmed = window.confirm(`确定删除任务「${task.title}」吗？该操作不可撤销。`);
    if (!confirmed) return;
    try {
      await deleteTask(task.id);
      if (selectedTaskId === task.id) {
        selectTask(null);
        setIsDetailsDrawerOpen(false);
      }
    } catch (err) {
      console.error(err);
    }
  };

  return (
    <section className="flex h-full flex-1 flex-col gap-6">
      <header className="flex flex-col gap-3 rounded-3xl border border-border/60 bg-background/80 p-6 shadow-sm">
        <div className="flex flex-wrap items-center justify-between gap-4">
          <div className="space-y-1">
            <div className="flex items-center gap-2">
              <Badge variant="secondary" className="bg-secondary/15 text-xs">
                <CalendarIcon className="mr-1.5 h-3.5 w-3.5" /> 日历视图
              </Badge>
              {session && (
                <Badge variant="outline" className="text-xs">
                  <Layers3 className="mr-1.5 h-3 w-3" />
                  {selectedLabel}
                </Badge>
              )}
            </div>
            <div className="flex items-center gap-2">
              <h1 className="text-2xl font-semibold text-foreground">任务日历</h1>
              <HelpPopover
                entryId="planning-center"
                triggerLabel="查看日历帮助说明"
                triggerClassName="ml-1"
              />
            </div>
            <p className="text-sm text-muted-foreground">
              月视图展示任务和规划时间块，点击日期查看详情。
            </p>
          </div>
          <div className="flex flex-col items-end gap-2 text-xs text-muted-foreground">
            {session && (
              <>
                <span>状态：{statusSummary}</span>
                {updatedAt && <span>更新：{TIME_FORMATTER.format(updatedAt)}</span>}
              </>
            )}
            <div className="flex items-center gap-2">
              <span>{tasks.length} 个任务</span>
              <span>·</span>
              <span>{planningBlocks.length} 个时间块</span>
            </div>
          </div>
        </div>
      </header>

      {/* 分析联动提示 */}
      {(analyticsIsDemo || !hasDeepseekKey) && (
        <section className="flex flex-col gap-2 rounded-3xl border border-primary/40 bg-primary/5 p-5 text-sm text-primary">
          <header className="flex flex-wrap items-center gap-2">
            <BarChart3 className="h-4 w-4" />
            <span className="font-semibold">规划 & 分析联动</span>
            {analyticsIsDemo && (
              <Badge variant="outline" className="border-primary/40 text-[11px] text-primary">
                示例数据
              </Badge>
            )}
          </header>
          <p className="text-xs text-primary/80">{analyticsSummaryCopy}</p>
          <div className="flex flex-wrap gap-2 pt-1">
            <Button asChild size="sm" className="h-8 px-3 text-[12px]">
              <Link to="/">查看智能分析</Link>
            </Button>
            <Button asChild size="sm" variant="ghost" className="h-8 px-3 text-[12px]">
              <Link to="/settings">配置 AI 偏好</Link>
            </Button>
          </div>
        </section>
      )}

      {/* 冲突提示 */}
      {conflictCount > 0 && (
        <div className="flex items-center gap-3 rounded-2xl border border-amber-500/40 bg-amber-500/10 p-4 text-sm text-amber-700">
          <AlertTriangle className="h-5 w-5 shrink-0" />
          <div className="flex-1">
            <span className="font-semibold">检测到 {conflictCount} 个时间冲突</span>
            <p className="text-xs text-amber-600 mt-1">
              前往任务中心的规划面板可以调整时间或解决冲突
            </p>
          </div>
          <Button asChild size="sm" variant="outline" className="border-amber-500/40">
            <Link to="/tasks">前往处理</Link>
          </Button>
        </div>
      )}

      {/* 日历主视图 */}
      <div className="grid gap-6 lg:grid-cols-[2fr_1fr]">
        <CalendarView
          tasks={tasks}
          planningBlocks={planningBlocks}
          onDateClick={handleDateClick}
          onTaskClick={handleTaskClick}
          onBlockClick={handleBlockClick}
          selectedDate={selectedDate}
        />

        {/* 侧边栏 */}
        <div className="space-y-4">
          {/* 选中日期详情 */}
          {selectedDate && (
            <DayDetailPanel
              date={selectedDate}
              tasks={selectedDateTasks}
              blocks={selectedDateBlocks}
              taskTitles={taskTitleMap}
              onClose={() => setSelectedDate(null)}
              onTaskClick={handleTaskClick}
              onBlockClick={handleBlockClick}
            />
          )}

          {/* 统计信息 */}
          {!selectedDate && (
            <div className="rounded-3xl border border-border/60 bg-card/80 p-5 shadow-sm space-y-4">
              <h3 className="text-sm font-semibold text-foreground">本月概览</h3>
              <div className="grid gap-3 text-sm">
                <div className="flex items-center justify-between">
                  <span className="text-muted-foreground">总任务数</span>
                  <Badge variant="secondary">{tasks.length}</Badge>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-muted-foreground">已完成</span>
                  <Badge variant="secondary" className="bg-emerald-500/15 text-emerald-600">
                    {tasks.filter((t) => t.status === 'done').length}
                  </Badge>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-muted-foreground">进行中</span>
                  <Badge variant="secondary" className="bg-amber-500/15 text-amber-600">
                    {tasks.filter((t) => t.status === 'in_progress').length}
                  </Badge>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-muted-foreground">规划时间块</span>
                  <Badge variant="secondary" className="bg-primary/15 text-primary">
                    {planningBlocks.length}
                  </Badge>
                </div>
              </div>
            </div>
          )}
        </div>
      </div>

      {/* 任务详情抽屉 */}
      <TaskDetailsDrawer
        open={isDetailsDrawerOpen && Boolean(selectedTask)}
        task={selectedTask}
        onOpenChange={setIsDetailsDrawerOpen}
        onEdit={() => {
          // 可以添加编辑功能
        }}
        onDelete={handleDeleteTask}
        onPlanTask={() => {
          // 可以添加规划功能
        }}
        isMutating={false}
      />
    </section>
  );
}
