import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';

export interface ProjectInfo {
  name: string;
  version: string;
  license: string;
  repositoryUrl: string;
  docsUrl: string;
  contributingUrl: string;
  communityUrl: string;
  isOpenSource: boolean;
  featuresAlwaysFree: boolean;
}

export interface DetectedPlugin {
  name: string;
  version?: string;
  source: string;
  enabled: boolean;
  permissions: string[];
}

export interface SystemInfo {
  os: string;
  appVersion: string;
  locale: string;
  timestamp: string;
}

export interface AnonymizedMetrics {
  totalTasks: number;
  completedTasks: number;
  averageCompletionTimeMinutes?: number;
  totalSessions: number;
  productivityScoreAvailable: boolean;
  workloadForecastsCount: number;
  wellnessEventsCount: number;
}

export interface FeedbackSummary {
  totalFeedbackCount: number;
  positiveCount: number;
  negativeCount: number;
  surfaces: string[];
  mostCommonIssues: string[];
}

export interface ExportBundle {
  systemInfo: SystemInfo;
  metrics: AnonymizedMetrics;
  feedbackSummary?: FeedbackSummary;
  plugins: DetectedPlugin[];
  checksum: string;
}

// Get project information
export function useProjectInfo() {
  return useQuery<ProjectInfo>({
    queryKey: ['community', 'projectInfo'],
    queryFn: async () => {
      const result = await invoke<ProjectInfo>('community_get_project_info');
      return result;
    },
    staleTime: Infinity, // Never stale - static information
  });
}

// Detect installed plugins
export function useDetectPlugins() {
  return useQuery<DetectedPlugin[]>({
    queryKey: ['community', 'plugins'],
    queryFn: async () => {
      const result = await invoke<DetectedPlugin[]>('community_detect_plugins');
      return result;
    },
    staleTime: 5 * 60 * 1000, // 5 minutes
  });
}

// Generate export bundle
export function useGenerateExportBundle() {
  return useMutation<ExportBundle, Error, boolean>({
    mutationFn: async (includeFeedback: boolean) => {
      const result = await invoke<ExportBundle>('community_generate_export_bundle', {
        includeFeedback,
      });
      return result;
    },
  });
}

// Save export to file
export function useSaveExportToFile() {
  const queryClient = useQueryClient();

  return useMutation<number, Error, { bundle: ExportBundle; filePath: string }>({
    mutationFn: async ({ bundle, filePath }) => {
      const bundleJson = JSON.stringify(bundle);
      const result = await invoke<number>('community_save_export_to_file', {
        bundleJson,
        filePath,
      });
      return result;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['community', 'exports'] });
    },
  });
}

// List previous exports
export function useListExports() {
  return useQuery<Array<[number, string, string]>>({
    queryKey: ['community', 'exports'],
    queryFn: async () => {
      const result = await invoke<Array<[number, string, string]>>('community_list_exports');
      return result;
    },
    staleTime: 1 * 60 * 1000, // 1 minute
  });
}
