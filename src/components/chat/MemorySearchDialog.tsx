import { useState } from 'react';
import { Search, Loader2, X, Calendar, MessageSquare } from 'lucide-react';
import { Button } from '../ui/button';
import type { MemorySearchResult } from '../../stores/chatStore';

interface MemorySearchDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onSearch: (query: string) => Promise<MemorySearchResult[]>;
}

export function MemorySearchDialog({ isOpen, onClose, onSearch }: MemorySearchDialogProps) {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<MemorySearchResult[]>([]);
  const [isSearching, setIsSearching] = useState(false);

  const handleSearch = async () => {
    if (!query.trim()) return;

    setIsSearching(true);
    try {
      const searchResults = await onSearch(query);
      setResults(searchResults);
    } finally {
      setIsSearching(false);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter') {
      handleSearch();
    }
  };

  const handleClose = () => {
    setQuery('');
    setResults([]);
    onClose();
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div className="w-full max-w-2xl rounded-2xl border border-border bg-background p-6 shadow-lg">
        {/* Header */}
        <div className="mb-4 flex items-center justify-between">
          <h2 className="text-lg font-semibold">搜索对话历史</h2>
          <Button variant="ghost" size="icon" onClick={handleClose}>
            <X className="h-4 w-4" />
          </Button>
        </div>

        {/* Search Input */}
        <div className="mb-4 flex gap-2">
          <div className="relative flex-1">
            <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
            <input
              type="text"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder="输入关键词搜索历史对话..."
              className="w-full rounded-lg border border-border bg-background py-2 pl-10 pr-4 text-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary"
            />
          </div>
          <Button onClick={handleSearch} disabled={!query.trim() || isSearching}>
            {isSearching ? (
              <Loader2 className="h-4 w-4 animate-spin" />
            ) : (
              <Search className="h-4 w-4" />
            )}
          </Button>
        </div>

        {/* Results */}
        <div className="max-h-96 space-y-3 overflow-y-auto">
          {results.length === 0 && !isSearching && query && (
            <div className="flex flex-col items-center justify-center py-8 text-center">
              <MessageSquare className="mb-2 h-12 w-12 text-muted-foreground/50" />
              <p className="text-sm text-muted-foreground">未找到相关对话</p>
            </div>
          )}

          {results.length === 0 && !query && (
            <div className="flex flex-col items-center justify-center py-8 text-center">
              <Search className="mb-2 h-12 w-12 text-muted-foreground/50" />
              <p className="text-sm text-muted-foreground">输入关键词开始搜索</p>
            </div>
          )}

          {results.map((result, index) => (
            <div
              key={`${result.conversationId}-${index}`}
              className="rounded-lg border border-border bg-card p-4 hover:bg-accent/50 transition-colors"
            >
              <div className="mb-2 flex items-center justify-between">
                <div className="flex items-center gap-2 text-xs text-muted-foreground">
                  <Calendar className="h-3 w-3" />
                  <span>
                    {new Date(result.timestamp).toLocaleString('zh-CN', {
                      year: 'numeric',
                      month: '2-digit',
                      day: '2-digit',
                      hour: '2-digit',
                      minute: '2-digit',
                    })}
                  </span>
                </div>
                <span className="text-xs text-muted-foreground">
                  相关度: {(result.relevanceScore * 100).toFixed(0)}%
                </span>
              </div>
              
              <div className="space-y-2">
                <div className="rounded-md bg-primary/5 p-2">
                  <p className="text-xs font-medium text-muted-foreground mb-1">用户:</p>
                  <p className="text-sm">{result.userMessage}</p>
                </div>
                
                <div className="rounded-md bg-muted/50 p-2">
                  <p className="text-xs font-medium text-muted-foreground mb-1">助手:</p>
                  <p className="text-sm">{result.assistantMessage}</p>
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
