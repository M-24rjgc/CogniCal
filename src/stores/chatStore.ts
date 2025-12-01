import { create } from 'zustand';
import {
  chatWithAgent,
  searchConversations as searchConversationsApi,
  exportConversation as exportConversationApi,
  clearConversation as clearConversationApi,
} from '../services/tauriApi';
import type { AppError } from '../services/tauriApi';
import { useTaskStore } from './taskStore';

export interface ChatMessage {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  timestamp: string;
  metadata?: ChatMessageMetadata;
}

export interface ChatMessageMetadata {
  tokensUsed?: Record<string, number>;
  latencyMs?: number;
  memoryEntriesUsed?: number;
  toolsExecuted?: string[];
  correlationId?: string;
  errors?: ErrorDetail[];
  memoryAvailable?: boolean;
}

export interface ErrorDetail {
  errorType: string;
  message: string;
  timestamp: string;
  context?: Record<string, string>;
}

export interface ToolCallStatus {
  id: string;
  toolName: string;
  status: 'pending' | 'executing' | 'completed' | 'failed';
  result?: unknown;
  error?: string;
}

type AgentToolCall = {
  id?: string;
  name?: string;
  result?: unknown;
};

export interface MemorySearchResult {
  conversationId: string;
  userMessage: string;
  assistantMessage: string;
  timestamp: string;
  relevanceScore: number;
  metadata?: {
    topics?: string;
  };
}

interface ChatStoreState {
  messages: ChatMessage[];
  conversationId: string;
  toolCalls: ToolCallStatus[];
  searchResults: MemorySearchResult[];
  isLoading: boolean;
  error: AppError | null;
  sendMessage: (content: string) => Promise<void>;
  clearMessages: () => void;
  clearError: () => void;
  searchConversations: (query: string) => Promise<MemorySearchResult[]>;
  exportConversation: () => Promise<string>;
  clearConversation: () => Promise<void>;
}

const TASK_SYNC_TOOLS = new Set([
  'create_task',
  'update_task',
  'delete_task',
  'create_time_block',
  'update_time_item',
  'quick_schedule',
  'associate_task_with_goal',
  'add_task_dependency',
  'remove_task_dependency',
]);

const syncStoresAfterTools = (toolsExecuted?: string[]) => {
  if (!toolsExecuted || toolsExecuted.length === 0) {
    return;
  }

  const shouldRefreshTasks = toolsExecuted.some((tool) => TASK_SYNC_TOOLS.has(tool));
  if (shouldRefreshTasks) {
    const { fetchTasks } = useTaskStore.getState();
    void fetchTasks();
  }
};

// Generate a unique conversation ID
const generateConversationId = (): string => {
  return `conv-${Date.now()}-${Math.random().toString(36).substring(2, 9)}`;
};

export const useChatStore = create<ChatStoreState>((set, get) => ({
  messages: [],
  conversationId: generateConversationId(),
  toolCalls: [],
  searchResults: [],
  isLoading: false,
  error: null,

  sendMessage: async (content: string) => {
    const userMessage: ChatMessage = {
      id: `user-${Date.now()}`,
      role: 'user',
      content,
      timestamp: new Date().toISOString(),
    };

    set({
      messages: [...get().messages, userMessage],
      isLoading: true,
      error: null,
      toolCalls: [],
    });

    try {
      const { conversationId } = get();
      const response = await chatWithAgent(conversationId, content);

      // Update tool call status if there are tool calls
      if (response.toolCalls && response.toolCalls.length > 0) {
        const toolCalls = response.toolCalls as AgentToolCall[];
        const toolCallStatuses: ToolCallStatus[] = toolCalls.map((toolCall) => ({
          id: toolCall.id ?? `tool-${Date.now()}-${Math.random()}`,
          toolName: toolCall.name ?? 'unknown',
          status: 'completed' as const,
          result: toolCall.result,
        }));

        set({ toolCalls: toolCallStatuses });
      }

      syncStoresAfterTools(response.metadata?.toolsExecuted);

      const assistantMessage: ChatMessage = {
        id: `assistant-${Date.now()}`,
        role: 'assistant',
        content: response.message,
        timestamp: new Date().toISOString(),
        metadata: {
          tokensUsed: response.metadata.tokensUsed,
          latencyMs: response.metadata.latencyMs,
          memoryEntriesUsed: response.metadata.memoryEntriesUsed,
          toolsExecuted: response.metadata.toolsExecuted,
          correlationId: response.metadata.correlationId,
          errors: response.metadata.errors,
          memoryAvailable: response.metadata.memoryAvailable,
        },
      };

      set({
        messages: [...get().messages, assistantMessage],
        isLoading: false,
      });
    } catch (error) {
      set({
        isLoading: false,
        error: error as AppError,
      });
    }
  },

  clearMessages: () => {
    set({
      messages: [],
      conversationId: generateConversationId(),
      toolCalls: [],
      searchResults: [],
      error: null,
    });
  },

  clearError: () => {
    set({ error: null });
  },

  searchConversations: async (query: string): Promise<MemorySearchResult[]> => {
    if (!query.trim()) {
      set({ searchResults: [] });
      return [];
    }

    try {
      set({ isLoading: true, error: null });
      const results = await searchConversationsApi(query);
      set({ searchResults: results, isLoading: false });
      return results;
    } catch (error) {
      set({
        isLoading: false,
        error: error as AppError,
        searchResults: [],
      });
      return [];
    }
  },

  exportConversation: async (): Promise<string> => {
    const { conversationId } = get();

    try {
      set({ isLoading: true, error: null });
      const exportPath = await exportConversationApi(conversationId);
      set({ isLoading: false });
      return exportPath;
    } catch (error) {
      set({
        isLoading: false,
        error: error as AppError,
      });
      throw error;
    }
  },

  clearConversation: async (): Promise<void> => {
    const { conversationId } = get();

    // Show confirmation dialog
    const confirmed = window.confirm(
      '确定要清除当前对话历史吗？此操作将归档对话记录，但不会永久删除。',
    );

    if (!confirmed) {
      return;
    }

    try {
      set({ isLoading: true, error: null });
      await clearConversationApi(conversationId);

      // Reset the chat state
      set({
        messages: [],
        conversationId: generateConversationId(),
        toolCalls: [],
        searchResults: [],
        isLoading: false,
        error: null,
      });
    } catch (error) {
      set({
        isLoading: false,
        error: error as AppError,
      });
      throw error;
    }
  },
}));
