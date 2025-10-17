import { expect, test } from '@playwright/test';

test('renders placeholder content', async ({ page }) => {
  await page.setContent('<main id="app">CogniCal Smoke Test</main>');
  await expect(page.locator('#app')).toContainText('CogniCal Smoke Test');
});
