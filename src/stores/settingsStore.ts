import { create } from 'zustand';
import {
  fetchAppSettings,
  fetchAiStatus,
  fetchDashboardConfig,
  updateAppSettings,
  updateDashboardConfig as updateDashboardConfigApi,
  type AppSettings,
  type UpdateAppSettingsInput,
  type AppError,
  isAppError,
  toAppError,
} from '../services/tauriApi';
import type { AiStatus } from '../types/settings';
import type { DashboardConfig, DashboardConfigInput } from '../types/dashboard';
import { DASHBOARD_MODULE_IDS } from '../types/dashboard';
import { DEFAULT_DASHBOARD_CONFIG, normalizeDashboardConfig } from '../utils/dashboardConfig';

interface SettingsStoreState {
  settings: AppSettings | null;
  isLoading: boolean;
  isSaving: boolean;
  aiStatus: AiStatus | null;
  isTestingAi: boolean;
  error: AppError | null;
  dashboardConfig: DashboardConfig;
  isDashboardConfigLoading: boolean;
  isDashboardConfigSaving: boolean;
  dashboardConfigError: AppError | null;
  loadSettings: () => Promise<AppSettings>;
  updateSettings: (input: UpdateAppSettingsInput) => Promise<AppSettings>;
  setSettings: (settings: AppSettings) => void;
  clearError: () => void;
  loadAiStatus: () => Promise<AiStatus>;
  testAiConnection: () => Promise<AiStatus>;
  setAiStatus: (status: AiStatus | null) => void;
  loadDashboardConfig: () => Promise<DashboardConfig>;
  saveDashboardConfig: (input: DashboardConfigInput) => Promise<DashboardConfig>;
  resetDashboardConfig: () => Promise<DashboardConfig>;
  setDashboardConfig: (config: DashboardConfig) => void;
  clearDashboardConfigError: () => void;
}

const createDefaultDashboardConfig = (): DashboardConfig =>
  normalizeDashboardConfig(DEFAULT_DASHBOARD_CONFIG);

const sanitizeDashboardConfigInput = (input: DashboardConfigInput): DashboardConfigInput => {
  const sanitized: DashboardConfigInput = {};

  if (input.modules) {
    for (const id of DASHBOARD_MODULE_IDS) {
      const override = input.modules[id];
      if (typeof override === 'boolean') {
        if (!sanitized.modules) {
          sanitized.modules = {};
        }
        sanitized.modules[id] = override;
      }
    }
  }

  if (Object.prototype.hasOwnProperty.call(input, 'lastUpdatedAt')) {
    if (typeof input.lastUpdatedAt === 'string' || input.lastUpdatedAt === null) {
      sanitized.lastUpdatedAt = input.lastUpdatedAt;
    }
  }

  return sanitized;
};

const buildOptimisticDashboardConfig = (
  current: DashboardConfig,
  patch: DashboardConfigInput,
): DashboardConfig => {
  const modules = { ...current.modules };

  if (patch.modules) {
    for (const id of DASHBOARD_MODULE_IDS) {
      const override = patch.modules[id];
      if (typeof override === 'boolean') {
        modules[id] = override;
      }
    }
  }

  const timestamp = Object.prototype.hasOwnProperty.call(patch, 'lastUpdatedAt')
    ? (patch.lastUpdatedAt ?? null)
    : new Date().toISOString();

  return normalizeDashboardConfig({
    modules,
    lastUpdatedAt: timestamp,
  });
};

const attachDashboardConfigToSettings = (
  settings: AppSettings | null,
  dashboardConfig: DashboardConfig,
): AppSettings | null => {
  if (!settings) {
    return settings;
  }
  return {
    ...settings,
    dashboardConfig,
  } satisfies AppSettings;
};

const resolveDashboardConfigFromSettings = (
  settings: AppSettings | null | undefined,
  fallback?: DashboardConfig,
): DashboardConfig => {
  if (settings && settings.dashboardConfig !== undefined) {
    return normalizeDashboardConfig(settings.dashboardConfig ?? undefined);
  }
  return fallback ?? createDefaultDashboardConfig();
};

export const useSettingsStore = create<SettingsStoreState>((set, get) => ({
  settings: null,
  isLoading: false,
  isSaving: false,
  aiStatus: null,
  isTestingAi: false,
  error: null,
  dashboardConfig: createDefaultDashboardConfig(),
  isDashboardConfigLoading: false,
  isDashboardConfigSaving: false,
  dashboardConfigError: null,
  async loadSettings() {
    set({ isLoading: true, error: null });
    try {
      const settings = await fetchAppSettings();
      const dashboardConfig = resolveDashboardConfigFromSettings(settings);
      const nextSettings = {
        ...settings,
        dashboardConfig,
      } satisfies AppSettings;
      set({ settings: nextSettings, dashboardConfig, isLoading: false });
      return nextSettings;
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
      const dashboardConfig = resolveDashboardConfigFromSettings(result, get().dashboardConfig);
      const nextSettings = {
        ...result,
        dashboardConfig,
      } satisfies AppSettings;
      set({ settings: nextSettings, dashboardConfig, isSaving: false });
      return nextSettings;
    } catch (error) {
      const appError = isAppError(error) ? error : toAppError(error);
      set({ error: appError, isSaving: false });
      throw appError;
    }
  },
  setSettings(settings) {
    const dashboardConfig = resolveDashboardConfigFromSettings(settings, get().dashboardConfig);
    set({
      settings: {
        ...settings,
        dashboardConfig,
      },
      dashboardConfig,
    });
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
  async loadDashboardConfig() {
    set({ isDashboardConfigLoading: true, dashboardConfigError: null });
    try {
      const config = await fetchDashboardConfig();
      set((state) => ({
        dashboardConfig: config,
        settings: attachDashboardConfigToSettings(state.settings, config),
        isDashboardConfigLoading: false,
      }));
      return config;
    } catch (error) {
      const appError = isAppError(error) ? error : toAppError(error);
      set({ dashboardConfigError: appError, isDashboardConfigLoading: false });
      throw appError;
    }
  },
  async saveDashboardConfig(input) {
    const sanitizedInput = sanitizeDashboardConfigInput(input);
    const previousConfig = get().dashboardConfig ?? createDefaultDashboardConfig();
    const optimisticConfig = buildOptimisticDashboardConfig(previousConfig, sanitizedInput);
    set({
      dashboardConfig: optimisticConfig,
      isDashboardConfigSaving: true,
      dashboardConfigError: null,
    });
    try {
      const config = await updateDashboardConfigApi(sanitizedInput);
      set((state) => ({
        dashboardConfig: config,
        settings: attachDashboardConfigToSettings(state.settings, config),
        isDashboardConfigSaving: false,
      }));
      return config;
    } catch (error) {
      const appError = isAppError(error) ? error : toAppError(error);
      set((state) => ({
        dashboardConfig: previousConfig,
        settings: attachDashboardConfigToSettings(state.settings, previousConfig),
        dashboardConfigError: appError,
        isDashboardConfigSaving: false,
      }));
      throw appError;
    }
  },
  async resetDashboardConfig() {
    const defaultConfig = createDefaultDashboardConfig();
    return get().saveDashboardConfig({
      modules: { ...defaultConfig.modules },
      lastUpdatedAt: null,
    });
  },
  setDashboardConfig(config) {
    const normalized = normalizeDashboardConfig(config);
    set((state) => ({
      dashboardConfig: normalized,
      settings: attachDashboardConfigToSettings(state.settings, normalized),
    }));
  },
  clearDashboardConfigError() {
    if (get().dashboardConfigError) {
      set({ dashboardConfigError: null });
    }
  },
}));
