import '@testing-library/jest-dom/vitest';
import { renderHook, act } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { useTaskForm } from '../hooks/useTaskForm';
import type { TaskParseResponse } from '../types/task';

const parseTaskMock = vi.hoisted(() => vi.fn());

vi.mock('../services/tauriApi', async () => {
  const actual =
    await vi.importActual<typeof import('../services/tauriApi')>('../services/tauriApi');
  return {
    ...actual,
    parseTask: parseTaskMock,
  };
});

describe('useTaskForm', () => {
  beforeEach(() => {
    parseTaskMock.mockReset();
  });

  const buildResponse = (): TaskParseResponse => ({
    payload: {
      title: 'AI Title',
      description: 'AI Description',
      tags: ['ai', 'demo'],
    },
    missingFields: ['dueAt'],
    ai: {
      source: 'live',
      generatedAt: '2025-10-16T08:00:00.000Z',
      summary: '自动填充字段摘要',
      metadata: {
        provider: {
          extra: {
            correlationId: 'corr-123',
          },
          tokensUsed: {
            prompt: 64,
          },
        },
      },
    },
  });

  it('applies AI result while preserving manual edits', async () => {
    const response = buildResponse();
    parseTaskMock.mockResolvedValueOnce(response);

    const { result } = renderHook(() => useTaskForm());

    await act(async () => {
      result.current.form.setValue('title', 'Manual Title', {
        shouldDirty: true,
      });
    });

    await act(async () => {
      const ret = await result.current.triggerAiParse('  自动化任务  ');
      expect(ret).toEqual(response);
    });

    expect(parseTaskMock).toHaveBeenCalledTimes(1);
    const sentRequest = parseTaskMock.mock.calls[0][0];
    expect(sentRequest.input).toBe('自动化任务');
    expect(sentRequest.context?.referenceDate).toBeDefined();

    expect(result.current.form.getValues('title')).toBe('Manual Title');
    expect(result.current.form.getValues('description')).toBe('AI Description');
    expect(result.current.form.getValues('tags')).toEqual(['ai', 'demo']);

    expect(result.current.aiState.status).toBe('success');
    expect(result.current.aiState.error).toBeNull();
    expect(result.current.aiState.correlationId).toBe('corr-123');
    expect(result.current.aiState.generatedAt).toBe('2025-10-16T08:00:00.000Z');
    expect(result.current.aiState.missingFields).toEqual(['dueAt']);
    expect(result.current.aiState.appliedFields).toContain('description');
    expect(result.current.aiState.appliedFields).not.toContain('title');
    expect(result.current.aiState.result).toEqual(response);
    expect(result.current.aiState.lastInput).toBe('自动化任务');
  });

  it('surfaces missing API key errors with correlation id', async () => {
    parseTaskMock.mockRejectedValueOnce({
      code: 'MISSING_API_KEY',
      message: 'DeepSeek API Key 未配置',
      details: { correlationId: 'key-missing-789' },
    });

    const { result } = renderHook(() => useTaskForm());

    await act(async () => {
      const value = await result.current.triggerAiParse('需要解析的任务');
      expect(value).toBeNull();
    });

    expect(parseTaskMock).toHaveBeenCalledTimes(1);
    expect(result.current.aiState.status).toBe('error');
    expect(result.current.aiState.error).toBe('DeepSeek API Key 未配置');
    expect(result.current.aiState.correlationId).toBe('key-missing-789');
    expect(result.current.aiState.appliedFields).toEqual([]);
    expect(result.current.aiState.missingFields).toEqual([]);
    expect(result.current.aiState.result).toBeNull();
  });

  it('rejects empty input without calling backend', async () => {
    const { result } = renderHook(() => useTaskForm());

    await act(async () => {
      const ret = await result.current.triggerAiParse('   ');
      expect(ret).toBeNull();
    });

    expect(parseTaskMock).not.toHaveBeenCalled();
    expect(result.current.aiState.status).toBe('error');
    expect(result.current.aiState.error).toBe('请输入待解析的任务描述');
    expect(result.current.aiState.correlationId).toBeUndefined();
  });
});
