import { useEffect, useMemo, useState } from 'react';
import {
  AlertCircle,
  AlertTriangle,
  Info,
  Loader2,
  RefreshCw,
  Settings2,
  Sparkles,
  WifiOff,
} from 'lucide-react';
import { Button } from '../ui/button';
import { Badge } from '../ui/badge';
import { usePlanning } from '../../hooks/usePlanning';
import { PlanOptionCard } from './PlanOptionCard';
import { ConflictResolutionSheet } from './ConflictResolutionSheet';
import { PersonalizationDialog } from './PersonalizationDialog';
import { pushToast } from '../../stores/uiStore';
import type { Task } from '../../types/task';
import {
  type PlanningOptionView,
  type PlanningPreferencesUpdateInput,
  type PlanningSessionView,
  type PreferenceSnapshot,
  type TimeBlockOverride,
} from '../../types/planning';
import { cn } from '../../lib/utils';
import { HelpPopover } from '../help/HelpPopover';

interface TaskPlanningPanelProps {
  tasks: Task[];
  selectedTaskId?: string | null;
  selectedTaskIds: string[];
  onSelectionChange: (taskIds: string[]) => void;
  onPlanApplied?: () => void;
  className?: string;
}

interface PreferencePreset {
  id: string;
  label: string;
  snapshot: PreferenceSnapshot;
}

const preferencePresets: PreferencePreset[] = [
  {
    id: 'default',
    label: '标准偏好',
    snapshot: {
      focusStartMinute: 9 * 60,
      focusEndMinute: 18 * 60,
      bufferMinutesBetweenBlocks: 30,
      preferCompactSchedule: false,
      avoidanceWindows: [],
    },
  },
  {
    id: 'focus_morning',
    label: '晨间深度工作',
    snapshot: {
      focusStartMinute: 8 * 60 + 30,
      focusEndMinute: 11 * 60,
      bufferMinutesBetweenBlocks: 20,
      preferCompactSchedule: false,
      avoidanceWindows: [],
    },
  },
  {
    id: 'compact',
    label: '紧凑模式',
    snapshot: {
      focusStartMinute: 9 * 60,
      focusEndMinute: 18 * 60,
      bufferMinutesBetweenBlocks: 5,
      preferCompactSchedule: true,
      avoidanceWindows: [],
    },
  },
];

