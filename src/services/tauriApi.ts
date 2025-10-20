import { invoke } from '@tauri-apps/api/core';
import { ZodError } from 'zod';
import {
  DEFAULT_PAGE_SIZE,
  type Task,
  type TaskFilters,
  type TaskListResponse,
  type TaskPayload,
  type TaskUpdatePayload,
  type TaskParseRequest,
  type TaskParseResponse,
} from '../types/task';
import {
  analyticsExportParamsSchema,
  analyticsQueryParamsSchema,
  appSettingsUpdateSchema,
  aiStatusSchema,
  formatZodError,
  parseTaskFilters,
  parseTaskPayload,
  parseTaskParseInput,
  parseTaskParseResult,
  taskUpdateSchema,
} from '../utils/validators';
import {
  applyPlanInputSchema,
  generatePlanInputSchema,
  parseAppliedPlan,
  parsePlanningSessionView,
  parsePreferenceSnapshot,
  planningPreferencesUpdateSchema,
  resolveConflictInputSchema,
  type AppliedPlan,
  type ApplyPlanInput,
  type GeneratePlanInput,
  type PlanningPreferencesUpdateInput,
  type PlanningSessionView,
  type PreferenceSnapshot,
  type ResolveConflictInput,
} from '../types/planning';
import {
  type AnalyticsExportParams,
  type AnalyticsExportResult,
  type AnalyticsGrouping,
  type AnalyticsHistoryResponse,
  type AnalyticsOverviewResponse,
  type AnalyticsQueryParams,
  type AnalyticsRangeKey,
} from '../types/analytics';
import {
  type AiStatus,
  type AppSettings,
  type CacheClearResult,
  type ThemePreference,
  type UpdateAppSettingsInput,
} from '../types/settings';
import {
  DASHBOARD_MODULE_IDS,
  type DashboardConfig,
  type DashboardConfigInput,
} from '../types/dashboard';
import { DEFAULT_DASHBOARD_CONFIG, normalizeDashboardConfig } from '../utils/dashboardConfig';

type CommandErrorPayload = {
  code: string;
  message?: string;
  details?: unknown;
};

const isDevelopment = import.meta.env.DEV;

const warnMockUsage = (() => {
  let warned = false;
  return () => {
    if (warned) return;
    warned = true;
    if (typeof console !== 'undefined' && typeof console.warn === 'function') {
      console.warn(
        '[tauriApi] 检测到桌面引擎未就绪，已启用内存数据模拟。打包或连接桌面运行时时将自动调用真实 Tauri Commands。',
      );
    }
  };
})();

export type AppErrorCode =
  | 'TAURI_UNAVAILABLE'
  | 'VALIDATION_ERROR'
  | 'NOT_FOUND'
  | 'CONFLICT'
  | 'NETWORK'
  | 'FORBIDDEN'
  | 'INVALID_REQUEST'
  | 'MISSING_API_KEY'
  | 'HTTP_TIMEOUT'
  | 'RATE_LIMITED'
  | 'INVALID_RESPONSE'
  | 'DEEPSEEK_UNAVAILABLE'
  | 'UNKNOWN';

export interface AppError {
  code: AppErrorCode;
  message: string;
  details?: unknown;
  correlationId?: string;
  cause?: unknown;
}

export const defaultErrorCopy: Record<AppErrorCode, { title: string; description?: string }> = {
  TAURI_UNAVAILABLE: {
    title: '应用未连接桌面引擎',
    description: '暂时无法调用本地能力，请重启应用或检查权限设置。',
  },
  VALIDATION_ERROR: {
    title: '数据校验失败',
    description: '提交的数据不符合要求，请检查后重新尝试。',
  },
  NOT_FOUND: {
    title: '资源未找到',
    description: '目标数据可能已被删除，请刷新后再试。',
  },
  CONFLICT: {
    title: '数据冲突',
    description: '当前操作与已有数据冲突，请刷新列表或稍后重试。',
  },
  NETWORK: {
    title: '网络异常',
    description: '检测到网络或进程中断，请确认网络连接。',
  },
  FORBIDDEN: {
    title: 'DeepSeek API 权限不足',
    description: '请在 DeepSeek 控制台检查账户权限或申请更高配额。',
  },
  INVALID_REQUEST: {
    title: '请求参数无效',
    description: '请检查输入内容或稍后再尝试解析任务。',
  },
  MISSING_API_KEY: {
    title: '未检测到 DeepSeek API Key',
    description: '请前往设置页面配置有效的 DeepSeek API Key。',
  },
  HTTP_TIMEOUT: {
    title: '网络连接超时',
    description: '连接 DeepSeek 超时，请检查网络后重试。',
  },
  RATE_LIMITED: {
    title: 'AI 请求过于频繁',
    description: 'DeepSeek 暂时拒绝请求，请稍后再试。',
  },
  INVALID_RESPONSE: {
    title: 'AI 返回数据格式异常',
    description: 'DeepSeek 响应格式无法解析，请稍后重试或联系管理员。',
  },
  DEEPSEEK_UNAVAILABLE: {
    title: 'DeepSeek 服务暂不可用',
    description: '服务可能正在维护，请稍后再试。',
  },
  UNKNOWN: {
    title: '发生未知错误',
    description: '可在日志中查看详情，或稍后再试。',
  },
};

type E2ECommandOutcome = { type: 'resolve'; value: unknown } | { type: 'reject'; error: unknown };

type E2EQueueEntry =
  | E2ECommandOutcome
  | ((payload?: Record<string, unknown>) => unknown | Promise<unknown>);

type E2EBridge = {
  queues?: Record<string, E2EQueueEntry[]>;
};

const getE2EBridge = (): E2EBridge | undefined => {
  const globalAny = globalThis as unknown as { __COGNICAL_E2E__?: E2EBridge };
  return globalAny.__COGNICAL_E2E__;
};

const isE2EOutcome = (value: unknown): value is E2ECommandOutcome => {
  if (typeof value !== 'object' || value === null) {
    return false;
  }
  const record = value as Record<string, unknown>;
  const type = record.type;
  return type === 'resolve' || type === 'reject';
};

const executeE2EEntry = async (
  entry: E2EQueueEntry,
  payload: Record<string, unknown> | undefined,
) => {
  if (typeof entry === 'function') {
    const result = await entry(payload);
    if (isE2EOutcome(result)) {
      if (result.type === 'reject') {
        throw result.error;
      }
      return result.value;
    }
    return result;
  }

  if (entry.type === 'reject') {
    throw entry.error;
  }

  return entry.value;
};

const tryConsumeE2EMock = async (
  command: string,
  payload: Record<string, unknown> | undefined,
): Promise<{ matched: true; value: unknown } | { matched: false }> => {
  const bridge = getE2EBridge();
  if (!bridge?.queues) {
    return { matched: false };
  }

  const queue = bridge.queues[command];
  if (!queue || queue.length === 0) {
    return { matched: false };
  }

  const entry = queue.shift();
  if (!entry) {
    return { matched: false };
  }

  const value = await executeE2EEntry(entry, payload);
  return { matched: true, value };
};

const COMMANDS = {
  LIST: 'tasks_list',
  CREATE: 'tasks_create',
  UPDATE: 'tasks_update',
  DELETE: 'tasks_delete',
  PARSE: 'tasks_parse_ai',
  AI_STATUS: 'ai_status',
  AI_RECOMMENDATIONS: 'ai_generate_recommendations',
  AI_PLAN_SCHEDULE: 'ai_plan_schedule',
  RECOMMENDATIONS_GENERATE: 'recommendations_generate',
  RECOMMENDATIONS_DECISION: 'recommendations_record_decision',
  PLANNING_GENERATE: 'planning_generate',
  PLANNING_APPLY: 'planning_apply',
  PLANNING_RESOLVE_CONFLICT: 'planning_resolve_conflict',
  PLANNING_PREFERENCES_GET: 'planning_preferences_get',
  PLANNING_PREFERENCES_UPDATE: 'planning_preferences_update',
  ANALYTICS_OVERVIEW_FETCH: 'analytics_overview_fetch',
  ANALYTICS_HISTORY_FETCH: 'analytics_history_fetch',
  ANALYTICS_REPORT_EXPORT: 'analytics_report_export',
  SETTINGS_GET: 'settings_get',
  SETTINGS_UPDATE: 'settings_update',
  DASHBOARD_CONFIG_GET: 'dashboard_config_get',
  DASHBOARD_CONFIG_UPDATE: 'dashboard_config_update',
  CACHE_CLEAR_ALL: 'cache_clear_all',
} as const;

const isTauriAvailable = () => {
  if (typeof window === 'undefined') return false;
  const tauriWindow = window as typeof window & {
    __TAURI_IPC__?: unknown;
    __TAURI_INTERNALS__?: unknown;
  };
  return Boolean(tauriWindow.__TAURI_IPC__ ?? tauriWindow.__TAURI_INTERNALS__);
};

const resolveErrorMessage = (code: AppErrorCode, provided?: string): string => {
  const trimmed = typeof provided === 'string' ? provided.trim() : '';
  if (trimmed.length > 0) {
    return trimmed;
  }
  return defaultErrorCopy[code]?.title ?? defaultErrorCopy.UNKNOWN.title;
};

const extractCorrelationId = (details: unknown): string | undefined => {
  if (!details) return undefined;

  const queue: unknown[] = [details];
  const visited = new WeakSet<object>();

  while (queue.length > 0) {
    const current = queue.shift();
    if (!current) continue;

    if (typeof current === 'string') {
      continue;
    }

    if (typeof current !== 'object') {
      continue;
    }

    const record = current as Record<string, unknown>;
    if (visited.has(record)) {
      continue;
    }
    visited.add(record);

    const value = record.correlationId;
    if (typeof value === 'string' && value.trim().length > 0) {
      return value.trim();
    }

    for (const key of Object.keys(record)) {
      const child = record[key];
      if (child && (typeof child === 'object' || typeof child === 'string')) {
        queue.push(child);
      }
    }
  }

  return undefined;
};

const createId = () =>
  typeof crypto !== 'undefined' && 'randomUUID' in crypto
    ? crypto.randomUUID()
    : `mock-task-${Math.random().toString(36).slice(2, 10)}`;

const defaultPreferenceSnapshot: PreferenceSnapshot = {
  focusStartMinute: 540, // 9:00 AM
  focusEndMinute: 1020, // 5:00 PM
  bufferMinutesBetweenBlocks: 30,
  preferCompactSchedule: false,
  avoidanceWindows: [],
};

