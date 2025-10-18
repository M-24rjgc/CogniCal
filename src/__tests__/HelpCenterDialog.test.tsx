import '@testing-library/jest-dom/vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { HelpCenterDialog } from '../components/help/HelpCenterDialog';
import { useOnboardingStore } from '../stores/onboardingStore';
import { useSettingsStore } from '../stores/settingsStore';
import { ONBOARDING_TOUR_STEPS } from '../utils/onboarding';

describe('HelpCenterDialog', () => {
  beforeEach(() => {
    useOnboardingStore.getState().resetProgress();
    useOnboardingStore.setState({ replayRequestToken: null });

    const settingsState = useSettingsStore.getState();
    useSettingsStore.setState({
      settings: null,
      isLoading: false,
      isSaving: false,
      aiStatus: null,
      isTestingAi: false,
      error: null,
      dashboardConfig: settingsState.dashboardConfig,
      isDashboardConfigLoading: false,
      isDashboardConfigSaving: false,
      dashboardConfigError: null,
    });
  });

  it('renders onboarding progress and wires quick actions', async () => {
    const user = userEvent.setup();
    const firstStep = ONBOARDING_TOUR_STEPS[0]!.id;

    useOnboardingStore.setState({
      progress: {
        version: 1,
        hasCompletedTour: false,
        completedStepIds: [firstStep],
        lastStepId: firstStep,
        dismissedAt: '2025-10-17T08:00:00.000Z',
      },
    });

    const settingsState = useSettingsStore.getState();
    useSettingsStore.setState({
      settings: {
        hasDeepseekKey: true,
        maskedDeepseekKey: '****ABCD',
        workdayStartMinute: 540,
        workdayEndMinute: 1080,
        themePreference: 'system',
        lastUpdatedAt: '2025-10-16T10:20:00.000Z',
        dashboardConfig: settingsState.dashboardConfig,
      },
      isLoading: false,
      error: null,
    });

    const onOpenShortcuts = vi.fn();
    const onNavigateToSettings = vi.fn();
    const onOpenDocs = vi.fn();
    const onOpenChange = vi.fn();
    const dispatchSpy = vi.spyOn(window, 'dispatchEvent');

    try {
      render(
        <HelpCenterDialog
          open
          onOpenChange={onOpenChange}
          shortcutGroups={[
            {
              title: '常用',
              shortcuts: [{ keys: 'Ctrl + K', description: '打开指令面板' }],
            },
          ]}
          onOpenShortcuts={onOpenShortcuts}
          onNavigateToSettings={onNavigateToSettings}
          onOpenDocs={onOpenDocs}
        />,
      );

      expect(screen.getByRole('heading', { name: '帮助与支持中心' })).toBeInTheDocument();
      expect(screen.getByText('互动引导进度')).toBeInTheDocument();
      expect(screen.getByText(`下一步：${ONBOARDING_TOUR_STEPS[1]!.title}`)).toBeInTheDocument();
      expect(screen.getByText('DeepSeek API 配置提醒')).toBeInTheDocument();
      expect(screen.getByText('****ABCD')).toBeInTheDocument();

      await user.click(screen.getByRole('button', { name: '查看全部' }));
      expect(onOpenShortcuts).toHaveBeenCalledTimes(1);

      await user.click(screen.getByRole('button', { name: '前往设置' }));
      expect(onNavigateToSettings).toHaveBeenCalledTimes(1);

      await user.click(screen.getByRole('button', { name: '查看配置指南' }));
      expect(onOpenDocs).toHaveBeenCalledTimes(1);

      await user.click(screen.getByRole('button', { name: '重新播放引导' }));
      expect(dispatchSpy).toHaveBeenCalled();
      const lastCall = dispatchSpy.mock.calls[dispatchSpy.mock.calls.length - 1];
      const dispatchedEvent = lastCall?.[0];
      expect(dispatchedEvent).toBeInstanceOf(CustomEvent);
      expect(dispatchedEvent?.type).toBe('onboarding:replay');
      expect(useOnboardingStore.getState().replayRequestToken).toBeTruthy();

      await user.click(screen.getByRole('button', { name: '重置进度' }));
      expect(useOnboardingStore.getState().progress.completedStepIds).toHaveLength(0);
      expect(useOnboardingStore.getState().replayRequestToken).toBeNull();
    } finally {
      dispatchSpy.mockRestore();
    }
  });
});
