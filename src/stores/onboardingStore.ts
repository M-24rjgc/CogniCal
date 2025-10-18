import { create, type StateCreator } from 'zustand';
import { createJSONStorage, persist } from 'zustand/middleware';
import {
  ONBOARDING_STEP_IDS,
  ONBOARDING_TOUR_STEPS,
  type OnboardingStepId,
  isOnboardingStepId,
  getFirstOnboardingStepId,
} from '../utils/onboarding';

const ONBOARDING_STORAGE_KEY = 'onboarding-store-state';
const ONBOARDING_PERSIST_VERSION = 1;

interface OnboardingProgress {
  version: number;
  hasCompletedTour: boolean;
  completedStepIds: OnboardingStepId[];
  lastStepId?: OnboardingStepId;
  dismissedAt?: string | null;
}

interface OnboardingStoreState {
  progress: OnboardingProgress;
  replayRequestToken: string | null;
  setHasCompletedTour: (completed: boolean) => void;
  markStepComplete: (stepId: OnboardingStepId) => void;
  recordDismissal: (stepId?: OnboardingStepId) => void;
  resetProgress: () => void;
  requestReplay: () => string;
  consumeReplayRequest: () => string | null;
  shouldAutoLaunchTour: () => boolean;
  getPendingStepId: () => OnboardingStepId | null;
}

const createId = () =>
  typeof crypto !== 'undefined' && 'randomUUID' in crypto
    ? crypto.randomUUID()
    : `tour-${Math.random().toString(36).slice(2, 10)}`;

const createInitialProgress = (): OnboardingProgress => ({
  version: ONBOARDING_PERSIST_VERSION,
  hasCompletedTour: false,
  completedStepIds: [],
  lastStepId: undefined,
  dismissedAt: null,
});

const sanitizeCompletedIds = (ids: OnboardingStepId[]): OnboardingStepId[] => {
  const set = new Set<OnboardingStepId>();
  for (const value of ids) {
    if (isOnboardingStepId(value)) {
      set.add(value);
    }
  }
  return Array.from(set);
};

const sanitizeProgress = (progress?: Partial<OnboardingProgress>): OnboardingProgress => {
  if (!progress || progress.version !== ONBOARDING_PERSIST_VERSION) {
    return createInitialProgress();
  }

  const completedStepIds = sanitizeCompletedIds(progress.completedStepIds ?? []);
  const allStepsCompleted = completedStepIds.length >= ONBOARDING_STEP_IDS.length;

  return {
    version: ONBOARDING_PERSIST_VERSION,
    hasCompletedTour: Boolean(progress.hasCompletedTour && allStepsCompleted),
    completedStepIds,
    lastStepId:
      progress.lastStepId && isOnboardingStepId(progress.lastStepId)
        ? progress.lastStepId
        : completedStepIds[completedStepIds.length - 1],
    dismissedAt: typeof progress.dismissedAt === 'string' ? progress.dismissedAt : null,
  } satisfies OnboardingProgress;
};

const memoryStorage: Storage = {
  get length() {
    return 0;
  },
  clear: () => undefined,
  getItem: () => null,
  key: () => null,
  removeItem: () => undefined,
  setItem: () => undefined,
};

type OnboardingStorePersist = Pick<OnboardingStoreState, 'progress'>;

const shouldAutoLaunch = (progress: OnboardingProgress): boolean => {
  if (progress.hasCompletedTour) {
    return false;
  }
  return !progress.dismissedAt;
};

const findPendingStep = (progress: OnboardingProgress): OnboardingStepId | null => {
  const completed = new Set(progress.completedStepIds);
  for (const step of ONBOARDING_TOUR_STEPS) {
    if (!completed.has(step.id)) {
      return step.id;
    }
  }
  return null;
};