type InternalAppSettings = {
  deepseekApiKey: string | null;
  workdayStartMinute: number;
  workdayEndMinute: number;
  themePreference: ThemePreference;
  lastUpdatedAt: string | null;
  aiFeedbackOptOut?: boolean;
  dashboardConfig: DashboardConfig;
};

const createDefaultAppSettings = (): InternalAppSettings => ({
  deepseekApiKey: null,
  workdayStartMinute: 9 * 60,
  workdayEndMinute: 18 * 60,
  themePreference: 'system',
  lastUpdatedAt: null,
  aiFeedbackOptOut: false,
  dashboardConfig: normalizeDashboardConfig(DEFAULT_DASHBOARD_CONFIG),
});

const memoryStore: {
  tasks: Task[];
  planningSession?: PlanningSessionView;
  preferences: Record<string, PreferenceSnapshot>;
  analytics?: AnalyticsOverviewResponse;
  lastAnalyticsRange?: AnalyticsRangeKey;
  settings: InternalAppSettings;
  aiStatus: AiStatus | null;
} = {
  tasks: [],
  preferences: {},
  settings: createDefaultAppSettings(),
  aiStatus: null,
};

const deepClone = <T>(value: T): T => JSON.parse(JSON.stringify(value));

const ensurePreferenceSnapshot = (preferenceId: string): PreferenceSnapshot => {
  const key = preferenceId.trim().length > 0 ? preferenceId : 'default';
  if (!memoryStore.preferences[key]) {
    memoryStore.preferences[key] = deepClone(defaultPreferenceSnapshot);
  }
  return memoryStore.preferences[key];
};

const ensureAppSettings = (): InternalAppSettings => {
  if (!memoryStore.settings) {
    memoryStore.settings = createDefaultAppSettings();
  }
  return memoryStore.settings;
};

const createAppError = (
  code: AppErrorCode,
  message: string,
  details?: unknown,
  cause?: unknown,
  correlationId?: string,
): AppError => ({
  code,
  message,
  details,
  cause,
  correlationId,
});

const KNOWN_APP_ERROR_CODES: AppErrorCode[] = [
  'TAURI_UNAVAILABLE',
  'VALIDATION_ERROR',
  'NOT_FOUND',
  'CONFLICT',
  'NETWORK',
  'FORBIDDEN',
  'INVALID_REQUEST',
  'MISSING_API_KEY',
  'HTTP_TIMEOUT',
  'RATE_LIMITED',
  'INVALID_RESPONSE',
  'DEEPSEEK_UNAVAILABLE',
  'UNKNOWN',
];

const normalizeCommandCode = (code: unknown): AppErrorCode => {
  if (typeof code !== 'string') return 'UNKNOWN';
  const normalized = code.toUpperCase();
  return (KNOWN_APP_ERROR_CODES as string[]).includes(normalized)
    ? (normalized as AppErrorCode)
    : 'UNKNOWN';
};

const tryParseCommandErrorString = (value: unknown): CommandErrorPayload | null => {
  if (typeof value !== 'string') return null;
  const start = value.indexOf('{');
  const end = value.lastIndexOf('}');
  if (start === -1 || end === -1 || end < start) {
    return null;
  }

  const jsonSegment = value.slice(start, end + 1);
  try {
    const parsed = JSON.parse(jsonSegment) as Record<string, unknown>;
    if (parsed && typeof parsed.code === 'string') {
      const details =
        parsed.details ??
        parsed.data ??
        (typeof parsed.payload === 'object' && parsed.payload
          ? (parsed.payload as Record<string, unknown>).details
          : undefined);

      return {
        code: parsed.code as string,
        message: typeof parsed.message === 'string' ? parsed.message : undefined,
        details,
      };
    }
  } catch (parseError) {
    if (typeof console !== 'undefined' && typeof console.debug === 'function') {
      console.debug('[tauriApi] 解析命令错误信息失败', parseError);
    }
  }

  return null;
};

const extractCommandErrorPayload = (error: unknown): CommandErrorPayload | null => {
  if (!error) return null;

  const visited = new WeakSet<object>();
  const queue: unknown[] = [error];

  while (queue.length > 0) {
    const current = queue.shift();
    if (!current) continue;

    if (typeof current === 'string') {
      const parsed = tryParseCommandErrorString(current);
      if (parsed) return parsed;
      continue;
    }

    if (typeof current !== 'object') {
      continue;
    }

    const record = current as Record<string, unknown>;
    if (visited.has(record)) {
      continue;
    }
    visited.add(record);

    if (typeof record.code === 'string') {
      const details =
        record.details ??
        record.data ??
        (typeof record.payload === 'object' && record.payload
          ? (record.payload as Record<string, unknown>).details
          : undefined);

      return {
        code: record.code,
        message: typeof record.message === 'string' ? record.message : undefined,
        details,
      };
    }

    const candidates = ['payload', 'cause', 'inner', 'data', 'error', 'source'] as const;
    for (const key of candidates) {
      if (key in record) {
        queue.push(record[key]);
      }
    }

    if (typeof record.message === 'string') {
      const parsed = tryParseCommandErrorString(record.message);
      if (parsed) return parsed;
    }
  }

  return null;
};

const mapCommandErrorPayload = (payload: CommandErrorPayload, cause?: unknown): AppError => {
  const code = normalizeCommandCode(payload.code);
  const message = resolveErrorMessage(code, payload.message);
  const correlationId = extractCorrelationId(payload.details);

  return createAppError(code, message, payload.details, cause ?? payload, correlationId);
};

const ensureSampleData = () => {
  if (memoryStore.tasks.length > 0) return;

  const now = new Date();
  const baseDate = now.toISOString();

  memoryStore.tasks = [
    {
      id: createId(),
      title: '接入 Tauri API',
      description: '封装 invoke，支持 CRUD 与错误映射。',
      status: 'in_progress',
      priority: 'high',
      plannedStartAt: baseDate,
      startAt: baseDate,
      dueAt: new Date(now.getTime() + 60 * 60 * 1000).toISOString(),
      completedAt: undefined,
      estimatedMinutes: 90,
      estimatedHours: 1.5,
      tags: ['foundation', 'tauri'],
      ownerId: 'system',
      isRecurring: false,
      recurrence: undefined,
      taskType: 'work',
      ai: {
        summary: '建议先定义数据模型与错误类型，再实现状态管理。',
        confidence: 0.82,
        complexityScore: 6,
        suggestedStartAt: baseDate,
        focusMode: { pomodoros: 3, recommendedSlots: [baseDate] },
        efficiencyPrediction: { expectedHours: 2.5, confidence: 0.72 },
        cotSteps: [
          {
            order: 0,
            title: '分析需求',
            detail: '确认需要支持的命令与错误类型。',
            outcome: '列出 LIST/CREATE/UPDATE/DELETE 命令。',
          },
          {
            order: 1,
            title: '制定方案',
            detail: '规划 invoke 封装与内存 mock 行为。',
            outcome: '确定 removeUndefinedDeep 与 warnMockUsage。',
          },
          {
            order: 2,
            title: '评估风险',
            detail: '检查离线模式和错误映射场景。',
            outcome: '确保提供 TAURI_UNAVAILABLE 提示。',
          },
        ],
        cotSummary: '按照先建模型再封装命令的顺序推进最稳妥。',
        source: 'live',
        generatedAt: baseDate,
      },
      externalLinks: [],
      createdAt: baseDate,
      updatedAt: baseDate,
    },
    {
      id: createId(),
      title: '构建任务状态 Store',
      description: '使用 Zustand 管理任务列表、过滤器与加载态。',
      status: 'todo',
      priority: 'medium',
      plannedStartAt: undefined,
      startAt: undefined,
      dueAt: undefined,
      completedAt: undefined,
      estimatedMinutes: 60,
      estimatedHours: 1,
      tags: ['zustand'],
      ownerId: 'system',
      isRecurring: false,
      recurrence: undefined,
      taskType: 'study',
      ai: {
        summary: '准备状态结构、派生 selectors，并考虑缓存。',
        confidence: 0.64,
        complexityScore: 4,
        focusMode: { pomodoros: 2, recommendedSlots: [] },
        efficiencyPrediction: { expectedHours: 1.2, confidence: 0.58 },
        cotSteps: [
          {
            order: 0,
            detail: '回顾现有 CRUD 的状态需求。',
            outcome: '识别 loading/error/filter 字段。',
          },
          {
            order: 1,
            detail: '规划 Zustand store 结构与 actions。',
            outcome: '定义 setTasks、setFilters 等方法。',
          },
        ],
        cotSummary: '先完善 store 接口，再接入组件。',
        source: 'cache',
        generatedAt: baseDate,
      },
      externalLinks: [],
      createdAt: baseDate,
      updatedAt: baseDate,
    },
  ];
};

const clamp = (value: number, min: number, max: number) => Math.min(max, Math.max(min, value));

const pseudoRandom = (seed: number) => {
  const raw = Math.sin(seed) * 10000;
  return raw - Math.floor(raw);
};

const deriveHistoryConfig = (
  range: AnalyticsRangeKey,
): { grouping: AnalyticsGrouping; pointCount: number; step: number } => {
  if (range === '7d') {
    return { grouping: 'day', pointCount: 7, step: 1 };
  }
  if (range === '30d') {
    return { grouping: 'week', pointCount: 5, step: 7 };
  }
  return { grouping: 'week', pointCount: 8, step: 7 };
};

const createMockAnalyticsHistory = (range: AnalyticsRangeKey): AnalyticsHistoryResponse => {
  const { grouping, pointCount, step } = deriveHistoryConfig(range);
  const now = new Date();
  const points: AnalyticsHistoryResponse['points'] = [];

  for (let index = pointCount - 1; index >= 0; index -= 1) {
    const daysBack = index * step;
    const date = new Date(now);
    date.setDate(now.getDate() - daysBack);

    const seed = index + pointCount * (range === '30d' ? 2 : range === '90d' ? 3 : 1);
    const completionRate = clamp(0.68 + pseudoRandom(seed) * 0.24, 0.45, 0.96);
    const productivityScore = Math.round(clamp(65 + pseudoRandom(seed + 1) * 28, 40, 95));
    const focusMinutes = Math.round(240 + pseudoRandom(seed + 2) * 150);
    const completedTasks = Math.max(1, Math.round(8 + pseudoRandom(seed + 3) * 6));
    const overdueTasks = Math.max(0, Math.round(pseudoRandom(seed + 4) * 3));

    points.push({
      date: date.toISOString(),
      productivityScore,
      completionRate: Number(completionRate.toFixed(3)),
      focusMinutes,
      completedTasks,
      overdueTasks,
    });
  }

  return { range, grouping, points };
};

