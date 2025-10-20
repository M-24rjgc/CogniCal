import { z, type ZodError, type ZodIssue } from 'zod';
import type { AiStatus } from '../types/settings';
import {
  DEFAULT_PAGE_SIZE,
  TASK_PRIORITIES,
  TASK_SORT_FIELDS,
  TASK_STATUSES,
  TASK_TYPES,
  TASK_PAYLOAD_FIELDS,
  type TaskFilters,
  type TaskPayload,
  type TaskParseRequest,
  type TaskParseResponse,
  type TaskType,
  type TaskPayloadField,
} from '../types/task';

const emptyToUndefined = (value: unknown) => {
  if (value === '' || value === null || typeof value === 'undefined') {
    return undefined;
  }
  if (typeof value === 'string') {
    const trimmed = value.trim();
    return trimmed.length === 0 ? undefined : trimmed;
  }
  return value;
};

const TASK_STATUS_SET = new Set<string>(TASK_STATUSES as readonly string[]);
const TASK_PRIORITY_SET = new Set<string>(TASK_PRIORITIES as readonly string[]);
const TASK_TYPE_SET = new Set<string>(TASK_TYPES as readonly string[]);

const normalizeEnumValue = (value: unknown, allowed: Set<string>): string | undefined => {
  if (value === '' || value === null || typeof value === 'undefined') {
    return undefined;
  }

  if (typeof value !== 'string') {
    return undefined;
  }

  const normalized = value
    .trim()
    .toLowerCase()
    .replace(/[\s-]+/g, '_');
  if (!normalized || !allowed.has(normalized)) {
    return undefined;
  }

  return normalized;
};

const toNumberIfPossible = (value: unknown) => {
  if (value === '' || value === null || typeof value === 'undefined') {
    return undefined;
  }
  if (typeof value === 'number') {
    return Number.isNaN(value) ? undefined : value;
  }
  if (typeof value === 'string') {
    const parsed = Number(value);
    return Number.isNaN(parsed) ? undefined : parsed;
  }
  return value;
};

const isoDateSchema = z
  .string({ required_error: '请输入 ISO 日期时间' })
  .trim()
  .refine((value) => !Number.isNaN(Date.parse(value)), '请输入有效的 ISO 日期时间');

const optionalIsoDateSchema = z.preprocess(emptyToUndefined, isoDateSchema.optional());

const recurrenceSchema = z
  .object({
    rule: z
      .string({ required_error: '重复规则不能为空' })
      .trim()
      .min(1, '重复规则不能为空')
      .max(256, '重复规则长度需小于 256 字符'),
    until: optionalIsoDateSchema,
  })
  .strict();

const aiFocusModeSchema = z
  .object({
    pomodoros: z.preprocess(
      toNumberIfPossible,
      z
        .number()
        .int('番茄钟数量需为整数')
        .min(1, '番茄钟数量至少为 1')
        .max(16, '番茄钟数量最多为 16'),
    ),
    recommendedSlots: z
      .preprocess(
        emptyToUndefined,
        z.array(isoDateSchema).max(10, '推荐时间段最多 10 个').optional(),
      )
      .optional(),
  })
  .strict();

const aiEfficiencyPredictionSchema = z
  .object({
    expectedHours: z.preprocess(
      toNumberIfPossible,
      z
        .number()
        .positive('预估工时需大于 0')
        .max(24 * 14, '预估工时最多为 14 天'),
    ),
    confidence: z.preprocess(
      toNumberIfPossible,
      z.number().min(0, '置信度不能小于 0').max(1, '置信度不能大于 1'),
    ),
  })
  .strict();

