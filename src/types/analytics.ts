import type { TaskPriority, TaskType } from './task';

export type AnalyticsRangeKey = '7d' | '30d' | '90d';
export type AnalyticsGrouping = 'day' | 'week';

export interface AnalyticsQueryParams {
  range: AnalyticsRangeKey;
  from?: string;
  to?: string;
  grouping?: AnalyticsGrouping;
}

export interface TrendPoint {
  date: string;
  completionRate: number;
  productivityScore: number;
  completedTasks: number;
  focusMinutes: number;
}

export interface TimeAllocationEntry {
  label: string;
  minutes: number;
  percentage: number;
}

export interface TimeAllocationBreakdown {
  byType: Array<{ type: TaskType; minutes: number; percentage: number }>;
  byPriority: Array<{ priority: TaskPriority; minutes: number; percentage: number }>;
  byStatus?: TimeAllocationEntry[];
}

export interface EfficiencySuggestion {
  id: string;
  title: string;
  summary: string;
  relatedTaskId?: string;
  relatedPlanId?: string;
  impact: 'low' | 'medium' | 'high';
  confidence: number;
  category: 'schedule' | 'focus' | 'collaboration' | 'planning';
}

export interface InsightCard {
  id: string;
  headline: string;
  detail: string;
  actionLabel?: string;
  actionHref?: string;
  severity: 'info' | 'success' | 'warning' | 'critical';
  relatedIds?: string[];
  generatedAt: string;
  source: 'ai' | 'rule' | 'manual';
}

export interface AnalyticsEfficiency {
  estimateAccuracy: number;
  onTimeRate: number;
  complexityCorrelation: number;
  suggestions: EfficiencySuggestion[];
}

export interface AnalyticsSummary {
  totalCompleted: number;
  completionRate: number;
  trendDelta: number;
  workloadPrediction: number;
  focusMinutes: number;
  overdueTasks: number;
}

export interface ZeroStateMeta {
  isEmpty: boolean;
  recommendedActions: string[];
  sampleDataAvailable: boolean;
  sampleDataLabel?: string;
  missingConfiguration?: string[];
}

export interface AnalyticsOverview {
  range: AnalyticsRangeKey;
  summary: AnalyticsSummary;
  trend: TrendPoint[];
  timeAllocation: TimeAllocationBreakdown;
  efficiency: AnalyticsEfficiency;
  insights: InsightCard[];
  zeroState: ZeroStateMeta;
  meta: {
    generatedAt: string;
    isDemo: boolean;
  };
}

export interface AnalyticsHistoryPoint {
  date: string;
  productivityScore: number;
  completionRate: number;
  focusMinutes: number;
  completedTasks: number;
  overdueTasks: number;
}

export interface AnalyticsHistoryResponse {
  range: AnalyticsRangeKey;
  grouping: AnalyticsGrouping;
  points: AnalyticsHistoryPoint[];
}

export type AnalyticsExportFormat = 'markdown' | 'json';

export interface AnalyticsExportParams {
  range: AnalyticsRangeKey;
  format: AnalyticsExportFormat;
  from?: string;
  to?: string;
}

export interface AnalyticsExportResult {
  filePath: string;
  format: AnalyticsExportFormat;
  generatedAt: string;
  isDemo: boolean;
}

export interface AnalyticsErrorSummary {
  code: string;
  message: string;
  hint?: string;
}

export interface AnalyticsOverviewResponse {
  overview: AnalyticsOverview;
  history: AnalyticsHistoryResponse;
  error?: AnalyticsErrorSummary | null;
}
