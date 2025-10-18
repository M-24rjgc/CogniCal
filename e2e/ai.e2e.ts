import { expect, test } from '@playwright/test';

type ParseSuccess = {
  payload: {
    title: string;
    description: string;
    tags: string[];
  };
  missingFields: string[];
  ai: {
    source: 'live' | 'cache';
    generatedAt: string;
    summary: string;
    metadata: Record<string, unknown>;
  };
};

type ParseError = {
  code: string;
  message: string;
  details?: Record<string, unknown>;
};

const createSuccessResponse = (overrides?: Partial<ParseSuccess>): ParseSuccess => ({
  payload: {
    title: 'AI 生成的季度复盘计划',
    description: '系统建议整理季度数据并完成复盘输出。',
    tags: ['ai', 'planning'],
    ...overrides?.payload,
  },
  missingFields: ['ownerId'],
  ai: {
    source: 'live',
    generatedAt: '2025-10-16T08:00:00.000Z',
    summary: 'AI 已根据输入内容提供关键字段建议。',
    metadata: {
      provider: {
        extra: {
          correlationId: overrides?.ai?.metadata?.provider
            ? (overrides.ai.metadata.provider as { extra?: { correlationId?: string } }).extra
                ?.correlationId
            : 'corr-success-001',
        },
        tokensUsed: {
          prompt: 96,
          completion: 48,
          total: 144,
        },
      },
    },
    ...overrides?.ai,
  },
  ...overrides,
});

const createTimeoutError = (): ParseError => ({
  code: 'HTTP_TIMEOUT',
  message: 'DeepSeek 请求超时',
  details: { correlationId: 'timeout-001' },
});

