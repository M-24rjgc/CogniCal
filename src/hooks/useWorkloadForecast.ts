import { useQuery } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';

export interface ContributingTask {
  taskId: string;
  title: string;
  estimatedHours: number;
  dueAt?: string;
  priority: string;
}

export interface WorkloadForecast {
  horizon: string;
  generatedAt: string;
  riskLevel: 'ok' | 'warning' | 'critical';
  totalHours: number;
  capacityThreshold: number;
  contributingTasks: ContributingTask[];
  confidence: number;
  recommendations: string[];
}

export function useWorkloadForecast(capacityThresholdHours?: number) {
  return useQuery<WorkloadForecast[]>({
    queryKey: ['workloadForecast', capacityThresholdHours],
    queryFn: async () => {
      const forecasts = await invoke<WorkloadForecast[]>('analytics_get_workload_forecast', {
        capacityThresholdHours,
      });
      return forecasts;
    },
    staleTime: 1000 * 60 * 60, // 1 hour
    gcTime: 1000 * 60 * 60 * 24, // 24 hours
  });
}

export function useLatestWorkloadForecasts() {
  return useQuery<WorkloadForecast[]>({
    queryKey: ['latestWorkloadForecasts'],
    queryFn: async () => {
      const forecasts = await invoke<WorkloadForecast[]>('analytics_get_latest_workload_forecasts');
      return forecasts;
    },
    staleTime: 1000 * 60 * 5, // 5 minutes
    gcTime: 1000 * 60 * 60, // 1 hour
    refetchOnMount: true,
  });
}
