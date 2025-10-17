import { expect, test } from '@playwright/test';
import { createServer, type ViteDevServer } from 'vite';

let server: ViteDevServer | undefined;
let baseUrl = 'http://127.0.0.1:4173/';

const withTrailingSlash = (value: string) => (value.endsWith('/') ? value : `${value}/`);

test.beforeAll(async () => {
  server = await createServer({
    logLevel: 'error',
    server: {
      host: '127.0.0.1',
      port: 4173,
    },
  });
  await server.listen();
  const resolved = server.resolvedUrls?.local?.[0];
  if (resolved) {
    baseUrl = withTrailingSlash(resolved);
  }
});

test.afterAll(async () => {
  await server?.close();
});

test.skip('user can complete intelligent planning journey', async ({ page }) => {
  const tasksUrl = `${baseUrl}#/tasks`;
  await page.goto(tasksUrl);

  await expect(page.getByRole('heading', { name: '任务管理中心' })).toBeVisible();
  await expect(page.getByRole('heading', { name: '智能规划中心' })).toBeVisible();

  const totalBadge = page
    .locator('span')
    .filter({ hasText: /^共 \d+ 条任务$/ })
    .first();
  await expect(totalBadge).toHaveText(/共 [1-9]\d* 条任务/, { timeout: 15000 });

  const taskSelect = page.locator('#planning-task-select');
  await expect(taskSelect).toBeVisible();

  const addTaskToPlanning = async (label: string) => {
    const optionLocator = page.locator('#planning-task-select option').filter({ hasText: label });
    await optionLocator.waitFor({ state: 'attached', timeout: 10000 });
    await taskSelect.selectOption({ label });
    await page.getByRole('button', { name: '添加' }).click();
    await expect(page.getByRole('button', { name: `移除任务 ${label}` })).toBeVisible();
    await expect(taskSelect).toHaveValue('');
  };

  await addTaskToPlanning('接入 Tauri API');
  await addTaskToPlanning('构建任务状态 Store');

  const generateButton = page.getByRole('button', { name: '生成方案' });
  await generateButton.click();

  const planToast = page.getByRole('alert').filter({ hasText: '规划方案已生成' });
  await expect(planToast).toBeVisible({ timeout: 10000 });

  await expect(page.getByText('方案 #1')).toBeVisible({ timeout: 10000 });
  await expect(page.getByText('方案 #2')).toBeVisible();
  await expect(page.getByText('冲突 1 项')).toBeVisible();

  await page
    .getByRole('button', { name: /^查看冲突$/ })
    .first()
    .click();

  const conflictDialog = page.getByRole('dialog', { name: '冲突处理助手' });
  await expect(conflictDialog).toBeVisible({ timeout: 5000 });
  await expect(conflictDialog.getByText('存在潜在冲突，请确认安排。')).toBeVisible();

  await conflictDialog.getByRole('button', { name: '标记为已处理' }).click();

  const resolveToast = page.getByRole('alert').filter({ hasText: '冲突已调整' });
  await expect(resolveToast).toBeVisible({ timeout: 10000 });
  await expect(conflictDialog.getByText('冲突已解决')).toBeVisible({ timeout: 5000 });

  await conflictDialog.locator('button').filter({ hasText: '关闭' }).last().click();
  await expect(conflictDialog).toBeHidden({ timeout: 5000 });

  const primaryOptionCard = page.locator('article').filter({ hasText: '方案 #1' }).first();
  await expect(primaryOptionCard.getByText('冲突已处理')).toBeVisible({ timeout: 5000 });

  await page.getByRole('button', { name: '应用方案' }).first().click();

  const applyToast = page.getByRole('alert').filter({ hasText: '方案已应用' });
  await expect(applyToast).toBeVisible({ timeout: 10000 });
  await expect(page.getByText('冲突 0')).toBeVisible({ timeout: 5000 });

  await page.getByRole('link', { name: '日历' }).click();

  await expect(page.getByRole('heading', { name: '规划时间线' })).toBeVisible({ timeout: 10000 });
  await expect(page.getByText(/已排程任务 · \d+ 个时间块/)).toBeVisible();
  await expect(page.getByText('冲突已清空')).toBeVisible();
});
