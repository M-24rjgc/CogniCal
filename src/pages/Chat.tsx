import { useCallback, useEffect, useRef, useState } from 'react';
import { Link } from 'react-router-dom';
import {
  Bot,
  Loader2,
  Send,
  Sparkles,
  Trash2,
  AlertCircle,
  Settings2,
  Download,
  Search,
  Activity,
} from 'lucide-react';
import { Button } from '../components/ui/button';
import { Badge } from '../components/ui/badge';
import { useSettingsStore } from '../stores/settingsStore';
import { useChatStore } from '../stores/chatStore';
import { MessageBubble } from '../components/chat/MessageBubble';
import { MemorySearchDialog } from '../components/chat/MemorySearchDialog';
import { ConversationExportDialog } from '../components/chat/ConversationExportDialog';


export default function ChatPage() {
  const [input, setInput] = useState('');
  const [showMetrics, setShowMetrics] = useState(false);
  const [isSearchDialogOpen, setIsSearchDialogOpen] = useState(false);
  const [isExportDialogOpen, setIsExportDialogOpen] = useState(false);

  const messagesEndRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const hasDeepseekKey = useSettingsStore((state) => state.settings?.hasDeepseekKey ?? false);
  const {
    messages,
    conversationId,
    toolCalls,
    isLoading,
    error,
    sendMessage,
    clearMessages,
    clearError,
    searchConversations,
    exportConversation,
  } = useChatStore();

  const scrollToBottom = useCallback(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, []);



  useEffect(() => {
    scrollToBottom();
  }, [messages, scrollToBottom]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!input.trim() || isLoading) return;

    const message = input.trim();
    setInput('');

    // 重置 textarea 高度
    if (textareaRef.current) {
      textareaRef.current.style.height = 'auto';
    }

    await sendMessage(message);
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSubmit(e);
    }
  };

  const handleInputChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    setInput(e.target.value);

    // 自动调整高度
    const textarea = e.target;
    textarea.style.height = 'auto';
    textarea.style.height = `${Math.min(textarea.scrollHeight, 200)}px`;
  };

  const handleClearChat = () => {
    if (messages.length === 0) return;
    if (window.confirm('确定要清空所有对话记录吗？')) {
      clearMessages();
    }
  };

  const handleExportConversation = async () => {
    return await exportConversation();
  };

  const handleSearchConversations = async (query: string) => {
    return await searchConversations(query);
  };

  return (
    <section className="flex h-full flex-1 flex-col gap-6">
      {/* 头部 */}
      <header className="flex flex-col gap-3 rounded-3xl border border-border/60 bg-background/80 p-6 shadow-sm">
        <div className="flex flex-wrap items-center justify-between gap-4">
          <div className="space-y-1">
            <div className="flex items-center gap-2">
              <Badge variant="secondary" className="bg-secondary/15 text-xs">
                <Sparkles className="mr-1.5 h-3.5 w-3.5" /> AI 助手
              </Badge>
              <Badge
                variant={hasDeepseekKey ? 'default' : 'destructive'}
                className="text-xs"
              >
                {hasDeepseekKey ? '已连接' : '未配置'}
              </Badge>
            </div>
            <h1 className="text-2xl font-semibold text-foreground">AI 对话</h1>
            <p className="text-sm text-muted-foreground">
              与 AI 助手自由对话，获取任务建议、时间管理技巧或其他帮助。
            </p>
          </div>
          <div className="flex items-center gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={() => setShowMetrics(!showMetrics)}
              title={showMetrics ? '隐藏性能指标' : '显示性能指标'}
            >
              <Activity className="mr-2 h-4 w-4" />
              {showMetrics ? '隐藏指标' : '显示指标'}
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={() => setIsSearchDialogOpen(true)}
              disabled={messages.length === 0}
            >
              <Search className="mr-2 h-4 w-4" />
              搜索历史
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={() => setIsExportDialogOpen(true)}
              disabled={messages.length === 0}
            >
              <Download className="mr-2 h-4 w-4" />
              导出对话
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={handleClearChat}
              disabled={messages.length === 0}
            >
              <Trash2 className="mr-2 h-4 w-4" />
              清空对话
            </Button>
            {!hasDeepseekKey && (
              <Button asChild size="sm">
                <Link to="/settings">
                  <Settings2 className="mr-2 h-4 w-4" />
                  配置 API
                </Link>
              </Button>
            )}
          </div>
        </div>
      </header>

      {/* 未配置提示 */}
      {!hasDeepseekKey && (
        <div className="flex items-center gap-3 rounded-2xl border border-amber-500/40 bg-amber-500/10 p-4 text-sm text-amber-700">
          <AlertCircle className="h-5 w-5 shrink-0" />
          <div className="flex-1">
            <span className="font-semibold">需要配置 DeepSeek API Key</span>
            <p className="text-xs text-amber-600 mt-1">
              前往设置页面配置 API Key 后即可开始对话
            </p>
          </div>
          <Button asChild size="sm" variant="outline" className="border-amber-500/40">
            <Link to="/settings">前往配置</Link>
          </Button>
        </div>
      )}

      {/* 记忆服务不可用提示 */}
      {messages.length > 0 && messages[messages.length - 1]?.metadata?.memoryAvailable === false && (
        <div className="flex items-center gap-3 rounded-2xl border border-blue-500/40 bg-blue-500/10 p-4 text-sm text-blue-700">
          <AlertCircle className="h-5 w-5 shrink-0" />
          <div className="flex-1">
            <span className="font-semibold">记忆服务当前不可用</span>
            <p className="text-xs text-blue-600 mt-1">
              AI 正在无状态模式下运行，无法记住之前的对话。您仍然可以进行对话和使用工具，但功能将受到限制。
            </p>
          </div>
        </div>
      )}

      {/* 对话区域 */}
      <div className="flex flex-1 flex-col gap-4 rounded-3xl border border-border/60 bg-card/80 p-6 shadow-sm overflow-hidden">
        {/* 消息列表 */}
        <div className="flex-1 overflow-y-auto space-y-4 pr-2">
          {messages.length === 0 ? (
            <div className="flex h-full flex-col items-center justify-center gap-4 text-center">
              <div className="rounded-full bg-primary/10 p-6">
                <Bot className="h-12 w-12 text-primary" />
              </div>
              <div className="space-y-2">
                <h3 className="text-lg font-semibold text-foreground">
                  开始与 AI 对话
                </h3>
                <p className="text-sm text-muted-foreground max-w-md">
                  你可以询问任何关于任务管理、时间规划的问题，或者寻求其他帮助。
                </p>
              </div>
              <div className="grid gap-2 text-sm">
                <p className="text-muted-foreground">试试这些问题：</p>
                <div className="grid gap-2 sm:grid-cols-2">
                  <button
                    className="rounded-xl border border-border/60 bg-background/80 px-4 py-3 text-left text-sm hover:border-primary/40 hover:bg-primary/5 transition"
                    onClick={() => setInput('如何提高工作效率？')}
                  >
                    💡 如何提高工作效率？
                  </button>
                  <button
                    className="rounded-xl border border-border/60 bg-background/80 px-4 py-3 text-left text-sm hover:border-primary/40 hover:bg-primary/5 transition"
                    onClick={() => setInput('帮我制定一个学习计划')}
                  >
                    📚 帮我制定一个学习计划
                  </button>
                  <button
                    className="rounded-xl border border-border/60 bg-background/80 px-4 py-3 text-left text-sm hover:border-primary/40 hover:bg-primary/5 transition"
                    onClick={() => setInput('如何平衡工作和生活？')}
                  >
                    ⚖️ 如何平衡工作和生活？
                  </button>
                  <button
                    className="rounded-xl border border-border/60 bg-background/80 px-4 py-3 text-left text-sm hover:border-primary/40 hover:bg-primary/5 transition"
                    onClick={() => setInput('推荐一些时间管理技巧')}
                  >
                    ⏰ 推荐一些时间管理技巧
                  </button>
                </div>
              </div>
            </div>
          ) : (
            <>
              {messages.map((message, index) => {
                // Get tool calls for this message if it's the last assistant message
                const isLastAssistantMessage = 
                  message.role === 'assistant' && 
                  index === messages.length - 1;
                const messageToolCalls = isLastAssistantMessage ? toolCalls : [];
                
                return (
                  <MessageBubble 
                    key={message.id} 
                    message={message}
                    toolCalls={messageToolCalls}
                    showMetrics={showMetrics}
                  />
                );
              })}
              {isLoading && (
                <div className="flex items-start gap-3">
                  <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-primary/10">
                    <Bot className="h-5 w-5 text-primary" />
                  </div>
                  <div className="flex-1 rounded-2xl border border-border/60 bg-background/80 px-4 py-3">
                    <div className="flex items-center gap-2 text-sm text-muted-foreground">
                      <Loader2 className="h-4 w-4 animate-spin" />
                      <span>AI 正在思考...</span>
                    </div>
                  </div>
                </div>
              )}
              <div ref={messagesEndRef} />
            </>
          )}
        </div>

        {/* 错误提示 */}
        {error && (
          <div className="flex items-center gap-2 rounded-xl border border-destructive/40 bg-destructive/10 px-4 py-2 text-sm text-destructive">
            <AlertCircle className="h-4 w-4 shrink-0" />
            <span className="flex-1">{error.message}</span>
            <Button
              variant="ghost"
              size="sm"
              className="h-6 px-2"
              onClick={clearError}
            >
              关闭
            </Button>
          </div>
        )}

        {/* 输入框 */}
        <form onSubmit={handleSubmit} className="flex gap-2">
          <textarea
            ref={textareaRef}
            value={input}
            onChange={handleInputChange}
            onKeyDown={handleKeyDown}
            placeholder={
              hasDeepseekKey
                ? '输入消息... (Shift + Enter 换行)'
                : '请先配置 DeepSeek API Key'
            }
            disabled={!hasDeepseekKey || isLoading}
            className="flex-1 resize-none rounded-xl border border-border/60 bg-background px-4 py-3 text-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary disabled:cursor-not-allowed disabled:opacity-50"
            rows={1}
            style={{ minHeight: '48px', maxHeight: '200px' }}
          />
          <Button
            type="submit"
            size="icon"
            disabled={!hasDeepseekKey || !input.trim() || isLoading}
            className="h-12 w-12 shrink-0"
          >
            {isLoading ? (
              <Loader2 className="h-5 w-5 animate-spin" />
            ) : (
              <Send className="h-5 w-5" />
            )}
          </Button>
        </form>
      </div>

      {/* Memory Search Dialog */}
      <MemorySearchDialog
        isOpen={isSearchDialogOpen}
        onClose={() => setIsSearchDialogOpen(false)}
        onSearch={handleSearchConversations}
      />

      {/* Conversation Export Dialog */}
      <ConversationExportDialog
        isOpen={isExportDialogOpen}
        onClose={() => setIsExportDialogOpen(false)}
        onExport={handleExportConversation}
        conversationId={conversationId}
        messageCount={messages.length}
      />
    </section>
  );
}
