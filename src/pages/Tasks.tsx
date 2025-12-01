import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { Link, useLocation, useNavigate } from 'react-router-dom';
import { BarChart3, Plus, RefreshCcw, Search, Repeat } from 'lucide-react';
import { TaskDetailsDrawer } from '../components/tasks/TaskDetailsDrawer';
import { TaskFormDialog } from '../components/tasks/TaskFormDialog';
import { TaskPlanningPanel } from '../components/tasks/TaskPlanningPanel';
import { TaskTable } from '../components/tasks/TaskTable';
import { RecurringTaskForm } from '../components/tasks/RecurringTaskForm';
import { TaskInstanceManager } from '../components/tasks/TaskInstanceManager';
import { TaskInstanceEditDialog } from '../components/tasks/TaskInstanceEditDialog';
import { DependencyGraphContainer } from '../components/tasks/DependencyGraphContainer';
import { Badge } from '../components/ui/badge';
import { Button } from '../components/ui/button';
import { Input } from '../components/ui/input';
import { Label } from '../components/ui/label';
import { useTaskForm, type TaskFormValues } from '../hooks/useTaskForm';
import { useTasks } from '../hooks/useTasks';
import { useRecurringTasks } from '../hooks/useRecurringTasks';
import { FOCUS_SEARCH_EVENT_NAME } from '../hooks/useKeyboardShortcuts';
import { useAnalyticsStore } from '../stores/analyticsStore';
import { useSettingsStore } from '../stores/settingsStore';
import type { Task, TaskAISource, TaskStatus, TaskInstance, RecurringTaskTemplate } from '../types/task';
import { TASK_STATUSES } from '../types/task';
import { pushToast } from '../stores/uiStore';
import type { AnalyticsRangeKey } from '../types/analytics';

const STATUS_LABELS: Record<TaskStatus, string> = {
  backlog: '待整理',
  todo: '待开始',
  in_progress: '进行中',
  blocked: '受阻',
  done: '已完成',
  archived: '已归档',
};

