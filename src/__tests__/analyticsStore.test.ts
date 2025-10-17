import { beforeEach, describe, expect, it } from 'vitest';
import type { AnalyticsExportResult } from '../types/analytics';
import type { AppError } from '../services/tauriApi';
import { useAnalyticsStore } from '../stores/analyticsStore';

type PersistApi = {
  clearStorage?: () => Promise<void> | void;
};

const getPersistApi = (): PersistApi | undefined => {
  return (useAnalyticsStore as typeof useAnalyticsStore & { persist?: PersistApi }).persist;
};

const resetState = () => {
  useAnalyticsStore.setState({
    range: '7d',
    grouping: 'day',
    isOnboardingComplete: false,
    exportStatus: 'idle',
    exportResult: null,
    exportError: null,
    lastRefreshedAt: null,
    isDemoData: false,
  });
};

describe('analyticsStore', () => {
  beforeEach(async () => {
    const persist = getPersistApi();
    if (persist?.clearStorage) {
      await persist.clearStorage();
    }
    resetState();
  });

  it('updates range and grouping preferences', () => {
    const store = useAnalyticsStore.getState();
    store.setRange('30d');
    store.setGrouping('week');

    const next = useAnalyticsStore.getState();
    expect(next.range).toBe('30d');
    expect(next.grouping).toBe('week');
  });

  it('tracks onboarding completion lifecycle', () => {
    const store = useAnalyticsStore.getState();
    expect(store.isOnboardingComplete).toBe(false);

    store.markOnboardingComplete();
    expect(useAnalyticsStore.getState().isOnboardingComplete).toBe(true);

    store.resetOnboarding();
    expect(useAnalyticsStore.getState().isOnboardingComplete).toBe(false);
  });

  it('manages export lifecycle and error state', () => {
    const previousError: AppError = { code: 'UNKNOWN', message: 'previous failure' };
    useAnalyticsStore.setState({ exportStatus: 'error', exportError: previousError });

    const store = useAnalyticsStore.getState();
    store.setExportStatus('loading');

    let next = useAnalyticsStore.getState();
    expect(next.exportStatus).toBe('loading');
    expect(next.exportError).toBeNull();

    const exportResult: AnalyticsExportResult = {
      filePath: 'reports/demo.md',
      format: 'markdown',
      generatedAt: new Date().toISOString(),
      isDemo: false,
    };
    store.setExportResult(exportResult);
    next = useAnalyticsStore.getState();
    expect(next.exportResult).toEqual(exportResult);

    const exportError: AppError = { code: 'NETWORK', message: 'failed to write' };
    store.setExportError(exportError);
    next = useAnalyticsStore.getState();
    expect(next.exportStatus).toBe('error');
    expect(next.exportError).toEqual(exportError);

    store.setExportError(null);
    next = useAnalyticsStore.getState();
    expect(next.exportStatus).toBe('idle');
    expect(next.exportError).toBeNull();
  });

  it('records refresh metadata and demo mode', () => {
    const store = useAnalyticsStore.getState();

    const timestamp = new Date().toISOString();
    store.setLastRefreshedAt(timestamp);
    store.setIsDemoData(true);

    const next = useAnalyticsStore.getState();
    expect(next.lastRefreshedAt).toBe(timestamp);
    expect(next.isDemoData).toBe(true);
  });
});
