import { z } from 'zod';

const nullishToUndefined = (value: unknown) =>
  value === null || typeof value === 'undefined' ? undefined : value;

const isoDateSchema = z
  .string({ required_error: '请输入 ISO 日期时间' })
  .trim()
  .refine((value) => !Number.isNaN(Date.parse(value)), '请输入有效的 ISO 日期时间');

const optionalIsoDateSchema = z.preprocess(nullishToUndefined, isoDateSchema.optional());

const optionalStringSchema = z.preprocess((value) => {
  if (value === null || typeof value === 'undefined') return undefined;
  if (typeof value !== 'string') return String(value);
  const trimmed = value.trim();
  return trimmed.length > 0 ? trimmed : undefined;
}, z.string().trim().optional());

const stringArraySchema = z.preprocess((value) => {
  if (!value) return [];
  if (Array.isArray(value)) {
    return value.filter((item): item is string => typeof item === 'string');
  }
  return [];
}, z.array(z.string()).default([]));

export const CONFLICT_SEVERITIES = ['low', 'medium', 'high'] as const;

export type ConflictSeverity = (typeof CONFLICT_SEVERITIES)[number];

export const conflictSeveritySchema = z.enum(CONFLICT_SEVERITIES);

export const scheduleConflictSchema = z
  .object({
    conflictType: z
      .string({ required_error: '冲突类型不能为空' })
      .trim()
      .min(1, '冲突类型不能为空'),
    severity: conflictSeveritySchema,
    message: z.string({ required_error: '冲突描述不能为空' }).trim().min(1, '冲突描述不能为空'),
    relatedBlockId: optionalStringSchema,
    relatedEventId: optionalStringSchema,
  })
  .strict();

export type ScheduleConflict = z.infer<typeof scheduleConflictSchema>;

export const planRationaleStepSchema = z
  .object({
    step: z
      .number({ required_error: '思维链步骤序号不能为空' })
      .int('思维链步骤序号需为整数')
      .min(0, '思维链步骤序号至少为 0'),
    thought: z
      .string({ required_error: '思维链思考内容不能为空' })
      .trim()
      .min(1, '思维链思考内容不能为空'),
    result: optionalStringSchema,
  })
  .strict();

export type PlanRationaleStep = z.infer<typeof planRationaleStepSchema>;

export const cotStepsSchema = z
  .preprocess((value) => {
    if (!value) return [];
    if (Array.isArray(value)) return value;
    return [];
  }, z.array(planRationaleStepSchema))
  .default([]);

export const avoidanceWindowSchema = z
  .object({
    weekday: z
      .number({ required_error: '请选择避免时间的星期' })
      .int('星期需为整数')
      .min(0, '星期需在 0-6 之间')
      .max(6, '星期需在 0-6 之间'),
    startMinute: z
      .number({ required_error: '请提供起始分钟数' })
      .int('分钟数需为整数')
      .min(0, '分钟数不能小于 0')
      .max(24 * 60, '分钟数需在 0-1440 之间'),
    endMinute: z
      .number({ required_error: '请提供结束分钟数' })
      .int('分钟数需为整数')
      .min(0, '分钟数不能小于 0')
      .max(24 * 60, '分钟数需在 0-1440 之间'),
  })
  .strict();

export type AvoidanceWindow = z.infer<typeof avoidanceWindowSchema>;

export const preferenceSnapshotSchema = z
  .object({
    focusStartMinute: z.preprocess(
      nullishToUndefined,
      z
        .number()
        .int('分钟需为整数')
        .min(0)
        .max(24 * 60)
        .optional(),
    ),
    focusEndMinute: z.preprocess(
      nullishToUndefined,
      z
        .number()
        .int('分钟需为整数')
        .min(0)
        .max(24 * 60)
        .optional(),
    ),
    bufferMinutesBetweenBlocks: z
      .number({ required_error: '请提供时间块之间的缓冲分钟数' })
      .int('缓冲时间需为整数')
      .min(0, '缓冲时间不能小于 0')
      .max(24 * 60, '缓冲时间最多为 24 小时'),
    preferCompactSchedule: z.boolean().default(false),
    avoidanceWindows: z.preprocess((value) => {
      if (!value) return [];
      if (Array.isArray(value)) return value;
      return [];
    }, z.array(avoidanceWindowSchema)),
  })
  .strict();

export type PreferenceSnapshot = z.infer<typeof preferenceSnapshotSchema>;

export const timeWindowSchema = z
  .object({
    startAt: isoDateSchema,
    endAt: isoDateSchema,
  })
  .strict();

