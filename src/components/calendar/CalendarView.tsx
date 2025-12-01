import { useMemo, useState } from 'react';
import { ChevronLeft, ChevronRight } from 'lucide-react';
import { Button } from '../ui/button';
import { Badge } from '../ui/badge';
import { cn } from '../../lib/utils';
import { extractDateKey, formatDateKey, isSameDay, parseDateTime } from '../../utils/date';
import type { Task } from '../../types/task';
import type { PlanningOptionView } from '../../types/planning';

interface CalendarViewProps {
  tasks: Task[];
  planningBlocks?: PlanningOptionView['blocks'];
  onDateClick?: (date: Date) => void;
  onTaskClick?: (task: Task) => void;
  onBlockClick?: (block: PlanningOptionView['blocks'][number]) => void;
  className?: string;
  selectedDate?: Date | null;
}

interface CalendarDay {
  date: Date;
  isCurrentMonth: boolean;
  isToday: boolean;
  tasks: Task[];
  blocks: PlanningOptionView['blocks'];
}

const WEEKDAYS = ['周一', '周二', '周三', '周四', '周五', '周六', '周日'];
const TIME_FORMATTER = new Intl.DateTimeFormat('zh-CN', {
  hour: '2-digit',
  minute: '2-digit',
});

export function CalendarView({
  tasks,
  planningBlocks = [],
  onDateClick,
  onTaskClick,
  onBlockClick,
  className,
  selectedDate,
}: CalendarViewProps) {
  const [currentDate, setCurrentDate] = useState(new Date());

  const { year, month } = useMemo(() => {
    return {
      year: currentDate.getFullYear(),
      month: currentDate.getMonth(),
    };
  }, [currentDate]);

  const calendarDays = useMemo<CalendarDay[]>(() => {
    const firstDay = new Date(year, month, 1);
    const lastDay = new Date(year, month + 1, 0);
    const daysInMonth = lastDay.getDate();

    // 获取第一天是星期几（0=周日，1=周一...）
    let firstDayOfWeek = firstDay.getDay();
    // 转换为周一开始（0=周一，6=周日）
    firstDayOfWeek = firstDayOfWeek === 0 ? 6 : firstDayOfWeek - 1;

    const days: CalendarDay[] = [];
    const today = new Date();
    today.setHours(0, 0, 0, 0);

    // 添加上个月的日期
    const prevMonthLastDay = new Date(year, month, 0).getDate();
    for (let i = firstDayOfWeek - 1; i >= 0; i--) {
      const date = new Date(year, month - 1, prevMonthLastDay - i);
      days.push({
        date,
        isCurrentMonth: false,
        isToday: date.getTime() === today.getTime(),
        tasks: getTasksForDate(date, tasks),
        blocks: getBlocksForDate(date, planningBlocks),
      });
    }

    // 添加当前月的日期
    for (let day = 1; day <= daysInMonth; day++) {
      const date = new Date(year, month, day);
      days.push({
        date,
        isCurrentMonth: true,
        isToday: date.getTime() === today.getTime(),
        tasks: getTasksForDate(date, tasks),
        blocks: getBlocksForDate(date, planningBlocks),
      });
    }

    // 添加下个月的日期，补齐到42天（6周）
    const remainingDays = 42 - days.length;
    for (let day = 1; day <= remainingDays; day++) {
      const date = new Date(year, month + 1, day);
      days.push({
        date,
        isCurrentMonth: false,
        isToday: date.getTime() === today.getTime(),
        tasks: getTasksForDate(date, tasks),
        blocks: getBlocksForDate(date, planningBlocks),
      });
    }

    return days;
  }, [year, month, tasks, planningBlocks]);

  const handlePrevMonth = () => {
    setCurrentDate(new Date(year, month - 1, 1));
  };

  const handleNextMonth = () => {
    setCurrentDate(new Date(year, month + 1, 1));
  };

  const handleToday = () => {
    setCurrentDate(new Date());
  };

  const monthLabel = new Intl.DateTimeFormat('zh-CN', {
    year: 'numeric',
    month: 'long',
  }).format(currentDate);

  return (
    <div className={cn('flex flex-col gap-4', className)}>
      {/* 日历头部 */}
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-semibold text-foreground">{monthLabel}</h2>
        <div className="flex items-center gap-2">
          <Button variant="outline" size="sm" onClick={handleToday}>
            今天
          </Button>
          <div className="flex items-center gap-1">
            <Button variant="ghost" size="icon" onClick={handlePrevMonth}>
              <ChevronLeft className="h-4 w-4" />
            </Button>
            <Button variant="ghost" size="icon" onClick={handleNextMonth}>
              <ChevronRight className="h-4 w-4" />
            </Button>
          </div>
        </div>
      </div>

      {/* 日历网格 */}
      <div className="rounded-2xl border border-border/60 bg-card/80 p-4 shadow-sm">
        {/* 星期标题 */}
        <div className="mb-2 grid grid-cols-7 gap-2">
          {WEEKDAYS.map((day) => (
            <div key={day} className="py-2 text-center text-xs font-semibold text-muted-foreground">
              {day}
            </div>
          ))}
        </div>

        {/* 日期网格 */}
        <div className="grid grid-cols-7 gap-2">
          {calendarDays.map((day, index) => (
            <CalendarDayCell
              key={index}
              day={day}
              onDateClick={onDateClick}
              onTaskClick={onTaskClick}
              onBlockClick={onBlockClick}
              selectedDate={selectedDate}
            />
          ))}
        </div>
      </div>

      {/* 图例 */}
      <div className="flex flex-wrap items-center gap-4 text-xs text-muted-foreground">
        <div className="flex items-center gap-2">
          <div className="h-3 w-3 rounded-sm bg-sky-500/20 border border-sky-500/40" />
          <span>有任务</span>
        </div>
        <div className="flex items-center gap-2">
          <div className="h-3 w-3 rounded-sm bg-primary/20 border border-primary/40" />
          <span>已规划</span>
        </div>
        <div className="flex items-center gap-2">
          <div className="h-3 w-3 rounded-sm bg-emerald-500/20 border border-emerald-500/40" />
          <span>已完成</span>
        </div>
      </div>
    </div>
  );
}

