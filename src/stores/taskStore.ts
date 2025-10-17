import { create } from 'zustand';
import {
  createTask,
  deleteTask,
  isAppError,
  listTasks,
  toAppError,
  updateTask,
  type AppError,
} from '../services/tauriApi';
import {
  DEFAULT_PAGE_SIZE,
  type Task,
  type TaskFilters,
  type TaskListResponse,
  type TaskPayload,
  type TaskUpdatePayload,
} from '../types/task';
import { parseTaskFilters } from '../utils/validators';

interface TaskStoreState {
  tasks: Task[];
  total: number;
  filters: TaskFilters;
  isLoading: boolean;
  isMutating: boolean;
  error: AppError | null;
  selectedTaskId: string | null;
  lastFetchedAt: string | null;
  fetchTasks: (filters?: Partial<TaskFilters>) => Promise<TaskListResponse | void>;
  createTask: (payload: TaskPayload) => Promise<Task | void>;
  updateTask: (id: string, payload: TaskUpdatePayload) => Promise<Task | void>;
  deleteTask: (id: string) => Promise<void>;
  selectTask: (id: string | null) => void;
  setFilters: (filters: Partial<TaskFilters>) => void;
  clearError: () => void;
  reset: () => void;
}

const createInitialFilters = (): TaskFilters => ({
  page: 1,
  pageSize: DEFAULT_PAGE_SIZE,
});

const createInitialState = (): Omit<
  TaskStoreState,
  | 'fetchTasks'
  | 'createTask'
  | 'updateTask'
  | 'deleteTask'
  | 'selectTask'
  | 'setFilters'
  | 'clearError'
  | 'reset'
> => ({
  tasks: [],
  total: 0,
  filters: createInitialFilters(),
  isLoading: false,
  isMutating: false,
  error: null,
  selectedTaskId: null,
  lastFetchedAt: null,
});

export const useTaskStore = create<TaskStoreState>((set, get) => ({
  ...createInitialState(),
  async fetchTasks(filters = {}) {
    set({ isLoading: true, error: null });

    try {
      const currentFilters = { ...get().filters, ...filters } satisfies TaskFilters;
      const normalized = parseTaskFilters(currentFilters);
      const response = await listTasks(normalized);

      set({
        tasks: response.items,
        total: response.total,
        filters: normalized,
        isLoading: false,
        error: null,
        lastFetchedAt: new Date().toISOString(),
      });

      return response;
    } catch (error) {
      const appError = toAppError(error, '获取任务失败');
      set({ isLoading: false, error: appError });
      return undefined;
    }
  },
  async createTask(payload) {
    set({ isMutating: true, error: null });
    try {
      const task = await createTask(payload);
      if (!task) return undefined;
      set((state) => ({
        tasks: [task, ...state.tasks],
        total: state.total + 1,
        isMutating: false,
        error: null,
        selectedTaskId: task.id,
      }));
      return task;
    } catch (error) {
      const appError = toAppError(error, '创建任务失败');
      set({ isMutating: false, error: appError });
      throw appError;
    }
  },
  async updateTask(id, payload) {
    set({ isMutating: true, error: null });
    try {
      const updated = await updateTask(id, payload);
      if (!updated) return undefined;
      set((state) => ({
        tasks: state.tasks.map((task) => (task.id === updated.id ? updated : task)),
        isMutating: false,
        error: null,
        selectedTaskId: updated.id,
      }));
      return updated;
    } catch (error) {
      const appError = toAppError(error, '更新任务失败');
      set({ isMutating: false, error: appError });
      throw appError;
    }
  },
  async deleteTask(id) {
    set({ isMutating: true, error: null });
    try {
      await deleteTask(id);
      set((state) => ({
        tasks: state.tasks.filter((task) => task.id !== id),
        total: Math.max(0, state.total - 1),
        isMutating: false,
        error: null,
        selectedTaskId: state.selectedTaskId === id ? null : state.selectedTaskId,
      }));
      if (get().tasks.length === 0) {
        await get().fetchTasks();
      }
    } catch (error) {
      const appError = toAppError(error, '删除任务失败');
      set({ isMutating: false, error: appError });
      throw appError;
    }
  },
  selectTask(id) {
    set({ selectedTaskId: id });
  },
  setFilters(filters) {
    const nextFilters = parseTaskFilters({ ...get().filters, ...filters });
    set({ filters: nextFilters });
  },
  clearError() {
    set({ error: null });
  },
  reset() {
    set({ ...createInitialState() });
  },
}));

export const selectTaskById = (id: string | null) => {
  if (!id) return null;
  const state = useTaskStore.getState();
  return state.tasks.find((task) => task.id === id) ?? null;
};

export const getTaskError = () => useTaskStore.getState().error;

export const isTaskStoreError = (error: unknown) => isAppError(error);
