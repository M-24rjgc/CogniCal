import { useEffect, useMemo, useState } from 'react';
import { Plus, Trash2 } from 'lucide-react';
import { Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle } from '../ui/dialog';
import { Button } from '../ui/button';
import { Input } from '../ui/input';
import { Label } from '../ui/label';
import {
  type AvoidanceWindow,
  type PlanningPreferencesUpdateInput,
  type PreferenceSnapshot,
} from '../../types/planning';
import { pushToast } from '../../stores/uiStore';

interface PersonalizationDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  snapshot: PreferenceSnapshot | undefined;
  preferenceId: string;
  isSaving: boolean;
  onSave: (payload: PlanningPreferencesUpdateInput) => Promise<void>;
}

interface WindowFormRow {
  id: string;
  weekday: string;
  startMinute: string;
  endMinute: string;
}

const weekdayOptions = [
  { value: '0', label: '周一' },
  { value: '1', label: '周二' },
  { value: '2', label: '周三' },
  { value: '3', label: '周四' },
  { value: '4', label: '周五' },
  { value: '5', label: '周六' },
  { value: '6', label: '周日' },
];

const defaultSnapshot: PreferenceSnapshot = {
  focusStartMinute: 540, // 9:00 AM
  focusEndMinute: 1020, // 5:00 PM
  bufferMinutesBetweenBlocks: 30,
  preferCompactSchedule: false,
  avoidanceWindows: [],
};

const createRowId = () =>
  typeof crypto !== 'undefined' && 'randomUUID' in crypto
    ? crypto.randomUUID()
    : `avoid-${Math.random().toString(36).slice(2, 10)}`;

const newRow = (): WindowFormRow => ({
  id: createRowId(),
  weekday: '0',
  startMinute: '540',
  endMinute: '720',
});

