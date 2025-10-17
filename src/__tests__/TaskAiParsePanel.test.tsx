import '@testing-library/jest-dom/vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import { TaskAiParsePanel } from '../components/tasks/TaskAiParsePanel';
import type { TaskFormAiState } from '../hooks/useTaskForm';
import type { TaskParseResponse } from '../types/task';

const createAiState = (overrides: Partial<TaskFormAiState> = {}): TaskFormAiState => ({
  status: 'idle',
  error: null,
  correlationId: undefined,
  generatedAt: undefined,
  missingFields: [],
  appliedFields: [],
  result: null,
  lastInput: undefined,
  ...overrides,
});

const scrollIntoViewMock = vi.fn();

beforeAll(() => {
  Object.defineProperty(window.HTMLElement.prototype, 'scrollIntoView', {
    configurable: true,
    value: scrollIntoViewMock,
  });
});

afterEach(() => {
  scrollIntoViewMock.mockClear();
});

describe('TaskAiParsePanel', () => {
  it('calls onParse with trimmed input', async () => {
    const user = userEvent.setup();
    const onParse = vi.fn().mockResolvedValue(undefined);

    render(<TaskAiParsePanel hasDeepseekKey aiState={createAiState()} onParse={onParse} />);

    const textarea = screen.getByPlaceholderText(/请输入任务的自然语言描述/);
    await user.type(textarea, '  编写周报计划  ');

    const parseButton = screen.getByRole('button', { name: 'AI 解析' });
    expect(parseButton).toBeEnabled();

    await user.click(parseButton);

    expect(onParse).toHaveBeenCalledWith('编写周报计划');
    expect(textarea).toHaveValue('编写周报计划');
  });

  it('disables parse when API key missing and shows warning', async () => {
    const user = userEvent.setup();
    const onParse = vi.fn();

    render(<TaskAiParsePanel hasDeepseekKey={false} aiState={createAiState()} onParse={onParse} />);

    const textarea = screen.getByPlaceholderText(/请输入任务的自然语言描述/);
    await user.type(textarea, '准备 OKR 回顾');

    const parseButton = screen.getByRole('button', { name: 'AI 解析' });
    expect(parseButton).toBeDisabled();
    expect(onParse).not.toHaveBeenCalled();
    expect(
      screen.getByText('未配置 API Key，AI 解析将直接失败。请前往设置页面完成配置后再试。'),
    ).toBeInTheDocument();
  });

  it('renders success feedback with applied and missing fields', () => {
    const response: TaskParseResponse = {
      payload: {},
      missingFields: ['dueAt'],
      ai: {
        source: 'live',
        generatedAt: '2025-10-16T08:00:00.000Z',
        metadata: {
          provider: {
            extra: { correlationId: 'corr-ui-123' },
          },
        },
      },
    };

    const aiState = createAiState({
      status: 'success',
      correlationId: 'corr-ui-123',
      generatedAt: response.ai.generatedAt,
      appliedFields: ['description', 'tags'],
      missingFields: response.missingFields,
      result: response,
      lastInput: '编写周报',
    });

    render(<TaskAiParsePanel aiState={aiState} hasDeepseekKey />);

    expect(screen.getByText('AI 已解析任务描述并回填表单字段。')).toBeInTheDocument();
    expect(screen.getByText('诊断 ID corr-ui-123')).toBeInTheDocument();
    expect(screen.getByText('诊断 ID：corr-ui-123')).toBeInTheDocument();
    expect(screen.getByText('已自动填充字段：')).toBeInTheDocument();
    expect(screen.getByText('描述')).toBeInTheDocument();
    expect(screen.getByText('标签')).toBeInTheDocument();
    expect(screen.getByText('仍需手动确认：')).toBeInTheDocument();
    expect(screen.getByText('截止时间')).toBeInTheDocument();
    expect(screen.getByText('API Key 已配置')).toBeInTheDocument();
  });

  it('renders error feedback with correlation id', () => {
    const aiState = createAiState({
      status: 'error',
      error: '网络连接超时',
      correlationId: 'err-456',
    });

    render(<TaskAiParsePanel aiState={aiState} hasDeepseekKey />);

    expect(screen.getByText('AI 解析失败：网络连接超时')).toBeInTheDocument();
    expect(screen.getByText('诊断 ID err-456')).toBeInTheDocument();
    expect(screen.getByText('诊断 ID：err-456')).toBeInTheDocument();
  });

  it('clears input and calls onClear callback', async () => {
    const user = userEvent.setup();
    const onClear = vi.fn();

    render(<TaskAiParsePanel hasDeepseekKey aiState={createAiState()} onClear={onClear} />);

    const textarea = screen.getByPlaceholderText(/请输入任务的自然语言描述/);
    await user.type(textarea, '清理缓存');
    expect(textarea).toHaveValue('清理缓存');

    const clearButton = screen.getByRole('button', { name: '清除输入' });
    await user.click(clearButton);

    expect(onClear).toHaveBeenCalledTimes(1);
    expect(textarea).toHaveValue('');
  });
});