export type TimeWindow = z.infer<typeof timeWindowSchema>;

export const existingEventSchema = z
  .object({
    id: z.string({ required_error: '事件 ID 不能为空' }).trim().min(1),
    startAt: isoDateSchema,
    endAt: isoDateSchema,
    eventType: optionalStringSchema,
  })
  .strict();

export type ExistingEvent = z.infer<typeof existingEventSchema>;

export const scheduleConstraintsSchema = z
  .object({
    planningStartAt: optionalIsoDateSchema,
    planningEndAt: optionalIsoDateSchema,
    availableWindows: z
      .preprocess((value) => (Array.isArray(value) ? value : []), z.array(timeWindowSchema))
      .default([]),
    existingEvents: z
      .preprocess((value) => (Array.isArray(value) ? value : []), z.array(existingEventSchema))
      .default([]),
    maxFocusMinutesPerDay: z.preprocess(
      nullishToUndefined,
      z
        .number()
        .int('分钟数需为整数')
        .min(30)
        .max(24 * 60)
        .optional(),
    ),
  })
  .strict();

export type ScheduleConstraints = z.infer<typeof scheduleConstraintsSchema>;

export const generatePlanInputSchema = z
  .object({
    taskIds: z
      .array(z.string().trim().min(1, '任务 ID 不能为空'))
      .nonempty('生成计划时至少需要一个任务'),
    constraints: z.preprocess(nullishToUndefined, scheduleConstraintsSchema.optional()),
    preferenceId: optionalStringSchema,
    seed: z.preprocess(
      nullishToUndefined,
      z
        .number()
        .int('随机种子需为整数')
        .min(0, '随机种子需为非负整数')
        .max(Number.MAX_SAFE_INTEGER, '随机种子过大')
        .optional(),
    ),
  })
  .strict();

export type GeneratePlanInput = z.infer<typeof generatePlanInputSchema>;

export const timeBlockOverrideSchema = z
  .object({
    blockId: z.string({ required_error: '时间块 ID 不能为空' }).trim().min(1),
    startAt: optionalIsoDateSchema,
    endAt: optionalIsoDateSchema,
    flexibility: optionalStringSchema,
  })
  .strict();

export type TimeBlockOverride = z.infer<typeof timeBlockOverrideSchema>;

export const applyPlanInputSchema = z
  .object({
    sessionId: z.string({ required_error: '会话 ID 不能为空' }).trim().min(1),
    optionId: z.string({ required_error: '方案 ID 不能为空' }).trim().min(1),
    overrides: z.preprocess(
      (value) => (Array.isArray(value) ? value : []),
      z.array(timeBlockOverrideSchema),
    ),
  })
  .strict();

export type ApplyPlanInput = z.infer<typeof applyPlanInputSchema>;

export const resolveConflictInputSchema = z
  .object({
    sessionId: z.string({ required_error: '会话 ID 不能为空' }).trim().min(1),
    optionId: z.string({ required_error: '方案 ID 不能为空' }).trim().min(1),
    adjustments: z.preprocess(
      (value) => (Array.isArray(value) ? value : []),
      z.array(timeBlockOverrideSchema),
    ),
  })
  .strict();

export type ResolveConflictInput = z.infer<typeof resolveConflictInputSchema>;

export const PLANNING_SESSION_STATUSES = ['pending', 'applied'] as const;

export type PlanningSessionStatus = (typeof PLANNING_SESSION_STATUSES)[number];

export const PLANNING_TIME_BLOCK_STATUSES = [
  'draft',
  'planned',
  'applied',
  'completed',
  'skipped',
  'conflicted',
] as const;

export type PlanningTimeBlockStatus = (typeof PLANNING_TIME_BLOCK_STATUSES)[number];

const optionRiskMetadataSchema = z
  .preprocess(
    (value) => {
      if (!value || typeof value !== 'object') return {};
      return value;
    },
    z
      .object({
        notes: z.array(z.string()).optional(),
        conflicts: z.array(scheduleConflictSchema).optional(),
      })
      .strip(),
  )
  .transform((value) => ({
    notes: Array.isArray(value.notes) ? value.notes : [],
    conflicts: Array.isArray(value.conflicts) ? value.conflicts : [],
  }));

export type PlanningOptionRiskMetadata = z.infer<typeof optionRiskMetadataSchema>;