test.describe('AI parse flow', () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(() => {
      const queues: Record<string, unknown[]> = Object.create(null);
      const bridge = {
        queues,
        push(command: string, entry: unknown) {
          if (!queues[command]) {
            queues[command] = [];
          }
          queues[command].push(entry);
        },
        clear(command?: string) {
          if (command) {
            queues[command] = [];
            return;
          }
          for (const key of Object.keys(queues)) {
            queues[key] = [];
          }
        },
      };

      (globalThis as unknown as { __COGNICAL_E2E__?: typeof bridge }).__COGNICAL_E2E__ = bridge;

      const completedProgress = {
        state: {
          progress: {
            version: 1,
            hasCompletedTour: true,
            completedStepIds: [
              'dashboard-overview',
              'task-quick-create',
              'ai-parse-panel',
              'planning-center',
              'settings-api-key',
            ],
            lastStepId: 'settings-api-key',
            dismissedAt: null,
          },
        },
        version: 1,
      } satisfies Record<string, unknown>;

      try {
        window.localStorage?.setItem('onboarding-store-state', JSON.stringify(completedProgress));
      } catch (error) {
        console.warn('[e2e] Failed to preset onboarding store', error);
      }
    });
  });

  test('covers missing key guidance, successful parse, and timeout retry', async ({ page }) => {
    const initialSuccess = createSuccessResponse();
    initialSuccess.ai.metadata = {
      provider: {
        extra: { correlationId: 'corr-success-001' },
        tokensUsed: { prompt: 96, completion: 48, total: 144 },
      },
    };
    initialSuccess.ai.summary = 'AI 建议回填任务字段。';

    const retrySuccess = createSuccessResponse({
      payload: {
        title: '新品发布会执行清单',
        description: '包含筹备发布会所需的关键步骤。',
        tags: ['ai', 'launch'],
      },
    });
    retrySuccess.ai.metadata = {
      provider: {
        extra: { correlationId: 'corr-retry-002' },
        tokensUsed: { prompt: 72, completion: 28, total: 100 },
      },
    };
    retrySuccess.ai.summary = 'AI 已重新生成任务建议。';

    const timeoutError = createTimeoutError();

    await page.goto('/#/tasks');
    await expect(page.getByRole('heading', { name: '任务管理中心' })).toBeVisible();

    await page.evaluate(() => {
      (
        window as unknown as { __COGNICAL_E2E__?: { clear: (command?: string) => void } }
      ).__COGNICAL_E2E__?.clear();
    });

    await page.getByRole('button', { name: '新建任务' }).click();
    const dialog = page.getByRole('dialog');
    await expect(dialog).toBeVisible();

    const parseButton = dialog.getByRole('button', { name: 'AI 解析', exact: true });
    await expect(parseButton).toBeDisabled();
    await expect(dialog.getByText(/未配置 API Key/)).toBeVisible();
    await expect(dialog.getByText(/请前往设置页面完成配置/)).toBeVisible();

    await page.keyboard.press('Escape');
    await expect(dialog).toBeHidden();

    await page.getByRole('link', { name: '前往设置' }).click();
    await page.waitForURL('**/#/settings');
    await expect(page.getByRole('heading', { name: '应用设置中心' })).toBeVisible();

    const keyField = page.getByRole('textbox', { name: 'DeepSeek API Key', exact: true });
    await keyField.fill('sk-test-123456789');
    await page.getByRole('button', { name: '保存设置' }).click();
    await expect(page.getByRole('button', { name: '清除密钥' })).toBeVisible();

    await page.evaluate(
      ({ success }) => {
        const bridge = (
          window as unknown as {
            __COGNICAL_E2E__?: {
              clear: (command?: string) => void;
              push: (command: string, entry: unknown) => void;
            };
          }
        ).__COGNICAL_E2E__;
        bridge?.clear('tasks_parse_ai');
        bridge?.push('tasks_parse_ai', { type: 'resolve', value: success });
      },
      { success: initialSuccess },
    );

    await page.getByRole('link', { name: '任务' }).click();
    await page.waitForURL('**/#/tasks');
    await expect(page.getByRole('heading', { name: '任务管理中心' })).toBeVisible();

    await page.getByRole('button', { name: '新建任务' }).click();
    const activeDialog = page.getByRole('dialog');
    await expect(activeDialog).toBeVisible();
    await expect(activeDialog.getByText('API Key 已配置')).toBeVisible();

    const textarea = activeDialog.getByPlaceholder(/请输入任务的自然语言描述/);
    await textarea.fill('编写季度总结报告并安排复盘会议');

    const actionableParseButton = activeDialog.getByRole('button', {
      name: 'AI 解析',
      exact: true,
    });
    await expect(actionableParseButton).toBeEnabled();
    await actionableParseButton.click();

    await expect(activeDialog.getByText('AI 已解析任务描述并回填表单字段。')).toBeVisible();
    await expect(activeDialog.getByText('诊断 ID：corr-success-001')).toBeVisible();
    await expect(activeDialog.getByText('已自动填充字段：')).toBeVisible();
    await expect(activeDialog.getByText('仍需手动确认：')).toBeVisible();

    await page.evaluate(
      ({ error, retry }) => {
        const bridge = (
          window as unknown as {
            __COGNICAL_E2E__?: {
              clear: (command?: string) => void;
              push: (command: string, entry: unknown) => void;
            };
          }
        ).__COGNICAL_E2E__;
        bridge?.clear('tasks_parse_ai');
        bridge?.push('tasks_parse_ai', { type: 'reject', error });
        bridge?.push('tasks_parse_ai', { type: 'resolve', value: retry });
      },
      { error: timeoutError, retry: retrySuccess },
    );

    await textarea.fill('筹备新品发布会并完成资料审批');

    await actionableParseButton.click();
    await expect(activeDialog.getByText('AI 解析失败：DeepSeek 请求超时')).toBeVisible();
    await expect(activeDialog.getByText('诊断 ID：timeout-001')).toBeVisible();

    await actionableParseButton.click();
    await expect(activeDialog.getByText('AI 已解析任务描述并回填表单字段。')).toBeVisible();
    await expect(activeDialog.getByText('诊断 ID：corr-retry-002')).toBeVisible();

    await expect(activeDialog.getByText('AI 解析失败：DeepSeek 请求超时')).not.toBeVisible();
  });
});
