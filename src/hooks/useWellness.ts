import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import type { WellnessEventRecord, WeeklySummary, WellnessResponse } from '@/types/wellness';

export type { WellnessEventRecord, WeeklySummary, WellnessResponse };

/**
 * Check and potentially generate a new wellness nudge
 */
export function useCheckNudge() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (): Promise<WellnessEventRecord | null> => {
      return await invoke('wellness_check_nudge');
    },
    onSuccess: () => {
      // Invalidate pending nudge query to refetch
      queryClient.invalidateQueries({ queryKey: ['wellness', 'pending'] });
    },
  });
}

/**
 * Get the current pending wellness nudge
 */
export function usePendingNudge(enabled: boolean = true) {
  return useQuery<WellnessEventRecord | null>({
    queryKey: ['wellness', 'pending'],
    queryFn: async () => {
      return await invoke('wellness_get_pending');
    },
    enabled,
    refetchInterval: 5 * 60 * 1000, // Refetch every 5 minutes
    staleTime: 2 * 60 * 1000, // Consider data stale after 2 minutes
  });
}

/**
 * Respond to a wellness nudge (Completed, Snoozed, Ignored)
 */
export function useRespondToNudge() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({
      id,
      response,
    }: {
      id: number;
      response: WellnessResponse;
    }): Promise<void> => {
      return await invoke('wellness_respond', { id, response });
    },
    onSuccess: () => {
      // Invalidate both pending and weekly summary
      queryClient.invalidateQueries({ queryKey: ['wellness', 'pending'] });
      queryClient.invalidateQueries({ queryKey: ['wellness', 'weekly'] });
    },
  });
}

/**
 * Get the weekly wellness summary
 */
export function useWeeklySummary(enabled: boolean = true) {
  return useQuery<WeeklySummary>({
    queryKey: ['wellness', 'weekly'],
    queryFn: async () => {
      return await invoke('wellness_get_weekly_summary');
    },
    enabled,
    staleTime: 10 * 60 * 1000, // Consider data stale after 10 minutes
  });
}
