import { beforeEach, describe, expect, it, vi } from 'vitest';
import {
  getPendingOnboardingStep,
  markOnboardingStepComplete,
  resetOnboardingProgress,
  selectOnboardingProgress,
  shouldAutoLaunchOnboardingTour,
  useOnboardingStore,
} from '../stores/onboardingStore';
import { ONBOARDING_TOUR_STEPS } from '../utils/onboarding';

describe('onboardingStore', () => {
  beforeEach(() => {
    resetOnboardingProgress();
    useOnboardingStore.setState({ replayRequestToken: null });
  });

  it('initialises with default progress and auto launch enabled', () => {
    const progress = selectOnboardingProgress();
    expect(progress.hasCompletedTour).toBe(false);
    expect(progress.completedStepIds).toHaveLength(0);
    expect(shouldAutoLaunchOnboardingTour()).toBe(true);
    expect(getPendingOnboardingStep()).toBe(ONBOARDING_TOUR_STEPS[0]!.id);
  });

  it('records completed steps and marks tour finished', () => {
    const allStepIds = ONBOARDING_TOUR_STEPS.map((step) => step.id);
    useOnboardingStore.getState().recordDismissal(allStepIds[0]);
    for (const stepId of allStepIds) {
      markOnboardingStepComplete(stepId);
    }

    const progress = selectOnboardingProgress();
    expect(progress.completedStepIds).toEqual(allStepIds);
    expect(progress.hasCompletedTour).toBe(true);
    expect(shouldAutoLaunchOnboardingTour()).toBe(false);
    expect(getPendingOnboardingStep()).toBeNull();
    expect(progress.dismissedAt).toBeNull();
    expect(progress.lastStepId).toBe(allStepIds[allStepIds.length - 1]);
  });

  it('disables auto launch after dismissal and re-enables after reset', () => {
    useOnboardingStore.getState().recordDismissal();
    expect(shouldAutoLaunchOnboardingTour()).toBe(false);

    resetOnboardingProgress();
    expect(shouldAutoLaunchOnboardingTour()).toBe(true);
  });

  it('issues a replay token that can be consumed once', () => {
    const store = useOnboardingStore.getState();
    store.recordDismissal();
    expect(shouldAutoLaunchOnboardingTour()).toBe(false);

    const token = store.requestReplay();
    expect(typeof token).toBe('string');
    expect(token.length).toBeGreaterThan(0);
    expect(shouldAutoLaunchOnboardingTour()).toBe(true);

    const consumed = store.consumeReplayRequest();
    expect(consumed).toBe(token);
    expect(store.consumeReplayRequest()).toBeNull();
    expect(useOnboardingStore.getState().replayRequestToken).toBeNull();
  });

  it('records dismissal metadata and keeps duplicate completions deduplicated', () => {
    vi.useFakeTimers();
    const timestamp = new Date('2025-10-18T08:15:00.000Z');
    vi.setSystemTime(timestamp);

    const firstStep = ONBOARDING_TOUR_STEPS[0]!.id;
    const secondStep = ONBOARDING_TOUR_STEPS[1]!.id;
    const store = useOnboardingStore.getState();

    store.markStepComplete(firstStep);
    store.recordDismissal(firstStep);
    store.markStepComplete(firstStep);
    store.markStepComplete(secondStep);

    const progress = selectOnboardingProgress();
    expect(progress.completedStepIds).toEqual([firstStep, secondStep]);
    expect(progress.lastStepId).toBe(secondStep);
    expect(progress.dismissedAt).toBe(timestamp.toISOString());

    vi.useRealTimers();
  });
});