const aiReasoningStepSchema = z
  .object({
    order: z.preprocess(
      toNumberIfPossible,
      z
        .number()
        .int('思维链步骤序号需为整数')
        .min(0, '思维链步骤序号至少为 0')
        .max(50, '思维链步骤最多支持 50 条'),
    ),
    title: z.preprocess(emptyToUndefined, z.string().trim().max(160).optional()),
    detail: z.preprocess(emptyToUndefined, z.string().trim().max(4000).optional()).optional(),
    outcome: z.preprocess(emptyToUndefined, z.string().trim().max(4000).optional()),
  })
  .strict();

const aiSharedShape = {
  summary: z.preprocess(emptyToUndefined, z.string().trim().max(4000).optional()),
  nextAction: z.preprocess(emptyToUndefined, z.string().trim().max(1000).optional()),
  confidence: z.preprocess(
    toNumberIfPossible,
    z.number().min(0, '置信度不能小于 0').max(1, '置信度不能大于 1').optional(),
  ),
  metadata: z.record(z.unknown()).optional(),
  complexityScore: z.preprocess(
    toNumberIfPossible,
    z.number().min(0, '复杂度评分不能小于 0').max(10, '复杂度评分不能大于 10').optional(),
  ),
  suggestedStartAt: optionalIsoDateSchema,
  focusMode: z.preprocess(emptyToUndefined, aiFocusModeSchema.optional()),
  efficiencyPrediction: z.preprocess(emptyToUndefined, aiEfficiencyPredictionSchema.optional()),
  cotSteps: z
    .preprocess(
      emptyToUndefined,
      z.array(aiReasoningStepSchema).max(20, '思维链步骤最多 20 条').optional(),
    )
    .optional(),
  cotSummary: z.preprocess(emptyToUndefined, z.string().trim().max(4000).optional()),
} as const;

const aiSchema = z
  .object({
    ...aiSharedShape,
    source: z.enum(['live', 'cache']).optional(),
    generatedAt: optionalIsoDateSchema,
  })
  .strict()
  .optional();

const aiResultSchema = z
  .object({
    ...aiSharedShape,
    source: z.enum(['live', 'cache']),
    generatedAt: isoDateSchema,
  })
  .strict();

const baseTaskObject = z
  .object({
    title: z
      .string({ required_error: '标题不能为空' })
      .trim()
      .min(1, '标题不能为空')
      .max(160, '标题长度需小于 160 字符'),
    description: z.preprocess(emptyToUndefined, z.string().trim().max(4000).optional()),
    status: z.enum(TASK_STATUSES).default('todo'),
    priority: z.enum(TASK_PRIORITIES).default('medium'),
    plannedStartAt: optionalIsoDateSchema,
    startAt: optionalIsoDateSchema,
    dueAt: optionalIsoDateSchema,
    completedAt: optionalIsoDateSchema,
    estimatedMinutes: z.preprocess(
      toNumberIfPossible,
      z
        .number()
        .int('预估时长需为整数（分钟）')
        .positive('预估时长需大于 0')
        .max(60 * 24 * 30, '预估时长最多为 30 天')
        .optional(),
    ),
    estimatedHours: z.preprocess(
      toNumberIfPossible,
      z
        .number()
        .positive('预估工时需大于 0')
        .max(24 * 30, '预估工时最多为 30 天')
        .optional(),
    ),
    tags: z
      .preprocess(
        (value) => {
          if (value === undefined || value === null) return [];
          return value;
        },
        z
          .array(z.string().trim().min(1, '标签不能为空').max(32, '单个标签长度需小于 32 字符'))
          .max(30, '标签数量最多 30 个'),
      )
      .default([]),
    ownerId: z.preprocess(emptyToUndefined, z.string().trim().max(64).optional()),
    isRecurring: z.boolean().default(false),
    recurrence: recurrenceSchema.optional(),
    taskType: z.enum(TASK_TYPES).default('other'),
    ai: aiSchema,
    externalLinks: z.preprocess(
      emptyToUndefined,
      z.array(z.string().trim().url('请填写合法的 URL')).max(20, '链接最多 20 个').optional(),
    ),
  })
  .strict();