const clampMinute = (value: number, fallback: number): number => {
  if (!Number.isFinite(value)) return fallback;
  const normalized = Math.round(value);
  if (normalized < 0) return 0;
  const maxMinute = 24 * 60 - 1;
  if (normalized > maxMinute) return maxMinute;
  return normalized;
};

const ensureValidEndMinute = (start: number, end: number): number => {
  if (end <= start) {
    return Math.min(24 * 60 - 1, start + 60);
  }
  return Math.min(end, 24 * 60 - 1);
};

const maskDeepseekKey = (key: string | null | undefined): string | null => {
  if (!key) return null;
  const trimmed = key.trim();
  if (!trimmed) return null;
  if (trimmed.length <= 4) return '*'.repeat(trimmed.length);
  const suffix = trimmed.slice(-4);
  return `${'*'.repeat(trimmed.length - 4)}${suffix}`;
};

const normalizeInternalSettings = (settings: InternalAppSettings): AppSettings => {
  const hasKey = Boolean(settings.deepseekApiKey && settings.deepseekApiKey.trim().length > 0);
  const maskedDeepseekKey = hasKey ? maskDeepseekKey(settings.deepseekApiKey) : null;
  const startMinute = clampMinute(settings.workdayStartMinute, 9 * 60);
  const endMinute = clampMinute(settings.workdayEndMinute, Math.min(18 * 60, startMinute + 8 * 60));
  const normalizedEndMinute = ensureValidEndMinute(startMinute, endMinute);
  const dashboardConfig = normalizeDashboardConfig(settings.dashboardConfig);

  const theme: ThemePreference =
    settings.themePreference === 'light' || settings.themePreference === 'dark'
      ? settings.themePreference
      : 'system';

  return {
    hasDeepseekKey: hasKey,
    maskedDeepseekKey,
    workdayStartMinute: startMinute,
    workdayEndMinute: normalizedEndMinute,
    themePreference: theme,
    lastUpdatedAt: settings.lastUpdatedAt ?? null,
    aiFeedbackOptOut: settings.aiFeedbackOptOut ?? false,
    dashboardConfig,
  } satisfies AppSettings;
};

const normalizeAppSettingsResponse = (payload: unknown): AppSettings => {
  const fallback = ensureAppSettings();
  if (!payload || typeof payload !== 'object') {
    return normalizeInternalSettings(fallback);
  }

  const input = payload as Partial<AppSettings> & {
    deepseekApiKey?: string | null;
    theme?: ThemePreference;
    workdayStartMinute?: number;
    workdayEndMinute?: number;
    workdayStartHour?: number;
    workdayEndHour?: number;
    aiFeedbackOptOut?: boolean;
  };

  const startMinuteCandidate =
    typeof input.workdayStartMinute === 'number'
      ? input.workdayStartMinute
      : typeof input.workdayStartHour === 'number'
        ? input.workdayStartHour * 60
        : fallback.workdayStartMinute;
  const workdayStartMinute = clampMinute(startMinuteCandidate, fallback.workdayStartMinute);

  const endMinuteCandidate =
    typeof input.workdayEndMinute === 'number'
      ? input.workdayEndMinute
      : typeof input.workdayEndHour === 'number'
        ? input.workdayEndHour * 60
        : fallback.workdayEndMinute;
  const workdayEndMinute = clampMinute(endMinuteCandidate, fallback.workdayEndMinute);
  const normalizedEnd = ensureValidEndMinute(workdayStartMinute, workdayEndMinute);

  const themeInput = input.themePreference ?? input.theme ?? fallback.themePreference;
  const themePreference: ThemePreference =
    themeInput === 'light' || themeInput === 'dark' ? themeInput : 'system';

  const maskedFromPayload =
    input.maskedDeepseekKey ?? maskDeepseekKey(input.deepseekApiKey ?? undefined);

  const hasKey = Boolean(
    input.hasDeepseekKey ??
      maskedFromPayload ??
      (typeof input.deepseekApiKey === 'string' && input.deepseekApiKey.trim().length > 0),
  );

  const lastUpdatedAt =
    typeof input.lastUpdatedAt === 'string'
      ? input.lastUpdatedAt
      : (fallback.lastUpdatedAt ?? null);

  const aiFeedbackOptOut =
    typeof input.aiFeedbackOptOut === 'boolean'
      ? input.aiFeedbackOptOut
      : fallback.aiFeedbackOptOut;

  const dashboardConfigPayload =
    'dashboardConfig' in input
      ? (input.dashboardConfig as DashboardConfig | DashboardConfigInput | null | undefined)
      : fallback.dashboardConfig;
  const normalizedDashboardConfig = normalizeDashboardConfig(
    dashboardConfigPayload ?? fallback.dashboardConfig,
  );

  memoryStore.settings = {
    deepseekApiKey: hasKey
      ? typeof input.deepseekApiKey === 'string'
        ? input.deepseekApiKey
        : (maskedFromPayload ?? null)
      : null,
    workdayStartMinute,
    workdayEndMinute: normalizedEnd,
    themePreference,
    lastUpdatedAt,
    aiFeedbackOptOut,
    dashboardConfig: normalizedDashboardConfig,
  } satisfies InternalAppSettings;

  return {
    hasDeepseekKey: hasKey,
    maskedDeepseekKey: hasKey ? (maskedFromPayload ?? '****') : null,
    workdayStartMinute,
    workdayEndMinute: normalizedEnd,
    themePreference,
    lastUpdatedAt,
    aiFeedbackOptOut,
    dashboardConfig: normalizedDashboardConfig,
  } satisfies AppSettings;
};

const createMockAnalyticsOverview = (range: AnalyticsRangeKey): AnalyticsOverviewResponse => {
  ensureSampleData();
  const history = createMockAnalyticsHistory(range);
  const points = history.points;
  const lastPoint =
    points[points.length - 1] ??
    ({
      date: new Date().toISOString(),
      productivityScore: 72,
      completionRate: 0.72,
      focusMinutes: 240,
      completedTasks: 6,
      overdueTasks: 1,
    } satisfies AnalyticsHistoryResponse['points'][number]);
  const firstPoint = points[0] ?? lastPoint;
  const totalCompleted = points.reduce((sum, item) => sum + item.completedTasks, 0);
  const totalFocusMinutes = points.reduce((sum, item) => sum + item.focusMinutes, 0);
  const totalOverdue = points.reduce((sum, item) => sum + item.overdueTasks, 0);
  const trendDelta = (lastPoint.completionRate - firstPoint.completionRate) * 100;

  const ensurePercentage = (minutes: number) =>
    totalFocusMinutes === 0 ? 0 : Math.round((minutes / totalFocusMinutes) * 1000) / 10;

  const typeRatios: Record<string, number> = {
    work: 0.52,
    study: 0.22,
    life: 0.18,
    other: 0.08,
  };
  let remainingMinutes = totalFocusMinutes;
  const byType = (['work', 'study', 'life', 'other'] as const).map((type, index) => {
    const ratio = typeRatios[type] ?? 0;
    const minutes =
      index === 3 ? Math.max(0, remainingMinutes) : Math.round(totalFocusMinutes * ratio);
    remainingMinutes -= minutes;
    return {
      type,
      minutes,
      percentage: ensurePercentage(minutes),
    };
  });

  const priorityRatios: Record<string, number> = {
    urgent: 0.1,
    high: 0.32,
    medium: 0.4,
    low: 0.18,
  };
  remainingMinutes = totalFocusMinutes;
  const byPriority = (['urgent', 'high', 'medium', 'low'] as const).map((priority, index) => {
    const ratio = priorityRatios[priority] ?? 0;
    const minutes =
      index === 3 ? Math.max(0, remainingMinutes) : Math.round(totalFocusMinutes * ratio);
    remainingMinutes -= minutes;
    return {
      priority,
      minutes,
      percentage: ensurePercentage(minutes),
    };
  });

  const timeAllocation = {
    byType,
    byPriority,
    byStatus: [
      {
        label: '按时完成',
        minutes: Math.round(totalFocusMinutes * lastPoint.completionRate),
        percentage: ensurePercentage(Math.round(totalFocusMinutes * lastPoint.completionRate)),
      },
      {
        label: '延迟完成',
        minutes: Math.round(totalFocusMinutes * (1 - lastPoint.completionRate)),
        percentage: ensurePercentage(
          Math.round(totalFocusMinutes * (1 - lastPoint.completionRate)),
        ),
      },
    ],
  } as AnalyticsOverviewResponse['overview']['timeAllocation'];

  const suggestions: AnalyticsOverviewResponse['overview']['efficiency']['suggestions'] = [
    {
      id: 'mock-focus-window',
      title: '调整专注窗口',
      summary: '下午 14:00-17:00 的完成率最高，可在该时段安排高优先级任务。',
      impact: 'high',
      confidence: Number(clamp(0.72 + pseudoRandom(totalCompleted + 2) * 0.2, 0, 1).toFixed(2)),
      category: 'focus',
    },
    {
      id: 'mock-plan-sync',
      title: '同步规划与执行',
      summary: '部分规划块未应用到任务，建议重新查看规划面板。',
      impact: 'medium',
      confidence: Number(clamp(0.6 + pseudoRandom(totalCompleted + 3) * 0.2, 0, 1).toFixed(2)),
      category: 'planning',
      relatedPlanId: memoryStore.planningSession?.session.id,
    },
  ];

  const nowIso = new Date().toISOString();
  const insights: AnalyticsOverviewResponse['overview']['insights'] = [
    {
      id: 'insight-completion-rate',
      headline: '完成率较上周期提升',
      detail: `完成率变化 ${trendDelta >= 0 ? '+' : ''}${trendDelta.toFixed(1)}%`,
      actionLabel: '查看任务',
      actionHref: '/tasks',
      severity: trendDelta >= 0 ? 'success' : 'warning',
      relatedIds: memoryStore.tasks.slice(0, 3).map((task) => task.id),
      generatedAt: nowIso,
      source: 'rule',
    },
    {
      id: 'insight-focus-distribution',
      headline: '专注时间主要集中在下午',
      detail: '下午时段的专注时间占比 58%，可在上午安排轻量任务。',
      actionLabel: '查看日历',
      actionHref: '/calendar',
      severity: 'info',
      generatedAt: nowIso,
      source: 'ai',
    },
  ];

  const zeroState = {
    isEmpty: memoryStore.tasks.length === 0,
    recommendedActions:
      memoryStore.tasks.length === 0 ? ['创建你的第一项任务', '应用一份规划方案'] : [],
    sampleDataAvailable: true,
    sampleDataLabel: '示例分析数据',
    missingConfiguration: [],
  } satisfies AnalyticsOverviewResponse['overview']['zeroState'];

  return {
    overview: {
      range,
      summary: {
        totalCompleted,
        completionRate: Number(lastPoint.completionRate.toFixed(3)),
        trendDelta: Number(trendDelta.toFixed(1)),
        workloadPrediction: Math.max(
          lastPoint.completedTasks,
          Math.round(lastPoint.completedTasks * 1.1 + pseudoRandom(totalCompleted + 1) * 3),
        ),
        focusMinutes: totalFocusMinutes,
        overdueTasks: totalOverdue,
      },
      trend: points.map((point) => ({
        date: point.date,
        completionRate: point.completionRate,
        productivityScore: point.productivityScore,
        completedTasks: point.completedTasks,
        focusMinutes: point.focusMinutes,
      })),
      timeAllocation,
      efficiency: {
        estimateAccuracy: Number(
          clamp(0.62 + pseudoRandom(totalCompleted + 4) * 0.26, 0, 1).toFixed(3),
        ),
        onTimeRate: Number(clamp(0.66 + pseudoRandom(totalCompleted + 5) * 0.28, 0, 1).toFixed(3)),
        complexityCorrelation: Number(
          clamp(0.42 + pseudoRandom(totalCompleted + 6) * 0.35, 0, 1).toFixed(3),
        ),
        suggestions,
      },
      insights,
      zeroState,
      meta: {
        generatedAt: nowIso,
        isDemo: true,
      },
    },
    history,
    error: null,
  };
};

