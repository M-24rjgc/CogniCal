import { useState } from 'react';
import { Brain, ChevronUp, ChevronDown, Calendar, Hash } from 'lucide-react';
import { Button } from '../ui/button';
import { Badge } from '../ui/badge';

interface MemoryEntry {
  id: string;
  conversationId: string;
  userMessage: string;
  assistantMessage: string;
  timestamp: string;
  metadata: Record<string, string>;
}

interface MemoryContextIndicatorProps {
  memoryEntriesUsed: number;
  memoryEntries?: MemoryEntry[];
  className?: string;
}

export function MemoryContextIndicator({
  memoryEntriesUsed,
  memoryEntries = [],
  className = '',
}: MemoryContextIndicatorProps) {
  const [isExpanded, setIsExpanded] = useState(false);

  if (memoryEntriesUsed === 0) {
    return null;
  }

  return (
    <div
      className={`rounded-lg border border-blue-200 bg-blue-50 dark:border-blue-800 dark:bg-blue-950/30 ${className}`}
    >
      <Button
        variant="ghost"
        size="sm"
        onClick={() => setIsExpanded(!isExpanded)}
        className="w-full justify-between p-3 h-auto text-blue-700 dark:text-blue-300 hover:bg-blue-100 dark:hover:bg-blue-900/50"
      >
        <div className="flex items-center gap-2">
          <Brain className="h-4 w-4" />
          <span className="text-sm font-medium">使用了 {memoryEntriesUsed} 条记忆上下文</span>
          <Badge
            variant="secondary"
            className="text-xs bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300"
          >
            记忆增强
          </Badge>
        </div>
        {isExpanded ? <ChevronUp className="h-4 w-4" /> : <ChevronDown className="h-4 w-4" />}
      </Button>

      {isExpanded && memoryEntries.length > 0 && (
        <div className="border-t border-blue-200 dark:border-blue-800 p-3 space-y-3">
          <div className="text-xs font-medium text-blue-700 dark:text-blue-300 mb-2">
            相关历史对话:
          </div>

          {memoryEntries.map((entry, _index) => (
            <div
              key={entry.id}
              className="rounded-md border border-blue-200 dark:border-blue-700 bg-white dark:bg-blue-950/50 p-3 space-y-2"
            >
              <div className="flex items-center justify-between text-xs text-blue-600 dark:text-blue-400">
                <div className="flex items-center gap-2">
                  <Calendar className="h-3 w-3" />
                  <span>
                    {new Date(entry.timestamp).toLocaleString('zh-CN', {
                      month: '2-digit',
                      day: '2-digit',
                      hour: '2-digit',
                      minute: '2-digit',
                    })}
                  </span>
                </div>
                <div className="flex items-center gap-1">
                  <Hash className="h-3 w-3" />
                  <span className="font-mono">{entry.conversationId.slice(0, 8)}</span>
                </div>
              </div>

              <div className="space-y-2">
                <div className="text-xs">
                  <div className="font-medium text-blue-700 dark:text-blue-300 mb-1">用户:</div>
                  <div className="text-gray-700 dark:text-gray-300 line-clamp-2">
                    {entry.userMessage}
                  </div>
                </div>

                <div className="text-xs">
                  <div className="font-medium text-blue-700 dark:text-blue-300 mb-1">助手:</div>
                  <div className="text-gray-700 dark:text-gray-300 line-clamp-2">
                    {entry.assistantMessage}
                  </div>
                </div>
              </div>

              {entry.metadata.topics && (
                <div className="flex flex-wrap gap-1">
                  {entry.metadata.topics.split(', ').map((topic, topicIndex) => (
                    <Badge
                      key={topicIndex}
                      variant="outline"
                      className="text-xs px-1.5 py-0.5 h-auto border-blue-300 text-blue-700 dark:border-blue-600 dark:text-blue-300"
                    >
                      {topic}
                    </Badge>
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
