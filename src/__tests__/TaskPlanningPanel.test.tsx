import '@testing-library/jest-dom/vitest';
import { describe, expect, it, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { TaskPlanningPanel } from '../components/tasks/TaskPlanningPanel';
import type {
  PlanningSessionView,
  PlanningOptionView,
  PreferenceSnapshot,
} from '../types/planning';
import type { Task } from '../types/task';

const { usePlanningMock, pushToastMock } = vi.hoisted(() => ({
  usePlanningMock: vi.fn(),
  pushToastMock: vi.fn(),
}));

vi.mock('../hooks/usePlanning', () => ({
  usePlanning: usePlanningMock,
}));

vi.mock('../stores/uiStore', () => ({
  pushToast: pushToastMock,
}));

const iso = (hourOffset = 0) => new Date(Date.UTC(2025, 4, 1, 8 + hourOffset, 0, 0)).toISOString();

const preference: PreferenceSnapshot = {
  focusStartMinute: 480,
  focusEndMinute: 660,
  bufferMinutesBetweenBlocks: 30,
  preferCompactSchedule: false,
  avoidanceWindows: [],
};

const sampleOption = (): PlanningOptionView => ({
  option: {
    id: 'option-1',
    sessionId: 'session-1',
    rank: 1,
    score: 92.3,
    summary: '紧凑早晨方案',
    cotSteps: [],
    riskNotes: { notes: ['午餐前完成关键任务'], conflicts: [] },
    isFallback: false,
    createdAt: iso(),
  },
  blocks: [
    {
      id: 'block-1',
      optionId: 'option-1',
      taskId: 'task-1',
      startAt: iso(),
      endAt: iso(2),
      flexibility: 'fixed',
      confidence: 0.9,
      conflictFlags: [],
      appliedAt: undefined,
      actualStartAt: undefined,
      actualEndAt: undefined,
      status: 'draft',
    },
  ],
  conflicts: [
    {
      conflictType: 'calendar-overlap',
      severity: 'medium',
      message: '下午例会冲突',
      relatedBlockId: 'block-1',
      relatedEventId: 'event-1',
    },
  ],
});

const sampleSession = (): PlanningSessionView => ({
  session: {
    id: 'session-1',
    taskIds: ['task-1'],
    constraints: undefined,
    generatedAt: iso(),
    status: 'pending',
    selectedOptionId: 'option-1',
    personalizationSnapshot: preference,
    createdAt: iso(),
    updatedAt: iso(),
  },
  options: [sampleOption()],
  conflicts: [
    {
      conflictType: 'calendar-overlap',
      severity: 'medium',
      message: '下午例会冲突',
      relatedBlockId: 'block-1',
      relatedEventId: 'event-1',
    },
  ],
  preferenceSnapshot: preference,
});

interface PlanningHookState {
  session: PlanningSessionView | null;
  selectedOption: PlanningOptionView | null;
  isGenerating: boolean;
  isApplying: boolean;
  isResolving: boolean;
  isPreferencesLoading: boolean;
  isPreferencesSaving: boolean;
  hasEventBridge: boolean;
  error: unknown;
  activePreferenceId: string;
  preferences: Record<string, PreferenceSnapshot>;
  currentPreference: PreferenceSnapshot | undefined;
  generatePlan: ReturnType<typeof vi.fn>;
  applyOption: ReturnType<typeof vi.fn>;
  resolveConflicts: ReturnType<typeof vi.fn>;
  loadPreferences: ReturnType<typeof vi.fn>;
  updatePreferences: ReturnType<typeof vi.fn>;
  setActivePreferenceId: ReturnType<typeof vi.fn>;
  clearError: ReturnType<typeof vi.fn>;
}

const defaultPlanningState = (): PlanningHookState => ({
  session: null,
  selectedOption: null,
  isGenerating: false,
  isApplying: false,
  isResolving: false,
  isPreferencesLoading: false,
  isPreferencesSaving: false,
  hasEventBridge: true,
  error: null,
  activePreferenceId: 'default',
  preferences: { default: preference },
  currentPreference: preference,
  generatePlan: vi.fn().mockResolvedValue(sampleSession()),
  applyOption: vi.fn().mockResolvedValue({
    option: sampleOption(),
    session: sampleSession().session,
    conflicts: [],
  }),
  resolveConflicts: vi.fn().mockResolvedValue(sampleSession()),
  loadPreferences: vi.fn().mockResolvedValue(preference),
  updatePreferences: vi.fn().mockResolvedValue(undefined),
  setActivePreferenceId: vi.fn(),
  clearError: vi.fn(),
});

const tasks: Task[] = [
  {
    id: 'task-1',
    title: '完成周报',
    description: '整理关键进展',
    status: 'todo',
    priority: 'high',
    plannedStartAt: iso(),
    startAt: undefined,
    dueAt: iso(6),
    completedAt: undefined,
    estimatedMinutes: 120,
    estimatedHours: 2,
    tags: ['report'],
    ownerId: undefined,
    taskType: 'work',
    isRecurring: false,
    recurrence: undefined,
    ai: undefined,
    externalLinks: [],
    createdAt: iso(-2),
    updatedAt: iso(-1),
  },
];

describe('TaskPlanningPanel', () => {
  beforeEach(() => {
    pushToastMock.mockReset();
    usePlanningMock.mockReset();
  });

  it('renders empty guidance when no session exists', () => {
    usePlanningMock.mockReturnValue(defaultPlanningState());

    render(
      <TaskPlanningPanel
        tasks={tasks}
        selectedTaskIds={[]}
        selectedTaskId={undefined}
        onSelectionChange={vi.fn()}
      />,
    );

    expect(screen.getByText('智能规划中心')).toBeInTheDocument();
    expect(screen.getByText('尚未选择任务，请从列表中添加。')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: '生成方案' })).toBeDisabled();
  });

  it('triggers generate and apply flows for existing session', async () => {
    const planningState = defaultPlanningState();
    const generateSpy = planningState.generatePlan as ReturnType<typeof vi.fn>;
    const applySpy = planningState.applyOption as ReturnType<typeof vi.fn>;
    planningState.session = sampleSession();

    usePlanningMock.mockReturnValue(planningState);

    const onPlanApplied = vi.fn();
    const onSelectionChange = vi.fn();
    const user = userEvent.setup();

    render(
      <TaskPlanningPanel
        tasks={tasks}
        selectedTaskIds={['task-1']}
        selectedTaskId={undefined}
        onSelectionChange={onSelectionChange}
        onPlanApplied={onPlanApplied}
      />,
    );

    expect(screen.getByText('冲突 1')).toBeInTheDocument();

    const generateButton = screen.getByRole('button', { name: '生成方案' });
    await user.click(generateButton);
    expect(generateSpy).toHaveBeenCalledWith({ taskIds: ['task-1'], preferenceId: 'default' });

    const applyButton = screen.getByRole('button', { name: '应用方案' });
    await user.click(applyButton);
    expect(applySpy).toHaveBeenCalledWith({
      sessionId: 'session-1',
      optionId: 'option-1',
      overrides: [],
    });

    expect(onPlanApplied).toHaveBeenCalledTimes(1);
  });

  it('displays offline fallback banner when options have fallback flag', () => {
    const planningState = defaultPlanningState();
    const sessionWithFallback = sampleSession();
    sessionWithFallback.options[0].option.isFallback = true;
    planningState.session = sessionWithFallback;

    usePlanningMock.mockReturnValue(planningState);

    render(
      <TaskPlanningPanel
        tasks={tasks}
        selectedTaskIds={['task-1']}
        selectedTaskId={undefined}
        onSelectionChange={vi.fn()}
      />,
    );

    expect(screen.getByText('离线回退模式')).toBeInTheDocument();
    expect(
      screen.getByText(/DeepSeek API 当前不可用，系统已使用最近的缓存建议/),
    ).toBeInTheDocument();
  });

  it('displays conflict resolution guidance when conflicts exist', () => {
    const planningState = defaultPlanningState();
    planningState.session = sampleSession();

    usePlanningMock.mockReturnValue(planningState);

    render(
      <TaskPlanningPanel
        tasks={tasks}
        selectedTaskIds={['task-1']}
        selectedTaskId={undefined}
        onSelectionChange={vi.fn()}
      />,
    );

    expect(screen.getByText('检测到时间冲突')).toBeInTheDocument();
    expect(screen.getByText(/调整时间/)).toBeInTheDocument();
    expect(screen.getByText(/拆分任务/)).toBeInTheDocument();
    expect(screen.getByText(/替换任务/)).toBeInTheDocument();
  });

  it('triggers reject action when user rejects an option', async () => {
    const planningState = defaultPlanningState();
    planningState.session = sampleSession();

    usePlanningMock.mockReturnValue(planningState);

    const user = userEvent.setup();
    const consoleLogSpy = vi.spyOn(console, 'log').mockImplementation(() => undefined);

    render(
      <TaskPlanningPanel
        tasks={tasks}
        selectedTaskIds={['task-1']}
        selectedTaskId={undefined}
        onSelectionChange={vi.fn()}
      />,
    );

    const rejectButton = screen.getByRole('button', { name: '拒绝' });
    await user.click(rejectButton);

    expect(consoleLogSpy).toHaveBeenCalledWith(
      '[TaskPlanningPanel] User rejected plan option',
      expect.objectContaining({
        action: 'rejected',
        optionId: 'option-1',
      }),
    );

    expect(pushToastMock).toHaveBeenCalledWith(
      expect.objectContaining({
        title: '方案已拒绝',
      }),
    );

    consoleLogSpy.mockRestore();
  });

  it('opens conflict resolution sheet when adjust time is clicked', async () => {
    const planningState = defaultPlanningState();
    planningState.session = sampleSession();

    usePlanningMock.mockReturnValue(planningState);

    const user = userEvent.setup();
    const consoleLogSpy = vi.spyOn(console, 'log').mockImplementation(() => undefined);

    render(
      <TaskPlanningPanel
        tasks={tasks}
        selectedTaskIds={['task-1']}
        selectedTaskId={undefined}
        onSelectionChange={vi.fn()}
      />,
    );

    // Open the conflict resolution dropdown
    const resolveButton = screen.getByRole('button', { name: /解决冲突/ });
    await user.click(resolveButton);

    // Click on adjust time option
    const adjustTimeOption = screen.getByRole('menuitem', { name: /调整时间/ });
    await user.click(adjustTimeOption);

    expect(consoleLogSpy).toHaveBeenCalledWith(
      '[TaskPlanningPanel] User requested time adjustment',
      expect.objectContaining({
        action: 'adjusted',
        adjustmentType: 'time',
      }),
    );

    expect(pushToastMock).toHaveBeenCalledWith(
      expect.objectContaining({
        title: '调整时间',
      }),
    );

    consoleLogSpy.mockRestore();
  });
});
