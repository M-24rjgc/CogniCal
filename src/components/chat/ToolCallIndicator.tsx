import { CheckCircle2, Loader2, XCircle, Wrench } from 'lucide-react';
import { cn } from '../../lib/utils';
import type { ToolCallStatus } from '../../stores/chatStore';

interface ToolCallIndicatorProps {
  toolCall: ToolCallStatus;
  className?: string;
}

export function ToolCallIndicator({ toolCall, className }: ToolCallIndicatorProps) {
  const getStatusIcon = () => {
    switch (toolCall.status) {
      case 'pending':
        return <Wrench className="h-3.5 w-3.5 text-muted-foreground" />;
      case 'executing':
        return <Loader2 className="h-3.5 w-3.5 animate-spin text-primary" />;
      case 'completed':
        return <CheckCircle2 className="h-3.5 w-3.5 text-green-600" />;
      case 'failed':
        return <XCircle className="h-3.5 w-3.5 text-destructive" />;
      default:
        return <Wrench className="h-3.5 w-3.5 text-muted-foreground" />;
    }
  };

  const getStatusText = () => {
    switch (toolCall.status) {
      case 'pending':
        return '等待执行';
      case 'executing':
        return '执行中';
      case 'completed':
        return '已完成';
      case 'failed':
        return '执行失败';
      default:
        return '未知状态';
    }
  };

  const getStatusColor = () => {
    switch (toolCall.status) {
      case 'pending':
        return 'border-muted-foreground/20 bg-muted/30';
      case 'executing':
        return 'border-primary/40 bg-primary/10';
      case 'completed':
        return 'border-green-600/40 bg-green-600/10';
      case 'failed':
        return 'border-destructive/40 bg-destructive/10';
      default:
        return 'border-muted-foreground/20 bg-muted/30';
    }
  };

  const formatResult = (result: unknown): string => {
    if (typeof result === 'string') {
      return result;
    }
    try {
      return String(JSON.stringify(result)).substring(0, 50) + '...';
    } catch {
      return String(result);
    }
  };

  return (
    <div
      className={cn(
        'inline-flex items-center gap-2 rounded-lg border px-3 py-1.5 text-xs',
        getStatusColor(),
        className
      )}
    >
      {getStatusIcon()}
      <div className="flex flex-col gap-0.5">
        <span className="font-medium">{toolCall.toolName}</span>
        <span className="text-[10px] text-muted-foreground">{getStatusText()}</span>
      </div>
      
      {toolCall.status === 'failed' && toolCall.error && (
        <div className="ml-2 max-w-xs truncate text-[10px] text-destructive">
          {toolCall.error}
        </div>
      )}
      
      {toolCall.status === 'completed' && toolCall.result !== undefined && (
        <div className="ml-2 max-w-xs truncate text-[10px] text-muted-foreground">
          {formatResult(toolCall.result)}
        </div>
      )}
    </div>
  );
}