const getMockAnalyticsResponse = (params: AnalyticsQueryParams): AnalyticsOverviewResponse => {
  const range = params.range ?? '7d';
  if (!memoryStore.analytics || memoryStore.lastAnalyticsRange !== range) {
    memoryStore.analytics = createMockAnalyticsOverview(range);
    memoryStore.lastAnalyticsRange = range;
  }
  return memoryStore.analytics;
};

const normalizeAnalyticsHistoryResponse = (
  payload: unknown,
  fallbackRange: AnalyticsRangeKey,
): AnalyticsHistoryResponse => {
  if (!payload || typeof payload !== 'object') {
    return createMockAnalyticsHistory(fallbackRange);
  }

  const history = payload as Partial<AnalyticsHistoryResponse>;
  const grouping: AnalyticsGrouping =
    history.grouping === 'day' || history.grouping === 'week'
      ? history.grouping
      : fallbackRange === '7d'
        ? 'day'
        : 'week';

  const points = Array.isArray(history.points)
    ? (history.points
        .map((point) => {
          if (!point || typeof point !== 'object') return null;
          const record = point as Partial<AnalyticsHistoryResponse['points'][number]>;
          return {
            date: typeof record.date === 'string' ? record.date : new Date().toISOString(),
            productivityScore:
              typeof record.productivityScore === 'number' ? record.productivityScore : 70,
            completionRate:
              typeof record.completionRate === 'number' ? clamp(record.completionRate, 0, 1) : 0.7,
            focusMinutes: typeof record.focusMinutes === 'number' ? record.focusMinutes : 240,
            completedTasks: typeof record.completedTasks === 'number' ? record.completedTasks : 6,
            overdueTasks: typeof record.overdueTasks === 'number' ? record.overdueTasks : 1,
          } satisfies AnalyticsHistoryResponse['points'][number];
        })
        .filter(Boolean) as AnalyticsHistoryResponse['points'])
    : [];

  if (points.length === 0) {
    return createMockAnalyticsHistory(fallbackRange);
  }

  return {
    range: history.range ?? fallbackRange,
    grouping,
    points,
  };
};

const normalizeAnalyticsOverviewResponse = (
  payload: unknown,
  fallbackRange: AnalyticsRangeKey,
): AnalyticsOverviewResponse => {
  if (!payload || typeof payload !== 'object') {
    return createMockAnalyticsOverview(fallbackRange);
  }

  const input = payload as Partial<AnalyticsOverviewResponse>;
  if (!input.overview) {
    return createMockAnalyticsOverview(fallbackRange);
  }

  const overview = input.overview;
  const history = normalizeAnalyticsHistoryResponse(
    input.history ?? null,
    overview.range ?? fallbackRange,
  );

  const zeroState = overview.zeroState ?? {
    isEmpty: false,
    recommendedActions: [],
    sampleDataAvailable: false,
  };

  const daysByRange: Record<AnalyticsRangeKey, number> = {
    '7d': 7,
    '30d': 30,
    '90d': 90,
  };

  const activeRange = overview.range ?? fallbackRange;
  const rangeDays = daysByRange[activeRange] ?? 7;
  const focusMinutesFromHistory = history.points.reduce((sum, point) => {
    const value = typeof point.focusMinutes === 'number' ? point.focusMinutes : 0;
    return sum + (Number.isFinite(value) ? Math.max(0, value) : 0);
  }, 0);

  const rawSummary = overview.summary ?? null;
  const rawFocusMinutes = rawSummary?.focusMinutes;
  const focusMinutesCandidates = [
    typeof rawFocusMinutes === 'number' && Number.isFinite(rawFocusMinutes)
      ? Math.max(0, rawFocusMinutes)
      : null,
    focusMinutesFromHistory > 0 ? focusMinutesFromHistory : null,
  ].filter((value): value is number => value !== null);
  const fallbackFocusMinutes = Math.min(rangeDays * 24 * 60, focusMinutesFromHistory);
  const normalizedFocusMinutes = focusMinutesCandidates.length
    ? Math.min(rangeDays * 24 * 60, Math.min(...focusMinutesCandidates))
    : Math.max(0, fallbackFocusMinutes);

  const totalCompletedFromHistory = history.points.reduce(
    (sum, point) => sum + point.completedTasks,
    0,
  );

  const totalOverdueFromHistory = history.points.reduce(
    (sum, point) => sum + point.overdueTasks,
    0,
  );

  const averageCompletionRateFromHistory =
    history.points.reduce((sum, point) => sum + point.completionRate, 0) /
    Math.max(1, history.points.length);

  const firstCompletionRate = history.points.length ? history.points[0].completionRate : 0;
  const lastCompletionRate = history.points.length
    ? history.points[history.points.length - 1].completionRate
    : 0;

  const sanitizedSummary: AnalyticsOverviewResponse['overview']['summary'] = {
    totalCompleted:
      typeof rawSummary?.totalCompleted === 'number' && Number.isFinite(rawSummary.totalCompleted)
        ? Math.max(0, Math.round(rawSummary.totalCompleted))
        : Math.max(0, Math.round(totalCompletedFromHistory)),
    completionRate:
      typeof rawSummary?.completionRate === 'number' && Number.isFinite(rawSummary.completionRate)
        ? clamp(rawSummary.completionRate, 0, 1)
        : clamp(averageCompletionRateFromHistory, 0, 1),
    trendDelta:
      typeof rawSummary?.trendDelta === 'number' && Number.isFinite(rawSummary.trendDelta)
        ? rawSummary.trendDelta
        : (lastCompletionRate - firstCompletionRate) * 100,
    workloadPrediction:
      typeof rawSummary?.workloadPrediction === 'number' &&
      Number.isFinite(rawSummary.workloadPrediction)
        ? Math.max(0, Math.round(rawSummary.workloadPrediction))
        : Math.max(0, Math.round(totalCompletedFromHistory / Math.max(1, rangeDays))),
    focusMinutes: normalizedFocusMinutes,
    overdueTasks:
      typeof rawSummary?.overdueTasks === 'number' && Number.isFinite(rawSummary.overdueTasks)
        ? Math.max(0, Math.round(rawSummary.overdueTasks))
        : Math.max(0, Math.round(totalOverdueFromHistory)),
  };

  return {
    overview: {
      range: overview.range ?? fallbackRange,
      summary: sanitizedSummary,
      trend:
        overview.trend ??
        history.points.map((point) => ({
          date: point.date,
          completionRate: point.completionRate,
          productivityScore: point.productivityScore,
          completedTasks: point.completedTasks,
          focusMinutes: point.focusMinutes,
        })),
      timeAllocation:
        overview.timeAllocation ??
        ({
          byType: [],
          byPriority: [],
          byStatus: [],
        } satisfies AnalyticsOverviewResponse['overview']['timeAllocation']),
      efficiency:
        overview.efficiency ??
        ({
          estimateAccuracy: 0,
          onTimeRate: 0,
          complexityCorrelation: 0,
          suggestions: [],
        } satisfies AnalyticsOverviewResponse['overview']['efficiency']),
      insights: overview.insights ?? [],
      zeroState: {
        isEmpty: Boolean(zeroState.isEmpty),
        recommendedActions: zeroState.recommendedActions ?? [],
        sampleDataAvailable: Boolean(zeroState.sampleDataAvailable),
        sampleDataLabel: zeroState.sampleDataLabel,
        missingConfiguration: zeroState.missingConfiguration ?? [],
      },
      meta: {
        generatedAt: overview.meta?.generatedAt ?? new Date().toISOString(),
        isDemo: Boolean(overview.meta?.isDemo),
      },
    },
    history,
    error: input.error ?? null,
  };
};

const normalizeAnalyticsExportResult = (
  payload: unknown,
  fallback: AnalyticsExportParams,
): AnalyticsExportResult => {
  const extension = fallback.format === 'markdown' ? 'md' : 'json';
  const defaultPath = `mock://analytics/report-${fallback.range}.${extension}`;

  if (!payload || typeof payload !== 'object') {
    return {
      filePath: defaultPath,
      format: fallback.format,
      generatedAt: new Date().toISOString(),
      isDemo: false,
    } satisfies AnalyticsExportResult;
  }

  const input = payload as Partial<AnalyticsExportResult>;
  const filePath =
    typeof input.filePath === 'string' && input.filePath.trim().length > 0
      ? input.filePath
      : defaultPath;

  return {
    filePath,
    format: input.format ?? fallback.format,
    generatedAt: input.generatedAt ?? new Date().toISOString(),
    isDemo: Boolean(input.isDemo),
  } satisfies AnalyticsExportResult;
};

const removeUndefinedDeep = <T>(value: T): T => {
  if (Array.isArray(value)) {
    return value.map((item) => removeUndefinedDeep(item)) as unknown as T;
  }

  if (value && typeof value === 'object') {
    const result: Record<string, unknown> = {};
    Object.entries(value as Record<string, unknown>).forEach(([key, val]) => {
      if (val === undefined) return;
      result[key] = removeUndefinedDeep(val);
    });
    return result as T;
  }

  return value;
};