export default function TasksPage() {
  const location = useLocation();
  const navigate = useNavigate();
  const {
    tasks,
    filters,
    isLoading,
    isMutating,
    lastFetchedAt,
    selectedTaskId,
    fetchTasks,
    setFilters,
    createTask,
    updateTask,
    deleteTask,
    selectTask,
  } = useTasks({ autoFetch: true });

  const [searchTerm, setSearchTerm] = useState(filters.search ?? '');
  const initialStatus = filters.statuses?.[0] ?? 'all';
  const [statusFilter, setStatusFilter] = useState<string>(initialStatus);
  const [complexityMin, setComplexityMin] = useState<string>(
    filters.complexityMin !== undefined ? String(filters.complexityMin) : '',
  );
  const [complexityMax, setComplexityMax] = useState<string>(
    filters.complexityMax !== undefined ? String(filters.complexityMax) : '',
  );
  const [selectedSources, setSelectedSources] = useState<TaskAISource[]>(filters.aiSources ?? []);
  const [isFormOpen, setIsFormOpen] = useState(false);
  const [formMode, setFormMode] = useState<'create' | 'edit'>('create');
  const [formError, setFormError] = useState<string | null>(null);
  const [isDetailsOpen, setIsDetailsOpen] = useState(false);
  const [editingTask, setEditingTask] = useState<Task | null>(null);
  const [planningTaskIds, setPlanningTaskIds] = useState<string[]>([]);
  const planningPanelRef = useRef<HTMLDivElement | null>(null);
  const searchInputRef = useRef<HTMLInputElement | null>(null);

  // Recurring task state
  const [isRecurringFormOpen, setIsRecurringFormOpen] = useState(false);
  const [recurringFormMode, setRecurringFormMode] = useState<'create' | 'edit'>('create');
  const [editingTemplate, setEditingTemplate] = useState<RecurringTaskTemplate | null>(null);
  const [recurringFormError, setRecurringFormError] = useState<string | null>(null);
  
  // Instance management state
  const [isInstanceManagerOpen, setIsInstanceManagerOpen] = useState(false);
  const [managingTemplate, setManagingTemplate] = useState<RecurringTaskTemplate | null>(null);
  const [isInstanceEditOpen, setIsInstanceEditOpen] = useState(false);
  const [editingInstance, setEditingInstance] = useState<TaskInstance | null>(null);
  const [instanceEditType, setInstanceEditType] = useState<'single' | 'series'>('single');

  const { form, resetForm, setFromTask, aiState, triggerAiParse, clearAiState } = useTaskForm();
  
  const {
    instances,
    isLoading: isRecurringLoading,
    isMutating: isRecurringMutating,
    createRecurringTask,
    updateRecurringTask,
    fetchInstances,
    updateInstance,
    deleteInstance,
    bulkUpdateInstances,
    bulkDeleteInstances,
  } = useRecurringTasks();

  const selectedTask = useMemo(
    () => tasks.find((task) => task.id === selectedTaskId) ?? null,
    [tasks, selectedTaskId],
  );

  const statusOptions = useMemo(() => ['all', ...TASK_STATUSES], []);

  const handleFilterSubmit = useCallback(
    (event: React.FormEvent<HTMLFormElement>) => {
      event.preventDefault();
      const normalizedSearch = searchTerm.trim();
      const normalizedStatus = statusFilter === 'all' ? undefined : [statusFilter as TaskStatus];
      const min = complexityMin.trim();
      const max = complexityMax.trim();
      const parsedMin = min === '' ? undefined : Number(min);
      const parsedMax = max === '' ? undefined : Number(max);
      const normalizeComplexityValue = (value: number | undefined) => {
        if (value === undefined || Number.isNaN(value)) {
          return undefined;
        }
        return Math.min(10, Math.max(0, value));
      };
      const normalizedMin = normalizeComplexityValue(parsedMin);
      const normalizedMax = normalizeComplexityValue(parsedMax);
      const normalizedSources = selectedSources.length ? selectedSources : undefined;
      const nextFilters = {
        search: normalizedSearch || undefined,
        statuses: normalizedStatus,
        complexityMin: normalizedMin,
        complexityMax: normalizedMax,
        aiSources: normalizedSources,
        page: 1,
      };
      setFilters(nextFilters);
      void fetchTasks(nextFilters);
    },
    [
      complexityMax,
      complexityMin,
      fetchTasks,
      searchTerm,
      selectedSources,
      setFilters,
      statusFilter,
    ],
  );

  const handleResetFilters = useCallback(() => {
    setSearchTerm('');
    setStatusFilter('all');
    setComplexityMin('');
    setComplexityMax('');
    setSelectedSources([]);
    const nextFilters = {
      search: undefined,
      statuses: undefined,
      complexityMin: undefined,
      complexityMax: undefined,
      aiSources: undefined,
      page: 1,
    };
    setFilters(nextFilters);
    void fetchTasks(nextFilters);
  }, [fetchTasks, setFilters]);

  useEffect(() => {
    setSearchTerm(filters.search ?? '');
    setStatusFilter(filters.statuses?.[0] ?? 'all');
    setComplexityMin(filters.complexityMin !== undefined ? String(filters.complexityMin) : '');
    setComplexityMax(filters.complexityMax !== undefined ? String(filters.complexityMax) : '');
    setSelectedSources(filters.aiSources ?? []);
  }, [filters]);

  const openCreateDialog = useCallback(() => {
    setFormMode('create');
    setEditingTask(null);
    setFormError(null);
    resetForm();
    setIsFormOpen(true);
  }, [resetForm]);

  const openEditDialog = useCallback(
    (task: Task) => {
      setFormMode('edit');
      setEditingTask(task);
      setFormError(null);
      setFromTask(task);
      setIsFormOpen(true);
    },
    [setFromTask],
  );

  const handleFormOpenChange = useCallback(
    (open: boolean) => {
      setIsFormOpen(open);
      if (!open) {
        setFormError(null);
        if (formMode === 'create') {
          resetForm();
        } else if (editingTask) {
          setFromTask(editingTask);
        }
      }
    },
    [editingTask, formMode, resetForm, setFromTask],
  );

  const handleDetailsOpenChange = useCallback((open: boolean) => {
    setIsDetailsOpen(open);
  }, []);

  const handleSelectTask = useCallback(
    (task: Task | null) => {
      if (task) {
        selectTask(task.id);
        setIsDetailsOpen(true);
      } else {
        selectTask(null);
      }
    },
    [selectTask],
  );

  const handlePlanTask = useCallback(
    (task: Task) => {
      setPlanningTaskIds((prev) => {
        if (prev.includes(task.id)) return prev;
        return [...prev, task.id];
      });
      selectTask(task.id);
      setIsDetailsOpen(true);
      pushToast({ title: `已将「${task.title}」加入规划`, variant: 'success' });
    },
    [selectTask],
  );

  const focusSearchInput = useCallback(() => {
    const node = searchInputRef.current;
    if (node) {
      node.focus();
      node.select();
    }
  }, []);

  useEffect(() => {
    const handleFocusSearch = () => {
      focusSearchInput();
    };
    window.addEventListener(FOCUS_SEARCH_EVENT_NAME, handleFocusSearch);
    return () => {
      window.removeEventListener(FOCUS_SEARCH_EVENT_NAME, handleFocusSearch);
    };
  }, [focusSearchInput]);

  useEffect(() => {
    const state = location.state as { intent?: string } | null;
    if (!state?.intent) {
      return;
    }

    if (state.intent === 'create-task') {
      openCreateDialog();
    } else if (state.intent === 'open-planning') {
      planningPanelRef.current?.scrollIntoView({ behavior: 'smooth', block: 'start' });
      pushToast({ title: '已定位至智能规划中心', variant: 'default', duration: 2500 });
    } else if (state.intent === 'focus-search') {
      focusSearchInput();
    }

    navigate(location.pathname, { replace: true });
  }, [focusSearchInput, location, navigate, openCreateDialog]);

  const handlePlanningSelectionChange = useCallback((nextIds: string[]) => {
    setPlanningTaskIds(nextIds);
  }, []);

  const handleFormSubmit = useCallback(
    async (values: TaskFormValues) => {
      setFormError(null);
      try {
        if (formMode === 'create') {
          const created = await createTask(values);
          if (created) {
            setIsFormOpen(false);
            setEditingTask(null);
            setIsDetailsOpen(true);
          }
        } else if (formMode === 'edit' && editingTask) {
          await updateTask(editingTask.id, values);
          setIsFormOpen(false);
        }
      } catch (err) {
        const message = (err as { message?: string })?.message ?? '提交失败，请稍后重试。';
        setFormError(message);
      }
    },
    [createTask, editingTask, formMode, updateTask],
  );

  const handleAiParse = useCallback(
    async (input: string) => {
      const baseContext = editingTask
        ? {
            existingTaskId: editingTask.id,
            metadata: { mode: formMode },
          }
        : {
            metadata: { mode: formMode },
          };

      return triggerAiParse(input, {
        context: baseContext,
      });
    },
    [editingTask, formMode, triggerAiParse],
  );

  const handleDeleteTask = useCallback(
    async (task: Task) => {
      const confirmed = window.confirm(`确定删除任务「${task.title}」吗？该操作不可撤销。`);
      if (!confirmed) return;
      try {
        await deleteTask(task.id);
        if (selectedTaskId === task.id) {
          selectTask(null);
          setIsDetailsOpen(false);
        }
      } catch (err) {
        // 错误已由 store 处理，避免未处理的 promise
        console.error(err);
      }
    },
    [deleteTask, selectTask, selectedTaskId],
  );

  // Recurring task handlers
  const openRecurringTaskDialog = useCallback(() => {
    setRecurringFormMode('create');
    setEditingTemplate(null);
    setRecurringFormError(null);
    setIsRecurringFormOpen(true);
  }, []);

  const handleManageRecurringInstances = useCallback(async (task: Task) => {
    if (!task.isRecurring) return;
    
    // In a real implementation, we would fetch the template and instances
    // For now, we'll create mock data
    const mockTemplate: RecurringTaskTemplate = {
      id: `template_${task.id}`,
      title: task.title,
      description: task.description,
      recurrenceRule: {
        frequency: 'daily',
        interval: 1,
        endType: 'never',
      },
      priority: task.priority,
      tags: task.tags,
      estimatedMinutes: task.estimatedMinutes,
      createdAt: task.createdAt,
      updatedAt: task.updatedAt,
      isActive: true,
    };
    
    setManagingTemplate(mockTemplate);
    await fetchInstances(mockTemplate.id);
    setIsInstanceManagerOpen(true);
  }, [fetchInstances]);

  const handleEditRecurringTemplate = useCallback((task: Task) => {
    if (!task.isRecurring) return;
    
    // In a real implementation, we would fetch the template
    const mockTemplate: RecurringTaskTemplate = {
      id: `template_${task.id}`,
      title: task.title,
      description: task.description,
      recurrenceRule: {
        frequency: 'daily',
        interval: 1,
        endType: 'never',
      },
      priority: task.priority,
      tags: task.tags,
      estimatedMinutes: task.estimatedMinutes,
      createdAt: task.createdAt,
      updatedAt: task.updatedAt,
      isActive: true,
    };
    
    setRecurringFormMode('edit');
    setEditingTemplate(mockTemplate);
    setRecurringFormError(null);
    setIsRecurringFormOpen(true);
  }, []);

  const handleRecurringFormSubmit = useCallback(async (values: any) => {
    setRecurringFormError(null);
    
    try {
      if (recurringFormMode === 'create') {
        await createRecurringTask(values);
        setIsRecurringFormOpen(false);
      } else if (recurringFormMode === 'edit' && editingTemplate) {
        await updateRecurringTask(editingTemplate.id, values);
        setIsRecurringFormOpen(false);
      }
    } catch (err) {
      const message = (err as { message?: string })?.message ?? '操作失败，请稍后重试。';
      setRecurringFormError(message);
    }
  }, [recurringFormMode, editingTemplate, createRecurringTask, updateRecurringTask]);

  const handleInstanceEdit = useCallback((instance: TaskInstance, editType: 'single' | 'series') => {
    setEditingInstance(instance);
    setInstanceEditType(editType);
    setIsInstanceEditOpen(true);
  }, []);

  const handleInstanceEditSubmit = useCallback(async (
    values: any,
    editType: 'single' | 'series',
    instanceId: string
  ) => {
    try {
      await updateInstance(instanceId, values, editType);
      setIsInstanceEditOpen(false);
      setEditingInstance(null);
    } catch (err) {
      console.error('Failed to update instance:', err);
    }
  }, [updateInstance]);

  const handleInstanceDelete = useCallback(async (instanceId: string, deleteType: 'single' | 'series') => {
    const confirmed = window.confirm(
      deleteType === 'single' 
        ? '确定删除此任务实例吗？' 
        : '确定删除整个重复任务系列吗？此操作不可撤销。'
    );
    if (!confirmed) return;
    
    try {
      await deleteInstance(instanceId, deleteType);
      if (deleteType === 'series') {
        setIsInstanceManagerOpen(false);
        setManagingTemplate(null);
      }
    } catch (err) {
      console.error('Failed to delete instance:', err);
    }
  }, [deleteInstance]);

  const lastFetchedLabel = lastFetchedAt
    ? new Date(lastFetchedAt).toLocaleString('zh-CN')
    : '尚未加载';

  const analyticsRange = useAnalyticsStore((state) => state.range);
  const analyticsLastRefreshed = useAnalyticsStore((state) => state.lastRefreshedAt);
  const analyticsIsDemo = useAnalyticsStore((state) => state.isDemoData);
  const analyticsOnboardingComplete = useAnalyticsStore((state) => state.isOnboardingComplete);
  const settingsLoading = useSettingsStore((state) => state.isLoading);
  const hasDeepseekKey = useSettingsStore((state) => state.settings?.hasDeepseekKey ?? false);

  const rangeLabels: Record<AnalyticsRangeKey, string> = {
    '7d': '近 7 天',
    '30d': '近 30 天',
    '90d': '近 90 天',
  };

  const analyticsRangeLabel = rangeLabels[analyticsRange] ?? rangeLabels['7d'];
  const analyticsHint = hasDeepseekKey
    ? analyticsLastRefreshed
      ? `最新分析于 ${new Date(analyticsLastRefreshed).toLocaleString('zh-CN')} 更新，覆盖 ${analyticsRangeLabel} 数据。`
      : '保存或完成任务后，前往分析仪表盘查看趋势与效率洞察。'
    : settingsLoading
      ? '正在加载设置，稍候即可启用智能分析。'
      : '配置 DeepSeek API Key 后，分析仪表盘可生成效率洞察与导出报告。';

  return (
    <section className="flex h-full flex-1 flex-col gap-6">
      <header className="flex flex-col gap-4 rounded-3xl border border-border/60 bg-background/80 p-6 shadow-sm backdrop-blur">
        <div className="flex flex-col gap-2 md:flex-row md:items-center md:justify-between">
          <div className="space-y-1">
            <div className="flex items-center gap-3">
              <span className="rounded-full bg-primary/10 px-3 py-1 text-xs font-semibold uppercase tracking-[0.28em] text-primary">
                Tasks
              </span>
              <Badge variant="muted" className="text-xs">
                共 {tasks.length} 条任务
              </Badge>
            </div>
            <h1 className="text-2xl font-semibold text-foreground">任务管理中心</h1>
            <p className="text-sm text-muted-foreground">
              管理任务、调整优先级、查看 AI 洞察。最近同步时间：{lastFetchedLabel}
            </p>
          </div>
          <div className="flex flex-wrap justify-end gap-2">
            <Button
              type="button"
              variant="outline"
              onClick={() => void fetchTasks(filters)}
              disabled={isLoading}
            >
              <RefreshCcw className="mr-2 h-4 w-4" /> 刷新
            </Button>
            <Button
              type="button"
              variant="outline"
              onClick={openRecurringTaskDialog}
              disabled={isMutating}
            >
              <Repeat className="mr-2 h-4 w-4" /> 重复任务
            </Button>
            <Button
              type="button"
              onClick={openCreateDialog}
              disabled={isMutating}
              data-onboarding="task-quick-create"
            >
              <Plus className="mr-2 h-4 w-4" /> 新建任务
            </Button>
          </div>
        </div>

        <form
          className="grid gap-4 rounded-2xl border border-border/60 bg-card/80 p-4 shadow-inner md:grid-cols-[minmax(0,2fr)_minmax(0,1fr)_minmax(0,1fr)_auto] md:items-end"
          onSubmit={handleFilterSubmit}
        >
          <div className="flex flex-col gap-2">
            <Label htmlFor="task-search">关键词</Label>
            <Input
              id="task-search"
              placeholder="搜索标题或描述"
              value={searchTerm}
              onChange={(event) => setSearchTerm(event.target.value)}
              ref={searchInputRef}
            />
          </div>

          <div className="flex flex-col gap-2">
            <Label htmlFor="task-status-filter">状态</Label>
            <select
              id="task-status-filter"
              className="h-10 w-full rounded-lg border border-border/60 bg-background px-3 text-sm text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary"
              value={statusFilter}
              onChange={(event) => setStatusFilter(event.target.value)}
            >
              {statusOptions.map((status) => (
                <option key={status} value={status}>
                  {status === 'all' ? '全部状态' : STATUS_LABELS[status as TaskStatus]}
                </option>
              ))}
            </select>
          </div>

          <div className="flex flex-col gap-3 rounded-xl border border-border/50 bg-muted/30 p-3">
            <div className="flex flex-col gap-2">
              <Label htmlFor="task-complexity-min">AI 复杂度</Label>
              <div className="grid grid-cols-2 gap-2">
                <Input
                  id="task-complexity-min"
                  type="number"
                  min={0}
                  max={10}
                  placeholder="下限"
                  value={complexityMin}
                  onChange={(event) => setComplexityMin(event.target.value)}
                />
                <Input
                  id="task-complexity-max"
                  type="number"
                  min={0}
                  max={10}
                  placeholder="上限"
                  value={complexityMax}
                  onChange={(event) => setComplexityMax(event.target.value)}
                />
              </div>
            </div>

            <div className="flex flex-col gap-2">
              <Label>AI 来源</Label>
              <div className="flex flex-wrap gap-3 text-sm text-muted-foreground">
                {[
                  { value: 'live' as TaskAISource, label: '实时' },
                  { value: 'cache' as TaskAISource, label: '缓存' },
                ].map((option) => {
                  const checked = selectedSources.includes(option.value);
                  return (
                    <label
                      key={option.value}
                      className="flex cursor-pointer items-center gap-2 rounded-lg border border-transparent px-2 py-1 hover:border-border/80 hover:bg-background"
                    >
                      <input
                        type="checkbox"
                        className="h-4 w-4 rounded border-border/80 text-primary focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-primary"
                        checked={checked}
                        onChange={(event) => {
                          setSelectedSources((prev) => {
                            if (event.target.checked) {
                              if (prev.includes(option.value)) {
                                return prev;
                              }
                              return [...prev, option.value];
                            }
                            return prev.filter((source) => source !== option.value);
                          });
                        }}
                      />
                      <span className="text-foreground">{option.label}</span>
                    </label>
                  );
                })}
              </div>
            </div>
          </div>

          <div className="flex gap-2">
            <Button type="submit" variant="secondary" className="flex-1">
              <Search className="mr-2 h-4 w-4" /> 应用筛选
            </Button>
            <Button type="button" variant="ghost" className="flex-1" onClick={handleResetFilters}>
              重置
            </Button>
          </div>
        </form>
      </header>

      <section className="flex flex-col gap-3 rounded-3xl border border-primary/40 bg-primary/5 p-5 text-sm text-primary">
        <header className="flex flex-wrap items-center gap-2">
          <BarChart3 className="h-4 w-4" />
          <span className="font-semibold">分析洞察联动</span>
          {analyticsIsDemo ? (
            <Badge variant="outline" className="border-primary/40 text-[11px] text-primary">
              示例数据
            </Badge>
          ) : null}
          {!analyticsOnboardingComplete ? (
            <Badge variant="outline" className="text-[11px]">
              待完成引导
            </Badge>
          ) : null}
        </header>
        <p className="text-xs text-primary/80">{analyticsHint}</p>
        <div className="flex flex-wrap gap-2 pt-1">
          <Button asChild size="sm" className="h-8 px-3 text-[12px]">
            <Link to="/">查看智能分析</Link>
          </Button>
          <Button asChild size="sm" variant="ghost" className="h-8 px-3 text-[12px]">
            <Link to="/settings">配置 AI 偏好</Link>
          </Button>
        </div>
      </section>

      <TaskTable
        tasks={tasks}
        isLoading={isLoading}
        isMutating={isMutating}
        selectedTaskId={selectedTaskId ?? undefined}
        onSelect={handleSelectTask}
        onViewDetails={(task) => handleSelectTask(task)}
        onEditTask={openEditDialog}
        onDeleteTask={handleDeleteTask}
        onPlanTask={handlePlanTask}
        onManageRecurringInstances={handleManageRecurringInstances}
        onEditRecurringTemplate={handleEditRecurringTemplate}
      />

      {/* Dependency Graph Visualization */}
      <section className="space-y-4">
        <div className="flex items-center justify-between">
          <h2 className="text-xl font-semibold">任务依赖关系图</h2>
        </div>
        <DependencyGraphContainer
          tasks={tasks}
          onTaskClick={(taskId) => {
            const task = tasks.find(t => t.id === taskId);
            if (task) handleSelectTask(task);
          }}
        />
      </section>

      <div ref={planningPanelRef}>
        <TaskPlanningPanel
          tasks={tasks}
          selectedTaskId={selectedTaskId ?? undefined}
          selectedTaskIds={planningTaskIds}
          onSelectionChange={handlePlanningSelectionChange}
          onPlanApplied={() => void fetchTasks(filters)}
        />
      </div>

      <TaskFormDialog
        open={isFormOpen}
        mode={formMode}
        form={form}
        onOpenChange={handleFormOpenChange}
        onSubmit={handleFormSubmit}
        isSubmitting={isMutating}
        serverError={formError}
        aiState={aiState}
        hasDeepseekKey={hasDeepseekKey}
        onTriggerAiParse={handleAiParse}
        onClearAiState={() => {
          clearAiState();
          pushToast({
            title: 'AI 状态已清除',
            variant: 'default',
            duration: 2500,
          });
        }}
      />

      <TaskDetailsDrawer
        open={isDetailsOpen && Boolean(selectedTask)}
        task={selectedTask}
        onOpenChange={handleDetailsOpenChange}
        onEdit={openEditDialog}
        onDelete={handleDeleteTask}
        onPlanTask={handlePlanTask}
        isMutating={isMutating}
      />

      {/* Recurring Task Dialogs */}
      <RecurringTaskForm
        open={isRecurringFormOpen}
        mode={recurringFormMode}
        template={editingTemplate || undefined}
        onOpenChange={setIsRecurringFormOpen}
        onSubmit={handleRecurringFormSubmit}
        isSubmitting={isRecurringMutating}
        serverError={recurringFormError}
      />

      <TaskInstanceManager
        open={isInstanceManagerOpen}
        template={managingTemplate || undefined}
        instances={instances}
        onOpenChange={setIsInstanceManagerOpen}
        onEditInstance={handleInstanceEdit}
        onDeleteInstance={handleInstanceDelete}
        onBulkStatusUpdate={bulkUpdateInstances}
        onBulkDelete={bulkDeleteInstances}
        isLoading={isRecurringLoading}
      />

      <TaskInstanceEditDialog
        open={isInstanceEditOpen}
        instance={editingInstance}
        template={managingTemplate || undefined}
        editType={instanceEditType}
        onOpenChange={setIsInstanceEditOpen}
        onSubmit={handleInstanceEditSubmit}
        onEditTypeChange={setInstanceEditType}
        isSubmitting={isRecurringMutating}
      />
    </section>
  );
}