export const planningOptionSchema = z
  .object({
    id: z.string({ required_error: '方案 ID 不能为空' }).trim().min(1),
    sessionId: z.string({ required_error: '会话 ID 不能为空' }).trim().min(1),
    rank: z.number({ required_error: '方案排名不能为空' }),
    score: z.preprocess(nullishToUndefined, z.number().optional()),
    summary: optionalStringSchema,
    cotSteps: cotStepsSchema.optional(),
    riskNotes: optionRiskMetadataSchema.optional(),
    isFallback: z.boolean({ required_error: '请指明是否为备选方案' }),
    createdAt: isoDateSchema,
  })
  .strict()
  .transform((value) => ({
    ...value,
    cotSteps: value.cotSteps ?? [],
    riskNotes: value.riskNotes ?? { notes: [], conflicts: [] },
  }));

export type PlanningOption = z.infer<typeof planningOptionSchema>;

export const planningTimeBlockSchema = z
  .object({
    id: z.string({ required_error: '时间块 ID 不能为空' }).trim().min(1),
    optionId: z.string({ required_error: '方案 ID 不能为空' }).trim().min(1),
    taskId: z.string({ required_error: '任务 ID 不能为空' }).trim().min(1),
    startAt: isoDateSchema,
    endAt: isoDateSchema,
    flexibility: optionalStringSchema,
    confidence: z.preprocess(nullishToUndefined, z.number().optional()),
    conflictFlags: stringArraySchema,
    appliedAt: optionalIsoDateSchema,
    actualStartAt: optionalIsoDateSchema,
    actualEndAt: optionalIsoDateSchema,
    status: z.enum(PLANNING_TIME_BLOCK_STATUSES),
  })
  .strict();

export type PlanningTimeBlock = z.infer<typeof planningTimeBlockSchema>;

export const planningOptionViewSchema = z
  .object({
    option: planningOptionSchema,
    blocks: z.array(planningTimeBlockSchema),
    conflicts: z.array(scheduleConflictSchema),
  })
  .strict();

export type PlanningOptionView = z.infer<typeof planningOptionViewSchema>;

export const planningSessionSchema = z
  .object({
    id: z.string({ required_error: '会话 ID 不能为空' }).trim().min(1),
    taskIds: z.array(z.string().trim().min(1)),
    constraints: z.preprocess(nullishToUndefined, scheduleConstraintsSchema.optional()).optional(),
    generatedAt: isoDateSchema,
    status: z.enum(PLANNING_SESSION_STATUSES),
    selectedOptionId: optionalStringSchema,
    personalizationSnapshot: z
      .preprocess(nullishToUndefined, preferenceSnapshotSchema.optional())
      .optional(),
    createdAt: isoDateSchema,
    updatedAt: isoDateSchema,
  })
  .strict();

export type PlanningSession = z.infer<typeof planningSessionSchema>;

export const planningSessionViewSchema = z
  .object({
    session: planningSessionSchema,
    options: z.array(planningOptionViewSchema),
    conflicts: z.array(scheduleConflictSchema),
    preferenceSnapshot: z.preprocess(nullishToUndefined, preferenceSnapshotSchema.optional()),
  })
  .strict();

export type PlanningSessionView = z.infer<typeof planningSessionViewSchema>;

export const appliedPlanSchema = z
  .object({
    session: planningSessionSchema,
    option: planningOptionViewSchema,
    conflicts: z.array(scheduleConflictSchema),
  })
  .strict();

export type AppliedPlan = z.infer<typeof appliedPlanSchema>;

export const planningPreferencesUpdateSchema = z
  .object({
    preferenceId: optionalStringSchema,
    snapshot: preferenceSnapshotSchema,
  })
  .strict();

export type PlanningPreferencesUpdateInput = z.infer<typeof planningPreferencesUpdateSchema>;

export const PLANNING_EVENT_NAMES = {
  GENERATED: 'planning://generated',
  APPLIED: 'planning://applied',
  CONFLICTS_RESOLVED: 'planning://conflicts-resolved',
  PREFERENCES_UPDATED: 'planning://preferences-updated',
} as const;

export type PlanningEventName = (typeof PLANNING_EVENT_NAMES)[keyof typeof PLANNING_EVENT_NAMES];

export const parsePreferenceSnapshot = (value: unknown): PreferenceSnapshot =>
  preferenceSnapshotSchema.parse(value);

export const parsePlanningSessionView = (value: unknown): PlanningSessionView =>
  planningSessionViewSchema.parse(value);

export const parseAppliedPlan = (value: unknown): AppliedPlan => appliedPlanSchema.parse(value);

export const parsePlanningOptionView = (value: unknown): PlanningOptionView =>
  planningOptionViewSchema.parse(value);
