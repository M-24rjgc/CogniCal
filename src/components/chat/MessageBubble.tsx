import { Bot, User, Clock, Zap, AlertTriangle, CloudOff } from 'lucide-react';
import { cn } from '../../lib/utils';
import { ToolCallIndicator } from './ToolCallIndicator';
import { ErrorDetailsPanel } from './ErrorDetailsPanel';
import { MemoryContextIndicator } from './MemoryContextIndicator';
import type { ChatMessage, ToolCallStatus } from '../../stores/chatStore';

interface MessageBubbleProps {
  message: ChatMessage;
  toolCalls?: ToolCallStatus[];
  showMetrics?: boolean;
}

export function MessageBubble({
  message,
  toolCalls = [],
  showMetrics = false,
}: MessageBubbleProps) {
  const isUser = message.role === 'user';
  const time = new Date(message.timestamp).toLocaleTimeString('zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
  });

  const hasMetadata =
    message.metadata &&
    (message.metadata.tokensUsed ||
      message.metadata.latencyMs !== undefined ||
      message.metadata.memoryEntriesUsed !== undefined ||
      (message.metadata.toolsExecuted && message.metadata.toolsExecuted.length > 0));

  const hasMemoryContext =
    message.metadata?.memoryEntriesUsed && message.metadata.memoryEntriesUsed > 0;
  const memoryUnavailable = message.metadata?.memoryAvailable === false;
  const hasErrors = message.metadata?.errors && message.metadata.errors.length > 0;

  return (
    <div className={cn('flex items-start gap-3', isUser && 'flex-row-reverse')}>
      {/* Avatar */}
      <div
        className={cn(
          'flex h-8 w-8 shrink-0 items-center justify-center rounded-full',
          isUser ? 'bg-primary text-primary-foreground' : 'bg-primary/10 text-primary',
        )}
      >
        {isUser ? <User className="h-5 w-5" /> : <Bot className="h-5 w-5" />}
      </div>

      {/* Message Content */}
      <div className={cn('flex-1 space-y-2', isUser && 'flex flex-col items-end')}>
        {/* Memory Unavailable Warning */}
        {!isUser && memoryUnavailable && (
          <div className="flex items-center gap-1.5 rounded-md border border-yellow-500/30 bg-yellow-500/10 px-2 py-1 text-xs text-yellow-600 dark:text-yellow-400">
            <CloudOff className="h-3.5 w-3.5" />
            <span>记忆服务不可用，当前为无状态模式</span>
          </div>
        )}

        {/* Memory Context Indicator */}
        {!isUser && hasMemoryContext && (
          <MemoryContextIndicator
            memoryEntriesUsed={message.metadata?.memoryEntriesUsed || 0}
            className="mb-2"
          />
        )}

        {/* Error Display - Use ErrorDetailsPanel when metrics enabled, show simple errors otherwise */}
        {!isUser && hasErrors && (
          <>
            {showMetrics ? (
              <ErrorDetailsPanel
                errors={message.metadata?.errors || []}
                correlationId={message.metadata?.correlationId}
              />
            ) : (
              <div className="space-y-1">
                {message.metadata?.errors
                  ?.filter((error) => error.errorType === 'tool_execution')
                  .map((error, idx) => (
                    <div
                      key={idx}
                      className="flex items-start gap-1.5 rounded-md border border-red-500/30 bg-red-500/10 px-2 py-1 text-xs text-red-600 dark:text-red-400"
                    >
                      <AlertTriangle className="mt-0.5 h-3.5 w-3.5 shrink-0" />
                      <div className="flex-1">
                        <div className="font-medium">工具执行失败</div>
                        <div className="text-xs opacity-90">{error.message}</div>
                      </div>
                    </div>
                  ))}
              </div>
            )}
          </>
        )}

        {/* Message Bubble */}
        <div
          className={cn(
            'inline-block max-w-[85%] rounded-2xl border px-4 py-3',
            isUser
              ? 'border-primary/40 bg-primary/10 text-foreground'
              : 'border-border/60 bg-background/80 text-foreground',
          )}
        >
          <p className="whitespace-pre-wrap text-sm leading-relaxed">{message.content}</p>
        </div>

        {/* Tool Calls Display */}
        {!isUser && toolCalls.length > 0 && (
          <div className="flex flex-wrap gap-2">
            {toolCalls.map((toolCall) => (
              <ToolCallIndicator key={toolCall.id} toolCall={toolCall} />
            ))}
          </div>
        )}

        {/* Metadata Display */}
        {!isUser && hasMetadata && showMetrics && (
          <div className="space-y-1">
            <div className="flex flex-wrap items-center gap-3 text-xs text-muted-foreground">
              {message.metadata?.latencyMs !== undefined && (
                <div className="flex items-center gap-1">
                  <Clock className="h-3 w-3" />
                  <span>{message.metadata.latencyMs}ms</span>
                </div>
              )}

              {message.metadata?.tokensUsed && (
                <div className="flex items-center gap-1">
                  <Zap className="h-3 w-3" />
                  <span>
                    {Object.values(message.metadata.tokensUsed).reduce((a, b) => a + b, 0)} tokens
                  </span>
                </div>
              )}

              {message.metadata?.correlationId && (
                <div className="flex items-center gap-1 font-mono">
                  <span className="opacity-60">ID:</span>
                  <span className="opacity-80">{message.metadata.correlationId.slice(0, 8)}</span>
                </div>
              )}
            </div>

            {/* Error count indicator */}
            {hasErrors && (
              <div className="text-xs text-muted-foreground">
                <span className="opacity-60">{message.metadata?.errors?.length} 个错误/警告</span>
              </div>
            )}
          </div>
        )}

        {/* Timestamp */}
        <span className="text-xs text-muted-foreground">{time}</span>
      </div>
    </div>
  );
}
