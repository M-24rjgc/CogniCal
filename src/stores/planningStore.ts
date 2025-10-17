import { create } from 'zustand';
import { listen } from '@tauri-apps/api/event';
import { z } from 'zod';
import {
  applyPlanningOption,
  generatePlanningSession,
  getPlanningPreferences,
  isAppError,
  PLANNING_EVENT_NAMES,
  resolvePlanningConflicts,
  toAppError,
  updatePlanningPreferences,
  type AppError,
  type ApplyPlanInput,
  type AppliedPlan,
  type GeneratePlanInput,
  type PlanningPreferencesUpdateInput,
  type PlanningSessionView,
  type PreferenceSnapshot,
  type ResolveConflictInput,
} from '../services/tauriApi';
import {
  parseAppliedPlan,
  parsePlanningSessionView,
  preferenceSnapshotSchema,
  scheduleConflictSchema,
} from '../types/planning';

const conflictsArraySchema = z.array(scheduleConflictSchema);

type SetFn<T> = {
  (partial: T | Partial<T> | ((state: T) => T | Partial<T>), replace?: false): void;
  (state: T | ((state: T) => T), replace: true): void;
};

const isTauriRuntime = () => {
  if (typeof window === 'undefined') return false;
  const tauriWindow = window as typeof window & {
    __TAURI_IPC__?: unknown;
    __TAURI_INTERNALS__?: unknown;
  };
  return Boolean(tauriWindow.__TAURI_IPC__ ?? tauriWindow.__TAURI_INTERNALS__);
};

type UnlistenFn = () => void;

let listenersReady = false;
let listenersPromise: Promise<void> | null = null;
let unlistenFns: UnlistenFn[] = [];

interface PlanningStoreState {
  session: PlanningSessionView | null;
  lastSessionId: string | null;
  activePreferenceId: string;
  preferences: Record<string, PreferenceSnapshot>;
  isGenerating: boolean;
  isApplying: boolean;
  isResolving: boolean;
  isPreferencesLoading: boolean;
  isPreferencesSaving: boolean;
  hasEventBridge: boolean;
  error: AppError | null;
  ensureEventBridge: () => Promise<void>;
  generatePlan: (input: GeneratePlanInput) => Promise<PlanningSessionView>;
  applyOption: (input: ApplyPlanInput) => Promise<AppliedPlan>;
  resolveConflicts: (input: ResolveConflictInput) => Promise<PlanningSessionView>;
  loadPreferences: (
    preferenceId?: string,
    options?: { force?: boolean },
  ) => Promise<PreferenceSnapshot>;
  updatePreferences: (payload: PlanningPreferencesUpdateInput) => Promise<void>;
  setActivePreferenceId: (preferenceId: string) => void;
  clearError: () => void;
  reset: () => void;
}

const detachEventBridge = () => {
  if (unlistenFns.length === 0) return;
  for (const unlisten of unlistenFns) {
    try {
      unlisten();
    } catch (error) {
      console.warn('[planningStore] failed to remove event listener', error);
    }
  }
  unlistenFns = [];
  listenersReady = false;
  listenersPromise = null;
};

