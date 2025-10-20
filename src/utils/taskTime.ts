import type { Task } from '../types/task';

export type TaskTimingPhase = 'upcoming' | 'in_progress' | 'completed' | 'unscheduled';

export interface TaskTimingInfo {
  start: Date | null;
  end: Date | null;
  phase: TaskTimingPhase;
  minutesUntilStart: number | null;
  minutesUntilEnd: number | null;
  nextTriggerMinutes: number | null;
  isOverdue: boolean;
}

const parseIsoToDate = (value?: string | null): Date | null => {
  if (!value) return null;
  const parsed = new Date(value);
  return Number.isNaN(parsed.getTime()) ? null : parsed;
};

const resolveTaskStart = (task: Task): Date | null => {
  return (
    parseIsoToDate(task.startAt) ??
    parseIsoToDate(task.plannedStartAt) ??
    parseIsoToDate(task.ai?.suggestedStartAt ?? null) ??
    parseIsoToDate(task.dueAt)
  );
};

const resolveTaskEnd = (task: Task, start: Date | null): Date | null => {
  return parseIsoToDate(task.dueAt) ?? parseIsoToDate(task.completedAt) ?? start;
};

const formatMinutesDiff = (minutes: number): string => {
  const value = Math.abs(minutes);
  if (value === 0) {
    return '0 分钟';
  }
  if (value < 60) {
    return `${value} 分钟`;
  }
  const hours = Math.round(value / 60);
  if (hours < 24) {
    return `${hours} 小时`;
  }
  const days = Math.round(hours / 24);
  return `${days} 天`;
};

const formatDateForDisplay = (date: Date | null, reference: Date): string => {
  if (!date) return '未设置';
  const sameDay =
    date.getFullYear() === reference.getFullYear() &&
    date.getMonth() === reference.getMonth() &&
    date.getDate() === reference.getDate();
  const options: Intl.DateTimeFormatOptions = sameDay
    ? { hour: '2-digit', minute: '2-digit' }
    : { month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' };
  return date.toLocaleString([], options);
};

export const getTaskTiming = (task: Task, now = new Date()): TaskTimingInfo => {
  const start = resolveTaskStart(task);
  const end = resolveTaskEnd(task, start);
  const nowMs = now.getTime();
  const startMs = start ? start.getTime() : null;
  const endMs = end ? end.getTime() : null;

  let phase: TaskTimingPhase = 'unscheduled';
  if (startMs !== null && startMs > nowMs) {
    phase = 'upcoming';
  } else if (startMs !== null && endMs !== null && nowMs >= startMs && nowMs <= endMs) {
    phase = 'in_progress';
  } else if (endMs !== null && endMs < nowMs) {
    phase = 'completed';
  } else if (endMs !== null && endMs >= nowMs) {
    phase = 'upcoming';
  }

  const minutesUntilStart = startMs !== null ? Math.round((startMs - nowMs) / 60000) : null;
  const minutesUntilEnd = endMs !== null ? Math.round((endMs - nowMs) / 60000) : null;

  const nextTriggerMinutes = (() => {
    if (minutesUntilStart !== null && minutesUntilStart >= 0) {
      return minutesUntilStart;
    }
    if (minutesUntilEnd !== null && minutesUntilEnd >= 0) {
      return minutesUntilEnd;
    }
    return null;
  })();

  const isOverdue = minutesUntilEnd !== null && minutesUntilEnd < 0;

  return {
    start,
    end,
    phase,
    minutesUntilStart,
    minutesUntilEnd,
    nextTriggerMinutes,
    isOverdue,
  };
};

export const formatTaskRelative = (info: TaskTimingInfo): string => {
  const { phase, minutesUntilStart, minutesUntilEnd } = info;

  if (phase === 'upcoming') {
    if (minutesUntilStart !== null) {
      if (minutesUntilStart <= 0) {
        return '即将开始';
      }
      return `${formatMinutesDiff(minutesUntilStart)}后开始`;
    }
    if (minutesUntilEnd !== null && minutesUntilEnd > 0) {
      return `${formatMinutesDiff(minutesUntilEnd)}后到期`;
    }
    if (minutesUntilEnd !== null && minutesUntilEnd <= 0) {
      return `${formatMinutesDiff(minutesUntilEnd)}前到期`;
    }
  }

  if (phase === 'in_progress') {
    if (minutesUntilEnd !== null && minutesUntilEnd > 0) {
      return `进行中，距结束 ${formatMinutesDiff(minutesUntilEnd)}`;
    }
    if (minutesUntilEnd !== null && minutesUntilEnd <= 0) {
      return '已超时';
    }
    return '进行中';
  }

  if (phase === 'completed') {
    if (minutesUntilEnd !== null) {
      return `${formatMinutesDiff(minutesUntilEnd)}前已结束`;
    }
    return '已结束';
  }

  if (minutesUntilEnd !== null) {
    if (minutesUntilEnd > 0) {
      return `${formatMinutesDiff(minutesUntilEnd)}后到期`;
    }
    return `${formatMinutesDiff(minutesUntilEnd)}前到期`;
  }

  return '未设置时间';
};

export const formatTaskTimeRange = (info: TaskTimingInfo, now = new Date()) => {
  return {
    startText: formatDateForDisplay(info.start, now),
    endText: formatDateForDisplay(info.end, now),
  };
};