const sanitizeDashboardConfigInput = (
  input: DashboardConfigInput | null | undefined,
): DashboardConfigInput => {
  if (!input || typeof input !== 'object') {
    return {};
  }

  const sanitized: DashboardConfigInput = {};

  if (input.modules && typeof input.modules === 'object') {
    let overrides: DashboardConfigInput['modules'] | undefined;
    for (const id of DASHBOARD_MODULE_IDS) {
      const override = input.modules[id];
      if (typeof override === 'boolean') {
        if (!overrides) {
          overrides = {};
        }
        overrides[id] = override;
      }
    }
    if (overrides) {
      sanitized.modules = overrides;
    }
  }

  if (typeof input.lastUpdatedAt === 'string' || input.lastUpdatedAt === null) {
    sanitized.lastUpdatedAt = input.lastUpdatedAt;
  }

  return sanitized;
};

const mergeDashboardConfig = (
  current: DashboardConfig,
  patch: DashboardConfigInput | null | undefined,
): DashboardConfig => {
  const sanitized = sanitizeDashboardConfigInput(patch);
  const modules = { ...current.modules };

  if (sanitized.modules) {
    for (const id of DASHBOARD_MODULE_IDS) {
      if (sanitized.modules[id] !== undefined) {
        modules[id] = sanitized.modules[id] as boolean;
      }
    }
  }

  const timestamp =
    sanitized.lastUpdatedAt === undefined ? new Date().toISOString() : sanitized.lastUpdatedAt;

  return normalizeDashboardConfig({
    modules,
    lastUpdatedAt: timestamp,
  });
};

const parseIsoToMs = (iso?: string | null): number | null => {
  if (!iso) return null;
  const timestamp = Date.parse(iso);
  return Number.isNaN(timestamp) ? null : timestamp;
};

const resolveTaskStart = (task: Task): number | null => {
  return (
    parseIsoToMs(task.startAt) ??
    parseIsoToMs(task.plannedStartAt) ??
    parseIsoToMs(task.ai?.suggestedStartAt ?? null) ??
    parseIsoToMs(task.dueAt)
  );
};

const resolveTaskEnd = (task: Task): number | null => {
  return parseIsoToMs(task.dueAt) ?? parseIsoToMs(task.completedAt) ?? resolveTaskStart(task);
};

const applyFilters = (tasks: Task[], filters: TaskFilters): Task[] => {
  let filtered = [...tasks];

  if (!filters.includeArchived) {
    filtered = filtered.filter((task) => task.status !== 'archived');
  }
  if (filters.statuses?.length) {
    filtered = filtered.filter((task) => filters.statuses?.includes(task.status));
  }
  if (filters.priorities?.length) {
    filtered = filtered.filter((task) => filters.priorities?.includes(task.priority));
  }
  if (filters.tags?.length) {
    filtered = filtered.filter((task) => task.tags.some((tag) => filters.tags?.includes(tag)));
  }
  if (filters.taskTypes?.length) {
    filtered = filtered.filter((task) =>
      task.taskType ? filters.taskTypes?.includes(task.taskType) : false,
    );
  }
  if (filters.ownerIds?.length) {
    filtered = filtered.filter((task) =>
      task.ownerId ? filters.ownerIds?.includes(task.ownerId) : false,
    );
  }
  if (filters.search) {
    const keyword = filters.search.toLowerCase();
    filtered = filtered.filter(
      (task) =>
        task.title.toLowerCase().includes(keyword) ||
        (task.description ? task.description.toLowerCase().includes(keyword) : false),
    );
  }
  if (filters.dueAfter) {
    const dueAfter = Date.parse(filters.dueAfter);
    filtered = filtered.filter((task) => {
      if (!task.dueAt) return false;
      const due = Date.parse(task.dueAt);
      return !Number.isNaN(dueAfter) && !Number.isNaN(due) && due >= dueAfter;
    });
  }
  if (filters.dueBefore) {
    const dueBefore = Date.parse(filters.dueBefore);
    filtered = filtered.filter((task) => {
      if (!task.dueAt) return false;
      const due = Date.parse(task.dueAt);
      return !Number.isNaN(dueBefore) && !Number.isNaN(due) && due <= dueBefore;
    });
  }
  const windowStart = parseIsoToMs(filters.windowStart ?? null);
  const windowEnd = parseIsoToMs(filters.windowEnd ?? null);
  if (windowStart !== null || windowEnd !== null) {
    filtered = filtered.filter((task) => {
      const taskStart = resolveTaskStart(task);
      const taskEnd = resolveTaskEnd(task);
      if (windowStart !== null && (taskEnd === null || taskEnd < windowStart)) {
        return false;
      }
      if (windowEnd !== null && (taskStart === null || taskStart > windowEnd)) {
        return false;
      }
      return true;
    });
  }
  if (filters.complexityMin !== undefined) {
    filtered = filtered.filter((task) => {
      const score = task.ai?.complexityScore;
      return typeof score === 'number' && score >= filters.complexityMin!;
    });
  }
  if (filters.complexityMax !== undefined) {
    filtered = filtered.filter((task) => {
      const score = task.ai?.complexityScore;
      return typeof score === 'number' && score <= filters.complexityMax!;
    });
  }
  if (filters.aiSuggestedAfter) {
    const after = Date.parse(filters.aiSuggestedAfter);
    filtered = filtered.filter((task) => {
      const suggested = task.ai?.suggestedStartAt ? Date.parse(task.ai.suggestedStartAt) : NaN;
      return !Number.isNaN(after) && !Number.isNaN(suggested) && suggested >= after;
    });
  }
  if (filters.aiSuggestedBefore) {
    const before = Date.parse(filters.aiSuggestedBefore);
    filtered = filtered.filter((task) => {
      const suggested = task.ai?.suggestedStartAt ? Date.parse(task.ai.suggestedStartAt) : NaN;
      return !Number.isNaN(before) && !Number.isNaN(suggested) && suggested <= before;
    });
  }
  if (filters.aiSources?.length) {
    filtered = filtered.filter((task) =>
      task.ai?.source ? filters.aiSources?.includes(task.ai.source) : false,
    );
  }

  if (filters.sortBy) {
    const dir = filters.sortOrder === 'desc' ? -1 : 1;
    filtered.sort((a, b) => {
      const field = filters.sortBy!;
      const aValue = a[field];
      const bValue = b[field];

      if (!aValue && !bValue) return 0;
      if (!aValue) return -dir;
      if (!bValue) return dir;

      if (typeof aValue === 'string' && typeof bValue === 'string') {
        return aValue.localeCompare(bValue) * dir;
      }

      return 0;
    });
  }

  return filtered;
};

const paginate = (tasks: Task[], page: number, pageSize: number): TaskListResponse => {
  const total = tasks.length;
  const start = (page - 1) * pageSize;
  const items = tasks.slice(start, start + pageSize);
  return { items, total, page, pageSize };
};

const mapUnknownError = (error: unknown): AppError => {
  if (!error) {
    return createAppError('UNKNOWN', '发生未知错误');
  }

  if (
    typeof error === 'object' &&
    error !== null &&
    'code' in (error as Record<string, unknown>) &&
    'message' in (error as Record<string, unknown>)
  ) {
    const record = error as Record<string, unknown>;
    const code = normalizeCommandCode(record.code);
    const message = resolveErrorMessage(
      code,
      typeof record.message === 'string' ? record.message : undefined,
    );
    const details = record.details ?? record.data;
    const correlationId = extractCorrelationId(details);
    return createAppError(code, message, details, error, correlationId);
  }

  if (error instanceof ZodError) {
    return createAppError('VALIDATION_ERROR', '字段校验失败', formatZodError(error));
  }

  const payload = extractCommandErrorPayload(error);
  if (payload) {
    return mapCommandErrorPayload(payload, error);
  }

  if (error instanceof Error) {
    if (error.message?.includes('__TAURI_IPC__')) {
      return createAppError(
        'TAURI_UNAVAILABLE',
        '桌面引擎暂不可用，无法调用本地服务',
        undefined,
        error,
      );
    }
    return createAppError(
      'UNKNOWN',
      resolveErrorMessage('UNKNOWN', error.message),
      undefined,
      error,
    );
  }

  if (typeof error === 'string') {
    return createAppError('UNKNOWN', resolveErrorMessage('UNKNOWN', error));
  }

  return createAppError('UNKNOWN', resolveErrorMessage('UNKNOWN'), error);
};

