import { zodResolver } from '@hookform/resolvers/zod';
import { useCallback, useMemo, useRef, useState } from 'react';
import { useForm, type Path, type SubmitHandler } from 'react-hook-form';
import type {
  Task,
  TaskPayload,
  TaskParseRequest,
  TaskParseResponse,
  TaskPayloadField,
} from '../types/task';
import { taskCreateSchema } from '../utils/validators';
import { parseTask, toAppError } from '../services/tauriApi';

export type TaskFormValues = TaskPayload;

const DEFAULT_VALUES: TaskFormValues = {
  title: '',
  description: '',
  status: 'todo',
  priority: 'medium',
  plannedStartAt: undefined,
  startAt: undefined,
  dueAt: undefined,
  completedAt: undefined,
  estimatedMinutes: undefined,
  estimatedHours: undefined,
  tags: [],
  ownerId: undefined,
  isRecurring: false,
  recurrence: undefined,
  taskType: 'other',
  ai: undefined,
  externalLinks: [],
};

export interface UseTaskFormOptions {
  defaultValues?: Partial<TaskFormValues>;
}

export interface TaskFormAiState {
  status: 'idle' | 'loading' | 'success' | 'error';
  error: string | null;
  correlationId?: string;
  generatedAt?: string;
  missingFields: TaskPayloadField[];
  appliedFields: TaskPayloadField[];
  result: TaskParseResponse | null;
  lastInput?: string;
}

const createInitialAiState = (): TaskFormAiState => ({
  status: 'idle',
  error: null,
  correlationId: undefined,
  generatedAt: undefined,
  missingFields: [],
  appliedFields: [],
  result: null,
  lastInput: undefined,
});

export interface TriggerAiParseOptions {
  context?: TaskParseRequest['context'];
}

const resolveCorrelationIdFromResult = (
  result: TaskParseResponse | null | undefined,
): string | undefined => {
  const metadata = result?.ai?.metadata;
  if (!metadata || typeof metadata !== 'object') {
    return undefined;
  }

  const provider = (metadata as Record<string, unknown>).provider;
  if (!provider || typeof provider !== 'object') {
    return undefined;
  }

  const extra = (provider as Record<string, unknown>).extra;
  if (!extra || typeof extra !== 'object') {
    return undefined;
  }

  const correlationId = (extra as Record<string, unknown>).correlationId;
  if (typeof correlationId === 'string' && correlationId.trim().length > 0) {
    return correlationId.trim();
  }

  return undefined;
};

