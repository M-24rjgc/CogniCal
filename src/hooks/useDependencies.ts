import { useCallback, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  TaskDependency,
  DependencyCreateInput,
  DependencyValidation,
  DependencyGraph,
  DependencyFilter,
  ReadyTask,
  DependencyType,
} from '../types/dependency';
import { pushToast } from '../stores/uiStore';

interface UseDependenciesReturn {
  dependencies: TaskDependency[];
  dependencyGraph: DependencyGraph | null;
  readyTasks: ReadyTask[];
  isLoading: boolean;
  isMutating: boolean;

  // Actions
  fetchDependencies: (filter?: DependencyFilter) => Promise<void>;
  fetchDependencyGraph: (filter?: DependencyFilter) => Promise<void>;
  fetchReadyTasks: () => Promise<void>;
  createDependency: (input: DependencyCreateInput) => Promise<TaskDependency | null>;
  deleteDependency: (dependencyId: string) => Promise<void>;
  updateDependencyType: (dependencyId: string, newType: DependencyType) => Promise<void>;
  validateDependency: (predecessorId: string, successorId: string) => Promise<DependencyValidation>;
}

export function useDependencies(): UseDependenciesReturn {
  const [dependencies, setDependencies] = useState<TaskDependency[]>([]);
  const [dependencyGraph, setDependencyGraph] = useState<DependencyGraph | null>(null);
  const [readyTasks, setReadyTasks] = useState<ReadyTask[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [isMutating, setIsMutating] = useState(false);

  const fetchDependencies = useCallback(async (filter?: DependencyFilter) => {
    setIsLoading(true);
    try {
      const result = await invoke<TaskDependency[]>('get_task_dependencies', { filter });
      setDependencies(result);
    } catch (error) {
      console.error('Failed to fetch dependencies:', error);
      pushToast({
        title: '获取依赖关系失败',
        description: error instanceof Error ? error.message : '未知错误',
        variant: 'error',
      });
    } finally {
      setIsLoading(false);
    }
  }, []);

  const fetchDependencyGraph = useCallback(async (filter?: DependencyFilter) => {
    setIsLoading(true);
    try {
      const result = await invoke<DependencyGraph>('get_dependency_graph', { filter });
      setDependencyGraph(result);
    } catch (error) {
      console.error('Failed to fetch dependency graph:', error);
      pushToast({
        title: '获取依赖图失败',
        description: error instanceof Error ? error.message : '未知错误',
        variant: 'error',
      });
    } finally {
      setIsLoading(false);
    }
  }, []);

  const fetchReadyTasks = useCallback(async () => {
    setIsLoading(true);
    try {
      const result = await invoke<ReadyTask[]>('get_ready_tasks');
      setReadyTasks(result);
    } catch (error) {
      console.error('Failed to fetch ready tasks:', error);
      pushToast({
        title: '获取可执行任务失败',
        description: error instanceof Error ? error.message : '未知错误',
        variant: 'error',
      });
    } finally {
      setIsLoading(false);
    }
  }, []);

  const createDependency = useCallback(
    async (input: DependencyCreateInput): Promise<TaskDependency | null> => {
      setIsMutating(true);
      try {
        const result = await invoke<TaskDependency>('add_dependency', { input });

        // Update local state
        setDependencies((prev) => [...prev, result]);

        // Refresh dependency graph if it exists
        if (dependencyGraph) {
          await fetchDependencyGraph();
        }

        pushToast({
          title: '依赖关系创建成功',
          variant: 'success',
        });

        return result;
      } catch (error) {
        console.error('Failed to create dependency:', error);
        pushToast({
          title: '创建依赖关系失败',
          description: error instanceof Error ? error.message : '未知错误',
          variant: 'error',
        });
        return null;
      } finally {
        setIsMutating(false);
      }
    },
    [dependencyGraph, fetchDependencyGraph],
  );

  const deleteDependency = useCallback(
    async (dependencyId: string) => {
      setIsMutating(true);
      try {
        await invoke('remove_dependency', { dependencyId });

        // Update local state
        setDependencies((prev) => prev.filter((dep) => dep.id !== dependencyId));

        // Refresh dependency graph if it exists
        if (dependencyGraph) {
          await fetchDependencyGraph();
        }

        pushToast({
          title: '依赖关系删除成功',
          variant: 'success',
        });
      } catch (error) {
        console.error('Failed to delete dependency:', error);
        pushToast({
          title: '删除依赖关系失败',
          description: error instanceof Error ? error.message : '未知错误',
          variant: 'error',
        });
      } finally {
        setIsMutating(false);
      }
    },
    [dependencyGraph, fetchDependencyGraph],
  );

  const validateDependency = useCallback(
    async (predecessorId: string, successorId: string): Promise<DependencyValidation> => {
      try {
        const result = await invoke<DependencyValidation>('validate_dependency', {
          predecessorId,
          successorId,
        });
        return result;
      } catch (error) {
        console.error('Failed to validate dependency:', error);
        return {
          isValid: false,
          errorMessage: error instanceof Error ? error.message : '验证失败',
          wouldCreateCycle: false,
        };
      }
    },
    [],
  );

  const updateDependencyType = useCallback(
    async (dependencyId: string, newType: DependencyType): Promise<void> => {
      setIsMutating(true);
      try {
        await invoke('update_dependency_type', {
          dependencyId,
          dependencyType: newType,
        });

        // Update local state
        setDependencies((prev) =>
          prev.map((dep) => (dep.id === dependencyId ? { ...dep, dependencyType: newType } : dep)),
        );

        // Refresh dependency graph if it exists
        if (dependencyGraph) {
          await fetchDependencyGraph();
        }

        pushToast({
          title: '依赖类型更新成功',
          variant: 'success',
        });
      } catch (error) {
        console.error('Failed to update dependency type:', error);
        pushToast({
          title: '更新依赖类型失败',
          description: error instanceof Error ? error.message : '未知错误',
          variant: 'error',
        });
      } finally {
        setIsMutating(false);
      }
    },
    [dependencyGraph, fetchDependencyGraph],
  );

  return {
    dependencies,
    dependencyGraph,
    readyTasks,
    isLoading,
    isMutating,
    fetchDependencies,
    fetchDependencyGraph,
    fetchReadyTasks,
    createDependency,
    deleteDependency,
    updateDependencyType,
    validateDependency,
  };
}
