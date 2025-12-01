import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
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
import { RecurrenceRuleBuilder } from './RecurrenceRuleBuilder';
import {
  TASK_PRIORITIES,
  RECURRENCE_FREQUENCIES,
  RECURRENCE_END_TYPES,
  type TaskPriority,
  type RecurringTaskTemplate,
} from '../../types/task';

// Form validation schema
const recurringTaskSchema = z.object({
  title: z.string().min(1, '任务标题不能为空'),
  description: z.string().optional(),
  priority: z.enum(TASK_PRIORITIES),
  tags: z.array(z.string()).default([]),
  estimatedMinutes: z.number().min(15).optional(),
  recurrenceRule: z.object({
    frequency: z.enum(RECURRENCE_FREQUENCIES),
    interval: z.number().min(1).max(365),
    endType: z.enum(RECURRENCE_END_TYPES),
    endDate: z.string().optional(),
    endCount: z.number().min(1).max(1000).optional(),
    weekdays: z.array(z.number().min(0).max(6)).optional(),
    monthDay: z.number().min(1).max(31).optional(),
    monthWeek: z.number().min(1).max(4).optional(),
    monthWeekday: z.number().min(0).max(6).optional(),
  }),
});

type RecurringTaskFormValues = z.infer<typeof recurringTaskSchema>;

interface RecurringTaskFormProps {
  open: boolean;
  mode: 'create' | 'edit';
  template?: RecurringTaskTemplate;
  onOpenChange: (open: boolean) => void;
  onSubmit: (values: RecurringTaskFormValues) => Promise<void> | void;
  isSubmitting?: boolean;
  serverError?: string | null;
}

const DEFAULT_VALUES: RecurringTaskFormValues = {
  title: '',
  description: '',
  priority: 'medium',
  tags: [],
  estimatedMinutes: undefined,
  recurrenceRule: {
    frequency: 'daily',
    interval: 1,
    endType: 'never',
    endDate: undefined,
    endCount: undefined,
    weekdays: [],
    monthDay: undefined,
    monthWeek: undefined,
    monthWeekday: undefined,
  },
};

export function RecurringTaskForm({
  open,
  mode,
  template,
  onOpenChange,
  onSubmit,
  isSubmitting = false,
  serverError,
}: RecurringTaskFormProps) {
  const form = useForm<RecurringTaskFormValues>({
    resolver: zodResolver(recurringTaskSchema),
    defaultValues: template ? templateToFormValues(template) : DEFAULT_VALUES,
  });

  const submitLabel = mode === 'create' ? '创建重复任务' : '保存修改';
  const title = mode === 'create' ? '新建重复任务' : '编辑重复任务';
  const description = mode === 'create' 
    ? '创建一个按规律重复的任务模板，系统将自动生成未来的任务实例。'
    : '修改重复任务模板，更改将应用到未来的任务实例。';

  const handleSubmit = form.handleSubmit(async (values) => {
    const normalized: RecurringTaskFormValues = {
      ...values,
      description: values.description?.trim() || undefined,
      tags: values.tags.map((tag) => tag.trim()).filter(Boolean),
    };
    await onSubmit(normalized);
  });

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-h-[90vh] max-w-4xl overflow-y-auto">
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
          <DialogDescription>{description}</DialogDescription>
        </DialogHeader>

        {serverError && (
          <div className="rounded-lg border border-destructive/50 bg-destructive/10 px-4 py-3 text-sm text-destructive">
            {serverError}
          </div>
        )}

        <Form {...form}>
          <form className="space-y-6" onSubmit={handleSubmit}>
            {/* Basic Task Information */}
            <div className="space-y-4">
              <h3 className="text-lg font-medium">基本信息</h3>
              
              <FormField
                control={form.control}
                name="title"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>任务标题</FormLabel>
                    <FormControl>
                      <Input placeholder="例如：每日站会、周报总结" {...field} />
                    </FormControl>
                    <FormDescription>重复任务的标题，将应用到所有生成的任务实例。</FormDescription>
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
                      <Textarea rows={3} placeholder="描述任务的具体内容和要求" {...field} />
                    </FormControl>
                    <FormDescription>可选，但建议填写以帮助理解任务内容。</FormDescription>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <div className="grid gap-4 md:grid-cols-3">
                <FormField
                  control={form.control}
                  name="priority"
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>优先级</FormLabel>
                      <FormControl>
                        <select
                          className="h-10 w-full rounded-lg border border-border/60 bg-background px-3 text-sm text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary"
                          value={field.value}
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
                          placeholder="例如：meeting, daily"
                          value={field.value.join(', ')}
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
                      <FormMessage />
                    </FormItem>
                  )}
                />
              </div>
            </div>

            {/* Recurrence Rule Configuration */}
            <RecurrenceRuleBuilder form={form} />

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



// Helper functions
function templateToFormValues(template: RecurringTaskTemplate): RecurringTaskFormValues {
  return {
    title: template.title,
    description: template.description || '',
    priority: template.priority,
    tags: template.tags,
    estimatedMinutes: template.estimatedMinutes,
    recurrenceRule: template.recurrenceRule,
  };
}

// Labels
const PRIORITY_LABELS: Record<TaskPriority, string> = {
  low: '低',
  medium: '中',
  high: '高',
  urgent: '紧急',
};