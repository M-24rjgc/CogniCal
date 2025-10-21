import { AlertTriangle, ChevronDown, ChevronUp, Copy, Check } from 'lucide-react';
import { useState } from 'react';
import { Button } from '../ui/button';
import type { ErrorDetail } from '../../stores/chatStore';

interface ErrorDetailsPanelProps {
  errors: ErrorDetail[];
  correlationId?: string;
}

export function ErrorDetailsPanel({ errors, correlationId }: ErrorDetailsPanelProps) {
  const [isExpanded, setIsExpanded] = useState(false);
  const [copiedId, setCopiedId] = useState(false);

  if (errors.length === 0) {
    return null;
  }

  const handleCopyCorrelationId = () => {
    if (correlationId) {
      navigator.clipboard.writeText(correlationId);
      setCopiedId(true);
      setTimeout(() => setCopiedId(false), 2000);
    }
  };

  return (
    <div className="rounded-lg border border-red-500/30 bg-red-500/5 p-3 text-sm">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <AlertTriangle className="h-4 w-4 text-red-600 dark:text-red-400" />
          <span className="font-medium text-red-600 dark:text-red-400">
            {errors.length} 个错误
          </span>
        </div>
        <Button
          variant="ghost"
          size="sm"
          onClick={() => setIsExpanded(!isExpanded)}
          className="h-6 px-2 text-xs"
        >
          {isExpanded ? (
            <>
              <ChevronUp className="mr-1 h-3 w-3" />
              隐藏详情
            </>
          ) : (
            <>
              <ChevronDown className="mr-1 h-3 w-3" />
              显示详情
            </>
          )}
        </Button>
      </div>

      {isExpanded && (
        <div className="mt-3 space-y-2">
          {correlationId && (
            <div className="flex items-center justify-between rounded border border-red-500/20 bg-red-500/5 px-2 py-1 text-xs">
              <div className="flex items-center gap-2">
                <span className="text-muted-foreground">关联 ID:</span>
                <code className="font-mono text-red-600 dark:text-red-400">
                  {correlationId}
                </code>
              </div>
              <Button
                variant="ghost"
                size="sm"
                onClick={handleCopyCorrelationId}
                className="h-5 px-1"
              >
                {copiedId ? (
                  <Check className="h-3 w-3 text-green-600" />
                ) : (
                  <Copy className="h-3 w-3" />
                )}
              </Button>
            </div>
          )}

          {errors.map((error, idx) => (
            <div
              key={idx}
              className="rounded border border-red-500/20 bg-red-500/5 p-2 text-xs"
            >
              <div className="mb-1 flex items-center justify-between">
                <span className="font-medium text-red-600 dark:text-red-400">
                  {error.errorType}
                </span>
                <span className="text-muted-foreground">
                  {new Date(error.timestamp).toLocaleTimeString('zh-CN')}
                </span>
              </div>
              <p className="text-red-600/90 dark:text-red-400/90">
                {error.message}
              </p>
              {error.context && Object.keys(error.context).length > 0 && (
                <div className="mt-2 space-y-1 border-t border-red-500/20 pt-2">
                  <span className="text-muted-foreground">上下文:</span>
                  {Object.entries(error.context).map(([key, value]) => (
                    <div key={key} className="flex gap-2">
                      <span className="text-muted-foreground">{key}:</span>
                      <span className="flex-1 break-all font-mono text-red-600/80 dark:text-red-400/80">
                        {value}
                      </span>
                    </div>
                  ))}
                </div>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
