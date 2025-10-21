/**
 * Python Memory Service API
 * Wrapper around tauri-plugin-python for memory operations
 */

import { callFunction } from 'tauri-plugin-python-api';

export interface MemoryStatus {
  initialized: boolean;
  kb_path: string | null;

  python_version: string;
  available: boolean;
}

export interface SearchResult {
  conversation_id: string;
  user_message: string;
  assistant_message: string;
  timestamp: string;
  relevance_score: number;
}

export interface MemorySearchResponse {
  success: boolean;
  query: string;
  results: SearchResult[];
  message?: string;
  error?: string;
}

export interface MemoryStoreResponse {
  success: boolean;
  conversation_id: string;
  message?: string;
  error?: string;
}

export interface MemoryInitResponse {
  success: boolean;
  message: string;
  kb_path?: string;

  error?: string;
}

/**
 * Initialize the memory service
 */
export async function initializeMemory(kbPath: string): Promise<MemoryInitResponse> {
  try {
    const result = await callFunction('initialize_memory', [kbPath]);
    return result as unknown as MemoryInitResponse;
  } catch (error) {
    console.error('Failed to initialize memory:', error);
    return {
      success: false,
      message: `Failed to initialize: ${error}`,
      error: String(error),
    };
  }
}

/**
 * Search conversation memory
 */
export async function searchMemory(
  query: string,
  limit: number = 5,
): Promise<MemorySearchResponse> {
  try {
    const result = await callFunction('search_memory', [query, limit]);
    return result as unknown as MemorySearchResponse;
  } catch (error) {
    console.error('Failed to search memory:', error);
    return {
      success: false,
      query,
      results: [],
      message: `Search failed: ${error}`,
      error: String(error),
    };
  }
}

/**
 * Store a conversation turn
 */
export async function storeConversation(
  conversationId: string,
  userMessage: string,
  assistantMessage: string,
  metadata?: Record<string, string>,
): Promise<MemoryStoreResponse> {
  try {
    const result = await callFunction('store_conversation', [
      conversationId,
      userMessage,
      assistantMessage,
      metadata || {},
    ]);
    return result as unknown as MemoryStoreResponse;
  } catch (error) {
    console.error('Failed to store conversation:', error);
    return {
      success: false,
      conversation_id: conversationId,
      message: `Storage failed: ${error}`,
      error: String(error),
    };
  }
}

/**
 * Get memory service status
 */
export async function getMemoryStatus(): Promise<MemoryStatus> {
  try {
    const result = await callFunction('get_memory_status', []);
    return result as unknown as MemoryStatus;
  } catch (error) {
    console.error('Failed to get memory status:', error);
    return {
      initialized: false,
      kb_path: null,

      python_version: 'unknown',
      available: false,
    };
  }
}

/**
 * Test Python plugin connection using Tauri command
 */
export async function testConnection(): Promise<{
  success: boolean;
  message: string;
  python_version?: string;
  platform?: string;
  error?: string;
}> {
  try {
    // Use Tauri invoke instead of callFunction for better error handling
    const { invoke } = await import('@tauri-apps/api/core');
    const result = await invoke('test_python_connection');
    return result as any;
  } catch (error) {
    console.error('Failed to test connection:', error);
    return {
      success: false,
      message: `Connection test failed: ${error}`,
      error: String(error),
    };
  }
}
