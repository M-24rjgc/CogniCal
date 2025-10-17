import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { AppError } from '../services/tauriApi';
import { useTaskStore } from '../stores/taskStore';
import type { Task, TaskListResponse } from '../types/task';

const { listTasksMock, createTaskMock, updateTaskMock, deleteTaskMock } = vi.hoisted(() => ({
  listTasksMock: vi.fn(),
  createTaskMock: vi.fn(),
  updateTaskMock: vi.fn(),
  deleteTaskMock: vi.fn(),
}));

vi.mock('../services/tauriApi', () => ({
  listTasks: listTasksMock,
  createTask: createTaskMock,
  updateTask: updateTaskMock,
  deleteTask: deleteTaskMock,
  toAppError: (error: unknown, fallback: string) => {
    if (error && typeof error === 'object' && 'code' in (error as Record<string, unknown>)) {
      return error as AppError;
    }
    return { code: 'UNKNOWN', message: fallback } satisfies AppError;
  },
  isAppError: (error: unknown) =>
    Boolean(error && typeof error === 'object' && 'code' in (error as Record<string, unknown>)),
}));

describe('taskStore', () => {
  beforeEach(() => {
    useTaskStore.getState().reset();
    listTasksMock.mockReset();
    createTaskMock.mockReset();
    updateTaskMock.mockReset();
    deleteTaskMock.mockReset();
  });

  it('loads tasks successfully', async () => {
    const task: Task = {
      id: 'demo-1',
      title: 'Demo Task',
      description: 'Testing fetch',
      status: 'todo',
      priority: 'medium',
      plannedStartAt: new Date().toISOString(),
      startAt: undefined,
      dueAt: undefined,
      completedAt: undefined,
      estimatedMinutes: 30,
      estimatedHours: 0.5,
      tags: [],
      ownerId: undefined,
      taskType: 'work',
      isRecurring: false,
      recurrence: undefined,
      ai: {
        summary: 'AI summary',
        complexityScore: 4.5,
        source: 'cache',
        generatedAt: new Date().toISOString(),
      },
      externalLinks: [],
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    };

    const response: TaskListResponse = {
      items: [task],
      total: 1,
      page: 1,
      pageSize: 20,
    };

    listTasksMock.mockResolvedValue(response);

    const store = useTaskStore.getState();
    await store.fetchTasks();

    const next = useTaskStore.getState();
    expect(next.tasks).toHaveLength(1);
    expect(next.tasks[0]).toMatchObject({ id: 'demo-1', title: 'Demo Task' });
    expect(next.tasks[0].ai?.summary).toBe('AI summary');
    expect(next.tasks[0].estimatedHours).toBe(0.5);
    expect(next.tasks[0].taskType).toBe('work');
    expect(next.total).toBe(1);
    expect(next.error).toBeNull();
  });

  it('captures errors from listTasks', async () => {
    listTasksMock.mockRejectedValue({
      code: 'NETWORK',
      message: 'network failed' satisfies string,
    });

    const store = useTaskStore.getState();
    await store.fetchTasks();

    const next = useTaskStore.getState();
    expect(next.error).not.toBeNull();
    expect(next.error?.code).toBe('NETWORK');
    expect(next.isLoading).toBe(false);
  });

  it('appends task on create', async () => {
    const created: Task = {
      id: 'created-1',
      title: 'Create Task',
      description: undefined,
      status: 'todo',
      priority: 'high',
      plannedStartAt: undefined,
      startAt: undefined,
      dueAt: undefined,
      completedAt: undefined,
      estimatedMinutes: 45,
      estimatedHours: 1,
      tags: [],
      ownerId: undefined,
      taskType: 'study',
      isRecurring: false,
      recurrence: undefined,
      ai: {
        generatedAt: new Date().toISOString(),
        source: 'live',
      },
      externalLinks: [],
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    };

    createTaskMock.mockResolvedValue(created);

    const store = useTaskStore.getState();
    await store.createTask({
      title: created.title,
      status: created.status,
      priority: created.priority,
      tags: [],
      isRecurring: false,
    });

    const next = useTaskStore.getState();
    expect(next.tasks[0].id).toBe('created-1');
    expect(next.selectedTaskId).toBe('created-1');
    expect(next.total).toBe(1);
  });

  it('updates tasks with new ai metadata', async () => {
    const initial: Task = {
      id: 'task-1',
      title: 'Original',
      description: undefined,
      status: 'todo',
      priority: 'medium',
      plannedStartAt: undefined,
      startAt: undefined,
      dueAt: undefined,
      completedAt: undefined,
      estimatedMinutes: 30,
      estimatedHours: undefined,
      tags: [],
      ownerId: undefined,
      taskType: 'other',
      isRecurring: false,
      recurrence: undefined,
      ai: undefined,
      externalLinks: [],
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    };

    const updated: Task = {
      ...initial,
      title: 'Updated',
      ai: {
        summary: 'AI Updated',
        nextAction: 'Do it',
        complexityScore: 7.2,
        source: 'cache',
        generatedAt: new Date().toISOString(),
      },
      updatedAt: new Date().toISOString(),
    };

    listTasksMock.mockResolvedValue({
      items: [initial],
      total: 1,
      page: 1,
      pageSize: 20,
    });

    updateTaskMock.mockResolvedValue(updated);

    const store = useTaskStore.getState();
    await store.fetchTasks();

    await store.updateTask(initial.id, { ai: updated.ai, title: updated.title });

    const next = useTaskStore.getState();
    expect(next.tasks[0].ai?.summary).toBe('AI Updated');
    expect(next.tasks[0].title).toBe('Updated');
    expect(next.selectedTaskId).toBe(updated.id);
  });

  it('propagates errors from mutations', async () => {
    createTaskMock.mockRejectedValue({
      code: 'VALIDATION_ERROR',
      message: 'invalid payload' satisfies string,
    });

    const store = useTaskStore.getState();

    await expect(
      store.createTask({
        title: '',
        status: 'todo',
        priority: 'medium',
        tags: [],
        isRecurring: false,
      }),
    ).rejects.toMatchObject({ code: 'VALIDATION_ERROR' });

    const next = useTaskStore.getState();
    expect(next.isMutating).toBe(false);
    expect(next.error?.code).toBe('VALIDATION_ERROR');
  });
});
