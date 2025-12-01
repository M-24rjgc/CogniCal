export const TASK_STATUSES = [
  'backlog',
  'todo',
  'in_progress',
  'blocked',
  'done',
  'archived',
] as const;

export type TaskStatus = (typeof TASK_STATUSES)[number];

export const TASK_PRIORITIES = ['low', 'medium', 'high', 'urgent'] as const;

export type TaskPriority = (typeof TASK_PRIORITIES)[number];

export const TASK_SORT_FIELDS = ['createdAt', 'dueAt', 'updatedAt', 'priority', 'status'] as const;

export type TaskSortField = (typeof TASK_SORT_FIELDS)[number];

export const TASK_TYPES = ['work', 'study', 'life', 'other'] as const;

export type TaskType = (typeof TASK_TYPES)[number];

export type TaskAISource = 'live' | 'cache';

export const TASK_PAYLOAD_FIELDS = [
  'title',
  'description',
  'status',
  'priority',
  'plannedStartAt',
  'startAt',
  'dueAt',
  'completedAt',
  'estimatedMinutes',
  'estimatedHours',
  'tags',
  'ownerId',
  'isRecurring',
  'recurrence',
  'taskType',
  'ai',
  'externalLinks',
] as const;

export type TaskPayloadField = (typeof TASK_PAYLOAD_FIELDS)[number];

export interface TaskRecurrence {
  /** ISO 8601 RRULE 字符串或自定义表达式 */
  rule: string;
  /** 重复结束日期（ISO 字符串，可选） */
  until?: string;
}

// Enhanced recurring task types for v2.0 features
export const RECURRENCE_FREQUENCIES = ['daily', 'weekly', 'monthly', 'yearly'] as const;
export type RecurrenceFrequency = (typeof RECURRENCE_FREQUENCIES)[number];

export const RECURRENCE_END_TYPES = ['never', 'date', 'count'] as const;
export type RecurrenceEndType = (typeof RECURRENCE_END_TYPES)[number];

export const WEEKDAYS = [
  { value: 0, label: 'Sunday', short: 'SU' },
  { value: 1, label: 'Monday', short: 'MO' },
  { value: 2, label: 'Tuesday', short: 'TU' },
  { value: 3, label: 'Wednesday', short: 'WE' },
  { value: 4, label: 'Thursday', short: 'TH' },
  { value: 5, label: 'Friday', short: 'FR' },
  { value: 6, label: 'Saturday', short: 'SA' },
] as const;

export interface RecurrenceRuleConfig {
  frequency: RecurrenceFrequency;
  interval: number;
  endType: RecurrenceEndType;
  endDate?: string;
  endCount?: number;
  weekdays?: number[]; // 0-6, Sunday-Saturday
  monthDay?: number; // 1-31
  monthWeek?: number; // 1-4, first-fourth
  monthWeekday?: number; // 0-6, for "first Monday" type patterns
}

export interface TaskInstance {
  id: string;
  templateId: string;
  instanceDate: string;
  title: string;
  description?: string;
  status: TaskStatus;
  priority: TaskPriority;
  dueAt?: string;
  completedAt?: string;
  isException: boolean; // Modified from template
  createdAt: string;
  updatedAt: string;
}

export interface RecurringTaskTemplate {
  id: string;
  title: string;
  description?: string;
  recurrenceRule: RecurrenceRuleConfig;
  priority: TaskPriority;
  tags: string[];
  estimatedMinutes?: number;
  createdAt: string;
  updatedAt: string;
  isActive: boolean;
}

export interface TaskAIInsights {
  /** AI 总结 */
  summary?: string;
  /** 下一步建议 */
  nextAction?: string;
  /** 0-1 之间的置信度 */
  confidence?: number;
  /** 额外上下文信息 */
  metadata?: Record<string, unknown>;
  /** 0-10 的复杂度评分 */
  complexityScore?: number;
  /** AI 建议的开始时间（ISO 字符串） */
  suggestedStartAt?: string;
  /** 专注模式建议 */
  focusMode?: TaskFocusModeRecommendation;
  /** 时间效率预测 */
  efficiencyPrediction?: TaskEfficiencyPrediction;
  /** 思维链步骤 */
  cotSteps?: TaskAIReasoningStep[];
  /** 思维链摘要 */
  cotSummary?: string;
  /** 结果来源（实时 / 缓存） */
  source?: TaskAISource;
  /** 生成时间（ISO 字符串） */
  generatedAt?: string;
}

export interface TaskAIReasoningStep {
  order: number;
  title?: string;
  detail?: string;
  outcome?: string;
}

export interface TaskFocusModeRecommendation {
  pomodoros: number;
  recommendedSlots?: string[];
}

export interface TaskEfficiencyPrediction {
  expectedHours: number;
  confidence: number;
}

export type TaskAIResult = TaskAIInsights &
  Required<Pick<TaskAIInsights, 'source' | 'generatedAt'>>;

export interface TaskBase {
  title: string;
  description?: string;
  status: TaskStatus;
  priority: TaskPriority;
  plannedStartAt?: string;
  startAt?: string;
  dueAt?: string;
  completedAt?: string;
  estimatedMinutes?: number;
  estimatedHours?: number;
  tags?: string[];
  ownerId?: string;
  isRecurring?: boolean;
  recurrence?: TaskRecurrence;
  taskType?: TaskType;
  ai?: TaskAIInsights;
  externalLinks?: string[];
}

export interface Task extends Omit<TaskBase, 'tags' | 'isRecurring'> {
  id: string;
  tags: string[];
  isRecurring: boolean;
  createdAt: string;
  updatedAt: string;
}

export type TaskPayload = Omit<TaskBase, 'status'> & {
  status?: TaskStatus;
};

export type TaskUpdatePayload = Partial<TaskPayload>;

export interface TaskFilters {
  search?: string;
  statuses?: TaskStatus[];
  priorities?: TaskPriority[];
  tags?: string[];
  taskTypes?: TaskType[];
  complexityMin?: number;
  complexityMax?: number;
  aiSuggestedAfter?: string;
  aiSuggestedBefore?: string;
  aiSources?: TaskAISource[];
  ownerIds?: string[];
  includeArchived?: boolean;
  dueAfter?: string;
  dueBefore?: string;
  windowStart?: string;
  windowEnd?: string;
  updatedAfter?: string;
  updatedBefore?: string;
  sortBy?: TaskSortField;
  sortOrder?: 'asc' | 'desc';
  page?: number;
  pageSize?: number;
}

export interface TaskListResponse {
  items: Task[];
  total: number;
  page: number;
  pageSize: number;
}

export interface TaskParseContext {
  timezone?: string;
  locale?: string;
  referenceDate?: string;
  existingTaskId?: string;
  metadata?: Record<string, unknown>;
  userPreferences?: Record<string, unknown>;
}

export interface TaskParseRequest {
  input: string;
  context?: TaskParseContext;
}

export interface TaskParseResponse {
  payload: Partial<TaskPayload>;
  missingFields: TaskPayloadField[];
  ai: TaskAIResult;
}

export const DEFAULT_PAGE_SIZE = 20;