const applyTaskBusinessRules = (data: { [key: string]: unknown }, ctx: z.RefinementCtx) => {
  const plannedStartAt = data.plannedStartAt as string | undefined;
  const startAt = data.startAt as string | undefined;
  const dueAt = data.dueAt as string | undefined;
  const isRecurring = Boolean(data.isRecurring);
  const recurrence = data.recurrence as { rule: string } | undefined;
  const ai = data.ai as
    | {
        confidence?: number;
        complexityScore?: number;
        suggestedStartAt?: string;
      }
    | undefined;

  if (startAt && dueAt) {
    const start = Date.parse(startAt);
    const due = Date.parse(dueAt);
    if (!Number.isNaN(start) && !Number.isNaN(due) && start > due) {
      ctx.addIssue({
        code: 'custom',
        path: ['dueAt'],
        message: '截止时间不能早于开始时间',
      });
    }
  }

  if (plannedStartAt && dueAt) {
    const planned = Date.parse(plannedStartAt);
    const due = Date.parse(dueAt);
    if (!Number.isNaN(planned) && !Number.isNaN(due) && planned > due) {
      ctx.addIssue({
        code: 'custom',
        path: ['plannedStartAt'],
        message: '计划开始时间不能晚于截止时间',
      });
    }
  }

  if (!isRecurring && recurrence) {
    ctx.addIssue({
      code: 'custom',
      path: ['recurrence'],
      message: '未开启循环时无需提供重复规则',
    });
  }

  if (ai?.confidence !== undefined && (ai.confidence < 0 || ai.confidence > 1)) {
    ctx.addIssue({
      code: 'custom',
      path: ['ai', 'confidence'],
      message: '置信度需在 0 到 1 之间',
    });
  }

  if (ai?.complexityScore !== undefined && (ai.complexityScore < 0 || ai.complexityScore > 10)) {
    ctx.addIssue({
      code: 'custom',
      path: ['ai', 'complexityScore'],
      message: '复杂度评分需在 0 到 10 之间',
    });
  }

  const suggestedStartAt = ai?.suggestedStartAt;
  if (suggestedStartAt && dueAt) {
    const suggested = Date.parse(suggestedStartAt);
    const due = Date.parse(dueAt);
    if (!Number.isNaN(suggested) && !Number.isNaN(due) && suggested > due) {
      ctx.addIssue({
        code: 'custom',
        path: ['ai', 'suggestedStartAt'],
        message: 'AI 建议开始时间不能晚于截止时间',
      });
    }
  }
};

export const taskCreateSchema = baseTaskObject.superRefine(applyTaskBusinessRules);

export const taskUpdateSchema = baseTaskObject
  .partial()
  .strict()
  .superRefine((value, ctx) => {
    const keys = Object.keys(value).filter((key) => value[key as keyof typeof value] !== undefined);
    if (keys.length === 0) {
      ctx.addIssue({
        code: 'custom',
        message: '至少需要更新一项字段',
        path: [],
      });
    }
    applyTaskBusinessRules(value, ctx);
  });

