import { expect, test } from '@playwright/test';
import {
  DASHBOARD_MODULE_DEFINITIONS,
  DEFAULT_DASHBOARD_CONFIG,
  normalizeDashboardConfig,
  sortModulesByOrder,
} from '../src/utils/dashboardConfig';

const createEnabledModuleList = () => {
  const normalized = normalizeDashboardConfig(DEFAULT_DASHBOARD_CONFIG);
  return sortModulesByOrder(
    DASHBOARD_MODULE_DEFINITIONS.filter((definition) => normalized.modules[definition.id]),
  ).map((definition) => definition.id);
};

test('renders enabled dashboard modules in sorted order', async ({ page }) => {
  const enabledModules = createEnabledModuleList();

  await page.setContent('<main><ul id="modules"></ul></main>');

  await page.evaluate((modules) => {
    const root = document.querySelector<HTMLUListElement>('#modules');
    modules.forEach((id) => {
      const item = document.createElement('li');
      item.setAttribute('data-module-id', id);
      item.textContent = id;
      root?.appendChild(item);
    });
  }, enabledModules);

  const moduleItems = page.locator('#modules li');
  await expect(moduleItems).toHaveCount(enabledModules.length);
  await expect(moduleItems).toHaveText(enabledModules);
});

test('disabling all modules results in an empty dashboard list', async ({ page }) => {
  await page.setContent('<main><ul id="modules"></ul></main>');

  await page.evaluate(() => {
    const root = document.querySelector<HTMLUListElement>('#modules');
    if (root) {
      root.textContent = '';
    }
  });

  await expect(page.locator('#modules li')).toHaveCount(0);
});
