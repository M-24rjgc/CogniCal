import { useCallback, useEffect, useMemo, useRef } from 'react';
import {
  type AppError,
  type ApplyPlanInput,
  type AppliedPlan,
  type GeneratePlanInput,
  type PlanningPreferencesUpdateInput,
  type PlanningSessionView,
  type PreferenceSnapshot,
  type ResolveConflictInput,
} from '../services/tauriApi';
import { usePlanningStore } from '../stores/planningStore';
import { notifyErrorToast, notifySuccessToast } from '../stores/uiStore';

interface UsePlanningOptions {
  autoAttachEvents?: boolean;
  autoLoadPreferences?: boolean;
  preferenceId?: string;
  suppressSuccessToast?: boolean;
}

interface PlanningHookResult {
  session: PlanningSessionView | null;
  selectedOption: PlanningSessionView['options'][number] | null;
  isGenerating: boolean;
  isApplying: boolean;
  isResolving: boolean;
  isPreferencesLoading: boolean;
  isPreferencesSaving: boolean;
  hasEventBridge: boolean;
  error: AppError | null;
  activePreferenceId: string;
  preferences: Record<string, PreferenceSnapshot>;
  currentPreference: PreferenceSnapshot | undefined;
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
}

export function usePlanning(options: UsePlanningOptions = {}): PlanningHookResult {
  const {
    autoAttachEvents = true,
    autoLoadPreferences = false,
    preferenceId,
    suppressSuccessToast = false,
  } = options;

  const session = usePlanningStore((state) => state.session);
  const isGenerating = usePlanningStore((state) => state.isGenerating);
  const isApplying = usePlanningStore((state) => state.isApplying);
  const isResolving = usePlanningStore((state) => state.isResolving);
  const isPreferencesLoading = usePlanningStore((state) => state.isPreferencesLoading);
  const isPreferencesSaving = usePlanningStore((state) => state.isPreferencesSaving);
  const hasEventBridge = usePlanningStore((state) => state.hasEventBridge);
  const error = usePlanningStore((state) => state.error);
  const activePreferenceId = usePlanningStore((state) => state.activePreferenceId);
  const preferences = usePlanningStore((state) => state.preferences);

  const generate = usePlanningStore((state) => state.generatePlan);
  const apply = usePlanningStore((state) => state.applyOption);
  const resolve = usePlanningStore((state) => state.resolveConflicts);
  const loadPrefs = usePlanningStore((state) => state.loadPreferences);
  const updatePrefs = usePlanningStore((state) => state.updatePreferences);
  const setActivePreference = usePlanningStore((state) => state.setActivePreferenceId);
  const clearError = usePlanningStore((state) => state.clearError);
  const ensureEvents = usePlanningStore((state) => state.ensureEventBridge);

  const lastErrorRef = useRef<AppError | null>(null);

  useEffect(() => {
    if (!autoAttachEvents) return;
    void ensureEvents();
  }, [autoAttachEvents, ensureEvents]);

  useEffect(() => {
    if (!preferenceId) return;
    setActivePreference(preferenceId);
  }, [preferenceId, setActivePreference]);

  useEffect(() => {
    if (!autoLoadPreferences) return;
    const targetId = preferenceId ?? activePreferenceId;
    void loadPrefs(targetId).catch(() => undefined);
  }, [autoLoadPreferences, loadPrefs, preferenceId, activePreferenceId]);

  useEffect(() => {
    if (!error) {
      lastErrorRef.current = null;
      return;
    }
    if (
      lastErrorRef.current &&
      lastErrorRef.current.code === error.code &&
      lastErrorRef.current.message === error.message
    ) {
      return;
    }
    notifyErrorToast(error);
    lastErrorRef.current = error;
  }, [error]);

  const generatePlan = useCallback(
    async (input: GeneratePlanInput) => {
      const result = await generate(input);
      if (!suppressSuccessToast) {
        notifySuccessToast('规划方案已生成', '系统已生成新的调度方案');
      }
      return result;
    },
    [generate, suppressSuccessToast],
  );

  const applyOption = useCallback(
    async (input: ApplyPlanInput) => {
      const result = await apply(input);
      if (!suppressSuccessToast) {
        notifySuccessToast('方案已应用', '已更新任务的计划时间');
      }
      return result;
    },
    [apply, suppressSuccessToast],
  );

  const resolveConflicts = useCallback(
    async (input: ResolveConflictInput) => {
      const result = await resolve(input);
      if (!suppressSuccessToast) {
        notifySuccessToast('冲突已调整', '最新冲突信息已同步');
      }
      return result;
    },
    [resolve, suppressSuccessToast],
  );

  const loadPreferences = useCallback(
    async (id?: string, opts?: { force?: boolean }) => loadPrefs(id, opts),
    [loadPrefs],
  );

  const updatePreferences = useCallback(
    async (payload: PlanningPreferencesUpdateInput) => {
      await updatePrefs(payload);
      if (!suppressSuccessToast) {
        notifySuccessToast('偏好已保存');
      }
    },
    [updatePrefs, suppressSuccessToast],
  );

  const setActivePreferenceId = useCallback(
    (id: string) => {
      setActivePreference(id);
    },
    [setActivePreference],
  );

  const selectedOption = useMemo(() => {
    if (!session) return null;
    const selectedId = session.session.selectedOptionId;
    if (!selectedId) return null;
    return session.options.find((option) => option.option.id === selectedId) ?? null;
  }, [session]);

  const currentPreference = useMemo(() => {
    return preferences[activePreferenceId] ?? session?.preferenceSnapshot;
  }, [preferences, activePreferenceId, session]);

  return {
    session,
    selectedOption,
    isGenerating,
    isApplying,
    isResolving,
    isPreferencesLoading,
    isPreferencesSaving,
    hasEventBridge,
    error,
    activePreferenceId,
    preferences,
    currentPreference,
    generatePlan,
    applyOption,
    resolveConflicts,
    loadPreferences,
    updatePreferences,
    setActivePreferenceId,
    clearError,
  };
}
