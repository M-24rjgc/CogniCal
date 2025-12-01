import { useState, useEffect, useCallback } from 'react';
import {
  Brain,
  Search,
  Download,
  Trash2,
  Settings,
  BarChart3,
  Calendar,
  FileText,
  Archive,
} from 'lucide-react';
import { Button } from '../components/ui/button';
import { Badge } from '../components/ui/badge';
import { MemoryBrowser } from '../components/chat/MemoryBrowser';
import { MemoryCleanupDialog } from '../components/chat/MemoryCleanupDialog';
import { searchConversations } from '../services/tauriApi';
import type { MemorySearchResult } from '../stores/chatStore';

interface MemoryStats {
  totalMemories: number;
  totalSize: string;
  oldestDate: string | null;
  newestDate: string | null;
  topTopics: Array<{ topic: string; count: number }>;
}

export default function MemoryManagementPage() {
  const [activeTab, setActiveTab] = useState<'browser' | 'stats' | 'settings'>('browser');
  const [isCleanupDialogOpen, setIsCleanupDialogOpen] = useState(false);
  const [memories, setMemories] = useState<MemorySearchResult[]>([]);
  const [stats, setStats] = useState<MemoryStats | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  const calculateStats = useCallback((memories: MemorySearchResult[]): MemoryStats => {
    if (memories.length === 0) {
      return {
        totalMemories: 0,
        totalSize: '0 KB',
        oldestDate: null,
        newestDate: null,
        topTopics: [],
      };
    }

    // Calculate total size (rough estimate)
    const totalChars = memories.reduce(
      (sum, memory) => sum + memory.userMessage.length + memory.assistantMessage.length,
      0,
    );
    const totalSize = `${Math.round(totalChars / 1024)} KB`;

    // Find date range
    const dates = memories
      .map((m) => new Date(m.timestamp))
      .sort((a, b) => a.getTime() - b.getTime());
    const oldestDate = dates[0]?.toLocaleDateString('zh-CN') || null;
    const newestDate = dates[dates.length - 1]?.toLocaleDateString('zh-CN') || null;

    // Calculate top topics
    const topicCounts: Record<string, number> = {};
    memories.forEach((memory) => {
      const topics = memory.metadata?.topics?.split(', ') || [];
      topics.forEach((topic: string) => {
        topicCounts[topic] = (topicCounts[topic] || 0) + 1;
      });
    });

    const topTopics = Object.entries(topicCounts)
      .map(([topic, count]) => ({ topic, count }))
      .sort((a, b) => b.count - a.count)
      .slice(0, 5);

    return {
      totalMemories: memories.length,
      totalSize,
      oldestDate,
      newestDate,
      topTopics,
    };
  }, []);

  const loadMemoryData = useCallback(async () => {
    setIsLoading(true);
    try {
      // Load all memories
      const allMemories = await searchConversations('');
      setMemories(allMemories);

      // Calculate stats
      const calculatedStats = calculateStats(allMemories);
      setStats(calculatedStats);
    } catch (error) {
      console.error('Failed to load memory data:', error);
    } finally {
      setIsLoading(false);
    }
  }, [calculateStats]);

  useEffect(() => {
    void loadMemoryData();
  }, [loadMemoryData]);

  const handleCleanup = async (olderThanDays: number) => {
    // This would call the backend cleanup API
    // For now, we'll simulate the cleanup
    const cutoffDate = new Date();
    cutoffDate.setDate(cutoffDate.getDate() - olderThanDays);

    const toDelete = memories.filter((memory) => new Date(memory.timestamp) < cutoffDate);

    // Simulate API call
    await new Promise((resolve) => setTimeout(resolve, 2000));

    if (toDelete.length > 0) {
      // Update local state (in real app, this would be handled by the backend)
      const remaining = memories.filter((memory) => new Date(memory.timestamp) >= cutoffDate);
      setMemories(remaining);
      setStats(calculateStats(remaining));

      return {
        success: true,
        message: `成功清理了 ${olderThanDays} 天前的记忆数据`,
        count: toDelete.length,
      };
    } else {
      return {
        success: true,
        message: `没有找到 ${olderThanDays} 天前的记忆数据`,
        count: 0,
      };
    }
  };

  const handleExportAll = async () => {
    try {
      // This would call the backend export API
      alert('导出功能正在开发中...');
    } catch (error) {
      console.error('Export failed:', error);
      alert('导出失败，请稍后重试');
    }
  };

  if (isLoading) {
    return (
      <section className="flex h-full flex-1 items-center justify-center">
        <div className="text-center">
          <Brain className="mx-auto mb-4 h-12 w-12 text-muted-foreground animate-pulse" />
          <p className="text-muted-foreground">加载记忆数据中...</p>
        </div>
      </section>
    );
  }

  return (
    <section className="flex h-full flex-1 flex-col gap-6">
      {/* Header */}
      <header className="flex flex-col gap-3 rounded-3xl border border-border/60 bg-background/80 p-6 shadow-sm">
        <div className="flex flex-wrap items-center justify-between gap-4">
          <div className="space-y-1">
            <div className="flex items-center gap-2">
              <Badge variant="secondary" className="bg-secondary/15 text-xs">
                <Brain className="mr-1.5 h-3.5 w-3.5" /> 记忆管理
              </Badge>
              <Badge variant="default" className="text-xs">
                {stats?.totalMemories || 0} 条记录
              </Badge>
            </div>
            <h1 className="text-2xl font-semibold text-foreground">AI 记忆管理</h1>
            <p className="text-sm text-muted-foreground">
              管理 AI 助手的对话记忆，包括搜索、导出和清理功能。
            </p>
          </div>
          <div className="flex items-center gap-2">
            <Button variant="outline" size="sm" onClick={handleExportAll}>
              <Download className="mr-2 h-4 w-4" />
              导出全部
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={() => setIsCleanupDialogOpen(true)}
              className="text-destructive hover:text-destructive"
            >
              <Trash2 className="mr-2 h-4 w-4" />
              清理数据
            </Button>
          </div>
        </div>

        {/* Tab Navigation */}
        <div className="flex gap-1 rounded-lg border border-border bg-muted/50 p-1">
          <button
            onClick={() => setActiveTab('browser')}
            className={`flex items-center gap-2 rounded-md px-3 py-2 text-sm font-medium transition-colors ${
              activeTab === 'browser'
                ? 'bg-background text-foreground shadow-sm'
                : 'text-muted-foreground hover:text-foreground'
            }`}
          >
            <Search className="h-4 w-4" />
            记忆浏览
          </button>
          <button
            onClick={() => setActiveTab('stats')}
            className={`flex items-center gap-2 rounded-md px-3 py-2 text-sm font-medium transition-colors ${
              activeTab === 'stats'
                ? 'bg-background text-foreground shadow-sm'
                : 'text-muted-foreground hover:text-foreground'
            }`}
          >
            <BarChart3 className="h-4 w-4" />
            统计信息
          </button>
          <button
            onClick={() => setActiveTab('settings')}
            className={`flex items-center gap-2 rounded-md px-3 py-2 text-sm font-medium transition-colors ${
              activeTab === 'settings'
                ? 'bg-background text-foreground shadow-sm'
                : 'text-muted-foreground hover:text-foreground'
            }`}
          >
            <Settings className="h-4 w-4" />
            设置
          </button>
        </div>
      </header>

      {/* Content */}
      <div className="flex-1 rounded-3xl border border-border/60 bg-card/80 shadow-sm overflow-hidden">
        {activeTab === 'browser' && <MemoryBrowser />}

        {activeTab === 'stats' && stats && (
          <div className="p-6 space-y-6">
            {/* Overview Cards */}
            <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
              <div className="rounded-lg border border-border bg-card p-4">
                <div className="flex items-center gap-2 mb-2">
                  <FileText className="h-4 w-4 text-muted-foreground" />
                  <span className="text-sm font-medium">总记录数</span>
                </div>
                <div className="text-2xl font-bold">{stats.totalMemories}</div>
              </div>

              <div className="rounded-lg border border-border bg-card p-4">
                <div className="flex items-center gap-2 mb-2">
                  <Archive className="h-4 w-4 text-muted-foreground" />
                  <span className="text-sm font-medium">存储大小</span>
                </div>
                <div className="text-2xl font-bold">{stats.totalSize}</div>
              </div>

              <div className="rounded-lg border border-border bg-card p-4">
                <div className="flex items-center gap-2 mb-2">
                  <Calendar className="h-4 w-4 text-muted-foreground" />
                  <span className="text-sm font-medium">最早记录</span>
                </div>
                <div className="text-lg font-semibold">{stats.oldestDate || '无'}</div>
              </div>

              <div className="rounded-lg border border-border bg-card p-4">
                <div className="flex items-center gap-2 mb-2">
                  <Calendar className="h-4 w-4 text-muted-foreground" />
                  <span className="text-sm font-medium">最新记录</span>
                </div>
                <div className="text-lg font-semibold">{stats.newestDate || '无'}</div>
              </div>
            </div>

            {/* Top Topics */}
            <div className="rounded-lg border border-border bg-card p-6">
              <h3 className="text-lg font-semibold mb-4">热门话题</h3>
              {stats.topTopics.length > 0 ? (
                <div className="space-y-3">
                  {stats.topTopics.map((item, index) => (
                    <div key={item.topic} className="flex items-center justify-between">
                      <div className="flex items-center gap-3">
                        <div className="flex h-6 w-6 items-center justify-center rounded-full bg-primary/10 text-xs font-medium text-primary">
                          {index + 1}
                        </div>
                        <span className="font-medium">{item.topic}</span>
                      </div>
                      <Badge variant="secondary">{item.count} 次</Badge>
                    </div>
                  ))}
                </div>
              ) : (
                <p className="text-muted-foreground">暂无话题数据</p>
              )}
            </div>
          </div>
        )}

        {activeTab === 'settings' && (
          <div className="p-6 space-y-6">
            <div className="space-y-4">
              <h3 className="text-lg font-semibold">记忆设置</h3>

              <div className="rounded-lg border border-border bg-card p-4 space-y-4">
                <div className="flex items-center justify-between">
                  <div>
                    <div className="font-medium">自动清理</div>
                    <div className="text-sm text-muted-foreground">
                      自动清理超过指定天数的记忆数据
                    </div>
                  </div>
                  <Button variant="outline" size="sm">
                    配置
                  </Button>
                </div>

                <div className="flex items-center justify-between">
                  <div>
                    <div className="font-medium">记忆上下文限制</div>
                    <div className="text-sm text-muted-foreground">限制每次对话使用的记忆条数</div>
                  </div>
                  <Button variant="outline" size="sm">
                    设置
                  </Button>
                </div>

                <div className="flex items-center justify-between">
                  <div>
                    <div className="font-medium">导出格式</div>
                    <div className="text-sm text-muted-foreground">选择记忆数据的默认导出格式</div>
                  </div>
                  <Button variant="outline" size="sm">
                    选择
                  </Button>
                </div>
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Cleanup Dialog */}
      <MemoryCleanupDialog
        isOpen={isCleanupDialogOpen}
        onClose={() => setIsCleanupDialogOpen(false)}
        onCleanup={handleCleanup}
        totalMemories={stats?.totalMemories || 0}
      />
    </section>
  );
}
