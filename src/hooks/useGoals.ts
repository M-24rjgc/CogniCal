import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Goal, GoalWithProgress, CreateGoalRequest, UpdateGoalRequest } from '../types/goal';

export function useGoals(parentGoalId?: string) {
  const [goals, setGoals] = useState<Goal[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchGoals = async () => {
    try {
      setLoading(true);
      const result = await invoke<Goal[]>('list_goals', { parentGoalId });
      setGoals(result);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to fetch goals');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchGoals();
  }, [parentGoalId]);

  const createGoal = async (request: CreateGoalRequest): Promise<Goal> => {
    const goal = await invoke<Goal>('create_goal', { request });
    await fetchGoals();
    return goal;
  };

  const updateGoal = async (id: string, request: UpdateGoalRequest): Promise<Goal> => {
    const goal = await invoke<Goal>('update_goal', { id, request });
    await fetchGoals();
    return goal;
  };

  const deleteGoal = async (id: string): Promise<void> => {
    await invoke('delete_goal', { id });
    await fetchGoals();
  };

  const associateTask = async (goalId: string, taskId: string): Promise<void> => {
    await invoke('associate_task_with_goal', { goalId, taskId });
  };

  const dissociateTask = async (goalId: string, taskId: string): Promise<void> => {
    await invoke('dissociate_task_from_goal', { goalId, taskId });
  };

  const getGoalWithProgress = async (id: string): Promise<GoalWithProgress> => {
    return await invoke<GoalWithProgress>('get_goal_with_progress', { id });
  };

  return {
    goals,
    loading,
    error,
    createGoal,
    updateGoal,
    deleteGoal,
    associateTask,
    dissociateTask,
    getGoalWithProgress,
    refetch: fetchGoals,
  };
}
