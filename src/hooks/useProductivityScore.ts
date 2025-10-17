import { useQuery, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import {
  ProductivityScoreRecord,
  ProductivityScoreHistoryResponse,
} from '../types/productivity.ts';

interface UseProductivityScoreOptions {
  date?: string;
  enabled?: boolean;
}

export function useProductivityScore(options: UseProductivityScoreOptions = {}) {
  const { date, enabled = true } = options;
  const queryClient = useQueryClient();

  const {
    data: score,
    isLoading,
    error,
    refetch,
  } = useQuery({
    queryKey: ['productivity-score', date],
    queryFn: async () => {
      const result = await invoke<ProductivityScoreRecord>('analytics_get_productivity_score', {
        date: date || undefined,
      });
      return result;
    },
    enabled,
    staleTime: 5 * 60 * 1000, // 5 minutes
    gcTime: 10 * 60 * 1000, // 10 minutes
  });

  const refreshScore = () => {
    return refetch();
  };

  const invalidateScore = () => {
    queryClient.invalidateQueries({ queryKey: ['productivity-score'] });
  };

  return {
    score,
    isLoading,
    error,
    refreshScore,
    invalidateScore,
  };
}

interface UseProductivityScoreHistoryOptions {
  startDate: string;
  endDate: string;
  enabled?: boolean;
}

export function useProductivityScoreHistory(options: UseProductivityScoreHistoryOptions) {
  const { startDate, endDate, enabled = true } = options;

  const {
    data: history,
    isLoading,
    error,
    refetch,
  } = useQuery({
    queryKey: ['productivity-score-history', startDate, endDate],
    queryFn: async () => {
      const result = await invoke<ProductivityScoreHistoryResponse>(
        'analytics_get_productivity_score_history',
        {
          startDate,
          endDate,
        },
      );
      return result;
    },
    enabled: enabled && !!(startDate && endDate),
    staleTime: 10 * 60 * 1000, // 10 minutes
    gcTime: 30 * 60 * 1000, // 30 minutes
  });

  return {
    history,
    isLoading,
    error,
    refetch,
  };
}

export function useLatestProductivityScore() {
  const {
    data: latestScore,
    isLoading,
    error,
    refetch,
  } = useQuery({
    queryKey: ['productivity-score-latest'],
    queryFn: async () => {
      const result = await invoke<ProductivityScoreRecord | null>(
        'analytics_get_latest_productivity_score',
      );
      return result;
    },
    staleTime: 5 * 60 * 1000, // 5 minutes
    gcTime: 10 * 60 * 1000, // 10 minutes
  });

  return {
    latestScore,
    isLoading,
    error,
    refetch,
  };
}

// Helper functions for score interpretation
export function getScoreLevel(
  score: number,
): 'excellent' | 'good' | 'moderate' | 'needs-improvement' {
  if (score >= 85) return 'excellent';
  if (score >= 70) return 'good';
  if (score >= 55) return 'moderate';
  return 'needs-improvement';
}

export function getScoreColor(score: number): string {
  const level = getScoreLevel(score);
  switch (level) {
    case 'excellent':
      return 'text-green-600 dark:text-green-400';
    case 'good':
      return 'text-blue-600 dark:text-blue-400';
    case 'moderate':
      return 'text-yellow-600 dark:text-yellow-400';
    case 'needs-improvement':
      return 'text-red-600 dark:text-red-400';
    default:
      return 'text-gray-600 dark:text-gray-400';
  }
}

export function getScoreBgColor(score: number): string {
  const level = getScoreLevel(score);
  switch (level) {
    case 'excellent':
      return 'bg-green-100 dark:bg-green-900/20';
    case 'good':
      return 'bg-blue-100 dark:bg-blue-900/20';
    case 'moderate':
      return 'bg-yellow-100 dark:bg-yellow-900/20';
    case 'needs-improvement':
      return 'bg-red-100 dark:bg-red-900/20';
    default:
      return 'bg-gray-100 dark:bg-gray-900/20';
  }
}

export function formatScoreExplanation(score: ProductivityScoreRecord): {
  summary: string;
  insights: string[];
  recommendations: string[];
} {
  const insights: string[] = [];
  const recommendations: string[] = [];

  // Analyze dimension scores
  const dimensions = score.dimensionScores;

  if (dimensions.completionRate >= 80) {
    insights.push('Strong task completion rate');
  } else if (dimensions.completionRate < 50) {
    insights.push('Task completion needs attention');
    recommendations.push('Focus on breaking down larger tasks into smaller, manageable pieces');
  }

  if (dimensions.onTimeRatio >= 75) {
    insights.push('Good on-time delivery');
  } else if (dimensions.onTimeRatio < 50) {
    insights.push('On-time completion could improve');
    recommendations.push('Review time estimates and set more realistic deadlines');
  }

  if (dimensions.focusConsistency >= 70) {
    insights.push('Consistent focus patterns');
  } else if (dimensions.focusConsistency < 50) {
    insights.push('Focus consistency varies');
    recommendations.push('Consider time-blocking techniques and minimize distractions');
  }

  if (dimensions.restBalance >= 60) {
    insights.push('Healthy work-life balance');
  } else if (dimensions.restBalance < 40) {
    insights.push('Work-life balance needs attention');
    recommendations.push(
      'Schedule regular breaks and maintain clear boundaries between work and personal time',
    );
  }

  if (dimensions.efficiencyRating >= 80) {
    insights.push('High efficiency in task execution');
  } else if (dimensions.efficiencyRating < 60) {
    insights.push('Efficiency can be improved');
    recommendations.push('Optimize your workflow and eliminate time-wasting activities');
  }

  const summary =
    score.explanation ||
    'Productivity score calculated based on your task completion patterns and work habits.';

  return {
    summary,
    insights,
    recommendations,
  };
}