const attachEventBridge = async (set: SetFn<PlanningStoreState>, get: () => PlanningStoreState) => {
  if (listenersReady) {
    set({ hasEventBridge: true });
    return;
  }

  if (!isTauriRuntime()) {
    set({ hasEventBridge: false });
    return;
  }

  if (listenersPromise) {
    await listenersPromise;
    return;
  }

  listenersPromise = (async () => {
    try {
      const unlistenGenerated = await listen(PLANNING_EVENT_NAMES.GENERATED, (event) => {
        try {
          const view = parsePlanningSessionView(event.payload);
          const preferenceId = get().activePreferenceId;
          set((state: PlanningStoreState) => {
            const nextPreferences = view.preferenceSnapshot
              ? { ...state.preferences, [preferenceId]: view.preferenceSnapshot }
              : state.preferences;
            const nextState: Partial<PlanningStoreState> = {
              session: view,
              lastSessionId: view.session.id,
              preferences: nextPreferences,
              error: null,
            };
            return nextState;
          });
        } catch (error) {
          console.warn('[planningStore] failed to parse planning generated payload', error);
        }
      });

      const unlistenApplied = await listen(PLANNING_EVENT_NAMES.APPLIED, (event) => {
        try {
          const applied = parseAppliedPlan(event.payload);
          set((state: PlanningStoreState) => {
            const previous = state.session;
            const hasSameSession = previous?.session.id === applied.session.id;
            const baseOptions = hasSameSession ? previous.options : [];
            const filtered = baseOptions.filter(
              (option) => option.option.id !== applied.option.option.id,
            );
            const mergedOptions = [...filtered, applied.option].sort(
              (a, b) => a.option.rank - b.option.rank,
            );
            const preferenceSnapshot =
              previous?.preferenceSnapshot ?? state.preferences[state.activePreferenceId];

            const nextSession: PlanningSessionView = {
              session: applied.session,
              options: mergedOptions,
              conflicts: applied.conflicts,
              preferenceSnapshot,
            };

            return {
              session: nextSession,
              lastSessionId: applied.session.id,
              error: null,
            } satisfies Partial<PlanningStoreState>;
          });
        } catch (error) {
          console.warn('[planningStore] failed to parse planning applied payload', error);
        }
      });

      const unlistenConflicts = await listen(PLANNING_EVENT_NAMES.CONFLICTS_RESOLVED, (event) => {
        try {
          const conflicts = conflictsArraySchema.parse(event.payload ?? []);
          set((state: PlanningStoreState) => {
            if (!state.session) {
              return {} satisfies Partial<PlanningStoreState>;
            }
            const selectedOptionId = state.session.session.selectedOptionId;
            const options = state.session.options.map((option) => {
              if (!selectedOptionId || option.option.id !== selectedOptionId) {
                return option;
              }
              return {
                ...option,
                conflicts,
              };
            });
            const nextSession: PlanningSessionView = {
              ...state.session,
              conflicts,
              options,
            };
            return {
              session: nextSession,
            } satisfies Partial<PlanningStoreState>;
          });
        } catch (error) {
          console.warn('[planningStore] failed to parse conflicts payload', error);
        }
      });

      const unlistenPreferences = await listen(
        PLANNING_EVENT_NAMES.PREFERENCES_UPDATED,
        (event) => {
          const rawId = typeof event.payload === 'string' ? event.payload : null;
          const preferenceId = rawId && rawId.trim().length > 0 ? rawId.trim() : 'default';
          set((state: PlanningStoreState) => {
            const nextPreferences = { ...state.preferences };
            delete nextPreferences[preferenceId];
            return {
              preferences: nextPreferences,
            } satisfies Partial<PlanningStoreState>;
          });

          if (preferenceId === get().activePreferenceId) {
            void get()
              .loadPreferences(preferenceId, { force: true })
              .catch((error) => {
                if (isAppError(error)) {
                  console.warn('[planningStore] failed to refresh preferences after event', error);
                } else {
                  console.warn('[planningStore] unexpected error refreshing preferences', error);
                }
              });
          }
        },
      );

      unlistenFns = [unlistenGenerated, unlistenApplied, unlistenConflicts, unlistenPreferences];
      listenersReady = true;
      set({ hasEventBridge: true });
    } catch (error) {
      console.warn('[planningStore] failed to attach planning event listeners', error);
      detachEventBridge();
    } finally {
      listenersPromise = null;
    }
  })();

  await listenersPromise;
};

const sanitizePreferenceId = (preferenceId?: string | null) => {
  if (!preferenceId) return 'default';
  const trimmed = preferenceId.trim();
  return trimmed.length > 0 ? trimmed : 'default';
};

