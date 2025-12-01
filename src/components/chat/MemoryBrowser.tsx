import { useState, useEffect } from 'react';
import {
  Search,
  Calendar,
  Hash,
  Brain,
  Trash2,
  Download,
  Filter,
  ChevronDown,
  ChevronRight,
  FileText,
  Clock,
  Tag,
} from 'lucide-react';
import { Button } from '../ui/button';
import { Badge } from '../ui/badge';
import { searchConversations } from '../../services/tauriApi';
import type { MemorySearchResult } from '../../stores/chatStore';

interface MemoryBrowserProps {
  onClose?: () => void;
}

interface GroupedMemories {
  [date: string]: MemorySearchResult[];
}

export function MemoryBrowser({ onClose }: MemoryBrowserProps) {
  const [searchQuery, setSearchQuery] = useState('');
  const [memories, setMemories] = useState<MemorySearchResult[]>([]);
  const [filteredMemories, setFilteredMemories] = useState<MemorySearchResult[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [selectedTopics, setSelectedTopics] = useState<string[]>([]);
  const [expandedDates, setExpandedDates] = useState<Set<string>>(new Set());
  const [sortBy, setSortBy] = useState<'date' | 'relevance'>('date');

  // Load all memories on component mount
  useEffect(() => {
    loadAllMemories();
  }, []);

  // Filter memories when search query or filters change
  useEffect(() => {
    const filterMemories = () => {
      let filtered = [...memories];

      // Apply text search filter
      if (searchQuery.trim()) {
        const query = searchQuery.toLowerCase();
        filtered = filtered.filter(
          (memory) =>
            memory.userMessage.toLowerCase().includes(query) ||
            memory.assistantMessage.toLowerCase().includes(query),
        );
      }

      // Apply topic filter
      if (selectedTopics.length > 0) {
        filtered = filtered.filter((memory) => {
          const memoryTopics = memory.metadata?.topics?.split(', ') || [];
          return selectedTopics.some((topic) => memoryTopics.includes(topic));
        });
      }

      // Sort memories
      filtered.sort((a, b) => {
        if (sortBy === 'date') {
          return new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime();
        } else {
          return b.relevanceScore - a.relevanceScore;
        }
      });

      setFilteredMemories(filtered);
    };

    filterMemories();
  }, [memories, searchQuery, selectedTopics, sortBy]);

  const loadAllMemories = async () => {
    setIsLoading(true);
    try {
      // Search with empty query to get all memories
      const allMemories = await searchConversations('');
      setMemories(allMemories);
    } catch (error) {
      console.error('Failed to load memories:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const groupMemoriesByDate = (memories: MemorySearchResult[]): GroupedMemories => {
    return memories.reduce((groups, memory) => {
      const date = new Date(memory.timestamp).toLocaleDateString('zh-CN', {
        year: 'numeric',
        month: '2-digit',
        day: '2-digit',
      });

      if (!groups[date]) {
        groups[date] = [];
      }
      groups[date].push(memory);
      return groups;
    }, {} as GroupedMemories);
  };

  const getAllTopics = (): string[] => {
    const topicsSet = new Set<string>();
    memories.forEach((memory) => {
      const topics = memory.metadata?.topics?.split(', ') || [];
      topics.forEach((topic: string) => topicsSet.add(topic));
    });
    return Array.from(topicsSet).sort();
  };

  const toggleTopic = (topic: string) => {
    setSelectedTopics((prev) =>
      prev.includes(topic) ? prev.filter((t) => t !== topic) : [...prev, topic],
    );
  };

  const toggleDateExpansion = (date: string) => {
    setExpandedDates((prev) => {
      const newSet = new Set(prev);
      if (newSet.has(date)) {
        newSet.delete(date);
      } else {
        newSet.add(date);
      }
      return newSet;
    });
  };

  const groupedMemories = groupMemoriesByDate(filteredMemories);
  const allTopics = getAllTopics();

  return (
    <div className="flex h-full flex-col bg-background">
      {/* Header */}
      <div className="border-b border-border p-4">
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center gap-2">
            <Brain className="h-5 w-5 text-primary" />
            <h2 className="text-lg font-semibold">记忆管理</h2>
            <Badge variant="secondary" className="text-xs">
              {memories.length} 条记录
            </Badge>
          </div>
          {onClose && (
            <Button variant="ghost" size="sm" onClick={onClose}>
              关闭
            </Button>
          )}
        </div>

        {/* Search and Filters */}
        <div className="space-y-3">
          {/* Search Input */}
          <div className="relative">
            <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="搜索对话内容..."
              className="w-full rounded-lg border border-border bg-background py-2 pl-10 pr-4 text-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary"
            />
          </div>

          {/* Filter Controls */}
          <div className="flex flex-wrap items-center gap-2">
            <div className="flex items-center gap-2">
              <Filter className="h-4 w-4 text-muted-foreground" />
              <span className="text-sm text-muted-foreground">排序:</span>
              <select
                value={sortBy}
                onChange={(e) => setSortBy(e.target.value as 'date' | 'relevance')}
                className="rounded border border-border bg-background px-2 py-1 text-sm"
              >
                <option value="date">按时间</option>
                <option value="relevance">按相关性</option>
              </select>
            </div>
          </div>

          {/* Topic Filter */}
          {allTopics.length > 0 && (
            <div className="space-y-2">
              <div className="text-sm font-medium text-muted-foreground">话题筛选:</div>
              <div className="flex flex-wrap gap-1">
                {allTopics.map((topic) => (
                  <Badge
                    key={topic}
                    variant={selectedTopics.includes(topic) ? 'default' : 'outline'}
                    className="cursor-pointer text-xs"
                    onClick={() => toggleTopic(topic)}
                  >
                    <Tag className="mr-1 h-3 w-3" />
                    {topic}
                  </Badge>
                ))}
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Memory List */}
      <div className="flex-1 overflow-y-auto p-4">
        {isLoading ? (
          <div className="flex items-center justify-center py-8">
            <div className="text-sm text-muted-foreground">加载中...</div>
          </div>
        ) : Object.keys(groupedMemories).length === 0 ? (
          <div className="flex flex-col items-center justify-center py-8 text-center">
            <Brain className="mb-2 h-12 w-12 text-muted-foreground/50" />
            <p className="text-sm text-muted-foreground">
              {searchQuery || selectedTopics.length > 0 ? '未找到匹配的记忆' : '暂无记忆记录'}
            </p>
          </div>
        ) : (
          <div className="space-y-4">
            {Object.entries(groupedMemories).map(([date, dateMemories]) => (
              <div key={date} className="space-y-2">
                {/* Date Header */}
                <button
                  onClick={() => toggleDateExpansion(date)}
                  className="flex w-full items-center gap-2 rounded-lg border border-border bg-card p-3 text-left hover:bg-accent/50 transition-colors"
                >
                  {expandedDates.has(date) ? (
                    <ChevronDown className="h-4 w-4" />
                  ) : (
                    <ChevronRight className="h-4 w-4" />
                  )}
                  <Calendar className="h-4 w-4 text-muted-foreground" />
                  <span className="font-medium">{date}</span>
                  <Badge variant="secondary" className="text-xs">
                    {dateMemories.length} 条
                  </Badge>
                </button>

                {/* Memory Items */}
                {expandedDates.has(date) && (
                  <div className="ml-6 space-y-2">
                    {dateMemories.map((memory, index) => (
                      <div
                        key={`${memory.conversationId}-${index}`}
                        className="rounded-lg border border-border bg-card p-4 hover:bg-accent/30 transition-colors"
                      >
                        {/* Memory Header */}
                        <div className="mb-3 flex items-center justify-between">
                          <div className="flex items-center gap-2 text-xs text-muted-foreground">
                            <Clock className="h-3 w-3" />
                            <span>
                              {new Date(memory.timestamp).toLocaleTimeString('zh-CN', {
                                hour: '2-digit',
                                minute: '2-digit',
                              })}
                            </span>
                            <Hash className="h-3 w-3" />
                            <span className="font-mono">{memory.conversationId.slice(0, 8)}</span>
                          </div>

                          {sortBy === 'relevance' && (
                            <Badge variant="outline" className="text-xs">
                              相关度: {(memory.relevanceScore * 100).toFixed(0)}%
                            </Badge>
                          )}
                        </div>

                        {/* Memory Content */}
                        <div className="space-y-3">
                          <div className="rounded-md bg-primary/5 p-3">
                            <div className="text-xs font-medium text-muted-foreground mb-1">
                              用户:
                            </div>
                            <div className="text-sm">{memory.userMessage}</div>
                          </div>

                          <div className="rounded-md bg-muted/50 p-3">
                            <div className="text-xs font-medium text-muted-foreground mb-1">
                              助手:
                            </div>
                            <div className="text-sm">{memory.assistantMessage}</div>
                          </div>
                        </div>

                        {/* Topics */}
                        {memory.metadata?.topics && (
                          <div className="mt-3 flex flex-wrap gap-1">
                            {memory.metadata.topics
                              .split(', ')
                              .map((topic: string, topicIndex: number) => (
                                <Badge
                                  key={topicIndex}
                                  variant="outline"
                                  className="text-xs px-1.5 py-0.5 h-auto"
                                >
                                  {topic}
                                </Badge>
                              ))}
                          </div>
                        )}

                        {/* Actions */}
                        <div className="mt-3 flex items-center gap-2">
                          <Button variant="ghost" size="sm" className="h-7 px-2 text-xs">
                            <FileText className="mr-1 h-3 w-3" />
                            查看详情
                          </Button>
                          <Button
                            variant="ghost"
                            size="sm"
                            className="h-7 px-2 text-xs text-destructive hover:text-destructive"
                          >
                            <Trash2 className="mr-1 h-3 w-3" />
                            删除
                          </Button>
                        </div>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Footer Actions */}
      <div className="border-t border-border p-4">
        <div className="flex items-center justify-between">
          <div className="text-sm text-muted-foreground">
            显示 {filteredMemories.length} / {memories.length} 条记录
          </div>
          <div className="flex gap-2">
            <Button variant="outline" size="sm">
              <Download className="mr-2 h-4 w-4" />
              导出全部
            </Button>
            <Button variant="outline" size="sm" className="text-destructive hover:text-destructive">
              <Trash2 className="mr-2 h-4 w-4" />
              清理旧记录
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}
