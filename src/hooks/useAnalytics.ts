import { useCallback, useEffect, useMemo, useRef } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import {
  exportAnalyticsReport,
  fetchAnalyticsHistory,
  fetchAnalyticsOverview,
  type AnalyticsExportParams,
  type AnalyticsExportResult,
  type AnalyticsHistoryResponse,
  type AnalyticsOverviewResponse,
  type AnalyticsQueryParams,
  type AnalyticsRangeKey,
  type AppError,
  isAppError,
  toAppError,
} from '../services/tauriApi';
import { useAnalyticsStore, type AnalyticsExportStatus } from '../stores/analyticsStore';
import { notifyErrorToast, notifySuccessToast } from '../stores/uiStore';

interface UseAnalyticsOptions {
  /** 手动控制查询是否启用 */
  enabled?: boolean;
  /** 自定义范围覆盖状态存储（用于一次性查看） */
  rangeOverride?: AnalyticsRangeKey;
}

type AnalyticsGroupingValue = NonNullable<AnalyticsQueryParams['grouping']>;

interface UseAnalyticsReturn {
  overview: AnalyticsOverviewResponse['overview'] | null;
  overviewResponse: AnalyticsOverviewResponse | null;
  history: AnalyticsHistoryResponse | null;
  isLoading: boolean;
  isHistoryLoading: boolean;
  isRefetching: boolean;
  error: AppError | null;
  range: AnalyticsRangeKey;
  grouping: AnalyticsQueryParams['grouping'];
  setRange: (range: AnalyticsRangeKey) => void;
  setGrouping: (grouping: AnalyticsGroupingValue) => void;
  refreshOverview: () => Promise<AnalyticsOverviewResponse>;
  refreshHistory: () => Promise<AnalyticsHistoryResponse>;
  exportReport: (params?: Partial<AnalyticsExportParams>) => void;
  exportStatus: AnalyticsExportStatus;
  exportResult: AnalyticsExportResult | null;
  exportError: AppError | null;
  isExporting: boolean;
  isOnboardingComplete: boolean;
  completeOnboarding: () => void;
  markOnboardingIncomplete: () => void;
  lastRefreshedAt: string | null;
  isDemoData: boolean;
  loadSampleData: () => Promise<AnalyticsOverviewResponse>;
}

