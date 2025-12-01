import { useState, useCallback, useMemo } from 'react';
import { UseFormReturn } from 'react-hook-form';
import { Card, CardContent, CardHeader, CardTitle } from '../ui/card';
import { Button } from '../ui/button';
import { Badge } from '../ui/badge';
import {
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '../ui/form';
import { Input } from '../ui/input';
import {
  RECURRENCE_FREQUENCIES,
  RECURRENCE_END_TYPES,
  WEEKDAYS,
  type RecurrenceRuleConfig,
  type RecurrenceFrequency,
  type RecurrenceEndType,
} from '../../types/task';

interface RecurrenceRuleBuilderProps {
  form: UseFormReturn<any>;
  fieldName?: string;
  showPreview?: boolean;
  maxPreviewItems?: number;
}

export function RecurrenceRuleBuilder({
  form,
  fieldName = 'recurrenceRule',
  showPreview = true,
  maxPreviewItems = 10,
}: RecurrenceRuleBuilderProps) {
  const [monthlyType, setMonthlyType] = useState<'day' | 'weekday'>('day');
  
  const frequency = form.watch(`${fieldName}.frequency`) as RecurrenceFrequency;
  const endType = form.watch(`${fieldName}.endType`) as RecurrenceEndType;
  const rule = form.watch(fieldName) as RecurrenceRuleConfig;

  // Generate preview occurrences
  const previewOccurrences = useMemo(() => {
    if (!rule || !showPreview) return [];
    return generateDetailedPreview(rule, maxPreviewItems);
  }, [rule, showPreview, maxPreviewItems]);

  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle className="text-base">重复规则配置</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          {/* Frequency and Interval */}
          <div className="grid gap-4 md:grid-cols-2">
            <FormField
              control={form.control}
              name={`${fieldName}.frequency`}
              render={({ field }) => (
                <FormItem>
                  <FormLabel>重复频率</FormLabel>
                  <FormControl>
                    <select
                      className="h-10 w-full rounded-lg border border-border/60 bg-background px-3 text-sm text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary"
                      value={field.value}
                      onChange={(e) => {
                        field.onChange(e.target.value);
                        // Reset frequency-specific fields when frequency changes
                        if (e.target.value !== 'weekly') {
                          form.setValue(`${fieldName}.weekdays`, []);
                        }
                        if (e.target.value !== 'monthly') {
                          form.setValue(`${fieldName}.monthDay`, undefined);
                          form.setValue(`${fieldName}.monthWeek`, undefined);
                          form.setValue(`${fieldName}.monthWeekday`, undefined);
                        }
                      }}
                      onBlur={field.onBlur}
                    >
                      {RECURRENCE_FREQUENCIES.map((freq) => (
                        <option key={freq} value={freq}>
                          {FREQUENCY_LABELS[freq]}
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
              name={`${fieldName}.interval`}
              render={({ field }) => (
                <FormItem>
                  <FormLabel>间隔</FormLabel>
                  <FormControl>
                    <Input
                      type="number"
                      min={1}
                      max={365}
                      value={field.value}
                      onChange={(event) => field.onChange(Number(event.target.value))}
                    />
                  </FormControl>
                  <FormDescription>
                    每 {field.value} {FREQUENCY_LABELS[frequency]} 重复一次
                  </FormDescription>
                  <FormMessage />
                </FormItem>
              )}
            />
          </div>

          {/* Frequency-specific options */}
          {frequency === 'weekly' && (
            <WeekdaySelector form={form} fieldName={`${fieldName}.weekdays`} />
          )}
          
          {frequency === 'monthly' && (
            <MonthlyPatternSelector
              form={form}
              fieldName={fieldName}
              monthlyType={monthlyType}
              onMonthlyTypeChange={setMonthlyType}
            />
          )}

          {/* End conditions */}
          <EndConditionSelector form={form} fieldName={fieldName} endType={endType} />
        </CardContent>
      </Card>

      {/* Preview Section */}
      {showPreview && previewOccurrences.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle className="text-base flex items-center gap-2">
              预览接下来的任务
              <Badge variant="secondary">{previewOccurrences.length} 个</Badge>
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              {previewOccurrences.map((occurrence, index) => (
                <div key={index} className="flex items-center gap-3 p-2 rounded-lg bg-muted/50">
                  <Badge variant="outline" className="min-w-[2rem] justify-center">
                    {index + 1}
                  </Badge>
                  <div className="flex-1">
                    <div className="font-medium text-sm">{occurrence.date}</div>
                    <div className="text-xs text-muted-foreground">{occurrence.description}</div>
                  </div>
                </div>
              ))}
              {rule.endType === 'never' && (
                <div className="text-xs text-muted-foreground text-center pt-2 border-t">
                  任务将无限期重复...
                </div>
              )}
            </div>
          </CardContent>
        </Card>
      )}

      {/* RRULE Preview */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base">生成的重复规则</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="font-mono text-sm bg-muted p-3 rounded-lg">
            {generateRRuleString(rule)}
          </div>
          <div className="text-xs text-muted-foreground mt-2">
            此规则遵循 iCalendar RRULE 标准 (RFC 5545)
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

// Weekday Selector Component
interface WeekdaySelectorProps {
  form: UseFormReturn<any>;
  fieldName: string;
}

function WeekdaySelector({ form, fieldName }: WeekdaySelectorProps) {
  const selectedWeekdays = form.watch(fieldName) || [];

  const toggleWeekday = useCallback((weekday: number) => {
    const current = selectedWeekdays;
    const updated = current.includes(weekday)
      ? current.filter((w: number) => w !== weekday)
      : [...current, weekday].sort();
    form.setValue(fieldName, updated);
  }, [selectedWeekdays, form, fieldName]);

  return (
    <FormField
      control={form.control}
      name={fieldName}
      render={() => (
        <FormItem>
          <FormLabel>重复日期</FormLabel>
          <FormControl>
            <div className="grid grid-cols-7 gap-2">
              {WEEKDAYS.map((weekday) => (
                <Button
                  key={weekday.value}
                  type="button"
                  variant={selectedWeekdays.includes(weekday.value) ? "default" : "outline"}
                  size="sm"
                  className="h-12 flex flex-col gap-1"
                  onClick={() => toggleWeekday(weekday.value)}
                >
                  <span className="text-xs">{weekday.short}</span>
                  <span className="text-xs">{weekday.label.slice(0, 3)}</span>
                </Button>
              ))}
            </div>
          </FormControl>
          <FormDescription>
            选择一周中的哪些天重复任务。如果不选择，默认使用当前星期。
          </FormDescription>
          <FormMessage />
        </FormItem>
      )}
    />
  );
}

// Monthly Pattern Selector Component
interface MonthlyPatternSelectorProps {
  form: UseFormReturn<any>;
  fieldName: string;
  monthlyType: 'day' | 'weekday';
  onMonthlyTypeChange: (type: 'day' | 'weekday') => void;
}

function MonthlyPatternSelector({
  form,
  fieldName,
  monthlyType,
  onMonthlyTypeChange,
}: MonthlyPatternSelectorProps) {
  return (
    <div className="space-y-4">
      <FormItem>
        <FormLabel>月重复方式</FormLabel>
        <div className="flex gap-2">
          <Button
            type="button"
            variant={monthlyType === 'day' ? "default" : "outline"}
            size="sm"
            onClick={() => {
              onMonthlyTypeChange('day');
              form.setValue(`${fieldName}.monthWeek`, undefined);
              form.setValue(`${fieldName}.monthWeekday`, undefined);
            }}
          >
            按日期 (如每月15号)
          </Button>
          <Button
            type="button"
            variant={monthlyType === 'weekday' ? "default" : "outline"}
            size="sm"
            onClick={() => {
              onMonthlyTypeChange('weekday');
              form.setValue(`${fieldName}.monthDay`, undefined);
            }}
          >
            按星期 (如每月第一个周一)
          </Button>
        </div>
      </FormItem>

      {monthlyType === 'day' && (
        <FormField
          control={form.control}
          name={`${fieldName}.monthDay`}
          render={({ field }) => (
            <FormItem>
              <FormLabel>每月第几天</FormLabel>
              <FormControl>
                <Input
                  type="number"
                  min={1}
                  max={31}
                  placeholder="例如：15"
                  value={field.value ?? ''}
                  onChange={(event) => {
                    const value = event.target.value;
                    field.onChange(value === '' ? undefined : Number(value));
                  }}
                />
              </FormControl>
              <FormDescription>输入 1-31 之间的数字，表示每月的第几天</FormDescription>
              <FormMessage />
            </FormItem>
          )}
        />
      )}

      {monthlyType === 'weekday' && (
        <div className="grid gap-4 md:grid-cols-2">
          <FormField
            control={form.control}
            name={`${fieldName}.monthWeek`}
            render={({ field }) => (
              <FormItem>
                <FormLabel>第几周</FormLabel>
                <FormControl>
                  <select
                    className="h-10 w-full rounded-lg border border-border/60 bg-background px-3 text-sm text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary"
                    value={field.value ?? ''}
                    onChange={(event) => {
                      const value = event.target.value;
                      field.onChange(value === '' ? undefined : Number(value));
                    }}
                  >
                    <option value="">选择周数</option>
                    <option value={1}>第一周</option>
                    <option value={2}>第二周</option>
                    <option value={3}>第三周</option>
                    <option value={4}>第四周</option>
                  </select>
                </FormControl>
                <FormMessage />
              </FormItem>
            )}
          />

          <FormField
            control={form.control}
            name={`${fieldName}.monthWeekday`}
            render={({ field }) => (
              <FormItem>
                <FormLabel>星期几</FormLabel>
                <FormControl>
                  <select
                    className="h-10 w-full rounded-lg border border-border/60 bg-background px-3 text-sm text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary"
                    value={field.value ?? ''}
                    onChange={(event) => {
                      const value = event.target.value;
                      field.onChange(value === '' ? undefined : Number(value));
                    }}
                  >
                    <option value="">选择星期</option>
                    {WEEKDAYS.map((weekday) => (
                      <option key={weekday.value} value={weekday.value}>
                        {weekday.label}
                      </option>
                    ))}
                  </select>
                </FormControl>
                <FormMessage />
              </FormItem>
            )}
          />
        </div>
      )}
    </div>
  );
}

// End Condition Selector Component
interface EndConditionSelectorProps {
  form: UseFormReturn<any>;
  fieldName: string;
  endType: RecurrenceEndType;
}

function EndConditionSelector({ form, fieldName, endType }: EndConditionSelectorProps) {
  return (
    <div className="space-y-4">
      <FormField
        control={form.control}
        name={`${fieldName}.endType`}
        render={({ field }) => (
          <FormItem>
            <FormLabel>结束条件</FormLabel>
            <FormControl>
              <select
                className="h-10 w-full rounded-lg border border-border/60 bg-background px-3 text-sm text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary"
                value={field.value}
                onChange={(e) => {
                  field.onChange(e.target.value);
                  // Clear end-specific fields when end type changes
                  if (e.target.value !== 'date') {
                    form.setValue(`${fieldName}.endDate`, undefined);
                  }
                  if (e.target.value !== 'count') {
                    form.setValue(`${fieldName}.endCount`, undefined);
                  }
                }}
                onBlur={field.onBlur}
              >
                {RECURRENCE_END_TYPES.map((endType) => (
                  <option key={endType} value={endType}>
                    {END_TYPE_LABELS[endType]}
                  </option>
                ))}
              </select>
            </FormControl>
            <FormMessage />
          </FormItem>
        )}
      />

      {endType === 'date' && (
        <FormField
          control={form.control}
          name={`${fieldName}.endDate`}
          render={({ field }) => (
            <FormItem>
              <FormLabel>结束日期</FormLabel>
              <FormControl>
                <Input
                  type="date"
                  value={field.value ? field.value.split('T')[0] : ''}
                  onChange={(event) => {
                    const value = event.target.value;
                    field.onChange(value ? `${value}T23:59:59.999Z` : undefined);
                  }}
                />
              </FormControl>
              <FormDescription>任务将在此日期后停止重复</FormDescription>
              <FormMessage />
            </FormItem>
          )}
        />
      )}

      {endType === 'count' && (
        <FormField
          control={form.control}
          name={`${fieldName}.endCount`}
          render={({ field }) => (
            <FormItem>
              <FormLabel>重复次数</FormLabel>
              <FormControl>
                <Input
                  type="number"
                  min={1}
                  max={1000}
                  placeholder="例如：10"
                  value={field.value ?? ''}
                  onChange={(event) => {
                    const value = event.target.value;
                    field.onChange(value === '' ? undefined : Number(value));
                  }}
                />
              </FormControl>
              <FormDescription>任务将重复指定次数后停止</FormDescription>
              <FormMessage />
            </FormItem>
          )}
        />
      )}
    </div>
  );
}

// Helper functions
interface PreviewOccurrence {
  date: string;
  description: string;
}

function generateDetailedPreview(rule: RecurrenceRuleConfig, count: number): PreviewOccurrence[] {
  const occurrences: PreviewOccurrence[] = [];
  const now = new Date();
  
  // This is a simplified preview generator
  // In a real implementation, this would use the same RRULE logic as the backend
  for (let i = 0; i < count; i++) {
    let nextDate = new Date(now);
    
    switch (rule.frequency) {
      case 'daily':
        nextDate.setDate(now.getDate() + (i * rule.interval));
        break;
      case 'weekly':
        if (rule.weekdays && rule.weekdays.length > 0) {
          // For weekly with specific weekdays, calculate next occurrence
          const dayOffset = i * rule.interval * 7;
          nextDate.setDate(now.getDate() + dayOffset);
        } else {
          nextDate.setDate(now.getDate() + (i * rule.interval * 7));
        }
        break;
      case 'monthly':
        if (rule.monthDay) {
          nextDate.setMonth(now.getMonth() + (i * rule.interval));
          nextDate.setDate(rule.monthDay);
        } else if (rule.monthWeek && rule.monthWeekday !== undefined) {
          nextDate.setMonth(now.getMonth() + (i * rule.interval));
          // Calculate the nth weekday of the month
          const firstDay = new Date(nextDate.getFullYear(), nextDate.getMonth(), 1);
          const firstWeekday = firstDay.getDay();
          const targetWeekday = rule.monthWeekday;
          const daysToAdd = (targetWeekday - firstWeekday + 7) % 7 + (rule.monthWeek - 1) * 7;
          nextDate.setDate(1 + daysToAdd);
        } else {
          nextDate.setMonth(now.getMonth() + (i * rule.interval));
        }
        break;
      case 'yearly':
        nextDate.setFullYear(now.getFullYear() + (i * rule.interval));
        break;
    }
    
    const dateStr = nextDate.toLocaleDateString('zh-CN', {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
      weekday: 'long'
    });
    
    let description = '';
    switch (rule.frequency) {
      case 'daily':
        description = `每 ${rule.interval} 天重复`;
        break;
      case 'weekly':
        if (rule.weekdays && rule.weekdays.length > 0) {
          const weekdayNames = rule.weekdays.map(w => WEEKDAYS.find(wd => wd.value === w)?.label).join('、');
          description = `每 ${rule.interval} 周的 ${weekdayNames}`;
        } else {
          description = `每 ${rule.interval} 周重复`;
        }
        break;
      case 'monthly':
        if (rule.monthDay) {
          description = `每 ${rule.interval} 月的第 ${rule.monthDay} 天`;
        } else if (rule.monthWeek && rule.monthWeekday !== undefined) {
          const weekdayName = WEEKDAYS.find(w => w.value === rule.monthWeekday)?.label;
          description = `每 ${rule.interval} 月的第 ${rule.monthWeek} 个${weekdayName}`;
        } else {
          description = `每 ${rule.interval} 月重复`;
        }
        break;
      case 'yearly':
        description = `每 ${rule.interval} 年重复`;
        break;
    }
    
    occurrences.push({ date: dateStr, description });
  }
  
  return occurrences;
}

function generateRRuleString(rule: RecurrenceRuleConfig): string {
  if (!rule) return '';
  
  const parts = [`FREQ=${rule.frequency.toUpperCase()}`];
  
  if (rule.interval > 1) {
    parts.push(`INTERVAL=${rule.interval}`);
  }
  
  if (rule.weekdays && rule.weekdays.length > 0) {
    const weekdayStrs = rule.weekdays.map(w => WEEKDAYS.find(wd => wd.value === w)?.short).join(',');
    parts.push(`BYDAY=${weekdayStrs}`);
  }
  
  if (rule.monthDay) {
    parts.push(`BYMONTHDAY=${rule.monthDay}`);
  }
  
  if (rule.monthWeek && rule.monthWeekday !== undefined) {
    const weekdayStr = WEEKDAYS.find(w => w.value === rule.monthWeekday)?.short;
    parts.push(`BYDAY=${rule.monthWeek}${weekdayStr}`);
  }
  
  if (rule.endType === 'date' && rule.endDate) {
    const endDate = new Date(rule.endDate);
    const utcStr = endDate.toISOString().replace(/[-:]/g, '').split('.')[0] + 'Z';
    parts.push(`UNTIL=${utcStr}`);
  }
  
  if (rule.endType === 'count' && rule.endCount) {
    parts.push(`COUNT=${rule.endCount}`);
  }
  
  return parts.join(';');
}

// Labels
const FREQUENCY_LABELS: Record<RecurrenceFrequency, string> = {
  daily: '天',
  weekly: '周',
  monthly: '月',
  yearly: '年',
};

const END_TYPE_LABELS: Record<RecurrenceEndType, string> = {
  never: '永不结束',
  date: '结束日期',
  count: '重复次数',
};