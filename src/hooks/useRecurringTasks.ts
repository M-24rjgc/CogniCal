import { useCallback, useState } from 'react';
import { 
  type RecurringTaskTemplate, 
  type TaskInstance, 
  type TaskStatus 
} from '../types/task';
import { notifyErrorToast, notifySuccessToast } from '../stores/uiStore';

// Mock API functions - these would be replaced with actual Tauri API calls
const mockApi = {
  async createRecurringTask(template: Omit<RecurringTaskTemplate, 'id' | 'createdAt' | 'updatedAt'>): Promise<RecurringTaskTemplate> {
    // This would call the Tauri backend
    const newTemplate: RecurringTaskTemplate = {
      ...template,
      id: `template_${Date.now()}`,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    };
    return new Promise(resolve => setTimeout(() => resolve(newTemplate), 500));
  },

  async updateRecurringTask(_id: string, updates: Partial<RecurringTaskTemplate>): Promise<RecurringTaskTemplate> {
    // This would call the Tauri backend
    const updatedTemplate: RecurringTaskTemplate = {
      id: _id,
      title: 'Updated Template',
      recurrenceRule: {
        frequency: 'daily',
        interval: 1,
        endType: 'never',
      },
      priority: 'medium',
      tags: [],
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
      isActive: true,
      ...updates,
    };
    return new Promise(resolve => setTimeout(() => resolve(updatedTemplate), 500));
  },

  async deleteRecurringTask(_id: string): Promise<void> {
    // This would call the Tauri backend
    return new Promise(resolve => setTimeout(resolve, 500));
  },

  async getTaskInstances(templateId: string): Promise<TaskInstance[]> {
    // This would call the Tauri backend
    const instances: TaskInstance[] = [
      {
        id: `instance_${Date.now()}_1`,
        templateId,
        instanceDate: new Date().toISOString(),
        title: 'Daily Standup',
        status: 'todo',
        priority: 'medium',
        isException: false,
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
      },
    ];
    return new Promise(resolve => setTimeout(() => resolve(instances), 500));
  },

  async updateTaskInstance(
    _instanceId: string, 
    updates: Partial<TaskInstance>, 
    editType: 'single' | 'series'
  ): Promise<TaskInstance> {
    // This would call the Tauri backend
    const updatedInstance: TaskInstance = {
      id: _instanceId,
      templateId: 'template_123',
      instanceDate: new Date().toISOString(),
      title: 'Updated Instance',
      status: 'todo',
      priority: 'medium',
      isException: editType === 'single',
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
      ...updates,
    };
    return new Promise(resolve => setTimeout(() => resolve(updatedInstance), 500));
  },

  async deleteTaskInstance(_instanceId: string, _deleteType: 'single' | 'series'): Promise<void> {
    // This would call the Tauri backend
    return new Promise(resolve => setTimeout(resolve, 500));
  },

  async bulkUpdateInstances(_instanceIds: string[], _updates: { status?: TaskStatus }): Promise<void> {
    // This would call the Tauri backend
    return new Promise(resolve => setTimeout(resolve, 500));
  },

  async bulkDeleteInstances(_instanceIds: string[]): Promise<void> {
    // This would call the Tauri backend
    return new Promise(resolve => setTimeout(resolve, 500));
  },
};

interface UseRecurringTasksOptions {
  templateId?: string;
  autoFetch?: boolean;
}

