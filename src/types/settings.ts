export type ThemePreference = 'system' | 'light' | 'dark';

export interface AppSettings {
  hasDeepseekKey: boolean;
  maskedDeepseekKey?: string | null;
  workdayStartMinute: number;
  workdayEndMinute: number;
  themePreference: ThemePreference;
  lastUpdatedAt: string | null;
  aiFeedbackOptOut?: boolean;
}

export interface UpdateAppSettingsInput {
  deepseekApiKey?: string | null;
  removeDeepseekKey?: boolean;
  workdayStartMinute?: number;
  workdayEndMinute?: number;
  themePreference?: ThemePreference;
  aiFeedbackOptOut?: boolean;
}

export interface AiProviderTelemetry {
  providerId?: string | null;
  model?: string | null;
  latencyMs?: number | null;
  tokensUsed?: Record<string, number> | null;
  extra?: Record<string, unknown> | null;
}

export interface AiStatus {
  status: 'online' | 'missing_key' | 'unavailable';
  hasApiKey: boolean;
  lastCheckedAt: string;
  latencyMs?: number | null;
  provider?: AiProviderTelemetry | null;
  message?: string | null;
}

export interface CacheClearResult {
  tasksCleared: number;
  planningSessionsCleared: number;
  recommendationsCleared: number;
  analyticsSnapshotsCleared: number;
  productivityScoresCleared: number;
  wellnessNudgesCleared: number;
  workloadForecastsCleared: number;
  aiFeedbackCleared: number;
  communityExportsCleared: number;
  aiCacheCleared: number;
}