export function TaskPlanningPanel({
  tasks,
  selectedTaskId,
  selectedTaskIds,
  onSelectionChange,
  onPlanApplied,
  className,
}: TaskPlanningPanelProps) {
  const [pendingTaskId, setPendingTaskId] = useState('');
  const [isPersonalizationOpen, setPersonalizationOpen] = useState(false);
  const [isConflictSheetOpen, setConflictSheetOpen] = useState(false);
  const [activeOptionId, setActiveOptionId] = useState<string | null>(null);

  const {
    session,
    isGenerating,
    isApplying,
    isResolving,
    isPreferencesLoading,
    isPreferencesSaving,
    hasEventBridge,
    activePreferenceId,
    preferences,
    currentPreference,
    generatePlan,
    applyOption,
    resolveConflicts,
    loadPreferences,
    updatePreferences,
    setActivePreferenceId,
  } = usePlanning({ autoAttachEvents: true, autoLoadPreferences: true });

  useEffect(() => {
    if (!session) {
      setActiveOptionId(null);
      return;
    }
    const nextSelected = session.session.selectedOptionId ?? session.options[0]?.option.id ?? null;
    setActiveOptionId(nextSelected);
  }, [session]);

  useEffect(() => {
    if (!selectedTaskId) return;
    if (selectedTaskIds.includes(selectedTaskId)) return;
    onSelectionChange([...selectedTaskIds, selectedTaskId]);
  }, [selectedTaskId, selectedTaskIds, onSelectionChange]);

  const selectableTasks = useMemo(
    () => tasks.filter((task) => !selectedTaskIds.includes(task.id)),
    [tasks, selectedTaskIds],
  );

  const selectedTasks = useMemo(
    () => tasks.filter((task) => selectedTaskIds.includes(task.id)),
    [tasks, selectedTaskIds],
  );

  const taskTitleMap = useMemo(() => {
    const entries: Record<string, string> = {};
    for (const task of tasks) {
      entries[task.id] = task.title;
    }
    return entries;
  }, [tasks]);

  const selectionMatchesSession = useMemo(() => {
    if (!session) return true;
    const sessionTaskIds = session.session.taskIds;
    if (sessionTaskIds.length !== selectedTaskIds.length) return false;
    const sessionSet = new Set(sessionTaskIds);
    return selectedTaskIds.every((id) => sessionSet.has(id));
  }, [session, selectedTaskIds]);

  const conflictCount = session?.conflicts.length ?? 0;

  const summary = buildSessionSummary(session);

  const handleAddTask = () => {
    if (!pendingTaskId) {
      pushToast({ title: '请选择需要加入规划的任务', variant: 'warning' });
      return;
    }
    if (selectedTaskIds.includes(pendingTaskId)) {
      pushToast({ title: '该任务已在规划列表中', variant: 'warning' });
      return;
    }
    onSelectionChange([...selectedTaskIds, pendingTaskId]);
    setPendingTaskId('');
  };

  const handleRemoveTask = (taskId: string) => {
    onSelectionChange(selectedTaskIds.filter((id) => id !== taskId));
  };

  const handleGeneratePlan = async () => {
    if (!selectedTaskIds.length) {
      pushToast({ title: '请至少选择一个任务', variant: 'warning' });
      return;
    }
    const taskIds = selectedTaskIds as [string, ...string[]];
    await generatePlan({ taskIds, preferenceId: activePreferenceId });
  };

  const handleApplyOption = async (option: PlanningOptionView) => {
    if (!session) return;

    // 记录用户接受决策
    console.log('[TaskPlanningPanel] User accepted plan option', {
      sessionId: session.session.id,
      optionId: option.option.id,
      action: 'accepted',
      timestamp: new Date().toISOString(),
    });

    await applyOption({ sessionId: session.session.id, optionId: option.option.id, overrides: [] });
    onPlanApplied?.();
  };

  const handleRejectOption = async (option: PlanningOptionView) => {
    if (!session) return;

    // 记录用户拒绝决策
    console.log('[TaskPlanningPanel] User rejected plan option', {
      sessionId: session.session.id,
      optionId: option.option.id,
      action: 'rejected',
      timestamp: new Date().toISOString(),
    });

    pushToast({
      title: '方案已拒绝',
      description: '您可以选择其他方案或重新生成',
      variant: 'default',
    });
  };

  const handleAdjustTime = async (option: PlanningOptionView) => {
    if (!session) return;

    // 记录用户调整时间操作
    console.log('[TaskPlanningPanel] User requested time adjustment', {
      sessionId: session.session.id,
      optionId: option.option.id,
      action: 'adjusted',
      adjustmentType: 'time',
      timestamp: new Date().toISOString(),
    });

    setActiveOptionId(option.option.id);
    setConflictSheetOpen(true);
    pushToast({
      title: '调整时间',
      description: '请在冲突解决面板中调整任务的时间段',
      variant: 'default',
    });
  };

  const handleSplitTask = async (option: PlanningOptionView) => {
    if (!session) return;

    // 记录用户拆分任务操作
    console.log('[TaskPlanningPanel] User requested task split', {
      sessionId: session.session.id,
      optionId: option.option.id,
      action: 'adjusted',
      adjustmentType: 'split',
      timestamp: new Date().toISOString(),
    });

    pushToast({
      title: '拆分任务',
      description: '任务拆分功能即将推出，敬请期待',
      variant: 'default',
    });
  };

  const handleReplaceTask = async (option: PlanningOptionView) => {
    if (!session) return;

    // 记录用户替换任务操作
    console.log('[TaskPlanningPanel] User requested task replacement', {
      sessionId: session.session.id,
      optionId: option.option.id,
      action: 'adjusted',
      adjustmentType: 'replace',
      timestamp: new Date().toISOString(),
    });

    pushToast({
      title: '替换任务',
      description: '任务替换功能即将推出，敬请期待',
      variant: 'default',
    });
  };

  const handleResolve = async (overrides: TimeBlockOverride[]) => {
    if (!session || !activeOptionId) return;
    await resolveConflicts({
      sessionId: session.session.id,
      optionId: activeOptionId,
      adjustments: overrides,
    });
  };

  const handleLoadPreferences = async () => {
    const targetId = activePreferenceId || 'default';
    try {
      await loadPreferences(targetId, { force: true });
      pushToast({ title: '偏好已刷新', variant: 'success' });
    } catch (error) {
      const description = error instanceof Error ? error.message : '请稍后重试';
      pushToast({ title: '刷新偏好失败', description, variant: 'error' });
    }
  };

  const handlePreferenceSave = async (payload: PlanningPreferencesUpdateInput) => {
    await updatePreferences(payload);
  };

  const presetPreferenceOptions = useMemo(() => {
    const existing = new Set(Object.keys(preferences));
    return preferencePresets.map((item) => ({
      ...item,
      exists: existing.has(item.id),
    }));
  }, [preferences]);

  const isBusy = isGenerating || isApplying || isResolving;
  const isPreferencesBusy = isPreferencesLoading || isPreferencesSaving;

  const handlePresetSelect = async (preset: (typeof presetPreferenceOptions)[number]) => {
    if (isPreferencesBusy) {
      return;
    }

    setActivePreferenceId(preset.id);

    try {
      await updatePreferences({ preferenceId: preset.id, snapshot: preset.snapshot });
      setPersonalizationOpen(true);
    } catch (error) {
      const description = error instanceof Error ? error.message : '请稍后重试';
      pushToast({ title: `应用${preset.label}失败`, description, variant: 'error' });
    }
  };

  return (
    <section
      className={cn(
        'flex flex-col gap-6 rounded-3xl border border-border/70 bg-card/80 p-6 shadow-sm',
        className,
      )}
      data-onboarding="planning-center"
    >
      <header className="flex flex-col gap-2">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <Sparkles className="h-5 w-5 text-primary" />
            <h2 className="text-lg font-semibold">智能规划中心</h2>
            <HelpPopover
              entryId="planning-center"
              triggerLabel="了解智能规划中心提示"
              triggerClassName="ml-1"
            />
          </div>
          <div className="flex items-center gap-2 text-xs text-muted-foreground">
            <Badge variant={hasEventBridge ? 'secondary' : 'outline'}>
              {hasEventBridge ? '事件同步已开启' : '事件桥未激活'}
            </Badge>
            {session ? (
              <Badge variant="secondary" className="bg-primary/10 text-primary">
                会话 #{session.session.id.slice(0, 8)}
              </Badge>
            ) : null}
          </div>
        </div>
        <p className="text-sm text-muted-foreground">
          选择需要排程的任务，生成多种时间方案，并按需调整冲突或个性化偏好。
        </p>
      </header>

      <section className="space-y-4 rounded-2xl border border-border/60 bg-background/90 p-4">
        <div className="flex flex-wrap items-end gap-3">
          <div className="flex flex-col gap-2">
            <label
              htmlFor="planning-task-select"
              className="text-xs font-semibold uppercase tracking-wide text-muted-foreground"
            >
              加入任务
            </label>
            <div className="flex items-center gap-2">
              <select
                id="planning-task-select"
                className="h-9 min-w-[200px] rounded-md border border-border/60 bg-background px-3 text-sm"
                value={pendingTaskId}
                onChange={(event) => setPendingTaskId(event.target.value)}
              >
                <option value="">选择任务</option>
                {selectableTasks.map((task) => (
                  <option key={task.id} value={task.id}>
                    {task.title}
                  </option>
                ))}
              </select>
              <Button type="button" variant="outline" size="sm" onClick={handleAddTask}>
                添加
              </Button>
            </div>
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <Button
              type="button"
              size="sm"
              onClick={handleGeneratePlan}
              disabled={isGenerating || !selectedTaskIds.length}
            >
              {isGenerating ? (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              ) : (
                <Sparkles className="mr-2 h-4 w-4" />
              )}
              生成方案
            </Button>
            <Button
              type="button"
              size="sm"
              variant="outline"
              onClick={() => setPersonalizationOpen(true)}
              disabled={isPreferencesSaving}
            >
              <Settings2 className="mr-2 h-4 w-4" /> 个性化偏好
            </Button>
            <Button
              type="button"
              size="sm"
              variant="ghost"
              onClick={handleLoadPreferences}
              disabled={isPreferencesLoading}
            >
              <RefreshCw className="mr-2 h-4 w-4" /> 刷新偏好
            </Button>
          </div>
        </div>

        <div className="flex flex-wrap items-center gap-2">
          {selectedTasks.length ? (
            selectedTasks.map((task) => (
              <Badge
                key={task.id}
                variant="secondary"
                className="flex items-center gap-1 rounded-full bg-muted/80 pr-1 text-xs"
              >
                <span>{task.title}</span>
                <button
                  type="button"
                  className="ml-1 rounded-full px-1 text-muted-foreground transition hover:bg-muted"
                  aria-label={`移除任务 ${task.title}`}
                  onClick={() => handleRemoveTask(task.id)}
                >
                  ×
                </button>
              </Badge>
            ))
          ) : (
            <span className="text-xs text-muted-foreground">尚未选择任务，请从列表中添加。</span>
          )}
        </div>

        {!selectionMatchesSession && session ? (
          <div className="flex items-center gap-2 rounded-2xl border border-amber-500/40 bg-amber-500/10 p-3 text-xs text-amber-700">
            <AlertTriangle className="h-4 w-4" />
            <span>当前选择与最新生成的方案不一致，建议重新生成以匹配最新任务列表。</span>
          </div>
        ) : null}
      </section>

      {session ? (
        <section className="space-y-4 rounded-2xl border border-border/60 bg-background/80 p-4">
          {session.options.some((opt) => opt.option.isFallback) ? (
            <div className="flex items-center gap-3 rounded-2xl border border-blue-500/40 bg-blue-500/10 p-3 text-sm text-blue-700 dark:text-blue-400">
              <WifiOff className="h-5 w-5 shrink-0" />
              <div className="flex flex-col gap-1">
                <span className="font-semibold">离线回退模式</span>
                <span className="text-xs">
                  DeepSeek API
                  当前不可用，系统已使用最近的缓存建议或启发式算法生成方案。在线后可重新生成以获得更优质的规划。
                </span>
              </div>
            </div>
          ) : null}

          {conflictCount > 0 ? (
            <div className="flex items-center gap-3 rounded-2xl border border-amber-500/40 bg-amber-500/5 p-3 text-sm">
              <Info className="h-5 w-5 shrink-0 text-amber-600" />
              <div className="flex flex-col gap-1.5 text-amber-700 dark:text-amber-400">
                <span className="font-semibold">检测到时间冲突</span>
                <span className="text-xs">您可以通过以下方式解决冲突：</span>
                <ul className="ml-4 list-disc space-y-0.5 text-xs">
                  <li>
                    <strong>调整时间</strong>：修改任务的计划时间段以避开冲突
                  </li>
                  <li>
                    <strong>拆分任务</strong>：将大任务拆分为多个小任务分散处理
                  </li>
                  <li>
                    <strong>替换任务</strong>：用其他优先级较低的任务替换当前任务
                  </li>
                </ul>
              </div>
            </div>
          ) : null}

          <div className="flex flex-wrap items-center justify-between gap-3">
            <div className="flex flex-col gap-1">
              <span className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
                最新规划
              </span>
              <div className="flex flex-wrap items-center gap-2 text-sm text-muted-foreground/90">
                <span>{summary.title}</span>
                <Badge variant="outline" className="text-xs">
                  生成时间 {summary.generatedAt}
                </Badge>
                <Badge variant="outline" className="text-xs">
                  任务 {session.session.taskIds.length} 项
                </Badge>
              </div>
            </div>
            <div className="flex items-center gap-2 text-xs text-muted-foreground">
              <Badge
                variant={conflictCount ? 'destructive' : 'secondary'}
                className="flex items-center gap-1"
              >
                <AlertTriangle className="h-3.5 w-3.5" /> 冲突 {conflictCount}
              </Badge>
              <Button
                type="button"
                size="sm"
                variant="outline"
                onClick={() => setConflictSheetOpen(true)}
                disabled={!conflictCount}
              >
                查看冲突
              </Button>
            </div>
          </div>

          <div className="grid gap-3 rounded-2xl border border-dashed border-border/60 bg-card/60 p-4 text-xs text-muted-foreground">
            <span className="font-medium text-foreground">偏好概览</span>
            {currentPreference ? (
              <ul className="grid gap-1 text-xs text-muted-foreground">
                <li>
                  深度工作：
                  {renderMinuteRange(
                    currentPreference.focusStartMinute,
                    currentPreference.focusEndMinute,
                  )}
                </li>
                <li>缓冲时间：{currentPreference.bufferMinutesBetweenBlocks} 分钟</li>
                <li>紧凑模式：{currentPreference.preferCompactSchedule ? '启用' : '关闭'}</li>
                <li>
                  避免时段：
                  {currentPreference.avoidanceWindows.length
                    ? currentPreference.avoidanceWindows
                        .map(
                          (window) =>
                            `${weekdayLabel(window.weekday)} ${minuteToTime(window.startMinute)}-${minuteToTime(window.endMinute)}`,
                        )
                        .join('、')
                    : '未配置'}
                </li>
              </ul>
            ) : (
              <span className="text-xs text-muted-foreground">尚未加载偏好，将使用默认设定。</span>
            )}
            <div className="flex flex-wrap gap-2 text-xs">
              {presetPreferenceOptions.map((option) => (
                <Button
                  key={option.id}
                  type="button"
                  size="sm"
                  variant={activePreferenceId === option.id ? 'default' : 'outline'}
                  onClick={() => handlePresetSelect(option)}
                  disabled={isPreferencesBusy && activePreferenceId !== option.id}
                >
                  {option.label}
                  {option.exists ? null : (
                    <span className="ml-1 text-[10px] text-muted-foreground">(新建)</span>
                  )}
                </Button>
              ))}
            </div>
          </div>

          <div className="grid gap-4 sm:grid-cols-2">
            {session.options.map((option) => (
              <PlanOptionCard
                key={option.option.id}
                option={option}
                isSelected={option.option.id === activeOptionId}
                disabled={isBusy}
                taskTitles={taskTitleMap}
                onSelect={setActiveOptionId}
                onApply={handleApplyOption}
                onShowConflicts={() => {
                  setActiveOptionId(option.option.id);
                  setConflictSheetOpen(true);
                }}
                onAdjustTime={handleAdjustTime}
                onSplitTask={handleSplitTask}
                onReplaceTask={handleReplaceTask}
                onReject={handleRejectOption}
              />
            ))}
          </div>

          {!session.options.length ? (
            <div className="flex items-center gap-2 rounded-2xl border border-dashed border-border/60 bg-muted/70 p-4 text-sm text-muted-foreground">
              <AlertCircle className="h-4 w-4" />
              暂无可用方案，尝试重新生成或调整任务选择。
            </div>
          ) : null}
        </section>
      ) : (
        <div className="flex items-center gap-3 rounded-2xl border border-dashed border-border/60 bg-muted/70 p-6 text-sm text-muted-foreground">
          <AlertCircle className="h-5 w-5" />
          还未生成规划方案。选择任务后点击“生成方案”开始智能排程。
        </div>
      )}

      <ConflictResolutionSheet
        open={isConflictSheetOpen}
        onOpenChange={setConflictSheetOpen}
        session={session}
        optionId={activeOptionId}
        onResolve={handleResolve}
        isResolving={isResolving}
      />

      <PersonalizationDialog
        open={isPersonalizationOpen}
        onOpenChange={setPersonalizationOpen}
        snapshot={currentPreference}
        preferenceId={activePreferenceId}
        isSaving={isPreferencesSaving}
        onSave={handlePreferenceSave}
      />
    </section>
  );
}

function buildSessionSummary(session: PlanningSessionView | null) {
  if (!session) {
    return {
      title: '尚未生成规划',
      generatedAt: '—',
    };
  }

  const generatedAt = new Date(session.session.generatedAt);
  return {
    title: `共 ${session.options.length} 个方案`,
    generatedAt: new Intl.DateTimeFormat('zh-CN', {
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    }).format(generatedAt),
  };
}

function renderMinuteRange(start?: number, end?: number) {
  if (start === undefined && end === undefined) return '未设定';
  const startLabel = start !== undefined ? minuteToTime(start) : '自由';
  const endLabel = end !== undefined ? minuteToTime(end) : '自由';
  return `${startLabel} - ${endLabel}`;
}

function minuteToTime(minute: number) {
  const hours = Math.floor(minute / 60)
    .toString()
    .padStart(2, '0');
  const minutes = (minute % 60).toString().padStart(2, '0');
  return `${hours}:${minutes}`;
}

function weekdayLabel(value: number) {
  const names = ['周一', '周二', '周三', '周四', '周五', '周六', '周日'];
  return names[value] ?? `周${value + 1}`;
}