export const usePlanningStore = create<PlanningStoreState>((set, get) => ({
  session: null,
  lastSessionId: null,
  activePreferenceId: 'default',
  preferences: {},
  isGenerating: false,
  isApplying: false,
  isResolving: false,
  isPreferencesLoading: false,
  isPreferencesSaving: false,
  hasEventBridge: false,
  error: null,
  async ensureEventBridge() {
    await attachEventBridge(set, get);
  },
  async generatePlan(input) {
    await get().ensureEventBridge();
    const preferenceId = sanitizePreferenceId(input.preferenceId);
    set({
      isGenerating: true,
      error: null,
      activePreferenceId: preferenceId,
    });

    try {
      const session = await generatePlanningSession(input);
      set((state: PlanningStoreState) => {
        const nextPreferences = session.preferenceSnapshot
          ? { ...state.preferences, [preferenceId]: session.preferenceSnapshot }
          : state.preferences;
        return {
          session,
          lastSessionId: session.session.id,
          isGenerating: false,
          error: null,
          preferences: nextPreferences,
        } satisfies Partial<PlanningStoreState>;
      });
      return session;
    } catch (error) {
      const appError = toAppError(error, '生成规划方案失败');
      set({ isGenerating: false, error: appError });
      throw appError;
    }
  },
  async applyOption(input) {
    await get().ensureEventBridge();
    set({ isApplying: true, error: null });
    try {
      const applied = await applyPlanningOption(input);
      set((state: PlanningStoreState) => {
        const previous = state.session;
        const hasSameSession = previous?.session.id === applied.session.id;
        const baseOptions = hasSameSession ? previous.options : [];
        const filtered = baseOptions.filter(
          (option) => option.option.id !== applied.option.option.id,
        );
        const mergedOptions = [...filtered, applied.option].sort(
          (a, b) => a.option.rank - b.option.rank,
        );
        const preferenceSnapshot =
          previous?.preferenceSnapshot ?? state.preferences[state.activePreferenceId];
        const nextSession: PlanningSessionView = {
          session: applied.session,
          options: mergedOptions,
          conflicts: applied.conflicts,
          preferenceSnapshot,
        };
        return {
          session: nextSession,
          lastSessionId: applied.session.id,
          isApplying: false,
          error: null,
        } satisfies Partial<PlanningStoreState>;
      });
      return applied;
    } catch (error) {
      const appError = toAppError(error, '应用规划方案失败');
      set({ isApplying: false, error: appError });
      throw appError;
    }
  },
  async resolveConflicts(input) {
    await get().ensureEventBridge();
    set({ isResolving: true, error: null });
    try {
      const session = await resolvePlanningConflicts(input);
      set((state: PlanningStoreState) => {
        const preferenceId = state.activePreferenceId;
        const nextPreferences = session.preferenceSnapshot
          ? { ...state.preferences, [preferenceId]: session.preferenceSnapshot }
          : state.preferences;
        return {
          session,
          lastSessionId: session.session.id,
          isResolving: false,
          error: null,
          preferences: nextPreferences,
        } satisfies Partial<PlanningStoreState>;
      });
      return session;
    } catch (error) {
      const appError = toAppError(error, '调整冲突失败');
      set({ isResolving: false, error: appError });
      throw appError;
    }
  },
  async loadPreferences(preferenceId, options) {
    await get().ensureEventBridge();
    const id = sanitizePreferenceId(preferenceId);
    const force = options?.force ?? false;
    const cached = get().preferences[id];
    if (cached && !force) {
      set({ activePreferenceId: id });
      return cached;
    }

    set({ isPreferencesLoading: true, error: null, activePreferenceId: id });
    try {
      const snapshot = await getPlanningPreferences(id);
      const parsed = preferenceSnapshotSchema.parse(snapshot);
      set(
        (state: PlanningStoreState) =>
          ({
            preferences: { ...state.preferences, [id]: parsed },
            isPreferencesLoading: false,
            error: null,
            activePreferenceId: id,
          }) satisfies Partial<PlanningStoreState>,
      );
      return parsed;
    } catch (error) {
      const appError = toAppError(error, '获取偏好设置失败');
      set({ isPreferencesLoading: false, error: appError });
      throw appError;
    }
  },
  async updatePreferences(payload) {
    await get().ensureEventBridge();
    const preferenceId = sanitizePreferenceId(payload.preferenceId);
    set({ isPreferencesSaving: true, error: null, activePreferenceId: preferenceId });

    try {
      await updatePlanningPreferences(payload);
      const parsed = preferenceSnapshotSchema.parse(payload.snapshot);
      set(
        (state: PlanningStoreState) =>
          ({
            preferences: { ...state.preferences, [preferenceId]: parsed },
            isPreferencesSaving: false,
            error: null,
          }) satisfies Partial<PlanningStoreState>,
      );
    } catch (error) {
      const appError = toAppError(error, '保存偏好设置失败');
      set({ isPreferencesSaving: false, error: appError });
      throw appError;
    }
  },
  setActivePreferenceId(preferenceId: string) {
    const id = sanitizePreferenceId(preferenceId);
    set({ activePreferenceId: id });
  },
  clearError() {
    set({ error: null });
  },
  reset() {
    detachEventBridge();
    set({
      session: null,
      lastSessionId: null,
      activePreferenceId: 'default',
      preferences: {},
      isGenerating: false,
      isApplying: false,
      isResolving: false,
      isPreferencesLoading: false,
      isPreferencesSaving: false,
      hasEventBridge: false,
      error: null,
    });
  },
}));