export const taskFiltersSchema = z
  .object({
    search: z.preprocess(emptyToUndefined, z.string().trim().max(160).optional()),
    statuses: z.array(z.enum(TASK_STATUSES)).nonempty('请选择至少一个状态').optional(),
    priorities: z.array(z.enum(TASK_PRIORITIES)).nonempty('请选择至少一个优先级').optional(),
    tags: z.array(z.string().trim().min(1)).optional(),
    taskTypes: z.array(z.enum(TASK_TYPES)).nonempty('请选择至少一个任务类型').optional(),
    complexityMin: z.preprocess(
      toNumberIfPossible,
      z.number().min(0, '复杂度下限不能小于 0').max(10, '复杂度下限不能大于 10').optional(),
    ),
    complexityMax: z.preprocess(
      toNumberIfPossible,
      z.number().min(0, '复杂度上限不能小于 0').max(10, '复杂度上限不能大于 10').optional(),
    ),
    aiSuggestedAfter: optionalIsoDateSchema,
    aiSuggestedBefore: optionalIsoDateSchema,
    aiSources: z
      .array(z.enum(['live', 'cache']))
      .nonempty('请选择至少一个来源')
      .optional(),
    ownerIds: z.array(z.string().trim().min(1)).optional(),
    includeArchived: z.boolean().optional(),
    dueAfter: optionalIsoDateSchema,
    dueBefore: optionalIsoDateSchema,
    windowStart: optionalIsoDateSchema,
    windowEnd: optionalIsoDateSchema,
    updatedAfter: optionalIsoDateSchema,
    updatedBefore: optionalIsoDateSchema,
    sortBy: z.enum(TASK_SORT_FIELDS).optional(),
    sortOrder: z.enum(['asc', 'desc']).optional(),
    page: z.preprocess(
      toNumberIfPossible,
      z.number().int('页码需为整数').min(1, '页码从 1 开始').optional(),
    ),
    pageSize: z.preprocess(
      toNumberIfPossible,
      z
        .number()
        .int('分页大小需为整数')
        .min(1, '分页大小至少为 1')
        .max(100, '分页大小最多为 100')
        .optional(),
    ),
  })
  .superRefine((data, ctx) => {
    if (data.dueAfter && data.dueBefore) {
      const after = Date.parse(data.dueAfter);
      const before = Date.parse(data.dueBefore);
      if (!Number.isNaN(after) && !Number.isNaN(before) && after > before) {
        ctx.addIssue({
          code: 'custom',
          path: ['dueBefore'],
          message: '截止结束时间需晚于开始时间',
        });
      }
    }

    if (data.updatedAfter && data.updatedBefore) {
      const after = Date.parse(data.updatedAfter);
      const before = Date.parse(data.updatedBefore);
      if (!Number.isNaN(after) && !Number.isNaN(before) && after > before) {
        ctx.addIssue({
          code: 'custom',
          path: ['updatedBefore'],
          message: '更新时间范围无效',
        });
      }
    }

    if (data.windowStart && data.windowEnd) {
      const start = Date.parse(data.windowStart);
      const end = Date.parse(data.windowEnd);
      if (!Number.isNaN(start) && !Number.isNaN(end) && start > end) {
        ctx.addIssue({
          code: 'custom',
          path: ['windowEnd'],
          message: '时间窗口结束需晚于开始时间',
        });
      }
    }

    if (
      data.complexityMin !== undefined &&
      data.complexityMax !== undefined &&
      data.complexityMin > data.complexityMax
    ) {
      ctx.addIssue({
        code: 'custom',
        path: ['complexityMax'],
        message: '复杂度上限需大于或等于下限',
      });
    }

    if (data.aiSuggestedAfter && data.aiSuggestedBefore) {
      const after = Date.parse(data.aiSuggestedAfter);
      const before = Date.parse(data.aiSuggestedBefore);
      if (!Number.isNaN(after) && !Number.isNaN(before) && after > before) {
        ctx.addIssue({
          code: 'custom',
          path: ['aiSuggestedBefore'],
          message: 'AI 建议时间范围无效',
        });
      }
    }
  });

const taskParseContextSchema = z
  .object({
    timezone: z.preprocess(emptyToUndefined, z.string().trim().max(64).optional()),
    locale: z.preprocess(emptyToUndefined, z.string().trim().max(32).optional()),
    referenceDate: optionalIsoDateSchema,
    existingTaskId: z.preprocess(emptyToUndefined, z.string().trim().max(64).optional()),
    metadata: z.record(z.unknown()).optional(),
    userPreferences: z.record(z.unknown()).optional(),
  })
  .strict();