const invokeOrMock = async <T>(command: string, payload?: Record<string, unknown>): Promise<T> => {
  const sanitizedPayload = payload
    ? (removeUndefinedDeep(payload) as Record<string, unknown>)
    : undefined;

  const e2eMatch = await tryConsumeE2EMock(command, sanitizedPayload);
  if (e2eMatch.matched) {
    return e2eMatch.value as T;
  }

  if (isTauriAvailable()) {
    try {
      return await invoke<T>(command, sanitizedPayload);
    } catch (error) {
      throw mapUnknownError(error);
    }
  }

  if (!isDevelopment) {
    throw createAppError('TAURI_UNAVAILABLE', '桌面引擎暂不可用，无法调用本地服务', {
      command,
    });
  }

  warnMockUsage();
  ensureSampleData();

  const filters = sanitizedPayload?.filters ? (sanitizedPayload.filters as TaskFilters) : undefined;
  const extractPlanningPayload = (
    value: Record<string, unknown> | undefined,
  ): Record<string, unknown> | undefined => {
    if (!value) return undefined;
    if ('payload' in value) {
      const nested = value.payload;
      if (nested && typeof nested === 'object') {
        return nested as Record<string, unknown>;
      }
    }
    return value;
  };

  switch (command) {
    case COMMANDS.SETTINGS_GET: {
      const settings = ensureAppSettings();
      return normalizeInternalSettings(settings) as T;
    }
    case COMMANDS.SETTINGS_UPDATE: {
      const settings = ensureAppSettings();
      const payloadInput = (sanitizedPayload?.payload ?? sanitizedPayload) as
        | UpdateAppSettingsInput
        | undefined;

      const next: InternalAppSettings = {
        ...settings,
      };

      if (payloadInput) {
        if (payloadInput.removeDeepseekKey) {
          next.deepseekApiKey = null;
        } else if (typeof payloadInput.deepseekApiKey === 'string') {
          const trimmed = payloadInput.deepseekApiKey.trim();
          next.deepseekApiKey = trimmed.length > 0 ? trimmed : null;
        }

        const payloadRecord = payloadInput as Record<string, unknown>;

        if (typeof payloadInput.workdayStartMinute === 'number') {
          next.workdayStartMinute = clampMinute(
            payloadInput.workdayStartMinute,
            settings.workdayStartMinute,
          );
        } else if (typeof payloadRecord.workdayStartHour === 'number') {
          next.workdayStartMinute = clampMinute(
            (payloadRecord.workdayStartHour as number) * 60,
            settings.workdayStartMinute,
          );
        }

        if (typeof payloadInput.workdayEndMinute === 'number') {
          next.workdayEndMinute = clampMinute(
            payloadInput.workdayEndMinute,
            settings.workdayEndMinute,
          );
        } else if (typeof payloadRecord.workdayEndHour === 'number') {
          next.workdayEndMinute = clampMinute(
            (payloadRecord.workdayEndHour as number) * 60,
            settings.workdayEndMinute,
          );
        }

        if (payloadInput.themePreference) {
          const theme = payloadInput.themePreference;
          if (theme === 'light' || theme === 'dark' || theme === 'system') {
            next.themePreference = theme;
          }
        }

        if (typeof payloadInput.aiFeedbackOptOut === 'boolean') {
          next.aiFeedbackOptOut = payloadInput.aiFeedbackOptOut;
        }
      }

      next.workdayEndMinute = ensureValidEndMinute(next.workdayStartMinute, next.workdayEndMinute);

      next.lastUpdatedAt = new Date().toISOString();
      memoryStore.settings = next;
      return normalizeInternalSettings(next) as T;
    }
    case COMMANDS.DASHBOARD_CONFIG_GET: {
      const settings = ensureAppSettings();
      return deepClone(settings.dashboardConfig) as T;
    }
    case COMMANDS.DASHBOARD_CONFIG_UPDATE: {
      const patch = (sanitizedPayload?.payload ?? sanitizedPayload) as
        | DashboardConfigInput
        | null
        | undefined;
      const settings = ensureAppSettings();
      const nextConfig = mergeDashboardConfig(settings.dashboardConfig, patch);
      memoryStore.settings = {
        ...settings,
        dashboardConfig: nextConfig,
      } satisfies InternalAppSettings;
      return deepClone(nextConfig) as T;
    }
    case COMMANDS.LIST: {
      const parsedFilters = filters ?? { page: 1, pageSize: DEFAULT_PAGE_SIZE };
      const filtered = applyFilters(memoryStore.tasks, parsedFilters);
      return paginate(
        filtered,
        parsedFilters.page ?? 1,
        parsedFilters.pageSize ?? DEFAULT_PAGE_SIZE,
      ) as T;
    }
    case COMMANDS.CREATE: {
      const parsed = sanitizedPayload?.payload as TaskPayload;
      const now = new Date().toISOString();
      const task: Task = {
        id: createId(),
        title: parsed.title,
        description: parsed.description,
        status: parsed.status ?? 'todo',
        priority: parsed.priority ?? 'medium',
        plannedStartAt: parsed.plannedStartAt,
        startAt: parsed.startAt,
        dueAt: parsed.dueAt,
        completedAt: parsed.completedAt,
        estimatedMinutes: parsed.estimatedMinutes,
        estimatedHours: parsed.estimatedHours,
        tags: parsed.tags ?? [],
        ownerId: parsed.ownerId,
        isRecurring: parsed.isRecurring ?? false,
        recurrence: parsed.recurrence,
        taskType: parsed.taskType,
        ai: parsed.ai,
        externalLinks: parsed.externalLinks ?? [],
        createdAt: now,
        updatedAt: now,
      };
      memoryStore.tasks = [task, ...memoryStore.tasks];
      return task as T;
    }
    case COMMANDS.UPDATE: {
      const { id } = (sanitizedPayload ?? {}) as { id: string };
      const updatePayload = (sanitizedPayload?.payload as Partial<Task> | undefined) ?? {};
      const index = memoryStore.tasks.findIndex((task) => task.id === id);
      if (index === -1) {
        throw createAppError('NOT_FOUND', '任务不存在');
      }
      const current = memoryStore.tasks[index];
      const updated: Task = {
        ...current,
        ...updatePayload,
        tags: updatePayload.tags ?? current.tags,
        isRecurring: updatePayload.isRecurring ?? current.isRecurring,
        updatedAt: new Date().toISOString(),
      };
      memoryStore.tasks[index] = updated;
      return updated as T;
    }
    case COMMANDS.DELETE: {
      const { id } = (sanitizedPayload ?? {}) as { id: string };
      const index = memoryStore.tasks.findIndex((task) => task.id === id);
      if (index === -1) {
        throw createAppError('NOT_FOUND', '任务不存在');
      }
      memoryStore.tasks.splice(index, 1);
      return undefined as T;
    }
    case COMMANDS.PARSE: {
      const { input, context } = (sanitizedPayload ?? {}) as Partial<TaskParseRequest>;
      if (!input || input.trim().length === 0) {
        throw createAppError('VALIDATION_ERROR', '待解析内容不能为空');
      }

      const now = new Date();
      const generatedAt = now.toISOString();
      const normalizedTitle = input.trim().slice(0, 120) || 'AI 解析任务';
      const suggestedStart = new Date(now.getTime() + 60 * 60 * 1000).toISOString();
      const dueAt = new Date(now.getTime() + 4 * 60 * 60 * 1000).toISOString();
      const metadata = context?.metadata as Record<string, unknown> | undefined;
      const aiSource: TaskParseResponse['ai']['source'] =
        typeof metadata?.useCache === 'boolean' && metadata.useCache ? 'cache' : 'live';

      const payload: Partial<TaskPayload> = {
        title: normalizedTitle,
        description: input.trim(),
        priority: 'medium',
        plannedStartAt: context?.referenceDate ?? generatedAt,
        startAt: suggestedStart,
        dueAt,
        estimatedMinutes: 120,
        estimatedHours: 2,
        taskType: 'work',
        tags: ['ai', 'draft'],
      };

      const aiResult: TaskParseResponse['ai'] = {
        summary: `系统建议以“${normalizedTitle}”为任务标题，并补充关键字段。`,
        nextAction: '确认主要目标是否准确，再拆分子任务。',
        confidence: 0.78,
        complexityScore: 5,
        suggestedStartAt: suggestedStart,
        focusMode: { pomodoros: 3, recommendedSlots: [suggestedStart] },
        efficiencyPrediction: { expectedHours: 2, confidence: 0.62 },
        cotSteps: [
          {
            order: 0,
            title: '提炼目标',
            detail: '解析输入文本中的核心动词与期望结果。',
            outcome: '得到任务主题与成功标准。',
          },
          {
            order: 1,
            title: '识别约束',
            detail: '扫描文本中的日期、优先级与外部依赖。',
            outcome: '确定计划开始与截止时间建议。',
          },
          {
            order: 2,
            title: '生成建议',
            detail: '结合经验库预估复杂度与所需专注模式。',
            outcome: '给出番茄钟数量与效率预测。',
          },
        ],
        cotSummary: '该任务需要专注执行，建议预留 3 个番茄钟完成准备与交付。',
        source: aiSource,
        generatedAt: generatedAt,
      };

      const missingFields: TaskParseResponse['missingFields'] = ['ownerId'];

      return {
        payload,
        ai: aiResult,
        missingFields,
      } as T;
    }
    case COMMANDS.AI_STATUS: {
      const settings = ensureAppSettings();
      const hasKey = Boolean(settings.deepseekApiKey && settings.deepseekApiKey.trim().length > 0);
      const nowIso = new Date().toISOString();
      const latency = hasKey
        ? Math.round(120 + Math.random() * 80)
        : Math.round(5 + Math.random() * 5);

      const status: AiStatus = {
        status: hasKey ? 'online' : 'missing_key',
        hasApiKey: hasKey,
        lastCheckedAt: nowIso,
        latencyMs: hasKey ? latency : null,
        provider: hasKey
          ? {
              providerId: 'deepseek',
              model: 'deepseek-chat',
              latencyMs: latency,
              tokensUsed: null,
              extra: { environment: 'mock' },
            }
          : null,
        message: hasKey ? null : '未检测到 DeepSeek API Key，请先在此处配置有效密钥。',
      };
      memoryStore.aiStatus = status;
      return status as T;
    }
    case COMMANDS.PLANNING_GENERATE: {
      const settings = ensureAppSettings();
      const hasKey = Boolean(settings.deepseekApiKey && settings.deepseekApiKey.trim().length > 0);
      if (!hasKey) {
        throw createAppError('MISSING_API_KEY', '未检测到 DeepSeek API Key，无法生成智能规划方案');
      }
      // 真实 DeepSeek 调用由 Rust 后端处理；仅开发环境调用此 mock
      throw createAppError('TAURI_UNAVAILABLE', '规划功能需要连接桌面服务');
    }
    case COMMANDS.PLANNING_APPLY: {
      const rawPayload = extractPlanningPayload(sanitizedPayload);
      try {
        const parsed = applyPlanInputSchema.parse(rawPayload ?? {});
        const existing = memoryStore.planningSession;
        if (!existing || existing.session.id !== parsed.sessionId) {
          throw createAppError('NOT_FOUND', '未找到对应的规划会话');
        }

        const sessionView = deepClone(existing);
        const optionIndex = sessionView.options.findIndex(
          (item) => item.option.id === parsed.optionId,
        );
        if (optionIndex === -1) {
          throw createAppError('NOT_FOUND', '未找到对应的规划方案');
        }

        const nowIso = new Date().toISOString();
        const targetOption = sessionView.options[optionIndex];
        const updatedOption = {
          ...targetOption,
          blocks: targetOption.blocks.map((block) => {
            const override = parsed.overrides.find((item) => item.blockId === block.id);
            return {
              ...block,
              startAt: override?.startAt ?? block.startAt,
              endAt: override?.endAt ?? block.endAt,
              flexibility: override?.flexibility ?? block.flexibility,
              appliedAt: nowIso,
              status: 'planned' as const,
              conflictFlags: [],
            };
          }),
          conflicts: [],
        };

        sessionView.options[optionIndex] = updatedOption;
        sessionView.session.status = 'applied';
        sessionView.session.selectedOptionId = updatedOption.option.id;
        sessionView.session.updatedAt = nowIso;
        sessionView.conflicts = [];

        memoryStore.planningSession = deepClone(sessionView);

        const appliedCandidate = {
          session: sessionView.session,
          option: updatedOption,
          conflicts: updatedOption.conflicts,
        };

        return parseAppliedPlan(appliedCandidate) as T;
      } catch (error) {
        if (error instanceof ZodError) {
          throw createAppError('VALIDATION_ERROR', '应用规划请求校验失败', formatZodError(error));
        }
        if (isAppError(error)) {
          throw error;
        }
        throw mapUnknownError(error);
      }
    }
    case COMMANDS.PLANNING_RESOLVE_CONFLICT: {
      const rawPayload = extractPlanningPayload(sanitizedPayload);
      try {
        const parsed = resolveConflictInputSchema.parse(rawPayload ?? {});
        const existing = memoryStore.planningSession;
        if (!existing || existing.session.id !== parsed.sessionId) {
          throw createAppError('NOT_FOUND', '未找到对应的规划会话');
        }

        const sessionView = deepClone(existing);
        const optionIndex = sessionView.options.findIndex(
          (item) => item.option.id === parsed.optionId,
        );
        if (optionIndex === -1) {
          throw createAppError('NOT_FOUND', '未找到对应的规划方案');
        }

        const updatedOption = {
          ...sessionView.options[optionIndex],
          blocks: sessionView.options[optionIndex].blocks.map((block) => {
            const override = parsed.adjustments.find((item) => item.blockId === block.id);
            return {
              ...block,
              startAt: override?.startAt ?? block.startAt,
              endAt: override?.endAt ?? block.endAt,
              flexibility: override?.flexibility ?? block.flexibility,
              conflictFlags: [],
            };
          }),
          conflicts: [],
        };

        sessionView.options[optionIndex] = updatedOption;
        sessionView.conflicts = [];
        sessionView.session.updatedAt = new Date().toISOString();

        memoryStore.planningSession = deepClone(sessionView);
        return parsePlanningSessionView(sessionView) as T;
      } catch (error) {
        if (error instanceof ZodError) {
          throw createAppError('VALIDATION_ERROR', '冲突调整请求校验失败', formatZodError(error));
        }
        if (isAppError(error)) {
          throw error;
        }
        throw mapUnknownError(error);
      }
    }
    case COMMANDS.PLANNING_PREFERENCES_GET: {
      const preferenceId =
        (sanitizedPayload?.preference_id as string | undefined) ??
        (sanitizedPayload?.preferenceId as string | undefined) ??
        'default';
      const snapshot = deepClone(ensurePreferenceSnapshot(preferenceId ?? 'default'));
      return parsePreferenceSnapshot(snapshot) as T;
    }
    case COMMANDS.PLANNING_PREFERENCES_UPDATE: {
      const rawPayload = extractPlanningPayload(sanitizedPayload);
      try {
        const parsed = planningPreferencesUpdateSchema.parse(rawPayload ?? {});
        const prefId =
          parsed.preferenceId && parsed.preferenceId.trim().length > 0
            ? parsed.preferenceId
            : 'default';
        memoryStore.preferences[prefId] = deepClone(parsed.snapshot);
        if (memoryStore.planningSession) {
          memoryStore.planningSession.preferenceSnapshot = deepClone(parsed.snapshot);
        }
        return undefined as T;
      } catch (error) {
        if (error instanceof ZodError) {
          throw createAppError('VALIDATION_ERROR', '偏好更新请求校验失败', formatZodError(error));
        }
        throw mapUnknownError(error);
      }
    }
    case COMMANDS.ANALYTICS_OVERVIEW_FETCH: {
      const range = (sanitizedPayload?.range as AnalyticsRangeKey | undefined) ?? '7d';
      const response = getMockAnalyticsResponse({
        range,
        from: sanitizedPayload?.from as string | undefined,
        to: sanitizedPayload?.to as string | undefined,
        grouping: sanitizedPayload?.grouping as AnalyticsGrouping | undefined,
      });
      return response as T;
    }
    case COMMANDS.ANALYTICS_HISTORY_FETCH: {
      const range = (sanitizedPayload?.range as AnalyticsRangeKey | undefined) ?? '7d';
      const history = getMockAnalyticsResponse({
        range,
        from: sanitizedPayload?.from as string | undefined,
        to: sanitizedPayload?.to as string | undefined,
        grouping: sanitizedPayload?.grouping as AnalyticsGrouping | undefined,
      }).history;
      return history as T;
    }
    case COMMANDS.ANALYTICS_REPORT_EXPORT: {
      const range = (sanitizedPayload?.range as AnalyticsRangeKey | undefined) ?? '7d';
      const format =
        (sanitizedPayload?.format as AnalyticsExportParams['format'] | undefined) ?? 'markdown';
      const result = normalizeAnalyticsExportResult(
        {
          filePath: `mock://analytics/report-${range}.${format === 'markdown' ? 'md' : 'json'}`,
          format,
          generatedAt: new Date().toISOString(),
          isDemo: true,
        },
        {
          range,
          format,
          from: sanitizedPayload?.from as string | undefined,
          to: sanitizedPayload?.to as string | undefined,
        },
      );
      return result as T;
    }
    default:
      throw createAppError('UNKNOWN', `未识别的命令：${command}`);
  }
};

