import { useCallback, useEffect, useRef } from 'react';
import type { TaskFilters, TaskPayload, TaskUpdatePayload } from '../types/task';
import { useTaskStore } from '../stores/taskStore';
import { notifyErrorToast, notifySuccessToast } from '../stores/uiStore';

interface UseTasksOptions {
  filters?: Partial<TaskFilters>;
  autoFetch?: boolean;
}

export function useTasks(options: UseTasksOptions = {}) {
  const { filters, autoFetch = true } = options;

  const tasks = useTaskStore((store) => store.tasks);
  const total = useTaskStore((store) => store.total);
  const appliedFilters = useTaskStore((store) => store.filters);
  const isLoading = useTaskStore((store) => store.isLoading);
  const isMutating = useTaskStore((store) => store.isMutating);
  const error = useTaskStore((store) => store.error);
  const selectedTaskId = useTaskStore((store) => store.selectedTaskId);
  const lastFetchedAt = useTaskStore((store) => store.lastFetchedAt);

  const fetchTasks = useTaskStore((store) => store.fetchTasks);
  const setFilters = useTaskStore((store) => store.setFilters);
  const create = useTaskStore((store) => store.createTask);
  const update = useTaskStore((store) => store.updateTask);
  const remove = useTaskStore((store) => store.deleteTask);
  const select = useTaskStore((store) => store.selectTask);
  const clearError = useTaskStore((store) => store.clearError);

  const fetchWithFilters = useCallback(
    (nextFilters?: Partial<TaskFilters>) => fetchTasks(nextFilters ?? filters ?? {}),
    [fetchTasks, filters],
  );

  const setFiltersOnly = useCallback(
    (nextFilters: Partial<TaskFilters>) => {
      setFilters(nextFilters);
    },
    [setFilters],
  );

  const selectTask = useCallback((id: string | null) => select(id), [select]);

  const lastErrorRef = useRef<typeof error>(null);

  useEffect(() => {
    if (!autoFetch) return;
    void fetchWithFilters(filters);
  }, [autoFetch, fetchWithFilters, filters]);

  useEffect(() => {
    if (!error) {
      lastErrorRef.current = null;
      return;
    }

    if (lastErrorRef.current === error) {
      return;
    }

    notifyErrorToast(error);
    lastErrorRef.current = error;
  }, [error]);

  const createTask = useCallback(
    async (payload: TaskPayload) => {
      const result = await create(payload);
      if (result) {
        notifySuccessToast('任务已创建', `「${result.title}」已添加到列表`);
      }
      return result;
    },
    [create],
  );

  const updateTask = useCallback(
    async (id: string, payload: TaskUpdatePayload) => {
      const result = await update(id, payload);
      if (result) {
        notifySuccessToast('任务已更新', `已保存「${result.title}」的最新内容`);
      }
      return result;
    },
    [update],
  );

  const deleteTask = useCallback(
    async (id: string) => {
      await remove(id);
      notifySuccessToast('任务已删除');
    },
    [remove],
  );

  return {
    tasks,
    total,
    filters: appliedFilters,
    isLoading,
    isMutating,
    error,
    selectedTaskId,
    lastFetchedAt,
    fetchTasks: fetchWithFilters,
    setFilters: setFiltersOnly,
    createTask,
    updateTask,
    deleteTask,
    selectTask,
    clearError,
  };
}
