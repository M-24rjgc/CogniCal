import {
  AlertCircle,
  CheckCircle2,
  InfoIcon,
  Loader2,
  RotateCcw,
  Sparkles,
  Wand2,
} from 'lucide-react';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import type { TaskFormAiState } from '../../hooks/useTaskForm';
import { formatTaskPayloadField } from '../../utils/taskLabels';
import { Badge } from '../ui/badge';
import { Button } from '../ui/button';
import { Textarea } from '../ui/textarea';
import { HelpPopover } from '../help/HelpPopover';

interface TaskAiParsePanelProps {
  aiState?: TaskFormAiState;
  hasDeepseekKey?: boolean;
  onParse?: (input: string) => Promise<unknown> | unknown;
  onClear?: () => void;
  disabled?: boolean;
  mode?: 'create' | 'edit';
}

export function TaskAiParsePanel({
  aiState,
  hasDeepseekKey = true,
  onParse,
  onClear,
  disabled = false,
  mode = 'create',
}: TaskAiParsePanelProps) {
  const [input, setInput] = useState(() => aiState?.lastInput ?? '');
  const textareaRef = useRef<HTMLTextAreaElement | null>(null);

  useEffect(() => {
    if (aiState?.lastInput !== undefined && aiState.lastInput !== input) {
      setInput(aiState.lastInput);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [aiState?.lastInput]);

  const isParsing = aiState?.status === 'loading';
  const normalizedInput = input.trim();

  const handleParse = useCallback(async () => {
    if (!onParse || !normalizedInput || disabled || !hasDeepseekKey) {
      return;
    }

    setInput(normalizedInput);
    await onParse(normalizedInput);

    if (textareaRef.current) {
      textareaRef.current.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
    }
  }, [disabled, hasDeepseekKey, normalizedInput, onParse]);

  const handleClear = useCallback(() => {
    onClear?.();
    setInput('');
  }, [onClear]);

  const canParse = Boolean(normalizedInput) && !isParsing && !disabled && hasDeepseekKey;

  const feedback = useMemo(() => {
    if (!aiState || aiState.status === 'idle') {
      return null;
    }

    if (aiState.status === 'loading') {
      return (
        <div className="flex items-center gap-2 rounded-lg border border-primary/40 bg-primary/10 px-3 py-2 text-sm text-primary">
          <Loader2 className="h-4 w-4 animate-spin" />
          正在解析任务描述，请稍候…
        </div>
      );
    }

    if (aiState.status === 'error') {
      return (
        <div className="space-y-2 rounded-lg border border-destructive/40 bg-destructive/10 px-3 py-2 text-sm text-destructive">
          <div className="flex items-start gap-2">
            <AlertCircle className="mt-0.5 h-4 w-4" />
            <div className="space-y-1">
              <p>AI 解析失败：{aiState.error ?? '未知错误'}</p>
              {aiState.correlationId ? (
                <p className="text-xs text-destructive/80">诊断 ID：{aiState.correlationId}</p>
              ) : null}
            </div>
          </div>
        </div>
      );
    }

    if (aiState.status === 'success') {
      const hasApplied = aiState.appliedFields.length > 0;
      const hasMissing = aiState.missingFields.length > 0;

      return (
        <div className="space-y-3 rounded-lg border border-emerald-400/60 bg-emerald-100/60 px-3 py-3 text-sm text-emerald-950 dark:border-emerald-400/50 dark:bg-emerald-900/30 dark:text-emerald-100">
          <div className="flex items-start gap-2">
            <CheckCircle2 className="mt-0.5 h-4 w-4" />
            <div className="space-y-2">
              <p>AI 已解析任务描述并回填表单字段。</p>
              {aiState.generatedAt ? (
                <p className="text-xs text-emerald-900/70 dark:text-emerald-100/70">
                  生成时间：{formatDateTime(aiState.generatedAt)}
                </p>
              ) : null}
              {aiState.correlationId ? (
                <p className="text-xs text-emerald-900/70 dark:text-emerald-100/70">
                  诊断 ID：{aiState.correlationId}
                </p>
              ) : null}
            </div>
          </div>

          {hasApplied ? (
            <div className="space-y-1 text-xs">
              <p className="text-muted-foreground">已自动填充字段：</p>
              <div className="flex flex-wrap gap-2">
                {aiState.appliedFields.map((field) => (
                  <Badge key={field} variant="secondary" className="text-[11px]">
                    {formatTaskPayloadField(field)}
                  </Badge>
                ))}
              </div>
            </div>
          ) : null}

          {hasMissing ? (
            <div className="space-y-1 text-xs">
              <p className="text-muted-foreground">仍需手动确认：</p>
              <div className="flex flex-wrap gap-2">
                {aiState.missingFields.map((field) => (
                  <Badge key={field} variant="outline" className="text-[11px]">
                    {formatTaskPayloadField(field)}
                  </Badge>
                ))}
              </div>
            </div>
          ) : null}
        </div>
      );
    }

    return null;
  }, [aiState]);

  return (
    <section
      className="space-y-4 rounded-2xl border border-border/80 bg-muted/20 p-4"
      data-onboarding="ai-parse-panel"
    >
      <header className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
        <div className="space-y-1">
          <div className="flex items-center gap-2 text-sm font-semibold text-foreground">
            <Sparkles className="h-4 w-4 text-primary" />
            DeepSeek 任务解析
            <Badge
              variant={hasDeepseekKey ? 'secondary' : 'destructive'}
              className="text-[11px] uppercase tracking-wide"
            >
              {hasDeepseekKey ? 'API Key 已配置' : 'API Key 未配置'}
            </Badge>
            {aiState?.correlationId ? (
              <Badge variant="outline" className="font-mono text-[11px] uppercase tracking-wide">
                诊断 ID {aiState.correlationId}
              </Badge>
            ) : null}
            <HelpPopover
              entryId="tasks-ai-panel"
              triggerLabel="查看 AI 解析功能说明"
              triggerClassName="ml-1"
            />
          </div>
          <p className="text-xs text-muted-foreground">
            输入任务描述，AI 将尝试补全必填字段并提供执行建议。
            {mode === 'edit' ? ' 重新解析可更新现有任务信息。' : ''}
          </p>
        </div>
      </header>

      <div className="flex items-start gap-2 rounded-lg border border-border/60 bg-background/80 p-3 text-xs text-muted-foreground">
        <InfoIcon className="mt-0.5 h-4 w-4 text-primary" />
        <div className="space-y-1">
          <p>建议描述包含任务目标、预计时长、优先级等关键信息，AI 会智能回填表单字段。</p>
          <p className="hidden text-muted-foreground/80 sm:block">
            示例：下周一上午准备季度复盘，需要 2 小时，重点整理风险清单。
          </p>
        </div>
      </div>

      <Textarea
        ref={textareaRef}
        rows={4}
        value={input}
        onChange={(event) => setInput(event.target.value)}
        placeholder="请输入任务的自然语言描述，例如：下周一上午准备季度复盘，需要 2 小时，重点整理风险清单。"
        disabled={disabled || isParsing}
      />

      <div className="flex flex-wrap gap-2">
        <Button type="button" onClick={handleParse} disabled={!canParse}>
          {isParsing ? (
            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
          ) : (
            <Wand2 className="mr-2 h-4 w-4" />
          )}
          {isParsing ? '解析中…' : 'AI 解析'}
        </Button>

        <Button
          type="button"
          variant="ghost"
          onClick={handleClear}
          disabled={isParsing || disabled}
        >
          <RotateCcw className="mr-2 h-4 w-4" /> 清除输入
        </Button>
      </div>

      {hasDeepseekKey ? null : (
        <p className="rounded-lg border border-destructive/40 bg-destructive/5 px-3 py-2 text-xs text-destructive">
          未配置 API Key，AI 解析将直接失败。请前往设置页面完成配置后再试。
        </p>
      )}

      {feedback}
    </section>
  );
}

function formatDateTime(value: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return new Intl.DateTimeFormat('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  }).format(date);
}