const taskParseRequestSchema = z
  .object({
    input: z
      .string({ required_error: '请输入待解析的任务描述' })
      .trim()
      .min(4, '请输入至少 4 个字符')
      .max(4000, '任务描述需少于 4000 字符'),
    context: z.preprocess(emptyToUndefined, taskParseContextSchema.optional()),
  })
  .strict();

const taskPayloadFieldEnum = z.enum(TASK_PAYLOAD_FIELDS);

const taskParsePayloadSchema = z
  .object({
    title: z.preprocess(emptyToUndefined, z.string().trim().max(160).optional()),
    description: z.preprocess(emptyToUndefined, z.string().trim().max(4000).optional()),
    status: z.preprocess(
      (value) => normalizeEnumValue(value, TASK_STATUS_SET),
      z.enum(TASK_STATUSES).optional(),
    ),
    priority: z.preprocess(
      (value) => normalizeEnumValue(value, TASK_PRIORITY_SET),
      z.enum(TASK_PRIORITIES).optional(),
    ),
    plannedStartAt: optionalIsoDateSchema,
    startAt: optionalIsoDateSchema,
    dueAt: optionalIsoDateSchema,
    completedAt: optionalIsoDateSchema,
    estimatedMinutes: z.preprocess(
      toNumberIfPossible,
      z
        .number()
        .int('预估时长需为整数（分钟）')
        .positive('预估时长需大于 0')
        .max(60 * 24 * 30, '预估时长最多为 30 天')
        .optional(),
    ),
    estimatedHours: z.preprocess(
      toNumberIfPossible,
      z
        .number()
        .positive('预估工时需大于 0')
        .max(24 * 30, '预估工时最多为 30 天')
        .optional(),
    ),
    tags: z.preprocess(
      emptyToUndefined,
      z
        .array(z.string().trim().min(1, '标签不能为空').max(32, '单个标签长度需小于 32 字符'))
        .max(30, '标签数量最多 30 个')
        .optional(),
    ),
    ownerId: z.preprocess(emptyToUndefined, z.string().trim().max(64).optional()),
    isRecurring: z.boolean().optional(),
    recurrence: recurrenceSchema.optional(),
    taskType: z
      .preprocess(emptyToUndefined, z.string().trim().max(64).optional())
      .transform((value) => {
        if (!value) {
          return undefined;
        }
        const normalized = value.trim().toLowerCase();
        return (TASK_TYPE_SET.has(normalized) ? (normalized as TaskType) : undefined) as
          | TaskType
          | undefined;
      }),
    ai: aiSchema,
    externalLinks: z.preprocess(
      emptyToUndefined,
      z.array(z.string().trim().url('请填写合法的 URL')).max(20, '链接最多 20 个').optional(),
    ),
  })
  .strict();

const missingFieldsSchema = z.preprocess(
  (value) => (value === undefined || value === null ? [] : value),
  z.array(taskPayloadFieldEnum).max(TASK_PAYLOAD_FIELDS.length, '缺失字段数量超过允许范围'),
);

const taskParseResponseSchema = z
  .object({
    payload: z.preprocess(
      (value) => (value === undefined || value === null ? {} : value),
      taskParsePayloadSchema,
    ),
    ai: aiResultSchema,
    missingFields: missingFieldsSchema,
  })
  .strict();

const aiProviderMetadataSchema = z
  .object({
    providerId: z.preprocess(emptyToUndefined, z.string().trim().max(128).optional()),
    model: z.preprocess(emptyToUndefined, z.string().trim().max(256).optional()),
    latencyMs: z.preprocess(toNumberIfPossible, z.number().min(0).optional()),
    tokensUsed: z
      .preprocess(
        emptyToUndefined,
        z.record(z.preprocess(toNumberIfPossible, z.number().min(0).optional())).optional(),
      )
      .optional(),
    extra: z.preprocess(emptyToUndefined, z.record(z.unknown()).optional()),
  })
  .partial()
  .strict()
  .nullable()
  .optional();