export function useAnalytics(options: UseAnalyticsOptions = {}): UseAnalyticsReturn {
  const { enabled = true, rangeOverride } = options;
  const queryClient = useQueryClient();

  const range = useAnalyticsStore((state) => state.range);
  const grouping = useAnalyticsStore((state) => state.grouping);
  const exportStatus = useAnalyticsStore((state) => state.exportStatus);
  const exportResult = useAnalyticsStore((state) => state.exportResult);
  const exportError = useAnalyticsStore((state) => state.exportError);
  const isOnboardingComplete = useAnalyticsStore((state) => state.isOnboardingComplete);
  const lastRefreshedAt = useAnalyticsStore((state) => state.lastRefreshedAt);
  const isDemoData = useAnalyticsStore((state) => state.isDemoData);

  const setRangeState = useAnalyticsStore((state) => state.setRange);
  const setGroupingState = useAnalyticsStore((state) => state.setGrouping);
  const markOnboardingComplete = useAnalyticsStore((state) => state.markOnboardingComplete);
  const resetOnboarding = useAnalyticsStore((state) => state.resetOnboarding);
  const setExportStatus = useAnalyticsStore((state) => state.setExportStatus);
  const setExportResult = useAnalyticsStore((state) => state.setExportResult);
  const setExportError = useAnalyticsStore((state) => state.setExportError);
  const setLastRefreshedAt = useAnalyticsStore((state) => state.setLastRefreshedAt);
  const setIsDemoData = useAnalyticsStore((state) => state.setIsDemoData);

  const activeRange = rangeOverride ?? range;
  const overviewErrorRef = useRef<string | null>(null);

  const overviewQueryKey = useMemo(
    () => ['analytics', 'overview', activeRange, grouping] as const,
    [activeRange, grouping],
  );
  const overviewQuery = useQuery<AnalyticsOverviewResponse>({
    queryKey: overviewQueryKey,
    queryFn: () => fetchAnalyticsOverview({ range: activeRange, grouping }),
    enabled,
    staleTime: 60_000,
    gcTime: 5 * 60_000,
  });

  const historyQueryKey = useMemo(
    () => ['analytics', 'history', activeRange, grouping ?? 'day'] as const,
    [activeRange, grouping],
  );
  const historyQuery = useQuery<AnalyticsHistoryResponse>({
    queryKey: historyQueryKey,
    queryFn: () => fetchAnalyticsHistory({ range: activeRange, grouping }),
    enabled,
    staleTime: 60_000,
    gcTime: 5 * 60_000,
  });

  useEffect(() => {
    if (!overviewQuery.data) return;
    const data = overviewQuery.data;
    setLastRefreshedAt(data.overview.meta.generatedAt);
    setIsDemoData(Boolean(data.overview.meta.isDemo));
    queryClient.setQueryData(historyQueryKey, data.history);
    overviewErrorRef.current = null;
  }, [overviewQuery.data, setLastRefreshedAt, setIsDemoData, queryClient, historyQueryKey]);

  useEffect(() => {
    if (!overviewQuery.error) return;
    const appError = isAppError(overviewQuery.error)
      ? overviewQuery.error
      : toAppError(overviewQuery.error);
    const signature = `${appError.code}:${appError.message}`;
    if (overviewErrorRef.current === signature) return;
    overviewErrorRef.current = signature;
    notifyErrorToast(appError);
  }, [overviewQuery.error]);

  const exportMutation = useMutation<AnalyticsExportResult, unknown, AnalyticsExportParams>({
    mutationFn: (payload) => exportAnalyticsReport(payload),
    onMutate: () => {
      setExportStatus('loading');
      setExportError(null);
    },
    onSuccess: (result) => {
      setExportResult(result);
      setExportStatus('success');
      notifySuccessToast('分析报告已导出', '已生成最新分析报告，快去查看吧');
    },
    onError: (error) => {
      const appError = isAppError(error) ? error : toAppError(error);
      setExportError(appError);
      notifyErrorToast(appError);
    },
  });

  const handleExport = useCallback(
    (params?: Partial<AnalyticsExportParams>) => {
      const payload: AnalyticsExportParams = {
        range: activeRange,
        format: params?.format ?? 'markdown',
        from: params?.from,
        to: params?.to,
      };
      exportMutation.mutate(payload);
    },
    [activeRange, exportMutation],
  );

  const refreshOverview = useCallback(async () => {
    const result = await overviewQuery.refetch();
    if (result.data) return result.data;
    throw toAppError(result.error ?? new Error('刷新分析概览失败'));
  }, [overviewQuery]);

  const refreshHistory = useCallback(async () => {
    const result = await historyQuery.refetch();
    if (result.data) return result.data;
    throw toAppError(result.error ?? new Error('刷新分析趋势失败'));
  }, [historyQuery]);

  const overviewData = overviewQuery.data ?? null;
  const historyData = historyQuery.data ?? null;

  const overview = overviewData ? overviewData.overview : null;
  const history = historyData;

  useEffect(() => {
    if (!overview) return;
    if (!overview.zeroState?.isEmpty) {
      markOnboardingComplete();
    }
  }, [overview, markOnboardingComplete]);

  const error = overviewQuery.error
    ? isAppError(overviewQuery.error)
      ? overviewQuery.error
      : toAppError(overviewQuery.error)
    : null;

  const isLoading = overviewQuery.isLoading && enabled;
  const isHistoryLoading = historyQuery.isLoading && enabled;
  const isRefetching = overviewQuery.isRefetching;

  const exportState = useMemo(() => exportStatus, [exportStatus]);
  const isExporting = exportState === 'loading' || exportMutation.isPending;

  const handleSetGrouping = useCallback(
    (next: AnalyticsGroupingValue) => {
      setGroupingState(next);
    },
    [setGroupingState],
  );

  const loadSampleData = useCallback(async () => {
    const data = await fetchAnalyticsOverview({ range: '7d', grouping });
    setRangeState('7d');
    queryClient.setQueryData(overviewQueryKey, data);
    queryClient.setQueryData(historyQueryKey, data.history);
    setLastRefreshedAt(data.overview.meta.generatedAt);
    setIsDemoData(Boolean(data.overview.meta.isDemo));
    if (!data.overview.zeroState.isEmpty) {
      markOnboardingComplete();
    }
    return data;
  }, [
    grouping,
    markOnboardingComplete,
    queryClient,
    setIsDemoData,
    setLastRefreshedAt,
    setRangeState,
    overviewQueryKey,
    historyQueryKey,
  ]);

  return {
    overview,
    overviewResponse: overviewQuery.data ?? null,
    history,
    isLoading,
    isHistoryLoading,
    isRefetching,
    error,
    range: activeRange,
    grouping,
    setRange: setRangeState,
    setGrouping: handleSetGrouping,
    refreshOverview,
    refreshHistory,
    exportReport: handleExport,
    exportStatus: exportState,
    exportResult,
    exportError,
    isExporting,
    isOnboardingComplete,
    completeOnboarding: markOnboardingComplete,
    markOnboardingIncomplete: resetOnboarding,
    lastRefreshedAt,
    isDemoData,
    loadSampleData,
  } as UseAnalyticsReturn;
}
