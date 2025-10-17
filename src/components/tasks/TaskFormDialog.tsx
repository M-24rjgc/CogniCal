import { useMemo } from 'react';
import { type UseFormReturn } from 'react-hook-form';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '../ui/dialog';
import { Button } from '../ui/button';
import {
  Form,
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '../ui/form';
import { Input } from '../ui/input';
import { Textarea } from '../ui/textarea';
import { TASK_PRIORITIES, TASK_STATUSES } from '../../types/task';
import type { TaskFormAiState, TaskFormValues } from '../../hooks/useTaskForm';
import { TaskAiParsePanel } from './TaskAiParsePanel';

interface TaskFormDialogProps {
  open: boolean;
  mode: 'create' | 'edit';
  form: UseFormReturn<TaskFormValues>;
  onOpenChange: (open: boolean) => void;
  onSubmit: (values: TaskFormValues) => Promise<void> | void;
  isSubmitting?: boolean;
  serverError?: string | null;
  aiState?: TaskFormAiState;
  hasDeepseekKey?: boolean;
  onTriggerAiParse?: (input: string) => Promise<unknown> | unknown;
  onClearAiState?: () => void;
}

export function TaskFormDialog({
  open,
  mode,
  form,
  onOpenChange,
  onSubmit,
  isSubmitting = false,
  serverError,
  aiState,
  hasDeepseekKey = true,
  onTriggerAiParse,
  onClearAiState,
}: TaskFormDialogProps) {
  const submitLabel = mode === 'create' ? '创建任务' : '保存修改';
  const title = mode === 'create' ? '新建任务' : '编辑任务';
  const description =
    mode === 'create'
      ? '填写任务的关键信息，提交后将自动保存至数据库。'
      : '更新任务信息，保存后会立即同步到任务列表。';

  const statusOptions = useMemo(() => TASK_STATUSES, []);

  const handleSubmit = form.handleSubmit(async (values) => {
    const normalized: TaskFormValues = {
      ...values,
      description: values.description?.trim() || undefined,
      tags: (values.tags ?? []).map((tag) => tag.trim()).filter(Boolean),
      externalLinks: (values.externalLinks ?? []).filter(Boolean),
    };
    await onSubmit(normalized);
  });

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
          <DialogDescription>{description}</DialogDescription>
        </DialogHeader>

        {serverError ? (
          <div className="rounded-lg border border-destructive/50 bg-destructive/10 px-4 py-3 text-sm text-destructive">
            {serverError}
          </div>
        ) : null}

        <Form {...form}>
          <form className="space-y-5" onSubmit={handleSubmit}>
            {aiState && onTriggerAiParse ? (
              <TaskAiParsePanel
                aiState={aiState}
                hasDeepseekKey={hasDeepseekKey}
                onParse={onTriggerAiParse}
                onClear={onClearAiState}
                disabled={isSubmitting}
                mode={mode}
              />
            ) : null}
            <FormField
              control={form.control}
              name="title"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>任务标题</FormLabel>
                  <FormControl>
                    <Input placeholder="例如：准备周会材料" {...field} />
                  </FormControl>
                  <FormDescription>保持简洁明了，便于快速识别。</FormDescription>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="description"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>任务描述</FormLabel>
                  <FormControl>
                    <Textarea rows={4} placeholder="补充任务的背景、目标等信息" {...field} />
                  </FormControl>
                  <FormDescription>可选，但建议填写以帮助未来的自己。</FormDescription>
                  <FormMessage />
                </FormItem>
              )}
            />

            <div className="grid gap-4 md:grid-cols-2">
              <FormField
                control={form.control}
                name="status"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>状态</FormLabel>
                    <FormControl>
                      <select
                        className="h-10 w-full rounded-lg border border-border/60 bg-background px-3 text-sm text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary"
                        value={field.value ?? 'todo'}
                        onChange={field.onChange}
                        onBlur={field.onBlur}
                      >
                        {statusOptions.map((status) => (
                          <option key={status} value={status}>
                            {STATUS_LABELS[status]}
                          </option>
                        ))}
                      </select>
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="priority"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>优先级</FormLabel>
                    <FormControl>
                      <select
                        className="h-10 w-full rounded-lg border border-border/60 bg-background px-3 text-sm text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary"
                        value={field.value ?? 'medium'}
                        onChange={field.onChange}
                        onBlur={field.onBlur}
                      >
                        {TASK_PRIORITIES.map((priority) => (
                          <option key={priority} value={priority}>
                            {PRIORITY_LABELS[priority]}
                          </option>
                        ))}
                      </select>
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </div>

            <div className="grid gap-4 md:grid-cols-2">
              <FormField
                control={form.control}
                name="startAt"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>计划开始</FormLabel>
                    <FormControl>
                      <Input
                        type="datetime-local"
                        value={toLocalDateInput(field.value)}
                        onChange={(event) => field.onChange(fromLocalDateInput(event.target.value))}
                      />
                    </FormControl>
                    <FormDescription>可选，用于提前预留时间段。</FormDescription>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="dueAt"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>截止时间</FormLabel>
                    <FormControl>
                      <Input
                        type="datetime-local"
                        value={toLocalDateInput(field.value)}
                        onChange={(event) => field.onChange(fromLocalDateInput(event.target.value))}
                      />
                    </FormControl>
                    <FormDescription>建议至少设置截止时间，便于提醒。</FormDescription>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </div>

            <div className="grid gap-4 md:grid-cols-2">
              <FormField
                control={form.control}
                name="estimatedMinutes"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>预估时长（分钟）</FormLabel>
                    <FormControl>
                      <Input
                        type="number"
                        min={15}
                        step={15}
                        value={field.value ?? ''}
                        onChange={(event) => {
                          const value = event.target.value;
                          field.onChange(value === '' ? undefined : Number(value));
                        }}
                      />
                    </FormControl>
                    <FormDescription>用于后续的时间块排程与番茄钟建议。</FormDescription>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="tags"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>标签</FormLabel>
                    <FormControl>
                      <Input
                        placeholder="例如：planning, research"
                        value={(field.value ?? []).join(', ')}
                        onChange={(event) => {
                          const value = event.target.value;
                          const tags = value
                            .split(',')
                            .map((tag) => tag.trim())
                            .filter(Boolean);
                          field.onChange(tags);
                        }}
                      />
                    </FormControl>
                    <FormDescription>使用逗号分隔标签，便于后续过滤。</FormDescription>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </div>

            <DialogFooter>
              <Button type="button" variant="ghost" onClick={() => onOpenChange(false)}>
                取消
              </Button>
              <Button type="submit" disabled={isSubmitting}>
                {isSubmitting ? '处理中…' : submitLabel}
              </Button>
            </DialogFooter>
          </form>
        </Form>
      </DialogContent>
    </Dialog>
  );
}

const STATUS_LABELS: Record<(typeof TASK_STATUSES)[number], string> = {
  backlog: '待整理',
  todo: '待开始',
  in_progress: '进行中',
  blocked: '受阻',
  done: '已完成',
  archived: '已归档',
};

const PRIORITY_LABELS: Record<(typeof TASK_PRIORITIES)[number], string> = {
  low: '低',
  medium: '中',
  high: '高',
  urgent: '紧急',
};

function toLocalDateInput(value?: string | null) {
  if (!value) return '';
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return '';
  const localDate = new Date(date.getTime() - date.getTimezoneOffset() * 60000);
  return localDate.toISOString().slice(0, 16);
}

function fromLocalDateInput(value: string): string | undefined {
  if (!value) return undefined;
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return undefined;
  return date.toISOString();
}