const aiStatusResponseSchema = z
  .object({
    status: z
      .preprocess(emptyToUndefined, z.enum(['online', 'missing_key', 'unavailable']).optional())
      .optional(),
    mode: z.enum(['online', 'offline', 'cache']).optional(),
    hasApiKey: z.boolean().default(false),
    lastCheckedAt: isoDateSchema,
    latencyMs: z.preprocess(toNumberIfPossible, z.number().min(0).optional()).nullable().optional(),
    provider: aiProviderMetadataSchema,
    message: z.preprocess(emptyToUndefined, z.string().trim().max(1000).optional()),
  })
  .strict();

export const aiStatusSchema = aiStatusResponseSchema.transform((value) => {
  const status: AiStatus['status'] = (() => {
    if (value.status) {
      return value.status;
    }
    if (!value.hasApiKey) {
      return 'missing_key';
    }
    switch (value.mode) {
      case 'online':
        return 'online';
      case 'offline':
        return 'unavailable';
      case 'cache':
        return 'online';
      default:
        return 'online';
    }
  })();

  const provider = (() => {
    if (!value.provider) {
      return null;
    }
    const tokensSource = value.provider.tokensUsed ?? null;
    const tokens = tokensSource
      ? Object.entries(tokensSource).reduce<Record<string, number>>((acc, [key, val]) => {
          if (typeof val === 'number' && Number.isFinite(val)) {
            acc[key] = val;
          }
          return acc;
        }, {})
      : null;

    return {
      providerId: value.provider.providerId ?? null,
      model: value.provider.model ?? null,
      latencyMs: value.provider.latencyMs ?? null,
      tokensUsed: tokens && Object.keys(tokens).length > 0 ? tokens : null,
      extra: value.provider.extra ?? null,
    } as AiStatus['provider'];
  })();

  const normalized: AiStatus = {
    status,
    hasApiKey: value.hasApiKey,
    lastCheckedAt: value.lastCheckedAt,
    latencyMs: value.latencyMs ?? null,
    provider,
    message: value.message ?? null,
  };

  return normalized;
});

export type TaskCreateInput = z.infer<typeof taskCreateSchema>;

export type TaskUpdateInput = z.infer<typeof taskUpdateSchema>;

export type TaskFiltersInput = z.infer<typeof taskFiltersSchema>;

export const DEFAULT_FILTERS: TaskFilters = {
  page: 1,
  pageSize: DEFAULT_PAGE_SIZE,
};

export function parseTaskPayload(payload: unknown): TaskPayload {
  return taskCreateSchema.parse(payload);
}

export function parseTaskFilters(filters: unknown): TaskFilters {
  const parsed = taskFiltersSchema.parse(filters ?? {});

  return {
    ...DEFAULT_FILTERS,
    ...parsed,
  } satisfies TaskFilters;
}

export function parseTaskParseInput(payload: unknown): TaskParseRequest {
  return taskParseRequestSchema.parse(payload) as TaskParseRequest;
}

export function parseTaskParseResult(payload: unknown): TaskParseResponse {
  const parsed = taskParseResponseSchema.parse(payload) as TaskParseResponse;
  return {
    ...parsed,
    missingFields: [...parsed.missingFields] as TaskPayloadField[],
  };
}

export function formatZodError(error: ZodError | ZodIssue[]): Record<string, string> {
  const issues = Array.isArray(error) ? error : error.issues;
  const formatted: Record<string, string> = {};

  for (const issue of issues) {
    const path = issue.path?.length ? issue.path.join('.') : '_root';
    if (!formatted[path]) {
      formatted[path] = issue.message;
    }
  }

  return formatted;
}

const analyticsRangeEnum = z.enum(['7d', '30d', '90d']);
const analyticsGroupingEnum = z.enum(['day', 'week']);
const analyticsExportFormatEnum = z.enum(['markdown', 'json']);

