import { create } from 'zustand';
import { chatWithAI } from '../services/tauriApi';
import type { AppError } from '../services/tauriApi';

export interface ChatMessage {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  timestamp: string;
}

interface ChatStoreState {
  messages: ChatMessage[];
  isLoading: boolean;
  error: AppError | null;
  sendMessage: (content: string) => Promise<void>;
  clearMessages: () => void;
  clearError: () => void;
}

export const useChatStore = create<ChatStoreState>((set, get) => ({
  messages: [],
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
    });

    try {
      const response = await chatWithAI(content);

      const assistantMessage: ChatMessage = {
        id: `assistant-${Date.now()}`,
        role: 'assistant',
        content: response.message,
        timestamp: new Date().toISOString(),
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
    set({ messages: [], error: null });
  },

  clearError: () => {
    set({ error: null });
  },
}));
