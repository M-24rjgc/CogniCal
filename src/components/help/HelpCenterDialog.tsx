import { useCallback, useEffect, useMemo } from 'react';
import { Dialog, DialogContent } from '../ui/dialog';
import HelpCenterContent, { type HelpResourceLinkItem } from './HelpCenterContent';
import type { ShortcutGroup } from '../keyboard/KeyboardShortcutsHelp';
import {
  CONTEXTUAL_HELP_ENTRIES,
  dispatchOnboardingReplayEvent,
  getOnboardingStepById,
  ONBOARDING_TOUR_STEPS,
} from '../../utils/onboarding';
import { useOnboardingStore } from '../../stores/onboardingStore';
import { useSettingsStore } from '../../stores/settingsStore';
import { pushToast } from '../../stores/uiStore';

const HELP_CENTER_DOCS_URL = 'https://docs.cognical.app/help-center';

export interface HelpCenterDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  shortcutGroups: ShortcutGroup[];
  onOpenShortcuts?: () => void;
  onNavigateToSettings?: () => void;
  onOpenDocs?: () => void;
  docsUrl?: string;
}

const isExternalLink = (href: string | undefined): boolean =>
  Boolean(href && /^https?:/i.test(href));

const formatDateTime = (value: string | null | undefined): string | null => {
  if (!value) return null;
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return null;
  }
  return new Intl.DateTimeFormat('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  }).format(date);
};

export function HelpCenterDialog({
  open,
  onOpenChange,
  shortcutGroups,
  onOpenShortcuts,
  onNavigateToSettings,
  onOpenDocs,
  docsUrl,
}: HelpCenterDialogProps) {
  const progress = useOnboardingStore((state) => state.progress);
  const requestReplay = useOnboardingStore((state) => state.requestReplay);
  const resetProgress = useOnboardingStore((state) => state.resetProgress);

  const settings = useSettingsStore((state) => state.settings);
  const isSettingsLoading = useSettingsStore((state) => state.isLoading);
  const settingsError = useSettingsStore((state) => state.error);
  const loadSettings = useSettingsStore((state) => state.loadSettings);

  useEffect(() => {
    if (!open) {
      return;
    }
    if (settings || isSettingsLoading) {
      return;
    }
    void loadSettings().catch(() => undefined);
  }, [open, settings, isSettingsLoading, loadSettings]);

  const pendingStep = useMemo(() => {
    if (progress.hasCompletedTour) {
      return null;
    }
    const completed = new Set(progress.completedStepIds);
    for (const step of ONBOARDING_TOUR_STEPS) {
      if (!completed.has(step.id)) {
        return step;
      }
    }
    return null;
  }, [progress.completedStepIds, progress.hasCompletedTour]);

  const lastCompletedTitle = useMemo(() => {
    if (!progress.lastStepId) {
      return null;
    }
    const step = getOnboardingStepById(progress.lastStepId);
    return step?.title ?? null;
  }, [progress.lastStepId]);

  const dismissedAtLabel = useMemo(
    () => formatDateTime(progress.dismissedAt ?? null),
    [progress.dismissedAt],
  );

  const handleReplayTour = useCallback(() => {
    requestReplay();
    dispatchOnboardingReplayEvent();
    pushToast({ title: '即将重新播放互动引导', variant: 'default' });
  }, [requestReplay]);

  const handleResetProgress = useCallback(() => {
    resetProgress();
    pushToast({ title: '已重置互动引导进度', variant: 'success' });
  }, [resetProgress]);

  const effectiveDocsUrl = docsUrl ?? HELP_CENTER_DOCS_URL;

  const handleOpenDocs = useCallback(() => {
    if (onOpenDocs) {
      onOpenDocs();
      return;
    }
    if (effectiveDocsUrl) {
      window.open(effectiveDocsUrl, '_blank', 'noopener');
    }
  }, [effectiveDocsUrl, onOpenDocs]);

  const handleNavigateToSettings = useCallback(() => {
    if (onNavigateToSettings) {
      onNavigateToSettings();
      return;
    }
    if (typeof window !== 'undefined') {
      window.location.hash = '#/settings';
    }
  }, [onNavigateToSettings]);

  const handleOpenShortcuts = useCallback(() => {
    if (onOpenShortcuts) {
      onOpenShortcuts();
      return;
    }
    pushToast({ title: '请使用 Shift + / 打开快捷键面板', variant: 'default' });
  }, [onOpenShortcuts]);

  const resources = useMemo<HelpResourceLinkItem[]>(() => {
    const unique = new Map<string, HelpResourceLinkItem>();
    for (const entry of Object.values(CONTEXTUAL_HELP_ENTRIES)) {
      if (!entry.links) continue;
      for (const link of entry.links) {
        if (!link.href) continue;
        const key = `${link.href}::${link.label}`;
        if (unique.has(key)) continue;
        unique.set(key, {
          label: link.label,
          href: link.href,
          external: link.external ?? isExternalLink(link.href),
        });
      }
    }
    return Array.from(unique.values()).slice(0, 8);
  }, []);

  const onboardingData = useMemo(
    () => ({
      totalSteps: ONBOARDING_TOUR_STEPS.length,
      completedSteps: progress.completedStepIds.length,
      hasCompletedTour: progress.hasCompletedTour,
      pendingStepTitle: pendingStep?.title ?? null,
      lastCompletedTitle,
      dismissedAtLabel,
    }),
    [dismissedAtLabel, lastCompletedTitle, pendingStep, progress],
  );

  const settingsData = useMemo(() => {
    const formatted = formatDateTime(settings?.lastUpdatedAt ?? null);
    return {
      hasDeepseekKey: settings?.hasDeepseekKey ?? false,
      maskedKey: settings?.maskedDeepseekKey ?? null,
      lastUpdatedLabel: formatted ?? '尚未保存',
      isLoading: isSettingsLoading,
      errorMessage: settingsError?.message ?? null,
    };
  }, [isSettingsLoading, settings, settingsError]);

  const shortcutPreview = useMemo(() => {
    if (shortcutGroups.length <= 4) {
      return shortcutGroups;
    }
    return shortcutGroups.slice(0, 4);
  }, [shortcutGroups]);

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-5xl">
        <HelpCenterContent
          onboarding={onboardingData}
          settings={settingsData}
          shortcutGroups={shortcutPreview}
          resources={resources}
          onReplayTour={handleReplayTour}
          onResetProgress={handleResetProgress}
          onOpenDocs={handleOpenDocs}
          onNavigateToSettings={handleNavigateToSettings}
          onOpenShortcuts={handleOpenShortcuts}
        />
      </DialogContent>
    </Dialog>
  );
}

export default HelpCenterDialog;