export const listTasks = async (filters?: Partial<TaskFilters>) => {
  try {
    const parsedFilters = parseTaskFilters(filters ?? {});
    return await invokeOrMock<TaskListResponse>(COMMANDS.LIST, { filters: parsedFilters });
  } catch (error) {
    throw mapUnknownError(error);
  }
};

export const createTask = async (payload: TaskPayload) => {
  try {
    const sanitized = parseTaskPayload(payload);
    const normalized = removeUndefinedDeep(sanitized);
    return await invokeOrMock<Task>(COMMANDS.CREATE, { payload: normalized });
  } catch (error) {
    if (error instanceof ZodError) {
      throw createAppError('VALIDATION_ERROR', '任务创建校验失败', formatZodError(error));
    }
    throw mapUnknownError(error);
  }
};

export const updateTask = async (id: string, payload: TaskUpdatePayload) => {
  try {
    const parsed = taskUpdateSchema.parse(payload);
    const normalized = removeUndefinedDeep(parsed);
    return await invokeOrMock<Task>(COMMANDS.UPDATE, { id, payload: normalized });
  } catch (error) {
    if (error instanceof ZodError) {
      throw createAppError('VALIDATION_ERROR', '任务更新校验失败', formatZodError(error));
    }
    throw mapUnknownError(error);
  }
};

export const deleteTask = async (id: string) => {
  try {
    await invokeOrMock<void>(COMMANDS.DELETE, { id });
  } catch (error) {
    throw mapUnknownError(error);
  }
};

export const parseTask = async (payload: TaskParseRequest) => {
  let parsedInput: TaskParseRequest;
  try {
    parsedInput = parseTaskParseInput(payload);
  } catch (error) {
    if (error instanceof ZodError) {
      throw createAppError('VALIDATION_ERROR', 'AI 解析请求校验失败', formatZodError(error));
    }
    throw mapUnknownError(error);
  }

  try {
    const normalized = removeUndefinedDeep(parsedInput);
    const response = await invokeOrMock<TaskParseResponse>(COMMANDS.PARSE, {
      request: normalized,
    } as unknown as Record<string, unknown>);
    return parseTaskParseResult(response);
  } catch (error) {
    if (error instanceof ZodError) {
      throw createAppError('UNKNOWN', 'AI 解析结果不符合预期', formatZodError(error));
    }
    if (isAppError(error)) {
      throw error;
    }
    throw mapUnknownError(error);
  }
};

export const generatePlanningSession = async (
  payload: GeneratePlanInput,
): Promise<PlanningSessionView> => {
  let parsedInput: GeneratePlanInput;
  try {
    parsedInput = generatePlanInputSchema.parse(payload);
  } catch (error) {
    if (error instanceof ZodError) {
      throw createAppError('VALIDATION_ERROR', '规划请求校验失败', formatZodError(error));
    }
    throw mapUnknownError(error);
  }

  try {
    const normalized = removeUndefinedDeep(parsedInput) as GeneratePlanInput;
    const response = await invokeOrMock<unknown>(COMMANDS.PLANNING_GENERATE, {
      payload: normalized as unknown as Record<string, unknown>,
    });
    return parsePlanningSessionView(response);
  } catch (error) {
    if (error instanceof ZodError) {
      throw createAppError('UNKNOWN', '规划会话解析失败', formatZodError(error));
    }
    if (isAppError(error)) {
      throw error;
    }
    throw mapUnknownError(error);
  }
};

export const applyPlanningOption = async (payload: ApplyPlanInput): Promise<AppliedPlan> => {
  let parsedInput: ApplyPlanInput;
  try {
    parsedInput = applyPlanInputSchema.parse(payload);
  } catch (error) {
    if (error instanceof ZodError) {
      throw createAppError('VALIDATION_ERROR', '应用规划请求校验失败', formatZodError(error));
    }
    throw mapUnknownError(error);
  }

  try {
    const normalized = removeUndefinedDeep(parsedInput) as ApplyPlanInput;
    const response = await invokeOrMock<unknown>(COMMANDS.PLANNING_APPLY, {
      payload: normalized as unknown as Record<string, unknown>,
    });
    return parseAppliedPlan(response);
  } catch (error) {
    if (error instanceof ZodError) {
      throw createAppError('UNKNOWN', '应用规划结果解析失败', formatZodError(error));
    }
    if (isAppError(error)) {
      throw error;
    }
    throw mapUnknownError(error);
  }
};

export const resolvePlanningConflicts = async (
  payload: ResolveConflictInput,
): Promise<PlanningSessionView> => {
  let parsedInput: ResolveConflictInput;
  try {
    parsedInput = resolveConflictInputSchema.parse(payload);
  } catch (error) {
    if (error instanceof ZodError) {
      throw createAppError('VALIDATION_ERROR', '冲突调整请求校验失败', formatZodError(error));
    }
    throw mapUnknownError(error);
  }

  try {
    const normalized = removeUndefinedDeep(parsedInput) as ResolveConflictInput;
    const response = await invokeOrMock<unknown>(COMMANDS.PLANNING_RESOLVE_CONFLICT, {
      payload: normalized as unknown as Record<string, unknown>,
    });
    return parsePlanningSessionView(response);
  } catch (error) {
    if (error instanceof ZodError) {
      throw createAppError('UNKNOWN', '规划会话解析失败', formatZodError(error));
    }
    if (isAppError(error)) {
      throw error;
    }
    throw mapUnknownError(error);
  }
};

export const getPlanningPreferences = async (
  preferenceId?: string,
): Promise<PreferenceSnapshot> => {
  try {
    const response = await invokeOrMock<unknown>(COMMANDS.PLANNING_PREFERENCES_GET, {
      preference_id: preferenceId,
    });
    return parsePreferenceSnapshot(response);
  } catch (error) {
    if (error instanceof ZodError) {
      throw createAppError('UNKNOWN', '偏好数据解析失败', formatZodError(error));
    }
    if (isAppError(error)) {
      throw error;
    }
    throw mapUnknownError(error);
  }
};

