import type { TaskPayloadField } from '../types/task';

const TASK_FIELD_LABELS: Partial<Record<TaskPayloadField, string>> = {
  title: '标题',
  description: '描述',
  status: '状态',
  priority: '优先级',
  plannedStartAt: '计划开始时间',
  startAt: '开始时间',
  dueAt: '截止时间',
  completedAt: '完成时间',
  estimatedMinutes: '预估时长（分钟）',
  estimatedHours: '预估时长（小时）',
  tags: '标签',
  ownerId: '负责人',
  isRecurring: '循环设置',
  recurrence: '循环规则',
  taskType: '任务类型',
  ai: 'AI 洞察',
  externalLinks: '相关链接',
};

export function formatTaskPayloadField(field: TaskPayloadField): string {
  return TASK_FIELD_LABELS[field] ?? field;
}
