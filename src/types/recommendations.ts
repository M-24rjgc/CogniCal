import { z } from 'zod';
import { scheduleConflictSchema } from './planning';

const isoDateTimeString = z
  .string({ required_error: '时间字段不能为空' })
  .trim()
  .refine((value) => !Number.isNaN(Date.parse(value)), '请输入有效的 ISO 时间字符串');

export const aiProviderMetadataSchema = z
  .object({
    providerId: z.string().trim().min(1).optional().nullable(),
    model: z.string().trim().min(1).optional().nullable(),
    latencyMs: z.number().nonnegative().optional().nullable(),
    tokensUsed: z
      .record(z.number().nonnegative())
      .catch(() => ({}))
      .optional()
      .nullable(),
    extra: z.unknown().optional().nullable(),
  })
  .partial();

export type AiProviderMetadata = z.infer<typeof aiProviderMetadataSchema>;

export const recommendationSourceSchema = z.enum(['deepseek', 'cached'] as const);

export type RecommendationSource = z.infer<typeof recommendationSourceSchema>;

export const recommendationNetworkStatusSchema = z.enum(['online', 'offline'] as const);

export type RecommendationNetworkStatus = z.infer<typeof recommendationNetworkStatusSchema>;

export const recommendationTimeBlockSchema = z
  .object({
    taskId: z.string().trim().min(1).optional().nullable(),
    title: z.string({ required_error: '任务标题不能为空' }).trim().min(1, '任务标题不能为空'),
    startTime: isoDateTimeString,
    endTime: isoDateTimeString,
    priority: z.number().int().optional(),
    estimatedDuration: z.number().int().nonnegative().optional(),
  })
  .strict();

export type RecommendationTimeBlock = z.infer<typeof recommendationTimeBlockSchema>;

export const recommendationPlanSchema = z
  .object({
    id: z.string({ required_error: '方案 ID 不能为空' }).trim().min(1, '方案 ID 不能为空'),
    name: z.string({ required_error: '方案名称不能为空' }).trim().min(1, '方案名称不能为空'),
    description: z.string({ required_error: '方案描述不能为空' }).trim().min(1, '方案描述不能为空'),
    timeBlocks: z.array(recommendationTimeBlockSchema).default([]),
    conflicts: z.array(scheduleConflictSchema).default([]),
    confidenceScore: z.number().nonnegative().max(1).optional(),
    source: recommendationSourceSchema,
    metadata: z.unknown().optional(),
  })
  .strict();

export type RecommendationPlan = z.infer<typeof recommendationPlanSchema>;

export const recommendationResponseSchema = z
  .object({
    sessionId: z.string({ required_error: '会话 ID 不能为空' }).trim().min(1, '会话 ID 不能为空'),
    plans: z.array(recommendationPlanSchema).default([]),
    source: recommendationSourceSchema,
    networkStatus: recommendationNetworkStatusSchema,
    generatedAt: isoDateTimeString,
    expiresAt: isoDateTimeString,
    conflictsDetected: z.number().int().nonnegative().default(0),
    hasFallbacks: z.boolean().default(false),
  })
  .strict();

export type RecommendationResponse = z.infer<typeof recommendationResponseSchema>;

export const recommendationInputSchema = z
  .object({
    taskIds: z
      .array(z.string().trim().min(1, '任务 ID 不能为空'))
      .nonempty('生成推荐至少需要一个任务'),
    constraints: z.unknown().optional(),
    preferenceId: z.string().trim().optional(),
    seed: z.number().int().nonnegative().optional(),
    forceRefresh: z.boolean().optional(),
  })
  .strict();

export type RecommendationInput = z.infer<typeof recommendationInputSchema>;

export const recommendationDecisionActionSchema = z.enum([
  'accepted',
  'rejected',
  'adjusted',
] as const);

export type RecommendationDecisionAction = z.infer<typeof recommendationDecisionActionSchema>;

export const recommendationDecisionInputSchema = z
  .object({
    sessionId: z.string({ required_error: '会话 ID 不能为空' }).trim().min(1),
    planId: z.string({ required_error: '方案 ID 不能为空' }).trim().min(1),
    action: recommendationDecisionActionSchema,
    adjustments: z.unknown().optional(),
    preferenceTags: z.array(z.string().trim().min(1)).optional(),
  })
  .strict();

export type RecommendationDecisionInput = z.infer<typeof recommendationDecisionInputSchema>;

export const recommendationDecisionRecordSchema = z
  .object({
    id: z.number().int().nonnegative(),
    sessionId: z.number().int().nonnegative(),
    userAction: recommendationDecisionActionSchema,
    adjustmentPayload: z.unknown().optional(),
    respondedAt: isoDateTimeString,
    preferenceTags: z.unknown().optional(),
  })
  .strict();

export type RecommendationDecisionRecord = z.infer<typeof recommendationDecisionRecordSchema>;

export const schedulePlanItemSchema = z
  .object({
    taskId: z.string().trim().min(1).optional().nullable(),
    title: z.string({ required_error: '时间块标题不能为空' }).trim().min(1),
    startAt: isoDateTimeString,
    endAt: isoDateTimeString,
    confidence: z.number().min(0).max(1).optional(),
    notes: z.string().trim().optional(),
  })
  .strict();

export type SchedulePlanItem = z.infer<typeof schedulePlanItemSchema>;

export const schedulePlanSchema = z
  .object({
    items: z.array(schedulePlanItemSchema).default([]),
    telemetry: aiProviderMetadataSchema.optional(),
  })
  .strict();

export type SchedulePlan = z.infer<typeof schedulePlanSchema>;

export const planScheduleInputSchema = z
  .object({
    tasks: z
      .array(
        z
          .object({
            id: z.string().trim().min(1).optional(),
            title: z.string().trim().min(1, '任务标题不能为空'),
            priority: z.string().trim().optional(),
            estimatedMinutes: z.number().int().nonnegative().optional(),
            dueAt: isoDateTimeString.optional(),
          })
          .strict(),
      )
      .nonempty('排程至少需要一个任务'),
    context: z.unknown().optional(),
  })
  .strict();

export type PlanScheduleInput = z.infer<typeof planScheduleInputSchema>;

export const parseRecommendationResponse = (value: unknown): RecommendationResponse =>
  recommendationResponseSchema.parse(value);

export const parseRecommendationDecisionRecord = (value: unknown): RecommendationDecisionRecord =>
  recommendationDecisionRecordSchema.parse(value);

export const parseSchedulePlan = (value: unknown): SchedulePlan => schedulePlanSchema.parse(value);
