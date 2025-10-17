import { beforeAll, beforeEach, afterAll, describe, expect, it, vi } from 'vitest';
import {
  PLANNING_EVENT_NAMES,
  type AppliedPlan,
  type PlanningSessionView,
  type PreferenceSnapshot,
} from '../types/planning';
import { usePlanningStore } from '../stores/planningStore';

type EventHandler = (event: { payload: unknown }) => void;

const { mocks } = vi.hoisted(() => {
  const eventHandlers = new Map<string, EventHandler>();
  const generatePlanningSessionMock = vi.fn();
  const applyPlanningOptionMock = vi.fn();
  const resolvePlanningConflictsMock = vi.fn();
  const getPlanningPreferencesMock = vi.fn();
  const updatePlanningPreferencesMock = vi.fn();

  return {
    mocks: {
      eventHandlers,
      generatePlanningSessionMock,
      applyPlanningOptionMock,
      resolvePlanningConflictsMock,
      getPlanningPreferencesMock,
      updatePlanningPreferencesMock,
    },
  };
});

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn((eventName: string, handler: EventHandler) => {
    mocks.eventHandlers.set(eventName, handler);
    return Promise.resolve(() => {
      mocks.eventHandlers.delete(eventName);
    });
  }),
}));

vi.mock('../services/tauriApi', () => ({
  generatePlanningSession: mocks.generatePlanningSessionMock,
  applyPlanningOption: mocks.applyPlanningOptionMock,
  resolvePlanningConflicts: mocks.resolvePlanningConflictsMock,
  getPlanningPreferences: mocks.getPlanningPreferencesMock,
  updatePlanningPreferences: mocks.updatePlanningPreferencesMock,
  PLANNING_EVENT_NAMES: {
    GENERATED: 'planning://generated',
    APPLIED: 'planning://applied',
    CONFLICTS_RESOLVED: 'planning://conflicts-resolved',
    PREFERENCES_UPDATED: 'planning://preferences-updated',
  },
  toAppError: (error: unknown, fallback: string) => {
    if (error && typeof error === 'object' && 'code' in (error as Record<string, unknown>)) {
      return error as { code: string; message: string };
    }
    return { code: 'UNKNOWN', message: fallback };
  },
  isAppError: (error: unknown) =>
    Boolean(error && typeof error === 'object' && 'code' in (error as Record<string, unknown>)),
}));

const createPreference = (overrides: Partial<PreferenceSnapshot> = {}): PreferenceSnapshot => ({
  focusStartMinute: 480,
  focusEndMinute: 660,
  bufferMinutesBetweenBlocks: 30,
  preferCompactSchedule: true,
  avoidanceWindows: [],
  ...overrides,
});

const iso = (hourOffset = 0) => new Date(Date.UTC(2025, 4, 1, 9 + hourOffset, 0, 0)).toISOString();

const basePreference = createPreference();

const createSession = (overrides: Partial<PlanningSessionView> = {}): PlanningSessionView => ({
  session: {
    id: 'session-1',
    taskIds: ['task-1'],
    constraints: undefined,
    generatedAt: iso(),
    status: 'pending',
    selectedOptionId: 'option-1',
    personalizationSnapshot: basePreference,
    createdAt: iso(),
    updatedAt: iso(),
    ...overrides.session,
  },
  options: [
    {
      option: {
        id: 'option-1',
        sessionId: 'session-1',
        rank: 1,
        score: 95.25,
        summary: '晨间深度方案',
        cotSteps: [],
        riskNotes: { notes: [], conflicts: [] },
        isFallback: false,
        createdAt: iso(),
        ...overrides.options?.[0]?.option,
      },
      blocks: [
        {
          id: 'block-1',
          optionId: 'option-1',
          taskId: 'task-1',
          startAt: iso(),
          endAt: iso(2),
          flexibility: 'fixed',
          confidence: 0.85,
          conflictFlags: [],
          appliedAt: undefined,
          actualStartAt: undefined,
          actualEndAt: undefined,
          status: 'draft',
          ...overrides.options?.[0]?.blocks?.[0],
        },
      ],
      conflicts: overrides.options?.[0]?.conflicts ?? [],
    },
  ],
  conflicts: overrides.conflicts ?? [],
  preferenceSnapshot: overrides.preferenceSnapshot ?? basePreference,
});