const ensureChronologicalRange = (value: { from?: string; to?: string }, ctx: z.RefinementCtx) => {
  if (value.from && value.to) {
    const fromTime = Date.parse(value.from);
    const toTime = Date.parse(value.to);
    if (!Number.isNaN(fromTime) && !Number.isNaN(toTime) && fromTime > toTime) {
      ctx.addIssue({
        code: 'custom',
        path: ['to'],
        message: '结束时间需晚于开始时间',
      });
    }
  }
};

export const analyticsQueryParamsSchema = z
  .object({
    range: analyticsRangeEnum,
    from: optionalIsoDateSchema,
    to: optionalIsoDateSchema,
    grouping: analyticsGroupingEnum.optional(),
  })
  .strict()
  .superRefine((value, ctx) => {
    ensureChronologicalRange(value, ctx);
  });

export const analyticsExportParamsSchema = z
  .object({
    range: analyticsRangeEnum,
    from: optionalIsoDateSchema,
    to: optionalIsoDateSchema,
    format: analyticsExportFormatEnum,
  })
  .strict()
  .superRefine((value, ctx) => {
    ensureChronologicalRange(value, ctx);
  });

export const appSettingsUpdateSchema = z
  .object({
    deepseekApiKey: z.preprocess(
      emptyToUndefined,
      z
        .string()
        .trim()
        .min(1, 'DeepSeek API Key 不能为空')
        .max(256, 'DeepSeek API Key 长度需小于 256 字符')
        .optional(),
    ),
    removeDeepseekKey: z.boolean().optional(),
    workdayStartMinute: z.preprocess(
      toNumberIfPossible,
      z
        .number()
        .int('工作日开始时间需为整数（分钟）')
        .min(0, '工作日开始时间需在 0~1440 分钟之间')
        .max(24 * 60 - 1, '工作日开始时间需在 0~1440 分钟之间')
        .optional(),
    ),
    workdayEndMinute: z.preprocess(
      toNumberIfPossible,
      z
        .number()
        .int('工作日结束时间需为整数（分钟）')
        .min(0, '工作日结束时间需在 0~1440 分钟之间')
        .max(24 * 60 - 1, '工作日结束时间需在 0~1440 分钟之间')
        .optional(),
    ),
    workdayStartHour: z.preprocess(
      toNumberIfPossible,
      z
        .number()
        .int('工作日开始小时需为整数')
        .min(0, '工作日开始小时需在 0~23 之间')
        .max(23, '工作日开始小时需在 0~23 之间')
        .optional(),
    ),
    workdayEndHour: z.preprocess(
      toNumberIfPossible,
      z
        .number()
        .int('工作日结束小时需为整数')
        .min(0, '工作日结束小时需在 0~23 之间')
        .max(23, '工作日结束小时需在 0~23 之间')
        .optional(),
    ),
    themePreference: z.enum(['system', 'light', 'dark']).optional(),
    aiFeedbackOptOut: z.boolean().optional(),
  })
  .strict()
  .superRefine((value, ctx) => {
    const startMinute =
      value.workdayStartMinute !== undefined
        ? value.workdayStartMinute
        : value.workdayStartHour !== undefined
          ? value.workdayStartHour * 60
          : undefined;
    const endMinute =
      value.workdayEndMinute !== undefined
        ? value.workdayEndMinute
        : value.workdayEndHour !== undefined
          ? value.workdayEndHour * 60
          : undefined;

    if (startMinute !== undefined && endMinute !== undefined && endMinute <= startMinute) {
      ctx.addIssue({
        code: 'custom',
        path: ['workdayEndMinute'],
        message: '工作日结束时间需晚于开始时间',
      });
    }

    if (value.removeDeepseekKey && value.deepseekApiKey) {
      ctx.addIssue({
        code: 'custom',
        path: ['deepseekApiKey'],
        message: '请勿同时提供密钥与清除指令',
      });
    }
  });