export function useTaskForm(options: UseTaskFormOptions = {}) {
  const initialValues = useMemo(
    () => ({
      ...DEFAULT_VALUES,
      ...options.defaultValues,
    }),
    [options.defaultValues],
  );

  const form = useForm<TaskFormValues>({
    resolver: zodResolver(taskCreateSchema),
    defaultValues: initialValues,
    mode: 'onSubmit',
    reValidateMode: 'onChange',
  });

  const aiResultRef = useRef<TaskParseResponse | null>(null);
  const lastInputRef = useRef<string>('');
  const [aiState, setAiState] = useState<TaskFormAiState>(() => createInitialAiState());

  const resetAiState = useCallback(() => {
    aiResultRef.current = null;
    lastInputRef.current = '';
    setAiState(createInitialAiState());
  }, []);

  const resetForm = useCallback(
    (values?: Partial<TaskFormValues>) => {
      form.reset({
        ...DEFAULT_VALUES,
        ...values,
      });
      resetAiState();
    },
    [form, resetAiState],
  );

  const setFromTask = useCallback(
    (task: Task | null | undefined) => {
      if (!task) {
        resetForm();
        return;
      }
      const { id: _id, createdAt: _createdAt, updatedAt: _updatedAt, ...rest } = task;
      resetAiState();
      resetForm({
        ...rest,
        description: rest.description ?? '',
        tags: rest.tags ?? [],
        externalLinks: rest.externalLinks ?? [],
        ownerId: rest.ownerId ?? undefined,
      });
    },
    [resetAiState, resetForm],
  );

  const handleSubmit = useCallback(
    (submit: SubmitHandler<TaskFormValues>) => form.handleSubmit(submit),
    [form],
  );

  const applyAiResult = useCallback(
    (
      result?: TaskParseResponse | null,
      options: { overrideManual?: boolean } = {},
    ): { applied: TaskPayloadField[] } => {
      const target = result ?? aiResultRef.current;
      if (!target) {
        return { applied: [] };
      }

      aiResultRef.current = target;

      const overrideManual = options.overrideManual ?? false;
      const appliedFields: TaskPayloadField[] = [];

      const payloadKeys = Object.keys(target.payload) as Array<keyof TaskFormValues>;

      for (const field of payloadKeys) {
        if (field === 'ai') continue;

        if (!overrideManual && form.getFieldState(field as Path<TaskFormValues>).isDirty) {
          continue;
        }

        const typedValue = target.payload[field] as TaskFormValues[typeof field] | undefined;

        form.setValue(field, typedValue, {
          shouldDirty: false,
          shouldValidate: false,
        });

        appliedFields.push(field as TaskPayloadField);
      }

      const shouldUpdateAi =
        overrideManual || !form.getFieldState('ai' as Path<TaskFormValues>).isDirty;

      if (shouldUpdateAi) {
        form.setValue(
          'ai',
          {
            ...(form.getValues('ai') ?? {}),
            ...target.ai,
          },
          { shouldDirty: false, shouldValidate: false },
        );
        appliedFields.push('ai');
      }

      const correlationId = resolveCorrelationIdFromResult(target);

      setAiState(() => ({
        status: 'success',
        error: null,
        correlationId,
        generatedAt: target.ai.generatedAt,
        missingFields: target.missingFields,
        appliedFields,
        result: target,
        lastInput: lastInputRef.current || undefined,
      }));

      return { applied: appliedFields };
    },
    [form],
  );

  const triggerAiParse = useCallback(
    async (input: string, options?: TriggerAiParseOptions) => {
      const normalizedInput = input.trim();
      lastInputRef.current = normalizedInput;

      if (!normalizedInput) {
        setAiState((prev) => ({
          ...prev,
          status: 'error',
          error: '请输入待解析的任务描述',
          correlationId: undefined,
          generatedAt: undefined,
          missingFields: [],
          appliedFields: [],
          result: aiResultRef.current,
          lastInput: normalizedInput,
        }));
        return null;
      }

      setAiState((prev) => ({
        ...prev,
        status: 'loading',
        error: null,
        correlationId: undefined,
        generatedAt: undefined,
        missingFields: [],
        appliedFields: [],
        result: aiResultRef.current,
        lastInput: normalizedInput,
      }));

      const resolvedContext = buildParseContext(options?.context);

      try {
        const response = await parseTask({ input: normalizedInput, context: resolvedContext });
        aiResultRef.current = response;
        applyAiResult(response);
        return response;
      } catch (error) {
        const mapped = toAppError(error, 'AI 解析失败');
        setAiState((prev) => ({
          ...prev,
          status: 'error',
          error: mapped.message,
          correlationId: mapped.correlationId,
          generatedAt: undefined,
          missingFields: [],
          appliedFields: [],
          result: aiResultRef.current,
          lastInput: normalizedInput,
        }));
        return null;
      }
    },
    [applyAiResult],
  );

  const clearAiState = useCallback(() => {
    resetAiState();
  }, [resetAiState]);

  return {
    form,
    resetForm,
    setFromTask,
    handleSubmit,
    aiState,
    triggerAiParse,
    applyAiResult,
    clearAiState,
  };
}

export type UseTaskFormReturn = ReturnType<typeof useTaskForm>;

function buildParseContext(
  context?: TaskParseRequest['context'],
): TaskParseRequest['context'] | undefined {
  const timezone = (() => {
    try {
      return Intl.DateTimeFormat().resolvedOptions().timeZone;
    } catch {
      return undefined;
    }
  })();

  const locale = (() => {
    if (typeof navigator !== 'undefined' && navigator.language) {
      return navigator.language;
    }
    return undefined;
  })();

  const merged: TaskParseRequest['context'] = {
    ...(context ?? {}),
  };

  if (!merged.timezone && timezone) {
    merged.timezone = timezone;
  }
  if (!merged.locale && locale) {
    merged.locale = locale;
  }
  if (!merged.referenceDate) {
    merged.referenceDate = new Date().toISOString();
  }

  const hasValues = Object.values(merged).some((value) => value !== undefined);
  return hasValues ? merged : undefined;
}