interface CalendarDayCellProps {
  day: CalendarDay;
  onDateClick?: (date: Date) => void;
  onTaskClick?: (task: Task) => void;
  onBlockClick?: (block: PlanningOptionView['blocks'][number]) => void;
  selectedDate?: Date | null;
}

function CalendarDayCell({
  day,
  onDateClick,
  onTaskClick,
  onBlockClick,
  selectedDate,
}: CalendarDayCellProps) {
  const dayNumber = day.date.getDate();
  const hasTasks = day.tasks.length > 0;
  const hasBlocks = day.blocks.length > 0;
  const hasContent = hasTasks || hasBlocks;

  const completedTasks = day.tasks.filter((t) => t.status === 'done').length;
  const totalTasks = day.tasks.length;
  const isSelected = isSameDay(day.date, selectedDate);

  return (
    <div
      role="button"
      tabIndex={0}
      className={cn(
        'group relative min-h-[100px] cursor-pointer rounded-xl border p-2 transition-all focus:outline-none focus-visible:ring-2 focus-visible:ring-primary/50',
        day.isCurrentMonth
          ? 'border-border/60 bg-background hover:border-primary/40 hover:bg-primary/5'
          : 'border-border/30 bg-muted/30 hover:border-border/50',
        isSelected
          ? 'border-primary bg-primary/10 ring-2 ring-primary/40'
          : day.isToday
            ? 'border-primary/40 ring-1 ring-primary/30'
            : null,
      )}
      onClick={() => onDateClick?.(day.date)}
      onKeyDown={(event) => {
        if (event.key === 'Enter' || event.key === ' ') {
          event.preventDefault();
          onDateClick?.(day.date);
        }
      }}
    >
      {/* 日期数字 */}
      <div className="flex items-center justify-between mb-2">
        <span
          className={cn(
            'flex h-6 w-6 items-center justify-center rounded-full text-sm font-medium',
            isSelected
              ? 'bg-primary text-primary-foreground'
              : day.isToday
                ? 'bg-primary/10 text-primary'
                : day.isCurrentMonth
                  ? 'text-foreground'
                  : 'text-muted-foreground',
          )}
        >
          {dayNumber}
        </span>
        {hasContent && (
          <Badge variant="secondary" className="h-5 px-1.5 text-[10px]">
            {totalTasks + day.blocks.length}
          </Badge>
        )}
      </div>

      {/* 任务和时间块列表 */}
      <div className="space-y-1">
        {/* 显示任务 */}
        {day.tasks.slice(0, 2).map((task) => (
          <button
            key={task.id}
            className={cn(
              'rounded px-1.5 py-0.5 text-[10px] leading-tight truncate cursor-pointer transition',
              task.status === 'done'
                ? 'bg-emerald-500/20 text-emerald-700 dark:text-emerald-300 hover:bg-emerald-500/30'
                : 'bg-sky-500/20 text-sky-700 dark:text-sky-300 hover:bg-sky-500/30',
            )}
            type="button"
            onClick={(e) => {
              e.stopPropagation();
              onTaskClick?.(task);
            }}
            title={task.title}
          >
            {task.title}
          </button>
        ))}

        {/* 显示规划时间块 */}
        {day.blocks.slice(0, 2).map((block) => (
          <button
            key={block.id}
            className="rounded px-1.5 py-0.5 text-[10px] leading-tight truncate cursor-pointer bg-primary/20 text-primary hover:bg-primary/30 transition"
            type="button"
            onClick={(e) => {
              e.stopPropagation();
              onBlockClick?.(block);
            }}
            title={`${formatTime(block.startAt)} - ${formatTime(block.endAt)}`}
          >
            📅 {formatTime(block.startAt)}
          </button>
        ))}

        {/* 更多指示器 */}
        {totalTasks + day.blocks.length > 4 && (
          <div className="text-[10px] text-muted-foreground text-center">
            +{totalTasks + day.blocks.length - 4} 更多
          </div>
        )}
      </div>

      {/* 完成进度指示器 */}
      {totalTasks > 0 && (
        <div className="absolute bottom-1 left-1 right-1">
          <div className="h-1 rounded-full bg-muted">
            <div
              className="h-full rounded-full bg-emerald-500 transition-all"
              style={{ width: `${(completedTasks / totalTasks) * 100}%` }}
            />
          </div>
        </div>
      )}
    </div>
  );
}

function getTasksForDate(date: Date, tasks: Task[]): Task[] {
  const dateKey = formatDateKey(date);
  return tasks.filter((task) => {
    const dueKey = task.dueAt ? extractDateKey(task.dueAt) : null;
    const startKey = task.startAt ? extractDateKey(task.startAt) : null;
    return dueKey === dateKey || startKey === dateKey;
  });
}

function getBlocksForDate(
  date: Date,
  blocks: PlanningOptionView['blocks'],
): PlanningOptionView['blocks'] {
  const dateKey = formatDateKey(date);
  return blocks.filter((block) => {
    const blockKey = extractDateKey(block.startAt);
    return blockKey === dateKey;
  });
}

function formatTime(dateStr: string): string {
  const date = parseDateTime(dateStr);
  return TIME_FORMATTER.format(date);
}
