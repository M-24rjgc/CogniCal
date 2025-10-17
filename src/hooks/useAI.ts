import { useCallback, useState } from 'react';
import { useSettingsStore } from '../stores/settingsStore';
import { isAppError, toAppError, type AppError } from '../services/tauriApi';
import type { AiStatus } from '../types/settings';

interface UseAIResult {
  aiStatus: AiStatus | null;
  isTesting: boolean;
  statusError: AppError | null;
  refreshStatus: () => Promise<AiStatus>;
  testConnection: () => Promise<AiStatus>;
  resetStatus: () => void;
  setStatus: (status: AiStatus | null) => void;
}

export function useAI(): UseAIResult {
  const aiStatus = useSettingsStore((state) => state.aiStatus);
  const isTesting = useSettingsStore((state) => state.isTestingAi);
  const loadAiStatus = useSettingsStore((state) => state.loadAiStatus);
  const testAiConnection = useSettingsStore((state) => state.testAiConnection);
  const setAiStatus = useSettingsStore((state) => state.setAiStatus);

  const [statusError, setStatusError] = useState<AppError | null>(null);

  const refreshStatus = useCallback(async () => {
    try {
      const status = await loadAiStatus();
      setStatusError(null);
      return status;
    } catch (error) {
      const appError = isAppError(error) ? error : toAppError(error);
      setStatusError(appError);
      throw appError;
    }
  }, [loadAiStatus]);

  const testConnection = useCallback(async () => {
    try {
      const status = await testAiConnection();
      setStatusError(null);
      return status;
    } catch (error) {
      const appError = isAppError(error) ? error : toAppError(error);
      setStatusError(appError);
      throw appError;
    }
  }, [testAiConnection]);

  const resetStatus = useCallback(() => {
    setAiStatus(null);
    setStatusError(null);
  }, [setAiStatus]);

  const setStatus = useCallback(
    (status: AiStatus | null) => {
      setAiStatus(status);
      setStatusError(null);
    },
    [setAiStatus],
  );

  return {
    aiStatus,
    isTesting,
    statusError,
    refreshStatus,
    testConnection,
    resetStatus,
    setStatus,
  };
}