export function useRecurringTasks(_options: UseRecurringTasksOptions = {}) {

  const [templates, setTemplates] = useState<RecurringTaskTemplate[]>([]);
  const [instances, setInstances] = useState<TaskInstance[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [isMutating, setIsMutating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const createRecurringTask = useCallback(async (
    template: Omit<RecurringTaskTemplate, 'id' | 'createdAt' | 'updatedAt'>
  ) => {
    setIsMutating(true);
    setError(null);
    
    try {
      const newTemplate = await mockApi.createRecurringTask(template);
      setTemplates(prev => [...prev, newTemplate]);
      notifySuccessToast('重复任务已创建', `「${newTemplate.title}」将按规则自动生成实例`);
      return newTemplate;
    } catch (err) {
      const message = err instanceof Error ? err.message : '创建重复任务失败';
      setError(message);
      notifyErrorToast({ code: 'UNKNOWN', message });
      throw err;
    } finally {
      setIsMutating(false);
    }
  }, []);

  const updateRecurringTask = useCallback(async (
    id: string, 
    updates: Partial<RecurringTaskTemplate>
  ) => {
    setIsMutating(true);
    setError(null);
    
    try {
      const updatedTemplate = await mockApi.updateRecurringTask(id, updates);
      setTemplates(prev => prev.map(t => t.id === id ? updatedTemplate : t));
      notifySuccessToast('重复任务已更新', `「${updatedTemplate.title}」的规则已生效`);
      return updatedTemplate;
    } catch (err) {
      const message = err instanceof Error ? err.message : '更新重复任务失败';
      setError(message);
      notifyErrorToast({ code: 'UNKNOWN', message });
      throw err;
    } finally {
      setIsMutating(false);
    }
  }, []);

  const deleteRecurringTask = useCallback(async (id: string) => {
    setIsMutating(true);
    setError(null);
    
    try {
      await mockApi.deleteRecurringTask(id);
      setTemplates(prev => prev.filter(t => t.id !== id));
      setInstances(prev => prev.filter(i => i.templateId !== id));
      notifySuccessToast('重复任务已删除', '相关的任务实例也已清理');
    } catch (err) {
      const message = err instanceof Error ? err.message : '删除重复任务失败';
      setError(message);
      notifyErrorToast({ code: 'UNKNOWN', message });
      throw err;
    } finally {
      setIsMutating(false);
    }
  }, []);

  const fetchInstances = useCallback(async (templateId: string) => {
    setIsLoading(true);
    setError(null);
    
    try {
      const fetchedInstances = await mockApi.getTaskInstances(templateId);
      setInstances(fetchedInstances);
      return fetchedInstances;
    } catch (err) {
      const message = err instanceof Error ? err.message : '获取任务实例失败';
      setError(message);
      notifyErrorToast({ code: 'UNKNOWN', message });
      return [];
    } finally {
      setIsLoading(false);
    }
  }, []);

  const updateInstance = useCallback(async (
    instanceId: string,
    updates: Partial<TaskInstance>,
    editType: 'single' | 'series'
  ) => {
    setIsMutating(true);
    setError(null);
    
    try {
      const updatedInstance = await mockApi.updateTaskInstance(instanceId, updates, editType);
      setInstances(prev => prev.map(i => i.id === instanceId ? updatedInstance : i));
      
      const message = editType === 'single' 
        ? '任务实例已更新' 
        : '重复任务系列已更新，将应用到未来实例';
      notifySuccessToast(message);
      return updatedInstance;
    } catch (err) {
      const message = err instanceof Error ? err.message : '更新任务实例失败';
      setError(message);
      notifyErrorToast({ code: 'UNKNOWN', message });
      throw err;
    } finally {
      setIsMutating(false);
    }
  }, []);

  const deleteInstance = useCallback(async (
    instanceId: string,
    deleteType: 'single' | 'series'
  ) => {
    setIsMutating(true);
    setError(null);
    
    try {
      await mockApi.deleteTaskInstance(instanceId, deleteType);
      
      if (deleteType === 'series') {
        // Remove all instances of the same template
        const instance = instances.find(i => i.id === instanceId);
        if (instance) {
          setInstances(prev => prev.filter(i => i.templateId !== instance.templateId));
          setTemplates(prev => prev.filter(t => t.id !== instance.templateId));
        }
      } else {
        setInstances(prev => prev.filter(i => i.id !== instanceId));
      }
      
      const message = deleteType === 'single' 
        ? '任务实例已删除' 
        : '重复任务系列已删除';
      notifySuccessToast(message);
    } catch (err) {
      const message = err instanceof Error ? err.message : '删除任务实例失败';
      setError(message);
      notifyErrorToast({ code: 'UNKNOWN', message });
      throw err;
    } finally {
      setIsMutating(false);
    }
  }, [instances]);

  const bulkUpdateInstances = useCallback(async (
    instanceIds: string[],
    status: TaskStatus
  ) => {
    setIsMutating(true);
    setError(null);
    
    try {
      await mockApi.bulkUpdateInstances(instanceIds, { status });
      setInstances(prev => prev.map(i => 
        instanceIds.includes(i.id) ? { ...i, status } : i
      ));
      notifySuccessToast('批量更新完成', `已更新 ${instanceIds.length} 个任务实例`);
    } catch (err) {
      const message = err instanceof Error ? err.message : '批量更新失败';
      setError(message);
      notifyErrorToast({ code: 'UNKNOWN', message });
      throw err;
    } finally {
      setIsMutating(false);
    }
  }, []);

  const bulkDeleteInstances = useCallback(async (instanceIds: string[]) => {
    setIsMutating(true);
    setError(null);
    
    try {
      await mockApi.bulkDeleteInstances(instanceIds);
      setInstances(prev => prev.filter(i => !instanceIds.includes(i.id)));
      notifySuccessToast('批量删除完成', `已删除 ${instanceIds.length} 个任务实例`);
    } catch (err) {
      const message = err instanceof Error ? err.message : '批量删除失败';
      setError(message);
      notifyErrorToast({ code: 'UNKNOWN', message });
      throw err;
    } finally {
      setIsMutating(false);
    }
  }, []);

  const clearError = useCallback(() => {
    setError(null);
  }, []);

  return {
    templates,
    instances,
    isLoading,
    isMutating,
    error,
    createRecurringTask,
    updateRecurringTask,
    deleteRecurringTask,
    fetchInstances,
    updateInstance,
    deleteInstance,
    bulkUpdateInstances,
    bulkDeleteInstances,
    clearError,
  };
}