export function PersonalizationDialog({
  open,
  onOpenChange,
  snapshot,
  preferenceId,
  isSaving,
  onSave,
}: PersonalizationDialogProps) {
  const [focusStart, setFocusStart] = useState('');
  const [focusEnd, setFocusEnd] = useState('');
  const [bufferMinutes, setBufferMinutes] = useState('30');
  const [preferCompact, setPreferCompact] = useState(false);
  const [windows, setWindows] = useState<WindowFormRow[]>([]);

  const mergedSnapshot = useMemo(() => ({ ...defaultSnapshot, ...(snapshot ?? {}) }), [snapshot]);

  useEffect(() => {
    if (!open) return;
    setFocusStart(
      mergedSnapshot.focusStartMinute !== undefined ? String(mergedSnapshot.focusStartMinute) : '',
    );
    setFocusEnd(
      mergedSnapshot.focusEndMinute !== undefined ? String(mergedSnapshot.focusEndMinute) : '',
    );
    setBufferMinutes(String(mergedSnapshot.bufferMinutesBetweenBlocks));
    setPreferCompact(Boolean(mergedSnapshot.preferCompactSchedule));
    setWindows(
      mergedSnapshot.avoidanceWindows.length
        ? mergedSnapshot.avoidanceWindows.map((window) => ({
            id: createRowId(),
            weekday: String(window.weekday),
            startMinute: String(window.startMinute),
            endMinute: String(window.endMinute),
          }))
        : [newRow()],
    );
  }, [open, mergedSnapshot]);

  const handleAddWindow = () => {
    setWindows((prev) => [...prev, newRow()]);
  };

  const handleRemoveWindow = (id: string) => {
    setWindows((prev) => (prev.length > 1 ? prev.filter((row) => row.id !== id) : prev));
  };

  const handleSave = async () => {
    const parsed = parseSnapshot({
      focusStart,
      focusEnd,
      bufferMinutes,
      preferCompact,
      windows,
    });

    if ('error' in parsed) {
      pushToast({ title: '偏好设置有误', description: parsed.error, variant: 'error' });
      return;
    }

    await onSave({ preferenceId, snapshot: parsed });
    onOpenChange(false);
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-h-[90vh] overflow-y-auto sm:max-w-xl">
        <DialogHeader>
          <DialogTitle>规划偏好设置</DialogTitle>
        </DialogHeader>

        <section className="space-y-4 text-sm">
          <div className="grid gap-2">
            <Label htmlFor="focus-start">深度工作开始分钟（0-1440）</Label>
            <Input
              id="focus-start"
              inputMode="numeric"
              value={focusStart}
              onChange={(event) => setFocusStart(event.target.value)}
              placeholder="如 540 表示 9:00"
            />
          </div>
          <div className="grid gap-2">
            <Label htmlFor="focus-end">深度工作结束分钟（0-1440）</Label>
            <Input
              id="focus-end"
              inputMode="numeric"
              value={focusEnd}
              onChange={(event) => setFocusEnd(event.target.value)}
              placeholder="如 1020 表示 17:00"
            />
          </div>
          <div className="grid gap-2">
            <Label htmlFor="buffer-minutes">任务间缓冲分钟数</Label>
            <Input
              id="buffer-minutes"
              inputMode="numeric"
              value={bufferMinutes}
              onChange={(event) => setBufferMinutes(event.target.value)}
              placeholder="默认 30"
            />
          </div>
          <div className="flex items-center justify-between rounded-2xl border border-border/70 bg-muted/40 p-3">
            <div className="flex flex-col">
              <span className="text-sm font-medium">更紧凑的时间安排</span>
              <span className="text-xs text-muted-foreground">
                勾选后 AI 将更倾向于连续安排任务，减少空档。
              </span>
            </div>
            <input
              aria-label="偏好紧凑安排"
              type="checkbox"
              checked={preferCompact}
              onChange={(event) => setPreferCompact(event.target.checked)}
              className="h-4 w-4 rounded border border-border/70 accent-primary"
            />
          </div>

          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <Label>避免时间窗口</Label>
              <Button type="button" variant="outline" size="sm" onClick={handleAddWindow}>
                <Plus className="mr-1.5 h-3.5 w-3.5" /> 添加时间段
              </Button>
            </div>
            <div className="grid gap-3">
              {windows.map((row) => (
                <div
                  key={row.id}
                  className="grid gap-3 rounded-2xl border border-border/60 bg-background/80 p-3 sm:grid-cols-5 sm:items-end"
                >
                  <div className="grid gap-1 text-xs">
                    <Label htmlFor={`weekday-${row.id}`}>星期</Label>
                    <select
                      id={`weekday-${row.id}`}
                      className="h-9 rounded-md border border-border/60 bg-background px-2 text-sm"
                      value={row.weekday}
                      onChange={(event) =>
                        setWindows((prev) =>
                          prev.map((item) =>
                            item.id === row.id ? { ...item, weekday: event.target.value } : item,
                          ),
                        )
                      }
                    >
                      {weekdayOptions.map((option) => (
                        <option key={option.value} value={option.value}>
                          {option.label}
                        </option>
                      ))}
                    </select>
                  </div>
                  <div className="grid gap-1 text-xs">
                    <Label htmlFor={`start-${row.id}`}>开始分钟</Label>
                    <Input
                      id={`start-${row.id}`}
                      inputMode="numeric"
                      value={row.startMinute}
                      onChange={(event) =>
                        setWindows((prev) =>
                          prev.map((item) =>
                            item.id === row.id
                              ? { ...item, startMinute: event.target.value }
                              : item,
                          ),
                        )
                      }
                    />
                  </div>
                  <div className="grid gap-1 text-xs">
                    <Label htmlFor={`end-${row.id}`}>结束分钟</Label>
                    <Input
                      id={`end-${row.id}`}
                      inputMode="numeric"
                      value={row.endMinute}
                      onChange={(event) =>
                        setWindows((prev) =>
                          prev.map((item) =>
                            item.id === row.id ? { ...item, endMinute: event.target.value } : item,
                          ),
                        )
                      }
                    />
                  </div>
                  <div className="flex items-center justify-end sm:col-span-2">
                    <Button
                      type="button"
                      variant="ghost"
                      size="sm"
                      onClick={() => handleRemoveWindow(row.id)}
                      disabled={windows.length <= 1}
                    >
                      <Trash2 className="mr-1.5 h-3.5 w-3.5" /> 删除
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </section>

        <DialogFooter className="flex flex-col gap-3 sm:flex-row sm:justify-end">
          <Button
            type="button"
            variant="secondary"
            onClick={() => onOpenChange(false)}
            disabled={isSaving}
          >
            取消
          </Button>
          <Button type="button" onClick={handleSave} disabled={isSaving}>
            保存偏好
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function parseSnapshot({
  focusStart,
  focusEnd,
  bufferMinutes,
  preferCompact,
  windows,
}: {
  focusStart: string;
  focusEnd: string;
  bufferMinutes: string;
  preferCompact: boolean;
  windows: WindowFormRow[];
}): PreferenceSnapshot | { error: string } {
  const convertMinute = (value: string, label: string): number | undefined | { error: string } => {
    if (!value.trim()) return undefined;
    const parsed = Number.parseInt(value, 10);
    if (Number.isNaN(parsed) || parsed < 0 || parsed > 24 * 60) {
      return { error: `${label} 需在 0-1440 范围内` };
    }
    return parsed;
  };

  const parsedFocusStart = convertMinute(focusStart, '开始分钟');
  if (typeof parsedFocusStart === 'object') return parsedFocusStart;
  const parsedFocusEnd = convertMinute(focusEnd, '结束分钟');
  if (typeof parsedFocusEnd === 'object') return parsedFocusEnd;

  const parsedBuffer = Number.parseInt(bufferMinutes, 10);
  if (Number.isNaN(parsedBuffer) || parsedBuffer < 0 || parsedBuffer > 24 * 60) {
    return { error: '缓冲分钟数需在 0-1440 范围内' };
  }

  const avoidanceWindows: AvoidanceWindow[] = [];
  for (const window of windows) {
    const weekday = Number.parseInt(window.weekday, 10);
    const startMinute = Number.parseInt(window.startMinute, 10);
    const endMinute = Number.parseInt(window.endMinute, 10);

    if (
      Number.isNaN(weekday) ||
      weekday < 0 ||
      weekday > 6 ||
      Number.isNaN(startMinute) ||
      Number.isNaN(endMinute) ||
      startMinute < 0 ||
      endMinute < 0 ||
      startMinute >= endMinute ||
      startMinute > 24 * 60 ||
      endMinute > 24 * 60
    ) {
      return { error: '请检查避免时间窗口的星期与时间范围设置' };
    }

    avoidanceWindows.push({ weekday, startMinute, endMinute });
  }

  return {
    focusStartMinute: parsedFocusStart,
    focusEndMinute: parsedFocusEnd,
    bufferMinutesBetweenBlocks: parsedBuffer,
    preferCompactSchedule: preferCompact,
    avoidanceWindows,
  } satisfies PreferenceSnapshot;
}