const onboardingStoreCreator: StateCreator<OnboardingStoreState> = (set, get) => ({
  progress: createInitialProgress(),
  replayRequestToken: null,
  setHasCompletedTour(completed) {
    set((state) => ({
      progress: {
        ...state.progress,
        hasCompletedTour: completed,
        dismissedAt: completed ? null : state.progress.dismissedAt,
        version: ONBOARDING_PERSIST_VERSION,
      },
    }));
  },
  markStepComplete(stepId) {
    if (!isOnboardingStepId(stepId)) {
      console.warn('[onboardingStore] Invalid step id:', stepId);
      return;
    }

    set((state) => {
      const completedSet = new Set(state.progress.completedStepIds);
      completedSet.add(stepId);
      const completedStepIds = Array.from(completedSet);
      const hasCompletedTour = completedStepIds.length >= ONBOARDING_STEP_IDS.length;

      return {
        progress: {
          version: ONBOARDING_PERSIST_VERSION,
          completedStepIds,
          hasCompletedTour,
          lastStepId: stepId,
          dismissedAt: hasCompletedTour ? null : (state.progress.dismissedAt ?? null),
        },
      } satisfies Partial<OnboardingStoreState>;
    });
  },
  recordDismissal(stepId) {
    set((state) => ({
      progress: {
        ...state.progress,
        dismissedAt: new Date().toISOString(),
        lastStepId: stepId && isOnboardingStepId(stepId) ? stepId : state.progress.lastStepId,
        version: ONBOARDING_PERSIST_VERSION,
      },
    }));
  },
  resetProgress() {
    set({
      progress: createInitialProgress(),
      replayRequestToken: null,
    });
  },
  requestReplay() {
    const token = createId();
    set((state) => ({
      replayRequestToken: token,
      progress: {
        ...state.progress,
        dismissedAt: null,
        version: ONBOARDING_PERSIST_VERSION,
      },
    }));
    return token;
  },
  consumeReplayRequest() {
    const token = get().replayRequestToken;
    if (!token) {
      return null;
    }
    set({ replayRequestToken: null });
    return token;
  },
  shouldAutoLaunchTour() {
    return shouldAutoLaunch(get().progress);
  },
  getPendingStepId() {
    return findPendingStep(get().progress);
  },
});

const persistedOnboardingStore = persist(onboardingStoreCreator, {
  name: ONBOARDING_STORAGE_KEY,
  version: ONBOARDING_PERSIST_VERSION,
  storage: createJSONStorage(() =>
    typeof window === 'undefined' ? memoryStorage : window.localStorage,
  ),
  partialize: (state): OnboardingStorePersist => ({
    progress: state.progress,
  }),
  migrate: (persisted) => {
    const progress = sanitizeProgress((persisted as OnboardingStorePersist | undefined)?.progress);
    return {
      progress,
      replayRequestToken: null,
    };
  },
}) as unknown as StateCreator<OnboardingStoreState>;

export const useOnboardingStore = create<OnboardingStoreState>(persistedOnboardingStore);

export const selectOnboardingProgress = () => useOnboardingStore.getState().progress;
export const selectHasCompletedOnboarding = () =>
  useOnboardingStore.getState().progress.hasCompletedTour;
export const shouldAutoLaunchOnboardingTour = () =>
  useOnboardingStore.getState().shouldAutoLaunchTour();
export const getPendingOnboardingStep = () => useOnboardingStore.getState().getPendingStepId();
export const requestOnboardingReplay = () => useOnboardingStore.getState().requestReplay();
export const consumeOnboardingReplayToken = () =>
  useOnboardingStore.getState().consumeReplayRequest();
export const resetOnboardingProgress = () => useOnboardingStore.getState().resetProgress();
export const markOnboardingStepComplete = (stepId: OnboardingStepId) =>
  useOnboardingStore.getState().markStepComplete(stepId);
export const recordOnboardingDismissal = (stepId?: OnboardingStepId) =>
  useOnboardingStore.getState().recordDismissal(stepId);
export const setOnboardingCompletion = (completed: boolean) =>
  useOnboardingStore.getState().setHasCompletedTour(completed);
export const getFirstPendingOnboardingStep = () =>
  findPendingStep(useOnboardingStore.getState().progress) ?? getFirstOnboardingStepId();
