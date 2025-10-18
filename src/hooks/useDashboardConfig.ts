import { useCallback, useEffect, useMemo, useRef } from 'react';
import { notifyErrorToast, notifySuccessToast } from '../stores/uiStore';
import { useSettingsStore } from '../stores/settingsStore';
import {
  DEFAULT_DASHBOARD_CONFIG,
  getEnabledDashboardModules,
  normalizeDashboardConfig,
} from '../utils/dashboardConfig';
import type { DashboardConfig, DashboardConfigInput, DashboardModuleId } from '../types/dashboard';
import { isAppError, toAppError, type AppError } from '../services/tauriApi';

interface UseDashboardConfigOptions {
  autoFetch?: boolean;
  notifyOnSuccess?: boolean;
}

const defaultConfigValue: DashboardConfig = normalizeDashboardConfig(DEFAULT_DASHBOARD_CONFIG);

export function useDashboardConfig(options: UseDashboardConfigOptions = {}) {
  const { autoFetch = true, notifyOnSuccess = true } = options;

  const config = useSettingsStore((state) => state.dashboardConfig);
  const isLoading = useSettingsStore((state) => state.isDashboardConfigLoading);
  const isSaving = useSettingsStore((state) => state.isDashboardConfigSaving);
  const storeError = useSettingsStore((state) => state.dashboardConfigError);

  const loadDashboardConfig = useSettingsStore((state) => state.loadDashboardConfig);
  const saveDashboardConfig = useSettingsStore((state) => state.saveDashboardConfig);
  const resetDashboardConfig = useSettingsStore((state) => state.resetDashboardConfig);
  const setDashboardConfig = useSettingsStore((state) => state.setDashboardConfig);
  const clearDashboardConfigError = useSettingsStore((state) => state.clearDashboardConfigError);

  const hasFetchedRef = useRef(false);
  const lastErrorRef = useRef<AppError | null>(null);

  const resolvedConfig = config ?? defaultConfigValue;

  const handleError = useCallback(
    (error: unknown) => {
      const appError = isAppError(error) ? error : toAppError(error);
      notifyErrorToast(appError);
      lastErrorRef.current = appError;
      return appError;
    },
    [lastErrorRef],
  );

  const refresh = useCallback(async () => {
    try {
      return await loadDashboardConfig();
    } catch (error) {
      throw handleError(error);
    }
  }, [handleError, loadDashboardConfig]);

  const update = useCallback(
    async (input: DashboardConfigInput) => {
      try {
        const result = await saveDashboardConfig(input);
        if (notifyOnSuccess) {
          notifySuccessToast('仪表盘配置已更新');
        }
        return result;
      } catch (error) {
        throw handleError(error);
      }
    },
    [handleError, notifyOnSuccess, saveDashboardConfig],
  );

  const reset = useCallback(async () => {
    try {
      const result = await resetDashboardConfig();
      if (notifyOnSuccess) {
        notifySuccessToast('已恢复仪表盘默认配置');
      }
      return result;
    } catch (error) {
      throw handleError(error);
    }
  }, [handleError, notifyOnSuccess, resetDashboardConfig]);

  const setDirectly = useCallback(
    (nextConfig: DashboardConfig) => {
      setDashboardConfig(nextConfig);
    },
    [setDashboardConfig],
  );

  const setModuleEnabled = useCallback(
    async (moduleId: DashboardModuleId, enabled: boolean) =>
      update({ modules: { [moduleId]: enabled } }),
    [update],
  );

  const toggleModule = useCallback(
    async (moduleId: DashboardModuleId) => {
      const current = resolvedConfig.modules[moduleId] ?? false;
      return update({ modules: { [moduleId]: !current } });
    },
    [resolvedConfig, update],
  );

  const enabledModules = useMemo(
    () => getEnabledDashboardModules(resolvedConfig),
    [resolvedConfig],
  );

  const isModuleEnabled = useCallback(
    (moduleId: DashboardModuleId) => resolvedConfig.modules[moduleId] ?? false,
    [resolvedConfig],
  );

  useEffect(() => {
    if (!autoFetch || hasFetchedRef.current) {
      return;
    }
    hasFetchedRef.current = true;
    void refresh().catch(() => undefined);
  }, [autoFetch, refresh]);

  useEffect(() => {
    if (!storeError) {
      lastErrorRef.current = null;
      return;
    }
    if (lastErrorRef.current === storeError) {
      return;
    }
    notifyErrorToast(storeError);
    lastErrorRef.current = storeError;
  }, [storeError]);

  const clearError = useCallback(() => {
    lastErrorRef.current = null;
    clearDashboardConfigError();
  }, [clearDashboardConfigError]);

  return {
    config: resolvedConfig,
    enabledModules,
    isLoading,
    isSaving,
    error: storeError,
    refresh,
    update,
    reset,
    setDashboardConfig: setDirectly,
    setModuleEnabled,
    toggleModule,
    isModuleEnabled,
    clearError,
  } as const;
}
