import { expect, test } from '@playwright/test';
import {
  ONBOARDING_REPLAY_EVENT,
  dispatchOnboardingReplayEvent,
  isOverlayActive,
} from '../src/utils/onboarding';

const dispatchReplaySource = dispatchOnboardingReplayEvent
  .toString()
  .replace('ONBOARDING_REPLAY_EVENT', JSON.stringify(ONBOARDING_REPLAY_EVENT));

const isOverlayActiveSource = isOverlayActive.toString();

const dispatchReplayEval = `(${dispatchReplaySource})`;
const overlayEval = `(${isOverlayActiveSource})`;

test.beforeEach(async ({ page }) => {
  await page.goto('about:blank');
  await page.evaluate(
    ([replaySource, overlaySource]) => {
      const replay = eval(replaySource) as () => void;
      const overlay = eval(overlaySource) as typeof isOverlayActive;

      (window as unknown as Record<string, unknown>).dispatchReplayFromSpec = replay;
      (window as unknown as Record<string, unknown>).isOverlayActiveFromSpec = overlay;
    },
    [dispatchReplayEval, overlayEval],
  );
});

test('emits onboarding replay event to window listeners', async ({ page }) => {
  const result = await page.evaluate((eventName) => {
    return new Promise<string>((resolve, reject) => {
      const status = document.createElement('div');
      status.id = 'status';
      status.textContent = 'idle';
      document.body.appendChild(status);

      const timeoutId = window.setTimeout(() => reject(new Error('event-timeout')), 4000);

      window.addEventListener(
        eventName,
        () => {
          window.clearTimeout(timeoutId);
          status.textContent = 'replay-received';
          resolve(status.textContent);
        },
        { once: true },
      );

      const replay = (window as unknown as { dispatchReplayFromSpec: () => void })
        .dispatchReplayFromSpec;
      replay();
    });
  }, ONBOARDING_REPLAY_EVENT);

  expect(result).toBe('replay-received');
  await expect(page.locator('#status')).toHaveText('replay-received');
});

test('identifies overlay activity states for help surfaces', async ({ page }) => {
  const results = await page.evaluate(() => {
    const overlay = (
      window as unknown as {
        isOverlayActiveFromSpec: typeof isOverlayActive;
      }
    ).isOverlayActiveFromSpec;
    return [
      overlay({
        isCommandPaletteOpen: false,
        isShortcutHelpOpen: false,
        isHelpCenterOpen: false,
      }),
      overlay({
        isCommandPaletteOpen: true,
        isShortcutHelpOpen: false,
        isHelpCenterOpen: false,
      }),
      overlay({
        isCommandPaletteOpen: false,
        isShortcutHelpOpen: true,
        isHelpCenterOpen: false,
      }),
      overlay({
        isCommandPaletteOpen: false,
        isShortcutHelpOpen: false,
        isHelpCenterOpen: true,
      }),
    ];
  });

  expect(results).toEqual([false, true, true, true]);
});
