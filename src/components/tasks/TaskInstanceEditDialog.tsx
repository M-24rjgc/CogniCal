import { useEffect } from 'react';
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
import { Badge } from '../ui/badge';
import { Card, CardContent, CardHeader, CardTitle } from '../ui/card';
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
import {
  TASK_STATUSES,
  TASK_PRIORITIES,
  type TaskInstance,
  type RecurringTaskTemplate,
  type TaskStatus,
  type TaskPriority,
} from '../../types/task';

// Form validation schema
const instanceEditSchema = z.object({
  title: z.string().min(1, '任务标题不能为空'),
  description: z.string().optional(),
  status: z.enum(TASK_STATUSES),
  priority: z.enum(TASK_PRIORITIES),
  dueAt: z.string().optional(),
  estimatedMinutes: z.number().min(15).optional(),
});

type InstanceEditFormValues = z.infer<typeof instanceEditSchema>;

interface TaskInstanceEditDialogProps {
  open: boolean;
  instance: TaskInstance | null;
  template: RecurringTaskTemplate | undefined;
  editType: 'single' | 'series';
  onOpenChange: (open: boolean) => void;
  onSubmit: (
    values: InstanceEditFormValues,
    editType: 'single' | 'series',
    instanceId: string
  ) => Promise<void> | void;
  onEditTypeChange: (editType: 'single' | 'series') => void;
  isSubmitting?: boolean;
  serverError?: string | null;
}

export function TaskInstanceEditDialog({
  open,
  instance,
  template,
  editType,
  onOpenChange,
  onSubmit,
  onEditTypeChange,
  isSubmitting = false,
  serverError,
}: TaskInstanceEditDialogProps) {
  const form = useForm<InstanceEditFormValues>({
    resolver: zodResolver(instanceEditSchema),
    defaultValues: {
      title: '',
      description: '',
      status: 'todo',
      priority: 'medium',
      dueAt: undefined,
      estimatedMinutes: undefined,
    },
  });

  // Update form when instance changes
  useEffect(() => {
    if (instance) {
      form.reset({
        title: instance.title,
        description: instance.description || '',
        status: instance.status,
        priority: instance.priority,
        dueAt: instance.dueAt ? toLocalDateInput(instance.dueAt) : undefined,
        estimatedMinutes: template?.estimatedMinutes,
      });
    }
  }, [instance, template, form]);

  const handleSubmit = form.handleSubmit(async (values) => {
    if (!instance) return;
    
    const normalized: InstanceEditFormValues = {
      ...values,
      description: values.description?.trim() || undefined,
      dueAt: values.dueAt ? fromLocalDateInput(values.dueAt) : undefined,
    };
    
    await onSubmit(normalized, editType, instance.id);
  });

  if (!instance || !template) return null;

  const title = editType === 'single' ? '编辑任务实例' : '编辑重复任务系列';
  const description = editType === 'single'
    ? '修改此单个任务实例，不会影响其他实例。'
    : '修改整个重复任务系列，将应用到所有未来的实例。';

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
          <DialogDescription>{description}</DialogDescription>
        </DialogHeader>

        {serverError && (
          <div className="rounded-lg border border-destructive/50 bg-destructive/10 px-4 py-3 text-sm text-destructive">
            {serverError}
          </div>
        )}

        {/* Edit Type Selector */}
        <Card>
          <CardHeader>
            <CardTitle className="text-base">编辑范围</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="flex gap-4">
              <Button
                type="button"
                variant={editType === 'single' ? "default" : "outline"}
                onClick={() => onEditTypeChange('single')}
                className="flex-1"
              >
                <div className="text-center">
                  <div className="font-medium">仅此实例</div>
                  <div className="text-xs opacity-80">只修改当前选中的任务</div>
                </div>
              </Button>
              <Button
                type="button"
                variant={editType === 'series' ? "default" : "outline"}
                onClick={() => onEditTypeChange('series')}
                className="flex-1"
              >
                <div className="text-center">
                  <div className="font-medium">整个系列</div>
                  <div className="text-xs opacity-80">修改所有未来的任务</div>
                </div>
              </Button>
            </div>
            
            {editType === 'single' && (
              <div className="mt-3 p-3 bg-blue-50 dark:bg-blue-950/20 rounded-lg">
                <div className="flex items-center gap-2">
                  <Badge variant="outline">例外实例</Badge>
                  <span className="text-sm">此实例将被标记为例外，独立于系列规则。</span>
                </div>
              </div>
            )}
            
            {editType === 'series' && (
              <div className="mt-3 p-3 bg-orange-50 dark:bg-orange-950/20 rounded-lg">
                <div className="flex items-center gap-2">
                  <Badge variant="outline">系列修改</Badge>
                  <span className="text-sm">更改将应用到所有未来的任务实例。</span>
                </div>
              </div>
            )}
          </CardContent>
        </Card>

        <Form {...form}>
          <form className="space-y-4" onSubmit={handleSubmit}>
            <FormField
              control={form.control}
              name="title"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>任务标题</FormLabel>
                  <FormControl>
                    <Input placeholder="任务标题" {...field} />
                  </FormControl>
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
                    <Textarea rows={3} placeholder="任务描述" {...field} />
                  </FormControl>
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
                        value={field.value}
                        onChange={field.onChange}
                        onBlur={field.onBlur}
                      >
                        {TASK_STATUSES.map((status) => (
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
            </div>

            <div className="grid gap-4 md:grid-cols-2">
              <FormField
                control={form.control}
                name="dueAt"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>截止时间</FormLabel>
                    <FormControl>
                      <Input
                        type="datetime-local"
                        value={field.value || ''}
                        onChange={field.onChange}
                      />
                    </FormControl>
                    <FormDescription>
                      {editType === 'single' ? '仅此实例的截止时间' : '所有未来实例的截止时间'}
                    </FormDescription>
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
            </div>

            <DialogFooter>
              <Button type="button" variant="ghost" onClick={() => onOpenChange(false)}>
                取消
              </Button>
              <Button type="submit" disabled={isSubmitting}>
                {isSubmitting ? '保存中…' : '保存修改'}
              </Button>
            </DialogFooter>
          </form>
        </Form>
      </DialogContent>
    </Dialog>
  );
}

// Helper functions
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

// Labels
const STATUS_LABELS: Record<TaskStatus, string> = {
  backlog: '待整理',
  todo: '待开始',
  in_progress: '进行中',
  blocked: '受阻',
  done: '已完成',
  archived: '已归档',
};

const PRIORITY_LABELS: Record<TaskPriority, string> = {
  low: '低',
  medium: '中',
  high: '高',
  urgent: '紧急',
};