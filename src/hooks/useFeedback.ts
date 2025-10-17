import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';

export interface AiFeedbackSurface {
  surface: 'score' | 'recommendation' | 'forecast';
}

export interface FeedbackSubmission {
  surface: string;
  sessionId?: string;
  sentiment: 'up' | 'down';
  note?: string;
  promptSnapshot: string;
  contextSnapshot: Record<string, unknown>;
}

export interface AiFeedback {
  id: number;
  surface: string;
  sessionId?: string;
  sentiment: 'up' | 'down';
  note?: string;
  promptSnapshot: string;
  contextSnapshot: Record<string, unknown>;
  createdAt: string;
  anonymized: boolean;
}

export interface SurfaceDigest {
  surface: string;
  positive: number;
  negative: number;
  satisfactionRate: number;
  sampleNotes: string[];
}

export interface WeeklyDigest {
  periodStart: string;
  periodEnd: string;
  totalFeedback: number;
  positiveCount: number;
  negativeCount: number;
  bySurface: SurfaceDigest[];
  insights: string[];
  adjustmentsMade: string[];
}

/**
 * Hook for submitting AI feedback
 */
export function useSubmitFeedback() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (submission: FeedbackSubmission) => {
      const feedbackId = await invoke<number>('feedback_submit', { submission });
      return feedbackId;
    },
    onSuccess: () => {
      // Invalidate queries to refresh feedback data
      queryClient.invalidateQueries({ queryKey: ['feedback'] });
      queryClient.invalidateQueries({ queryKey: ['feedback-digest'] });
    },
  });
}

/**
 * Hook for checking if user has opted out
 */
export function useCheckOptOut() {
  return useQuery({
    queryKey: ['feedback-opt-out'],
    queryFn: async () => {
      const optedOut = await invoke<boolean>('feedback_check_opt_out');
      return optedOut;
    },
    staleTime: 1000 * 60 * 5, // 5 minutes
  });
}

/**
 * Hook for getting recent feedback for a surface
 */
export function useRecentFeedback(surface: string, limit?: number) {
  return useQuery({
    queryKey: ['feedback', 'recent', surface, limit],
    queryFn: async () => {
      const feedback = await invoke<AiFeedback[]>('feedback_get_recent', {
        surface,
        limit: limit || 10,
      });
      return feedback;
    },
    staleTime: 1000 * 60 * 2, // 2 minutes
  });
}

/**
 * Hook for getting weekly digest
 */
export function useWeeklyDigest() {
  return useQuery({
    queryKey: ['feedback-digest'],
    queryFn: async () => {
      const digest = await invoke<WeeklyDigest | null>('feedback_get_weekly_digest');
      return digest;
    },
    staleTime: 1000 * 60 * 10, // 10 minutes
  });
}

/**
 * Hook for purging all feedback
 */
export function usePurgeFeedback() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async () => {
      const deletedCount = await invoke<number>('feedback_purge_all');
      return deletedCount;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['feedback'] });
      queryClient.invalidateQueries({ queryKey: ['feedback-digest'] });
    },
  });
}

/**
 * Hook for getting feedback statistics
 */
export function useFeedbackStats(surface?: string) {
  return useQuery({
    queryKey: ['feedback-stats', surface],
    queryFn: async () => {
      const stats = await invoke<Record<string, unknown>>('feedback_get_stats', {
        surface: surface || null,
      });
      return stats;
    },
    staleTime: 1000 * 60 * 5, // 5 minutes
  });
}
