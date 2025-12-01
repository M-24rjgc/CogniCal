
import { Repeat, Calendar, Clock } from 'lucide-react';
import { Badge } from '../ui/badge';
import { Button } from '../ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '../ui/dropdown-menu';
import {
  type Task,
  type TaskInstance,
  type RecurringTaskTemplate,
  type RecurrenceFrequency,
} from '../../types/task';

interface RecurringTaskIndicatorProps {
  task: Task;
  instance?: TaskInstance;
  template?: RecurringTaskTemplate;
  onManageInstances?: () => void;
  onEditTemplate?: () => void;
  onViewSeries?: () => void;
  compact?: boolean;
}

export function RecurringTaskIndicator({
  task,
  instance,
  template,
  onManageInstances,
  onEditTemplate,
  onViewSeries,
  compact = false,
}: RecurringTaskIndicatorProps) {
  if (!task.isRecurring && !instance) {
    return null;
  }

  const isInstance = Boolean(instance);
  const recurrenceRule = template?.recurrenceRule || task.recurrence;
  
  if (compact) {
    return (
      <div className="flex items-center gap-1">
        <Badge variant="outline" className="text-xs">
          <Repeat className="mr-1 h-3 w-3" />
          {isInstance ? '实例' : '重复'}
        </Badge>
        {instance?.isException && (
          <Badge variant="outline" className="text-xs text-orange-600">
            例外
          </Badge>
        )}
      </div>
    );
  }

  return (
    <div className="flex items-center gap-2">
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button variant="ghost" size="sm" className="h-auto p-1">
            <div className="flex items-center gap-2">
              <Badge variant="outline" className="text-xs">
                <Repeat className="mr-1 h-3 w-3" />
                {isInstance ? '重复任务实例' : '重复任务'}
              </Badge>
              {instance?.isException && (
                <Badge variant="outline" className="text-xs text-orange-600">
                  <Clock className="mr-1 h-3 w-3" />
                  例外实例
                </Badge>
              )}
            </div>
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="start">
          {recurrenceRule && (
            <div className="px-2 py-1.5 text-xs text-muted-foreground">
              {getRecurrenceDescription(recurrenceRule)}
            </div>
          )}
          <DropdownMenuSeparator />
          {onViewSeries && (
            <DropdownMenuItem onClick={onViewSeries}>
              <Calendar className="mr-2 h-4 w-4" />
              查看系列
            </DropdownMenuItem>
          )}
          {onManageInstances && (
            <DropdownMenuItem onClick={onManageInstances}>
              <Repeat className="mr-2 h-4 w-4" />
              管理实例
            </DropdownMenuItem>
          )}
          {onEditTemplate && (
            <DropdownMenuItem onClick={onEditTemplate}>
              <Clock className="mr-2 h-4 w-4" />
              编辑模板
            </DropdownMenuItem>
          )}
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  );
}

// Helper function to generate human-readable recurrence description
function getRecurrenceDescription(rule: any): string {
  if (!rule) return '重复任务';
  
  const frequency = rule.frequency as RecurrenceFrequency;
  const interval = rule.interval || 1;
  
  const frequencyLabels: Record<RecurrenceFrequency, string> = {
    daily: '天',
    weekly: '周',
    monthly: '月',
    yearly: '年',
  };
  
  let description = `每 ${interval} ${frequencyLabels[frequency]}`;
  
  if (frequency === 'weekly' && rule.weekdays?.length > 0) {
    const weekdayNames = rule.weekdays.map((w: number) => {
      const weekdays = ['日', '一', '二', '三', '四', '五', '六'];
      return weekdays[w];
    }).join('、');
    description += ` (${weekdayNames})`;
  }
  
  if (frequency === 'monthly') {
    if (rule.monthDay) {
      description += ` 第${rule.monthDay}天`;
    } else if (rule.monthWeek && rule.monthWeekday !== undefined) {
      const weekdays = ['日', '一', '二', '三', '四', '五', '六'];
      description += ` 第${rule.monthWeek}个${weekdays[rule.monthWeekday]}`;
    }
  }
  
  if (rule.endType === 'count' && rule.endCount) {
    description += ` (共${rule.endCount}次)`;
  } else if (rule.endType === 'date' && rule.endDate) {
    const endDate = new Date(rule.endDate);
    description += ` (至${endDate.toLocaleDateString('zh-CN')})`;
  }
  
  return description;
}