import { expect, test } from '@playwright/test';

declare global {
  interface Window {
    toggleOnboarding: (complete: boolean) => void;
  }
}

test('analytics onboarding flag toggles zero state visibility', async ({ page }) => {
  await page.addInitScript(() => {
    const store = new Map();
    Object.defineProperty(window, 'localStorage', {
      value: {
        getItem(key: string) {
          return store.has(key) ? (store.get(key) as string) : null;
        },
        setItem(key: string, value: string) {
          store.set(key, value);
        },
        removeItem(key: string) {
          store.delete(key);
        },
        clear() {
          store.clear();
        },
        key(index: number) {
          return Array.from(store.keys())[index] ?? null;
        },
        get length() {
          return store.size;
        },
      },
      configurable: true,
    });
  });

  await page.goto('about:blank');
  await page.setContent(`
    <main>
      <section id="zero-state" data-visible="true">Complete onboarding</section>
    </main>
    <script>
      const STORAGE_KEY = 'analytics-store-state';
      function render() {
        const raw = localStorage.getItem(STORAGE_KEY);
        const parsed = raw ? JSON.parse(raw) : { state: { isOnboardingComplete: false } };
        const visible = !parsed.state?.isOnboardingComplete;
        const section = document.querySelector('#zero-state');
        if (!section) return;
        section.setAttribute('data-visible', String(visible));
        section.textContent = visible ? 'Complete onboarding' : 'Analytics Ready';
      }
      window.toggleOnboarding = (complete) => {
        const payload = {
          state: {
            range: '7d',
            grouping: 'day',
            isOnboardingComplete: complete,
            exportStatus: 'idle',
            exportResult: null,
            exportError: null,
            lastRefreshedAt: null,
            isDemoData: false
          },
          version: 0
        };
        localStorage.setItem(STORAGE_KEY, JSON.stringify(payload));
        render();
      };
      render();
    </script>
  `);

  const zeroState = page.locator('#zero-state');
  await expect(zeroState).toHaveAttribute('data-visible', 'true');

  await page.evaluate(() => window.toggleOnboarding(true));
  await expect(zeroState).toHaveText('Analytics Ready');
  await expect(zeroState).toHaveAttribute('data-visible', 'false');

  const stored = await page.evaluate(() => localStorage.getItem('analytics-store-state'));
  const parsed = stored ? JSON.parse(stored) : { state: { isOnboardingComplete: false } };
  expect(parsed.state.isOnboardingComplete).toBe(true);

  await page.evaluate(() => window.toggleOnboarding(false));
  await expect(zeroState).toHaveAttribute('data-visible', 'true');
});
