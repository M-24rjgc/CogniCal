import { create } from 'zustand';
import {
  fetchAppSettings,
  fetchAiStatus,
  updateAppSettings,
  type AppSettings,
  type UpdateAppSettingsInput,
  type AppError,
  isAppError,
  toAppError,
} from '../services/tauriApi';
import type { AiStatus } from '../types/settings';

interface SettingsStoreState {
  settings: AppSettings | null;
  isLoading: boolean;
  isSaving: boolean;
  aiStatus: AiStatus | null;
  isTestingAi: boolean;
  error: AppError | null;
  loadSettings: () => Promise<AppSettings>;
  updateSettings: (input: UpdateAppSettingsInput) => Promise<AppSettings>;
  setSettings: (settings: AppSettings) => void;
  clearError: () => void;
  loadAiStatus: () => Promise<AiStatus>;
  testAiConnection: () => Promise<AiStatus>;
  setAiStatus: (status: AiStatus | null) => void;
}

export const useSettingsStore = create<SettingsStoreState>((set, get) => ({
  settings: null,
  isLoading: false,
  isSaving: false,
  aiStatus: null,
  isTestingAi: false,
  error: null,
  async loadSettings() {
    set({ isLoading: true, error: null });
    try {
      const settings = await fetchAppSettings();
      set({ settings, isLoading: false });
      return settings;
    } catch (error) {
      const appError = isAppError(error) ? error : toAppError(error);
      set({ error: appError, isLoading: false });
      throw appError;
    }
  },
  async updateSettings(input) {
    set({ isSaving: true, error: null });
    try {
      const result = await updateAppSettings(input);
      set({ settings: result, isSaving: false });
      return result;
    } catch (error) {
      const appError = isAppError(error) ? error : toAppError(error);
      set({ error: appError, isSaving: false });
      throw appError;
    }
  },
  setSettings(settings) {
    set({ settings });
  },
  clearError() {
    if (get().error) {
      set({ error: null });
    }
  },
  async loadAiStatus() {
    try {
      const status = await fetchAiStatus();
      set({ aiStatus: status });
      return status;
    } catch (error) {
      const appError = isAppError(error) ? error : toAppError(error);
      set({ aiStatus: null });
      throw appError;
    }
  },
  async testAiConnection() {
    set({ isTestingAi: true });
    try {
      const status = await fetchAiStatus();
      set({ aiStatus: status, isTestingAi: false });
      return status;
    } catch (error) {
      const appError = isAppError(error) ? error : toAppError(error);
      set({ isTestingAi: false });
      throw appError;
    }
  },
  setAiStatus(status) {
    set({ aiStatus: status });
  },
}));
