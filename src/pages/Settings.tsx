import { useEffect, useMemo, useRef, useState } from 'react';
import { z } from 'zod';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { AlertCircle, CheckCircle2, KeyRound, Loader2, RefreshCw, Satellite } from 'lucide-react';
import { useToast } from '../providers/toast-provider';
import { useTheme, type ThemeMode } from '../providers/theme-provider';
import { useSettingsStore } from '../stores/settingsStore';
import { usePurgeFeedback } from '../hooks/useFeedback';
import { useAI } from '../hooks/useAI';
import { Badge, type BadgeProps } from '../components/ui/badge';
import { Button } from '../components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '../components/ui/card';
import {
  Form,
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '../components/ui/form';
import { Input } from '../components/ui/input';
import { Skeleton } from '../components/ui/skeleton';
import type { AiStatus, ThemePreference, UpdateAppSettingsInput } from '../types/settings';
import { isAppError, type AppError } from '../services/tauriApi';
import DashboardSettingsForm from '../components/settings/DashboardSettingsForm';

import { HelpPopover } from '../components/help/HelpPopover';

const timePattern = /^([01]\d|2[0-3]):[0-5]\d$/;
const THEME_VALUES = ['system', 'light', 'dark'] as const;
const THEME_OPTIONS: Array<{ value: ThemePreference; label: string }> = [
  { value: 'system', label: '跟随系统' },
  { value: 'light', label: '浅色' },
  { value: 'dark', label: '深色' },
];

type AiConnectionState = 'online' | 'missing_key' | 'unavailable' | 'unknown';

const AI_STATUS_LABELS: Record<AiConnectionState, string> = {
  online: '在线',
  missing_key: '未配置',
  unavailable: '不可用',
  unknown: '待检测',
};

const AI_STATUS_BADGE_VARIANTS: Record<AiConnectionState, BadgeProps['variant']> = {
  online: 'default',
  missing_key: 'destructive',
  unavailable: 'secondary',
  unknown: 'muted',
};

const deriveConnectionState = (
  status: AiStatus | null,
  hasApiKey: boolean,
  error: AppError | null,
): AiConnectionState => {
  if (!hasApiKey || status?.status === 'missing_key' || error?.code === 'MISSING_API_KEY') {
    return 'missing_key';
  }

  if (error) {
    return 'unavailable';
  }

  if (!status) {
    return 'unknown';
  }

  if (status.status === 'unavailable') {
    return 'unavailable';
  }

  if (status.status === 'online') {
    return 'online';
  }

  return 'unknown';
};

const settingsFormSchema = z
  .object({
    deepseekKey: z
      .string()
      .max(256, '密钥长度需小于 256 字符')
      .optional()
      .transform((value) => value ?? ''),
    workdayStart: z.string().regex(timePattern, '请选择有效的开始时间'),
    workdayEnd: z.string().regex(timePattern, '请选择有效的结束时间'),
    themePreference: z.enum(THEME_VALUES),
  })
  .superRefine((value, ctx) => {
    const startMinute = timeStringToMinute(value.workdayStart);
    const endMinute = timeStringToMinute(value.workdayEnd);
    if (startMinute !== null && endMinute !== null && endMinute <= startMinute) {
      ctx.addIssue({
        code: 'custom',
        path: ['workdayEnd'],
        message: '结束时间需晚于开始时间',
      });
    }
  });

type SettingsFormValues = z.infer<typeof settingsFormSchema>;

export default function SettingsPage() {
  const settings = useSettingsStore((state) => state.settings);
  const isLoading = useSettingsStore((state) => state.isLoading);
  const isSaving = useSettingsStore((state) => state.isSaving);
  const error = useSettingsStore((state) => state.error);
  const loadSettings = useSettingsStore((state) => state.loadSettings);
  const updateSettings = useSettingsStore((state) => state.updateSettings);
  const clearError = useSettingsStore((state) => state.clearError);

  const { notify } = useToast();
  const { setTheme } = useTheme();
  const {
    aiStatus,
    isTesting: isTestingAi,
    statusError: aiStatusError,
    refreshStatus,
    testConnection,
  } = useAI();

  const [isRefreshingAiStatus, setIsRefreshingAiStatus] = useState(false);

  const isLoadingRef = useRef(false);
  const hasAttemptedRef = useRef(false);
  const hasLoadedAiStatusRef = useRef(false);

  useEffect(() => {
    if (settings || isLoadingRef.current || hasAttemptedRef.current) {
      return;
    }

    const loadSettingsOnce = async () => {
      if (isLoadingRef.current) {
        return;
      }
      isLoadingRef.current = true;
      hasAttemptedRef.current = true;

      clearError();
      try {
        await loadSettings();
      } catch (err) {
        const message = isAppError(err) ? err.message : '请稍后重试。';
        notify({ title: '加载设置失败', description: message, variant: 'error' });
      } finally {
        isLoadingRef.current = false;
      }
    };

    void loadSettingsOnce();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [settings, clearError, loadSettings]);

  useEffect(() => {
    if (isLoading) return;
    if (hasLoadedAiStatusRef.current) return;
    hasLoadedAiStatusRef.current = true;
    void refreshStatus().catch(() => undefined);
  }, [isLoading, refreshStatus]);

  const defaultValues = useMemo<SettingsFormValues>(
    () => ({
      deepseekKey: '',
      workdayStart: minuteToTimeString(settings?.workdayStartMinute ?? 9 * 60),
      workdayEnd: minuteToTimeString(settings?.workdayEndMinute ?? 18 * 60),
      themePreference: settings?.themePreference ?? 'system',
    }),
    [settings],
  );

  const form = useForm<SettingsFormValues>({
    resolver: zodResolver(settingsFormSchema),
    defaultValues,
  });

  useEffect(() => {
    form.reset(defaultValues);
  }, [defaultValues, form]);

  const handleSubmit = async (values: SettingsFormValues) => {
    clearError();
    const startMinute = timeStringToMinute(values.workdayStart);
    const endMinute = timeStringToMinute(values.workdayEnd);
    if (startMinute === null || endMinute === null) return;

    const payload: UpdateAppSettingsInput = {
      workdayStartMinute: startMinute,
      workdayEndMinute: endMinute,
      themePreference:
       values.themePreference,
    };

    if (values.deepseekKey && values.deepseekKey.trim().length > 0) {
      payload.deepseekApiKey = values.deepseekKey.trim();
    }

    try {
      const result = await updateSettings(payload);
      notify({
        title: '设置已更新',
        description: '配置已保存并将在桌面端同步。',
        variant: 'success',
      });
      form.reset({
        deepseekKey: '',
        workdayStart: minuteToTimeString(result.workdayStartMinute),
        workdayEnd: minuteToTimeString(result.workdayEndMinute),
        themePreference: result.themePreference,
      });
      setTheme(result.themePreference as ThemeMode);
      try {
        await refreshStatus();
      } catch (statusErr) {
        const message = isAppError(statusErr) ? statusErr.message : '请稍后重试。';
        notify({
          title: '刷新 AI 状态失败',
          description: message,
          variant: 'warning',
        });
      }
    } catch (err) {
      const message = isAppError(err) ? err.message : '请稍后重试。';
      notify({ title: '保存失败', description: message, variant: 'error' });
    }
  };

  const handleRemoveKey = async () => {
    clearError();
    try {
      const result = await updateSettings({ removeDeepseekKey: true });
      notify({
        title: '已清除 DeepSeek 密钥',
        description: '后续需要重新填写密钥以启用 AI 功能。',
        variant: 'warning',
      });
      form.reset({
        deepseekKey: '',
        workdayStart: minuteToTimeString(result.workdayStartMinute),
        workdayEnd: minuteToTimeString(result.workdayEndMinute),
        themePreference: result.themePreference,
      });
      try {
        await refreshStatus();
      } catch (statusErr) {
        const message = isAppError(statusErr) ? statusErr.message : '请稍后重试。';
        notify({
          title: '刷新 AI 状态失败',
          description: message,
          variant: 'warning',
        });
      }
    } catch (err) {
      const message = isAppError(err) ? err.message : '请稍后重试。';
      notify({ title: '清除失败', description: message, variant: 'error' });
    }
  };

  const handleRefreshStatus = async () => {
    if (isAiStatusBusy) return;
    setIsRefreshingAiStatus(true);
    try {
      await refreshStatus();
    } catch (err) {
      const message = isAppError(err) ? err.message : '请稍后重试。';
      notify({ title: '刷新状态失败', description: message, variant: 'error' });
    } finally {
      setIsRefreshingAiStatus(false);
    }
  };

  const handleTestConnection = async () => {
    try {
      const status = await testConnection();
      const derivedState = deriveConnectionState(status, status.hasApiKey, null);
      const label = AI_STATUS_LABELS[derivedState];
      const variant =
        derivedState === 'online'
          ? 'success'
          : derivedState === 'missing_key'
            ? 'warning'
            : 'error';
      const description = (() => {
        if (derivedState === 'online') {
          return status.latencyMs ? `服务可用，延迟约 ${status.latencyMs} ms。` : '服务可用。';
        }
        if (derivedState === 'missing_key') {
          return '未检测到有效的 DeepSeek API Key，请先配置密钥。';
        }
        return status.message ?? 'DeepSeek 服务暂不可用，请稍后再试。';
      })();
      notify({
        title: `DeepSeek 状态：${label}`,
        description,
        variant,
      });
    } catch (err) {
      const message = isAppError(err) ? err.message : '请稍后重试。';
      notify({ title: '测试连接失败', description: message, variant: 'error' });
    }
  };

  const deepseekStatus = settings?.hasDeepseekKey ? '已配置' : '未配置';
  const deepseekBadgeVariant = settings?.hasDeepseekKey ? 'secondary' : 'destructive';
  const maskedKey = settings?.maskedDeepseekKey ?? null;
  const lastUpdatedLabel = settings?.lastUpdatedAt
    ? new Date(settings.lastUpdatedAt).toLocaleString('zh-CN')
    : '尚未保存';
  const showSkeleton = isLoading && !settings;

  const connectionState = deriveConnectionState(
    aiStatus,
    settings?.hasDeepseekKey ?? false,
    aiStatusError,
  );
  const statusLabel = AI_STATUS_LABELS[connectionState];
  const statusBadgeVariant = AI_STATUS_BADGE_VARIANTS[connectionState];
  const statusLatency = aiStatus?.latencyMs ?? aiStatus?.provider?.latencyMs ?? null;
  const latencyLabel = statusLatency !== null ? `${statusLatency} ms` : '—';
  const providerLabel = aiStatus?.provider?.providerId
    ? aiStatus.provider.providerId
    : connectionState === 'online'
      ? 'DeepSeek'
      : '—';
  const modelLabel = aiStatus?.provider?.model
    ? aiStatus.provider.model
    : connectionState === 'online'
      ? 'deepseek-chat'
      : '—';
  const lastCheckedLabel = aiStatus
    ? new Date(aiStatus.lastCheckedAt).toLocaleString('zh-CN')
    : '尚未检测';
  const rawStatusMessage = aiStatus?.message ?? null;
  const unavailableMessage =
    connectionState === 'unavailable'
      ? (aiStatusError?.message ?? rawStatusMessage ?? 'DeepSeek 服务暂不可用，请稍后再试。')
      : null;
  const missingKeyMessage =
    connectionState === 'missing_key'
      ? '未检测到有效的 DeepSeek API Key，请在左侧填写密钥并保存后再刷新状态。'
      : null;
  const onlineInfoMessage =
    connectionState === 'online' && rawStatusMessage ? rawStatusMessage : null;
  const statusHelper = (() => {
    switch (connectionState) {
      case 'online':
        return statusLatency !== null ? `服务可用，最近延迟 ${latencyLabel}` : '服务可用。';
      case 'missing_key':
        return '尚未配置 DeepSeek API Key。';
      case 'unavailable':
        return unavailableMessage ?? 'DeepSeek 服务暂不可用，请稍后再试。';
      default:
        return '点击“刷新”获取最新的连接状态。';
    }
  })();
  const statusHelperClass = (() => {
    switch (connectionState) {
      case 'online':
        return 'text-emerald-600 dark:text-emerald-300';
      case 'missing_key':
        return 'text-amber-600';
      case 'unavailable':
        return 'text-destructive';
      default:
        return 'text-muted-foreground';
    }
  })();
  const isAiStatusBusy = isTestingAi || isRefreshingAiStatus;

  return (
    <section className="flex h-full flex-1 flex-col gap-6">
      <header className="flex flex-col gap-4 rounded-3xl border border-border/70 bg-background/80 p-6 shadow-sm">
        <div className="flex flex-wrap items-center justify-between gap-4">
          <div className="space-y-2">
            <div className="flex items-center gap-2">
              <Badge variant={deepseekBadgeVariant} className="flex items-center gap-1">
                <KeyRound className="h-3.5 w-3.5" /> DeepSeek {deepseekStatus}
              </Badge>
            </div>
            <div className="flex items-center gap-2">
              <h1 className="text-2xl font-semibold text-foreground">应用设置中心</h1>
              <HelpPopover
                entryId="settings-api-key"
                triggerLabel="查看配置 DeepSeek API Key 帮助"
                triggerClassName="ml-1"
              />
            </div>
            <p className="text-sm text-muted-foreground">
              管理 AI 接入、工作时间段与主题偏好，确保分析仪表盘能够获得完整上下文。
            </p>
          </div>
          <div className="flex flex-col items-end gap-1 text-xs text-muted-foreground">
            <span>最近更新：{lastUpdatedLabel}</span>
            <Button
              type="button"
              variant="outline"
              size="sm"
              className="inline-flex items-center gap-1"
              onClick={async () => {
                hasAttemptedRef.current = false;
                clearError();
                try {
                  await loadSettings();
                  await refreshStatus();
                } catch (err) {
                  const message = isAppError(err) ? err.message : '请稍后重试。';
                  notify({ title: '加载设置失败', description: message, variant: 'error' });
                }
              }}
              disabled={isLoading}
            >
              {isLoading ? (
                <Loader2 className="h-3.5 w-3.5 animate-spin" />
              ) : (
                <RefreshCw className="h-3.5 w-3.5" />
              )}
              重新加载
            </Button>
          </div>
        </div>
      </header>

      {error ? (
        <div className="rounded-2xl border border-destructive/40 bg-destructive/10 p-4 text-sm text-destructive">
          <div className="flex items-center gap-2">
            <AlertCircle className="h-4 w-4" />
            <span>{error.message}</span>
          </div>
        </div>
      ) : null}

      {showSkeleton ? (
        <div className="grid gap-6 lg:grid-cols-[minmax(0,3fr)_minmax(0,2fr)]">
          <Skeleton className="h-[420px] w-full rounded-3xl" />
          <Skeleton className="h-[320px] w-full rounded-3xl" />
        </div>
      ) : (
        <div className="grid gap-6 lg:grid-cols-[minmax(0,3fr)_minmax(0,2fr)]">
          <div className="flex flex-col gap-4">
            <Card className="rounded-3xl border-border/70 bg-card/80 shadow-sm">
              <CardHeader>
                <CardTitle className="text-lg">核心配置</CardTitle>
                <p className="text-sm text-muted-foreground">
                  保存后将立即同步到本地引擎，并影响分析仪表盘与规划建议。
                </p>
              </CardHeader>
              <CardContent>
                <Form {...form}>
                  <form className="grid gap-6" onSubmit={form.handleSubmit(handleSubmit)}>
                    <FormField
                      control={form.control}
                      name="deepseekKey"
                      render={({ field }) => (
                        <FormItem data-onboarding="settings-api-key">
                          <FormLabel>DeepSeek API Key</FormLabel>
                          <FormControl>
                            <Input
                              type="password"
                              autoComplete="off"
                              placeholder={
                                settings?.hasDeepseekKey
                                  ? '重新输入可更新现有密钥'
                                  : '例如：sk-xxxx...'
                              }
                              {...field}
                            />
                          </FormControl>
                          <FormDescription>
                            仅用于本地调用，不会上传至云端。
                            {maskedKey ? ` 当前掩码：${maskedKey}` : ''}
                          </FormDescription>
                          <FormMessage />
                        </FormItem>
                      )}
                    />

                    <div className="grid gap-4 sm:grid-cols-2">
                      <FormField
                        control={form.control}
                        name="workdayStart"
                        render={({ field }) => (
                          <FormItem>
                            <FormLabel>工作日开始时间</FormLabel>
                            <FormControl>
                              <Input type="time" step={60} {...field} />
                            </FormControl>
                            <FormDescription>影响计划生成与专注建议。</FormDescription>
                            <FormMessage />
                          </FormItem>
                        )}
                      />
                      <FormField
                        control={form.control}
                        name="workdayEnd"
                        render={({ field }) => (
                          <FormItem>
                            <FormLabel>工作日结束时间</FormLabel>
                            <FormControl>
                              <Input type="time" step={60} {...field} />
                            </FormControl>
                            <FormDescription>需晚于开始时间以获得有效日程。</FormDescription>
                            <FormMessage />
                          </FormItem>
                        )}
                      />
                    </div>

                    <FormField
                      control={form.control}
                      name="themePreference"
                      render={({ field }) => (
                        <FormItem>
                          <FormLabel>主题偏好</FormLabel>
                          <FormControl>
                            <select
                              className="h-10 w-full rounded-lg border border-border/60 bg-background px-3 text-sm text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary"
                              value={field.value}
                              onChange={field.onChange}
                            >
                              {THEME_OPTIONS.map((option) => (
                                <option key={option.value} value={option.value}>
                                  {option.label}
                                </option>
                              ))}
                            </select>
                          </FormControl>
                          <FormDescription>保存后将立即应用到当前界面。</FormDescription>
                          <FormMessage />
                        </FormItem>
                      )}
                    />

                    <div className="flex flex-wrap items-center justify-between gap-3">
                      <div className="flex items-center gap-2 text-xs text-muted-foreground">
                        <CheckCircle2 className="h-4 w-4 text-emerald-500" />
                        设置保存后自动加载，刷新应用即可回显。
                      </div>
                      <div className="flex flex-wrap gap-2">
                        {settings?.hasDeepseekKey ? (
                          <Button
                            type="button"
                            variant="outline"
                            onClick={handleRemoveKey}
                            disabled={isSaving}
                          >
                            清除密钥
                          </Button>
                        ) : null}
                        <Button type="submit" disabled={isSaving}>
                          {isSaving ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : null}
                          保存设置
                        </Button>
                      </div>
                    </div>
                  </form>
                </Form>
              </CardContent>
            </Card>

            <DashboardSettingsForm />


          </div>

          <aside className="flex flex-col gap-4">
            <Card className="rounded-3xl border-border/70 bg-card/80 shadow-sm">
              <CardHeader>
                <CardTitle className="flex items-center gap-2 text-base">
                  <Satellite className="h-4 w-4 text-primary" />
                  AI 连接状态
                </CardTitle>
                <p className="text-sm text-muted-foreground">
                  检查 DeepSeek 接入情况，确认密钥配置与服务可用性。
                </p>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="flex flex-wrap items-center justify-between gap-3">
                  <div className="space-y-2">
                    <Badge variant={statusBadgeVariant} className="flex items-center gap-1">
                      {isAiStatusBusy ? (
                        <Loader2 className="h-3 w-3 animate-spin" />
                      ) : (
                        <Satellite className="h-3 w-3" />
                      )}
                      {statusLabel}
                    </Badge>
                    <p className="text-xs text-muted-foreground">最后检测：{lastCheckedLabel}</p>
                    <p className={`text-xs ${statusHelperClass}`}>{statusHelper}</p>
                  </div>
                  <div className="flex flex-wrap gap-2">
                    <Button
                      type="button"
                      variant="outline"
                      size="sm"
                      onClick={handleRefreshStatus}
                      disabled={isAiStatusBusy}
                    >
                      {isRefreshingAiStatus ? (
                        <Loader2 className="mr-2 h-3.5 w-3.5 animate-spin" />
                      ) : (
                        <RefreshCw className="mr-2 h-3.5 w-3.5" />
                      )}
                      刷新
                    </Button>
                    <Button
                      type="button"
                      size="sm"
                      onClick={handleTestConnection}
                      disabled={isTestingAi}
                    >
                      {isTestingAi ? (
                        <Loader2 className="mr-2 h-3.5 w-3.5 animate-spin" />
                      ) : (
                        <CheckCircle2 className="mr-2 h-3.5 w-3.5" />
                      )}
                      测试连接
                    </Button>
                  </div>
                </div>

                {connectionState === 'online' && aiStatus ? (
                  <dl className="grid gap-2 text-xs text-muted-foreground">
                    <div className="flex items-center justify-between">
                      <dt className="text-foreground">提供者</dt>
                      <dd>{providerLabel}</dd>
                    </div>
                    <div className="flex items-center justify-between">
                      <dt className="text-foreground">模型</dt>
                      <dd>{modelLabel}</dd>
                    </div>
                    <div className="flex items-center justify-between">
                      <dt className="text-foreground">延迟</dt>
                      <dd>{latencyLabel}</dd>
                    </div>
                  </dl>
                ) : null}

                {onlineInfoMessage ? (
                  <div className="rounded-lg border border-amber-300/40 bg-amber-500/10 px-3 py-2 text-xs text-amber-600">
                    {onlineInfoMessage}
                  </div>
                ) : null}

                {missingKeyMessage ? (
                  <div className="rounded-lg border border-amber-300/40 bg-amber-500/10 px-3 py-2 text-xs text-amber-600">
                    {missingKeyMessage}
                  </div>
                ) : null}

                {unavailableMessage ? (
                  <div className="flex items-center gap-2 rounded-lg border border-destructive/40 bg-destructive/10 px-3 py-2 text-xs text-destructive">
                    <AlertCircle className="h-3.5 w-3.5" />
                    <span>{unavailableMessage}</span>
                  </div>
                ) : null}
              </CardContent>
            </Card>

            <Card className="rounded-3xl border-border/60 bg-background/80">
              <CardHeader>
                <CardTitle className="text-base">为什么需要这些设置？</CardTitle>
              </CardHeader>
              <CardContent className="space-y-3 text-sm text-muted-foreground">
                <p>
                  DeepSeek API Key
                  用于生成任务洞察、效率建议、规划偏好等智能服务；为空时分析仪表盘会进入零状态并提供示例数据。
                </p>
                <p>
                  工作日时间段将用于智能规划、冲突检测与专注区块建议，建议按照真实工作节奏填写。
                </p>
                <p>主题偏好会同步到全局主题管理器，并在桌面端下次启动时沿用。</p>
              </CardContent>
            </Card>

            <Card className="rounded-3xl border-primary/40 bg-primary/5">
              <CardHeader>
                <CardTitle className="text-base text-primary">与分析仪表盘的联动</CardTitle>
              </CardHeader>
              <CardContent className="space-y-2 text-sm text-primary">
                <p>配置完成后，仪表盘的导出报告、效率洞察与冲突提醒会根据你的偏好动态刷新。</p>
                <p>若尚未准备真实数据，可在仪表盘中加载示例数据体验完整流程。</p>
              </CardContent>
            </Card>

            <FeedbackPrivacyCard />

            <DataManagementCard />
          </aside>
        </div>
      )}
    </section>
  );
}

function DataManagementCard() {
  const [isClearing, setIsClearing] = useState(false);
  const { notify } = useToast();

  const handleClearAllCache = async () => {
    if (
      !confirm(
        '⚠️ 确定要清除所有缓存数据吗？\n\n这将删除:\n• 所有任务\n• 规划会话\n• 推荐记录\n• 分析快照\n• 效率评分\n• 健康提醒\n• 工作量预测\n• AI 反馈\n• AI 缓存\n• 社区导出记录\n\n⚠️ 此操作不可撤销！设置和 API 密钥将保留。',
      )
    ) {
      return;
    }

    setIsClearing(true);
    try {
      const { clearAllCache } = await import('../services/tauriApi');
      const result = await clearAllCache();

      const total =
        result.tasksCleared +
        result.planningSessionsCleared +
        result.recommendationsCleared +
        result.analyticsSnapshotsCleared +
        result.productivityScoresCleared +
        result.wellnessNudgesCleared +
        result.workloadForecastsCleared +
        result.aiFeedbackCleared +
        result.communityExportsCleared +
        result.aiCacheCleared;

      notify({
        title: '缓存已清除',
        description: `已删除 ${total} 条记录 (任务: ${result.tasksCleared}, 规划: ${result.planningSessionsCleared}, AI 缓存: ${result.aiCacheCleared})`,
        variant: 'success',
      });
    } catch (error) {
      const message = error instanceof Error ? error.message : '未知错误';
      notify({
        title: '清除失败',
        description: message,
        variant: 'error',
      });
    } finally {
      setIsClearing(false);
    }
  };

  return (
    <Card className="rounded-3xl border-destructive/40 bg-destructive/5">
      <CardHeader>
        <CardTitle className="text-base text-destructive">数据管理</CardTitle>
      </CardHeader>
      <CardContent className="space-y-3">
        <p className="text-sm text-muted-foreground">
          清除所有本地缓存数据以重置应用状态。此操作将删除任务、规划、分析等所有记录,但保留设置和
          API 密钥。
        </p>
        <Button
          variant="destructive"
          size="sm"
          className="w-full"
          onClick={handleClearAllCache}
          disabled={isClearing}
        >
          {isClearing ? <Loader2 className="mr-2 h-3.5 w-3.5 animate-spin" /> : null}
          清除所有缓存数据
        </Button>
        <p className="text-xs text-destructive/80">⚠️ 此操作不可撤销,请谨慎使用</p>
      </CardContent>
    </Card>
  );
}

function FeedbackPrivacyCard() {
  const settings = useSettingsStore((state) => state.settings);
  const updateSettings = useSettingsStore((state) => state.updateSettings);
  const { notify } = useToast();
  const { mutate: purgeFeedback, isPending: isPurging } = usePurgeFeedback();

  const [isOptOutLoading, setIsOptOutLoading] = useState(false);
  const isOptedOut = settings?.aiFeedbackOptOut === true;

  const handleOptOutToggle = async () => {
    setIsOptOutLoading(true);
    try {
      await updateSettings({ aiFeedbackOptOut: !isOptedOut });
      notify({
        title: isOptedOut ? '已启用 AI 反馈' : '已禁用 AI 反馈',
        description: isOptedOut
          ? '你可以继续为 AI 功能提供反馈'
          : '不再收集反馈数据，现有数据仍会保留',
        variant: 'default',
      });
    } catch (error) {
      notify({
        title: '设置失败',
        description: error instanceof Error ? error.message : '未知错误',
        variant: 'error',
      });
    } finally {
      setIsOptOutLoading(false);
    }
  };

  const handlePurge = () => {
    if (!confirm('确定要永久删除所有 AI 反馈数据吗？此操作不可撤销。')) {
      return;
    }

    purgeFeedback(undefined, {
      onSuccess: (deletedCount) => {
        notify({
          title: '数据已清除',
          description: `已删除 ${deletedCount} 条反馈记录`,
          variant: 'default',
        });
      },
      onError: (error) => {
        notify({
          title: '清除失败',
          description: error instanceof Error ? error.message : '未知错误',
          variant: 'error',
        });
      },
    });
  };

  return (
    <Card className="rounded-3xl border-border/60 bg-background/80">
      <CardHeader>
        <CardTitle className="text-base">AI 反馈与隐私</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <p className="text-sm font-medium">反馈收集</p>
              <p className="text-xs text-muted-foreground">为 AI 功能提供 👍/👎 反馈</p>
            </div>
            <Button
              variant={isOptedOut ? 'outline' : 'default'}
              size="sm"
              onClick={handleOptOutToggle}
              disabled={isOptOutLoading}
            >
              {isOptOutLoading ? (
                <Loader2 className="h-3.5 w-3.5 animate-spin" />
              ) : isOptedOut ? (
                '已禁用'
              ) : (
                '已启用'
              )}
            </Button>
          </div>
          <p className="text-xs text-muted-foreground">反馈数据仅存储在本地，用于生成改进建议</p>
        </div>

        <div className="space-y-2">
          <p className="text-sm font-medium">数据管理</p>
          <Button
            variant="destructive"
            size="sm"
            className="w-full"
            onClick={handlePurge}
            disabled={isPurging || isOptedOut}
          >
            {isPurging ? <Loader2 className="mr-2 h-3.5 w-3.5 animate-spin" /> : null}
            清除所有反馈数据
          </Button>
          <p className="text-xs text-muted-foreground">永久删除所有历史反馈记录</p>
        </div>
      </CardContent>
    </Card>
  );
}

function timeStringToMinute(value: string): number | null {
  if (!timePattern.test(value)) return null;
  const [hours, minutes] = value.split(':');
  const parsedHours = Number.parseInt(hours ?? '0', 10);
  const parsedMinutes = Number.parseInt(minutes ?? '0', 10);
  if (Number.isNaN(parsedHours) || Number.isNaN(parsedMinutes)) return null;
  const total = parsedHours * 60 + parsedMinutes;
  return Math.max(0, Math.min(total, 24 * 60));
}

function minuteToTimeString(totalMinutes: number): string {
  const normalized = Math.max(0, Math.min(Math.round(totalMinutes), 24 * 60 - 1));
  const hours = Math.floor(normalized / 60);
  const minutes = normalized % 60;
  return `${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}`;
}
