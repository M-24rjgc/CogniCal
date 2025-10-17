import { create, type StateCreator } from 'zustand';
import { createJSONStorage, persist } from 'zustand/middleware';
import type {
  AnalyticsExportResult,
  AnalyticsGrouping,
  AnalyticsRangeKey,
} from '../types/analytics';
import type { AppError } from '../services/tauriApi';
import type { ProductivityScoreRecord } from '../types/productivity.ts';

export type AnalyticsExportStatus = 'idle' | 'loading' | 'success' | 'error';

interface AnalyticsStoreState {
  range: AnalyticsRangeKey;
  grouping: AnalyticsGrouping;
  isOnboardingComplete: boolean;
  exportStatus: AnalyticsExportStatus;
  exportResult: AnalyticsExportResult | null;
  exportError: AppError | null;
  lastRefreshedAt: string | null;
  isDemoData: boolean;

  // Productivity score related state
  currentProductivityScore: ProductivityScoreRecord | null;
  productivityScoreLoading: boolean;
  productivityScoreError: AppError | null;

  setRange: (range: AnalyticsRangeKey) => void;
  setGrouping: (grouping: AnalyticsGrouping) => void;
  markOnboardingComplete: () => void;
  resetOnboarding: () => void;
  setExportStatus: (status: AnalyticsExportStatus) => void;
  setExportResult: (result: AnalyticsExportResult | null) => void;
  setExportError: (error: AppError | null) => void;
  setLastRefreshedAt: (timestamp: string) => void;
  setIsDemoData: (isDemo: boolean) => void;

  // Productivity score actions
  setCurrentProductivityScore: (score: ProductivityScoreRecord | null) => void;
  setProductivityScoreLoading: (loading: boolean) => void;
  setProductivityScoreError: (error: AppError | null) => void;
}

const createInitialState = (): Omit<
  AnalyticsStoreState,
  | 'setRange'
  | 'setGrouping'
  | 'markOnboardingComplete'
  | 'resetOnboarding'
  | 'setExportStatus'
  | 'setExportResult'
  | 'setExportError'
  | 'setLastRefreshedAt'
  | 'setIsDemoData'
  | 'setCurrentProductivityScore'
  | 'setProductivityScoreLoading'
  | 'setProductivityScoreError'
> => ({
  range: '7d',
  grouping: 'day',
  isOnboardingComplete: false,
  exportStatus: 'idle',
  exportResult: null,
  exportError: null,
  lastRefreshedAt: null,
  isDemoData: false,

  // Productivity score state
  currentProductivityScore: null,
  productivityScoreLoading: false,
  productivityScoreError: null,
});

type AnalyticsStorePersist = Pick<
  AnalyticsStoreState,
  'range' | 'grouping' | 'isOnboardingComplete' | 'lastRefreshedAt' | 'isDemoData'
>;

const analyticsStoreCreator = ((set) => ({
  ...createInitialState(),
  setRange(range) {
    set({ range });
  },
  setGrouping(grouping) {
    set({ grouping });
  },
  markOnboardingComplete() {
    set({ isOnboardingComplete: true });
  },
  resetOnboarding() {
    set({ isOnboardingComplete: false });
  },
  setExportStatus(status) {
    set((state) => ({
      exportStatus: status,
      exportError: status === 'loading' ? null : state.exportError,
    }));
  },
  setExportResult(result) {
    set({ exportResult: result });
  },
  setExportError(error) {
    set({ exportError: error ?? null, exportStatus: error ? 'error' : 'idle' });
  },
  setLastRefreshedAt(timestamp) {
    set({ lastRefreshedAt: timestamp });
  },
  setIsDemoData(isDemo) {
    set({ isDemoData: isDemo });
  },

  // Productivity score actions
  setCurrentProductivityScore(score) {
    set({ currentProductivityScore: score });
  },
  setProductivityScoreLoading(loading) {
    set({ productivityScoreLoading: loading });
  },
  setProductivityScoreError(error) {
    set({ productivityScoreError: error });
  },
})) satisfies StateCreator<AnalyticsStoreState>;

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

const persistedAnalyticsStore = persist(analyticsStoreCreator, {
  name: 'analytics-store-state',
  storage: createJSONStorage(() =>
    typeof window === 'undefined' ? memoryStorage : window.localStorage,
  ),
  partialize: (state: AnalyticsStoreState): AnalyticsStorePersist => ({
    range: state.range,
    grouping: state.grouping,
    isOnboardingComplete: state.isOnboardingComplete,
    lastRefreshedAt: state.lastRefreshedAt,
    isDemoData: state.isDemoData,
  }),
}) as unknown as StateCreator<AnalyticsStoreState>;

export const useAnalyticsStore = create<AnalyticsStoreState>(persistedAnalyticsStore);