export const updatePlanningPreferences = async (
  payload: PlanningPreferencesUpdateInput,
): Promise<void> => {
  let parsedInput: PlanningPreferencesUpdateInput;
  try {
    parsedInput = planningPreferencesUpdateSchema.parse(payload);
  } catch (error) {
    if (error instanceof ZodError) {
      throw createAppError('VALIDATION_ERROR', '偏好更新请求校验失败', formatZodError(error));
    }
    throw mapUnknownError(error);
  }

  try {
    const normalized = removeUndefinedDeep(parsedInput) as PlanningPreferencesUpdateInput;
    await invokeOrMock<void>(COMMANDS.PLANNING_PREFERENCES_UPDATE, {
      payload: normalized as unknown as Record<string, unknown>,
    });
  } catch (error) {
    if (isAppError(error)) {
      throw error;
    }
    throw mapUnknownError(error);
  }
};

export const fetchAnalyticsOverview = async (
  params: AnalyticsQueryParams,
): Promise<AnalyticsOverviewResponse> => {
  try {
    const parsedParams = analyticsQueryParamsSchema.parse(params);
    const payload = removeUndefinedDeep(parsedParams) as unknown as Record<string, unknown>;
    const response = await invokeOrMock<unknown>(COMMANDS.ANALYTICS_OVERVIEW_FETCH, payload);
    const normalized = normalizeAnalyticsOverviewResponse(response, parsedParams.range);
    memoryStore.analytics = normalized;
    memoryStore.lastAnalyticsRange = normalized.overview.range;
    return normalized;
  } catch (error) {
    throw toAppError(error, '获取分析概览失败');
  }
};

export const fetchDashboardConfig = async (): Promise<DashboardConfig> => {
  try {
    const response = await invokeOrMock<unknown>(COMMANDS.DASHBOARD_CONFIG_GET, undefined);
    const normalized = normalizeDashboardConfig(
      (response ?? undefined) as DashboardConfig | DashboardConfigInput | null | undefined,
    );
    const current = ensureAppSettings();
    memoryStore.settings = {
      ...current,
      dashboardConfig: normalized,
    } satisfies InternalAppSettings;
    return normalized;
  } catch (error) {
    throw toAppError(error, '加载仪表盘配置失败');
  }
};

export const updateDashboardConfig = async (
  payload: DashboardConfigInput,
): Promise<DashboardConfig> => {
  try {
    const sanitizedPayload = sanitizeDashboardConfigInput(payload);
    const normalizedPayload = removeUndefinedDeep(sanitizedPayload) as Record<string, unknown>;
    const response = await invokeOrMock<unknown>(COMMANDS.DASHBOARD_CONFIG_UPDATE, {
      payload: normalizedPayload,
    });
    const normalized = normalizeDashboardConfig(
      (response ?? undefined) as DashboardConfig | DashboardConfigInput | null | undefined,
    );
    const current = ensureAppSettings();
    memoryStore.settings = {
      ...current,
      dashboardConfig: normalized,
    } satisfies InternalAppSettings;
    return normalized;
  } catch (error) {
    throw toAppError(error, '更新仪表盘配置失败');
  }
};

export const fetchAppSettings = async (): Promise<AppSettings> => {
  try {
    const response = await invokeOrMock<unknown>(COMMANDS.SETTINGS_GET, undefined);
    return normalizeAppSettingsResponse(response);
  } catch (error) {
    throw toAppError(error, '加载应用设置失败');
  }
};

export const fetchAiStatus = async (): Promise<AiStatus> => {
  try {
    const response = await invokeOrMock<unknown>(COMMANDS.AI_STATUS, undefined);
    const normalized = aiStatusSchema.parse(response);
    memoryStore.aiStatus = normalized;
    return normalized;
  } catch (error) {
    if (error instanceof ZodError) {
      throw createAppError('UNKNOWN', 'AI 状态解析失败', formatZodError(error));
    }
    if (isAppError(error)) {
      throw error;
    }
    throw mapUnknownError(error);
  }
};

export const updateAppSettings = async (payload: UpdateAppSettingsInput): Promise<AppSettings> => {
  try {
    const parsedInput = appSettingsUpdateSchema.parse(payload);

    const normalized: UpdateAppSettingsInput = {};
    const trimmedKey =
      typeof parsedInput.deepseekApiKey === 'string' ? parsedInput.deepseekApiKey.trim() : '';

    if (typeof parsedInput.themePreference === 'string') {
      normalized.themePreference = parsedInput.themePreference;
    }

    if (typeof parsedInput.workdayStartMinute === 'number') {
      normalized.workdayStartMinute = parsedInput.workdayStartMinute;
    } else if (typeof parsedInput.workdayStartHour === 'number') {
      normalized.workdayStartMinute = parsedInput.workdayStartHour * 60;
    }

    if (typeof parsedInput.workdayEndMinute === 'number') {
      normalized.workdayEndMinute = parsedInput.workdayEndMinute;
    } else if (typeof parsedInput.workdayEndHour === 'number') {
      normalized.workdayEndMinute = parsedInput.workdayEndHour * 60;
    }

    if (trimmedKey) {
      normalized.deepseekApiKey = trimmedKey;
    }

    if (parsedInput.removeDeepseekKey) {
      normalized.removeDeepseekKey = true;
      normalized.deepseekApiKey = null;
    }

    const sanitized = removeUndefinedDeep(normalized) as Record<string, unknown>;

    const response = await invokeOrMock<unknown>(COMMANDS.SETTINGS_UPDATE, { payload: sanitized });
    let result = normalizeAppSettingsResponse(response);

    const intendsToRemoveKey = Boolean(parsedInput.removeDeepseekKey);
    const intendsToSetKey = Boolean(trimmedKey);

    if (intendsToRemoveKey) {
      const current = ensureAppSettings();
      memoryStore.settings = {
        ...current,
        deepseekApiKey: null,
        lastUpdatedAt: result.lastUpdatedAt ?? new Date().toISOString(),
        workdayStartMinute: result.workdayStartMinute,
        workdayEndMinute: result.workdayEndMinute,
        themePreference: result.themePreference,
        aiFeedbackOptOut: result.aiFeedbackOptOut ?? current.aiFeedbackOptOut ?? false,
      } satisfies InternalAppSettings;
      if (result.hasDeepseekKey || result.maskedDeepseekKey) {
        result = {
          ...result,
          hasDeepseekKey: false,
          maskedDeepseekKey: null,
        } satisfies AppSettings;
      }
    }

    if (intendsToSetKey) {
      const current = ensureAppSettings();
      const masked = maskDeepseekKey(trimmedKey) ?? '****';
      memoryStore.settings = {
        ...current,
        deepseekApiKey: trimmedKey,
        lastUpdatedAt: result.lastUpdatedAt ?? new Date().toISOString(),
        workdayStartMinute: result.workdayStartMinute,
        workdayEndMinute: result.workdayEndMinute,
        themePreference: result.themePreference,
        aiFeedbackOptOut: result.aiFeedbackOptOut ?? current.aiFeedbackOptOut ?? false,
      } satisfies InternalAppSettings;
      if (!result.hasDeepseekKey || !result.maskedDeepseekKey) {
        result = {
          ...result,
          hasDeepseekKey: true,
          maskedDeepseekKey: masked,
        } satisfies AppSettings;
      }
    }

    return result;
  } catch (error) {
    throw toAppError(error, '更新应用设置失败');
  }
};

export const fetchAnalyticsHistory = async (
  params: AnalyticsQueryParams,
): Promise<AnalyticsHistoryResponse> => {
  try {
    const parsedParams = analyticsQueryParamsSchema.parse(params);
    const payload = removeUndefinedDeep(parsedParams) as unknown as Record<string, unknown>;
    const response = await invokeOrMock<unknown>(COMMANDS.ANALYTICS_HISTORY_FETCH, payload);
    return normalizeAnalyticsHistoryResponse(response, parsedParams.range);
  } catch (error) {
    throw toAppError(error, '获取历史趋势失败');
  }
};

export const exportAnalyticsReport = async (
  params: AnalyticsExportParams,
): Promise<AnalyticsExportResult> => {
  try {
    const parsedParams = analyticsExportParamsSchema.parse(params) as AnalyticsExportParams;
    const payload = removeUndefinedDeep(parsedParams) as unknown as Record<string, unknown>;
    const response = await invokeOrMock<unknown>(COMMANDS.ANALYTICS_REPORT_EXPORT, payload);
    return normalizeAnalyticsExportResult(response, parsedParams);
  } catch (error) {
    throw toAppError(error, '导出分析报告失败');
  }
};

export const toAppError = (error: unknown, fallback = '发生未知错误'): AppError => {
  if (!error) {
    return createAppError('UNKNOWN', fallback);
  }
  const mapped = mapUnknownError(error);
  if (!mapped.message || mapped.message === '发生未知错误') {
    return { ...mapped, message: fallback };
  }
  return mapped;
};

export const isAppError = (error: unknown): error is AppError =>
  Boolean(error) &&
  typeof error === 'object' &&
  'code' in (error as Record<string, unknown>) &&
  typeof (error as Record<string, unknown>).code === 'string';

export type {
  GeneratePlanInput,
  ApplyPlanInput,
  ResolveConflictInput,
  PlanningSessionView,
  AppliedPlan,
  PreferenceSnapshot,
  PlanningPreferencesUpdateInput,
} from '../types/planning';

export type {
  AnalyticsOverviewResponse,
  AnalyticsHistoryResponse,
  AnalyticsExportParams,
  AnalyticsExportResult,
  AnalyticsQueryParams,
  AnalyticsRangeKey,
  AnalyticsGrouping,
} from '../types/analytics';

export type { AppSettings, UpdateAppSettingsInput, CacheClearResult } from '../types/settings';

export { PLANNING_EVENT_NAMES, type PlanningEventName } from '../types/planning';

export const clearAllCache = async (): Promise<CacheClearResult> => {
  if (isTauriAvailable()) {
    try {
      const result = await invoke<CacheClearResult>(COMMANDS.CACHE_CLEAR_ALL);
      return result;
    } catch (error) {
      throw mapUnknownError(error);
    }
  }

  warnMockUsage();

  // Mock result in development
  return {
    tasksCleared: 0,
    planningSessionsCleared: 0,
    recommendationsCleared: 0,
    analyticsSnapshotsCleared: 0,
    productivityScoresCleared: 0,
    wellnessNudgesCleared: 0,
    workloadForecastsCleared: 0,
    aiFeedbackCleared: 0,
    communityExportsCleared: 0,
    aiCacheCleared: 0,
  };
};