describe('planningStore', () => {
  let originalWindow: typeof window | undefined;

  beforeAll(() => {
    originalWindow = globalThis.window;
    globalThis.window = {
      __TAURI_INTERNALS__: {},
    } as unknown as typeof window;
  });

  afterAll(() => {
    if (originalWindow) {
      globalThis.window = originalWindow;
    } else {
      // @ts-expect-error allow cleanup when window was undefined
      delete globalThis.window;
    }
  });

  beforeEach(() => {
    usePlanningStore.getState().reset();
    mocks.eventHandlers.clear();
    mocks.generatePlanningSessionMock.mockReset();
    mocks.applyPlanningOptionMock.mockReset();
    mocks.resolvePlanningConflictsMock.mockReset();
    mocks.getPlanningPreferencesMock.mockReset();
    mocks.updatePlanningPreferencesMock.mockReset();
  });

  it('generates plan and caches preference snapshot', async () => {
    const session = createSession();
    mocks.generatePlanningSessionMock.mockResolvedValue(session);

    const store = usePlanningStore.getState();
    const result = await store.generatePlan({ taskIds: ['task-1'] });

    expect(result.session.id).toBe('session-1');
    const state = usePlanningStore.getState();
    expect(state.session?.session.id).toBe('session-1');
    expect(state.preferences.default).toEqual(basePreference);
    expect(state.isGenerating).toBe(false);
    expect(state.error).toBeNull();
  });

  it('applies option and merges session view', async () => {
    const session = createSession();
    usePlanningStore.setState((state) => ({
      ...state,
      session,
      lastSessionId: session.session.id,
      preferences: { default: basePreference },
    }));

    const applied: AppliedPlan = {
      session: {
        ...session.session,
        status: 'applied',
        updatedAt: iso(),
      },
      option: {
        option: {
          id: 'option-1',
          sessionId: 'session-1',
          rank: 1,
          score: 90,
          summary: '更新后',
          cotSteps: [],
          riskNotes: { notes: ['注意午间会议'], conflicts: [] },
          isFallback: false,
          createdAt: iso(),
        },
        blocks: [
          {
            id: 'block-1',
            optionId: 'option-1',
            taskId: 'task-1',
            startAt: iso(1),
            endAt: iso(3),
            flexibility: 'fixed',
            confidence: 0.9,
            conflictFlags: [],
            appliedAt: iso(),
            actualStartAt: undefined,
            actualEndAt: undefined,
            status: 'planned',
          },
        ],
        conflicts: [
          {
            conflictType: 'calendar-overlap',
            severity: 'high',
            message: '与午餐冲突',
            relatedBlockId: 'block-1',
            relatedEventId: 'event-1',
          },
        ],
      },
      conflicts: [
        {
          conflictType: 'calendar-overlap',
          severity: 'high',
          message: '与午餐冲突',
          relatedBlockId: 'block-1',
          relatedEventId: 'event-1',
        },
      ],
    };

    mocks.applyPlanningOptionMock.mockResolvedValue(applied);

    const store = usePlanningStore.getState();
    const result = await store.applyOption({
      sessionId: 'session-1',
      optionId: 'option-1',
      overrides: [],
    });

    expect(result.option.option.summary).toBe('更新后');
    const state = usePlanningStore.getState();
    expect(state.session?.conflicts).toHaveLength(1);
    expect(state.session?.options[0].option.summary).toBe('更新后');
    expect(state.lastSessionId).toBe('session-1');
    expect(state.isApplying).toBe(false);
  });

  it('resolves conflicts and updates cached snapshot', async () => {
    const session = createSession({
      conflicts: [
        {
          conflictType: 'calendar-overlap',
          severity: 'high',
          message: '初始冲突',
          relatedBlockId: 'block-1',
          relatedEventId: 'event-1',
        },
      ],
    });

    usePlanningStore.setState((state) => ({
      ...state,
      session,
      lastSessionId: session.session.id,
      preferences: { default: basePreference },
    }));

    const resolvedSnapshot = createPreference({ bufferMinutesBetweenBlocks: 45 });
    const resolvedSession = createSession({
      conflicts: [],
      preferenceSnapshot: resolvedSnapshot,
    });

    mocks.resolvePlanningConflictsMock.mockResolvedValue(resolvedSession);

    const store = usePlanningStore.getState();
    const result = await store.resolveConflicts({
      sessionId: 'session-1',
      optionId: 'option-1',
      adjustments: [],
    });

    expect(result.conflicts).toHaveLength(0);
    const state = usePlanningStore.getState();
    expect(state.session?.conflicts).toHaveLength(0);
    expect(state.preferences.default.bufferMinutesBetweenBlocks).toBe(45);
    expect(state.isResolving).toBe(false);
  });

  it('loads preferences and serves cached data', async () => {
    mocks.getPlanningPreferencesMock.mockResolvedValue(
      createPreference({ bufferMinutesBetweenBlocks: 25 }),
    );

    const store = usePlanningStore.getState();
    const first = await store.loadPreferences('focus_morning');
    expect(first.bufferMinutesBetweenBlocks).toBe(25);
    expect(mocks.getPlanningPreferencesMock).toHaveBeenCalledTimes(1);

    const second = await store.loadPreferences('focus_morning');
    expect(second.bufferMinutesBetweenBlocks).toBe(25);
    expect(mocks.getPlanningPreferencesMock).toHaveBeenCalledTimes(1);
  });

  it('maps errors via toAppError when generation fails', async () => {
    mocks.generatePlanningSessionMock.mockRejectedValue({
      code: 'NETWORK',
      message: '网络异常',
    });

    const store = usePlanningStore.getState();
    await expect(store.generatePlan({ taskIds: ['task-1'] })).rejects.toMatchObject({
      code: 'NETWORK',
    });

    const state = usePlanningStore.getState();
    expect(state.error?.code).toBe('NETWORK');
    expect(state.isGenerating).toBe(false);
  });

  it('handles generated event payloads from event bridge', async () => {
    const session = createSession();
    const store = usePlanningStore.getState();
    await store.ensureEventBridge();

    const handler = mocks.eventHandlers.get(PLANNING_EVENT_NAMES.GENERATED);
    expect(handler).toBeDefined();
    handler?.({ payload: session });

    const state = usePlanningStore.getState();
    expect(state.session?.session.id).toBe('session-1');
    expect(state.lastSessionId).toBe('session-1');
  });
});
