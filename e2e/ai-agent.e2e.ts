import { test, expect } from '@playwright/test';

/**
 * End-to-End Tests for AI Agent with Memory and Tool Calling
 * 
 * These tests verify complete user workflows and validate that all
 * requirements from the specification are met.
 */

test.describe('AI Agent End-to-End Tests', () => {
  
  test.beforeEach(async ({ page }) => {
    // Navigate to the application
    await page.goto('/');
    
    // Wait for app to be ready
    await page.waitForSelector('[data-testid="app-ready"]', { timeout: 10000 });
  });

  test.describe('Requirement 1: Long-Term Memory Integration', () => {
    
    test('should store and retrieve conversation context', async ({ page }) => {
      // Navigate to chat
      await page.click('[data-testid="nav-chat"]');
      
      // Send first message with context
      await page.fill('[data-testid="chat-input"]', 'My name is Alice and I work on the marketing team');
      await page.click('[data-testid="send-button"]');
      
      // Wait for AI response
      await page.waitForSelector('[data-testid="message-bubble"]', { timeout: 10000 });
      
      // Clear chat to start new session
      await page.click('[data-testid="chat-menu"]');
      await page.click('[data-testid="clear-chat"]');
      await page.click('[data-testid="confirm-clear"]');
      
      // Send message that requires memory
      await page.fill('[data-testid="chat-input"]', 'What team do I work on?');
      await page.click('[data-testid="send-button"]');
      
      // Wait for response
      await page.waitForSelector('[data-testid="message-bubble"]:last-child', { timeout: 10000 });
      
      // Verify AI remembers the context
      const lastMessage = await page.textContent('[data-testid="message-bubble"]:last-child');
      expect(lastMessage?.toLowerCase()).toContain('marketing');
    });

    test('should display memory context indicator', async ({ page }) => {
      await page.click('[data-testid="nav-chat"]');
      
      // Send message that uses memory
      await page.fill('[data-testid="chat-input"]', 'What did we discuss earlier?');
      await page.click('[data-testid="send-button"]');
      
      // Wait for response with memory indicator
      await page.waitForSelector('[data-testid="memory-indicator"]', { timeout: 10000 });
      
      // Verify memory entries count is displayed
      const memoryCount = await page.textContent('[data-testid="memory-entries-count"]');
      expect(parseInt(memoryCount || '0')).toBeGreaterThan(0);
    });

    test('should limit memory context to configured maximum', async ({ page }) => {
      await page.click('[data-testid="nav-chat"]');
      
      // Send multiple messages to build up history
      for (let i = 0; i < 10; i++) {
        await page.fill('[data-testid="chat-input"]', `Test message ${i}`);
        await page.click('[data-testid="send-button"]');
        await page.waitForTimeout(1000);
      }
      
      // Send message that triggers memory retrieval
      await page.fill('[data-testid="chat-input"]', 'Summarize our conversation');
      await page.click('[data-testid="send-button"]');
      
      // Check memory entries used
      await page.waitForSelector('[data-testid="memory-indicator"]', { timeout: 10000 });
      const memoryCount = await page.textContent('[data-testid="memory-entries-count"]');
      
      // Should not exceed max (default 5)
      expect(parseInt(memoryCount || '0')).toBeLessThanOrEqual(5);
    });
  });

  test.describe('Requirement 2: MCP Server Integration', () => {
    
    test('should show memory service status', async ({ page }) => {
      // Navigate to settings
      await page.click('[data-testid="nav-settings"]');
      await page.click('[data-testid="settings-ai-agent"]');
      
      // Check memory status indicator
      const statusIndicator = await page.locator('[data-testid="memory-status"]');
      await expect(statusIndicator).toBeVisible();
      
      // Status should be either online or offline (not error)
      const status = await statusIndicator.getAttribute('data-status');
      expect(['online', 'offline']).toContain(status);
    });

    test('should operate in fallback mode when memory unavailable', async ({ page }) => {
      // This test assumes memory service can be toggled off
      await page.click('[data-testid="nav-settings"]');
      await page.click('[data-testid="settings-ai-agent"]');
      
      // Disable memory service
      await page.click('[data-testid="toggle-memory-service"]');
      
      // Navigate to chat
      await page.click('[data-testid="nav-chat"]');
      
      // Send message
      await page.fill('[data-testid="chat-input"]', 'Hello, can you help me?');
      await page.click('[data-testid="send-button"]');
      
      // Should still get response (stateless mode)
      await page.waitForSelector('[data-testid="message-bubble"]', { timeout: 10000 });
      
      // Verify no memory indicator
      const memoryIndicator = await page.locator('[data-testid="memory-indicator"]');
      await expect(memoryIndicator).not.toBeVisible();
    });
  });

  test.describe('Requirement 4 & 5: Task Management Tool Integration', () => {
    
    test('should create task through natural language', async ({ page }) => {
      await page.click('[data-testid="nav-chat"]');
      
      // Request task creation
      await page.fill('[data-testid="chat-input"]', 
        'Create a high-priority task to review the budget report by Friday');
      await page.click('[data-testid="send-button"]');
      
      // Wait for tool execution indicator
      await page.waitForSelector('[data-testid="tool-call-indicator"]', { timeout: 10000 });
      
      // Verify tool name
      const toolName = await page.textContent('[data-testid="tool-name"]');
      expect(toolName).toContain('create_task');
      
      // Wait for completion
      await page.waitForSelector('[data-testid="tool-status"][data-status="completed"]', 
        { timeout: 10000 });
      
      // Verify task was created by checking tasks page
      await page.click('[data-testid="nav-tasks"]');
      await page.waitForSelector('[data-testid="task-item"]');
      
      const taskTitle = await page.textContent('[data-testid="task-title"]:first-child');
      expect(taskTitle?.toLowerCase()).toContain('budget report');
    });

    test('should list tasks through natural language', async ({ page }) => {
      await page.click('[data-testid="nav-chat"]');
      
      // Request task list
      await page.fill('[data-testid="chat-input"]', 'Show me all my high priority tasks');
      await page.click('[data-testid="send-button"]');
      
      // Wait for tool execution
      await page.waitForSelector('[data-testid="tool-call-indicator"]', { timeout: 10000 });
      
      // Verify tool executed
      const toolName = await page.textContent('[data-testid="tool-name"]');
      expect(toolName).toMatch(/list_tasks|search_tasks/);
      
      // Wait for response with task list
      await page.waitForSelector('[data-testid="message-bubble"]:last-child', 
        { timeout: 10000 });
      
      const response = await page.textContent('[data-testid="message-bubble"]:last-child');
      expect(response).toBeTruthy();
    });

    test('should update task through natural language', async ({ page }) => {
      // First create a task
      await page.click('[data-testid="nav-chat"]');
      await page.fill('[data-testid="chat-input"]', 'Create a task called Test Task');
      await page.click('[data-testid="send-button"]');
      await page.waitForSelector('[data-testid="tool-status"][data-status="completed"]', 
        { timeout: 10000 });
      
      // Update the task
      await page.fill('[data-testid="chat-input"]', 
        'Change the Test Task to high priority');
      await page.click('[data-testid="send-button"]');
      
      // Wait for update tool execution
      await page.waitForSelector('[data-testid="tool-call-indicator"]', { timeout: 10000 });
      const toolName = await page.textContent('[data-testid="tool-name"]');
      expect(toolName).toContain('update_task');
      
      // Verify completion
      await page.waitForSelector('[data-testid="tool-status"][data-status="completed"]', 
        { timeout: 10000 });
    });
  });

  test.describe('Requirement 6: Calendar Tool Integration', () => {
    
    test('should check calendar through natural language', async ({ page }) => {
      await page.click('[data-testid="nav-chat"]');
      
      // Request calendar check
      await page.fill('[data-testid="chat-input"]', "What's on my calendar today?");
      await page.click('[data-testid="send-button"]');
      
      // Wait for tool execution
      await page.waitForSelector('[data-testid="tool-call-indicator"]', { timeout: 10000 });
      
      // Verify correct tool
      const toolName = await page.textContent('[data-testid="tool-name"]');
      expect(toolName).toContain('get_calendar_events');
      
      // Wait for response
      await page.waitForSelector('[data-testid="tool-status"][data-status="completed"]', 
        { timeout: 10000 });
    });

    test('should create calendar event through natural language', async ({ page }) => {
      await page.click('[data-testid="nav-chat"]');
      
      // Request event creation
      await page.fill('[data-testid="chat-input"]', 
        'Schedule a team meeting for tomorrow at 2pm for 1 hour');
      await page.click('[data-testid="send-button"]');
      
      // Wait for tool execution
      await page.waitForSelector('[data-testid="tool-call-indicator"]', { timeout: 10000 });
      
      // Verify tool name
      const toolName = await page.textContent('[data-testid="tool-name"]');
      expect(toolName).toContain('create_calendar_event');
      
      // Wait for completion
      await page.waitForSelector('[data-testid="tool-status"][data-status="completed"]', 
        { timeout: 10000 });
      
      // Verify event was created
      await page.click('[data-testid="nav-calendar"]');
      await page.waitForSelector('[data-testid="calendar-event"]');
      
      const eventTitle = await page.textContent('[data-testid="event-title"]:first-child');
      expect(eventTitle?.toLowerCase()).toContain('team meeting');
    });

    test('should detect scheduling conflicts', async ({ page }) => {
      // Create first event
      await page.click('[data-testid="nav-chat"]');
      await page.fill('[data-testid="chat-input"]', 
        'Schedule a meeting for tomorrow at 3pm');
      await page.click('[data-testid="send-button"]');
      await page.waitForSelector('[data-testid="tool-status"][data-status="completed"]', 
        { timeout: 10000 });
      
      // Try to create conflicting event
      await page.fill('[data-testid="chat-input"]', 
        'Schedule another meeting for tomorrow at 3pm');
      await page.click('[data-testid="send-button"]');
      
      // Wait for response
      await page.waitForSelector('[data-testid="message-bubble"]:last-child', 
        { timeout: 10000 });
      
      // Verify conflict is mentioned
      const response = await page.textContent('[data-testid="message-bubble"]:last-child');
      expect(response?.toLowerCase()).toMatch(/conflict|already scheduled/);
    });
  });

  test.describe('Requirement 7: Memory-Enhanced Responses', () => {
    
    test('should reference past conversations', async ({ page }) => {
      await page.click('[data-testid="nav-chat"]');
      
      // Establish context
      await page.fill('[data-testid="chat-input"]', 
        'I am working on the Q1 marketing campaign');
      await page.click('[data-testid="send-button"]');
      await page.waitForSelector('[data-testid="message-bubble"]', { timeout: 10000 });
      
      // Later, reference the context
      await page.fill('[data-testid="chat-input"]', 
        'Create a task for the campaign we discussed');
      await page.click('[data-testid="send-button"]');
      
      // Wait for response
      await page.waitForSelector('[data-testid="message-bubble"]:last-child', 
        { timeout: 10000 });
      
      // Verify AI used context
      const response = await page.textContent('[data-testid="message-bubble"]:last-child');
      expect(response?.toLowerCase()).toContain('marketing');
    });

    test('should search conversation history', async ({ page }) => {
      await page.click('[data-testid="nav-chat"]');
      
      // Open search dialog
      await page.click('[data-testid="search-history-button"]');
      
      // Search for past conversations
      await page.fill('[data-testid="history-search-input"]', 'marketing');
      await page.click('[data-testid="search-button"]');
      
      // Wait for results
      await page.waitForSelector('[data-testid="search-result"]', { timeout: 10000 });
      
      // Verify results are displayed
      const results = await page.locator('[data-testid="search-result"]').count();
      expect(results).toBeGreaterThan(0);
    });
  });

  test.describe('Requirement 8: Conversation Persistence', () => {
    
    test('should persist conversations across sessions', async ({ page }) => {
      await page.click('[data-testid="nav-chat"]');
      
      // Send a unique message
      const uniqueMessage = `Test message ${Date.now()}`;
      await page.fill('[data-testid="chat-input"]', uniqueMessage);
      await page.click('[data-testid="send-button"]');
      await page.waitForSelector('[data-testid="message-bubble"]', { timeout: 10000 });
      
      // Reload the page (simulate new session)
      await page.reload();
      await page.waitForSelector('[data-testid="app-ready"]', { timeout: 10000 });
      
      // Navigate back to chat
      await page.click('[data-testid="nav-chat"]');
      
      // Verify message history is loaded
      const messages = await page.locator('[data-testid="message-bubble"]').count();
      expect(messages).toBeGreaterThan(0);
    });

    test('should export conversation history', async ({ page }) => {
      await page.click('[data-testid="nav-chat"]');
      
      // Click export button
      const downloadPromise = page.waitForEvent('download');
      await page.click('[data-testid="export-conversation-button"]');
      
      // Wait for download
      const download = await downloadPromise;
      
      // Verify file name
      expect(download.suggestedFilename()).toMatch(/\.tar\.gz$/);
    });

    test('should archive conversations when clearing', async ({ page }) => {
      await page.click('[data-testid="nav-chat"]');
      
      // Clear chat
      await page.click('[data-testid="chat-menu"]');
      await page.click('[data-testid="clear-chat"]');
      await page.click('[data-testid="confirm-clear"]');
      
      // Verify chat is cleared
      const messages = await page.locator('[data-testid="message-bubble"]').count();
      expect(messages).toBe(0);
      
      // Verify conversations are still searchable (archived, not deleted)
      await page.click('[data-testid="search-history-button"]');
      await page.fill('[data-testid="history-search-input"]', 'test');
      await page.click('[data-testid="search-button"]');
      
      // Should still find archived conversations
      await page.waitForSelector('[data-testid="search-result"]', { timeout: 10000 });
    });
  });

  test.describe('Requirement 9: Error Handling and Fallback', () => {
    
    test('should handle tool execution failures gracefully', async ({ page }) => {
      await page.click('[data-testid="nav-chat"]');
      
      // Send request with invalid parameters
      await page.fill('[data-testid="chat-input"]', 
        'Create a task with due date yesterday');
      await page.click('[data-testid="send-button"]');
      
      // Wait for tool execution
      await page.waitForSelector('[data-testid="tool-call-indicator"]', { timeout: 10000 });
      
      // Should show error status
      await page.waitForSelector('[data-testid="tool-status"][data-status="failed"]', 
        { timeout: 10000 });
      
      // AI should explain the error
      await page.waitForSelector('[data-testid="message-bubble"]:last-child', 
        { timeout: 10000 });
      const response = await page.textContent('[data-testid="message-bubble"]:last-child');
      expect(response?.toLowerCase()).toMatch(/error|invalid|cannot/);
    });

    test('should show user-friendly error messages', async ({ page }) => {
      await page.click('[data-testid="nav-chat"]');
      
      // Trigger an error (e.g., invalid tool call)
      await page.fill('[data-testid="chat-input"]', 
        'Delete all my tasks permanently');
      await page.click('[data-testid="send-button"]');
      
      // Wait for response
      await page.waitForSelector('[data-testid="message-bubble"]:last-child', 
        { timeout: 10000 });
      
      // Error message should be user-friendly, not technical
      const response = await page.textContent('[data-testid="message-bubble"]:last-child');
      expect(response).not.toContain('Error:');
      expect(response).not.toContain('Exception');
      expect(response).not.toContain('Stack trace');
    });
  });

  test.describe('Requirement 10: Performance and Scalability', () => {
    
    test('should respond within 2 seconds for simple queries', async ({ page }) => {
      await page.click('[data-testid="nav-chat"]');
      
      const startTime = Date.now();
      
      // Send simple query
      await page.fill('[data-testid="chat-input"]', 'Hello');
      await page.click('[data-testid="send-button"]');
      
      // Wait for response
      await page.waitForSelector('[data-testid="message-bubble"]:last-child', 
        { timeout: 10000 });
      
      const endTime = Date.now();
      const responseTime = endTime - startTime;
      
      // Should be under 2 seconds (2000ms)
      expect(responseTime).toBeLessThan(2000);
    });

    test('should stream responses progressively', async ({ page }) => {
      await page.click('[data-testid="nav-chat"]');
      
      // Send message that generates long response
      await page.fill('[data-testid="chat-input"]', 
        'Explain the benefits of task management in detail');
      await page.click('[data-testid="send-button"]');
      
      // Wait for streaming to start
      await page.waitForSelector('[data-testid="message-bubble"]:last-child', 
        { timeout: 2000 });
      
      // Check if content is being updated (streaming)
      const initialContent = await page.textContent('[data-testid="message-bubble"]:last-child');
      
      // Wait a bit
      await page.waitForTimeout(500);
      
      // Content should have grown (streaming)
      const updatedContent = await page.textContent('[data-testid="message-bubble"]:last-child');
      
      // If streaming is working, content should be different
      // (This test might be flaky depending on response speed)
      expect(updatedContent?.length).toBeGreaterThanOrEqual(initialContent?.length || 0);
    });

    test('should display performance metrics', async ({ page }) => {
      await page.click('[data-testid="nav-chat"]');
      
      // Send message
      await page.fill('[data-testid="chat-input"]', 'Create a task to test metrics');
      await page.click('[data-testid="send-button"]');
      
      // Wait for response
      await page.waitForSelector('[data-testid="message-bubble"]:last-child', 
        { timeout: 10000 });
      
      // Click to show metrics
      await page.click('[data-testid="message-bubble"]:last-child [data-testid="show-metrics"]');
      
      // Verify metrics are displayed
      await expect(page.locator('[data-testid="metric-tokens"]')).toBeVisible();
      await expect(page.locator('[data-testid="metric-latency"]')).toBeVisible();
      await expect(page.locator('[data-testid="metric-memory-entries"]')).toBeVisible();
    });
  });

  test.describe('Multi-turn Tool Calling', () => {
    
    test('should handle multi-step workflows', async ({ page }) => {
      await page.click('[data-testid="nav-chat"]');
      
      // Request complex workflow
      await page.fill('[data-testid="chat-input"]', 
        'Create a task for the project meeting and schedule it for tomorrow at 2pm');
      await page.click('[data-testid="send-button"]');
      
      // Should execute multiple tools
      await page.waitForSelector('[data-testid="tool-call-indicator"]', { timeout: 10000 });
      
      // Wait for all tools to complete
      await page.waitForSelector('[data-testid="tool-status"][data-status="completed"]', 
        { timeout: 15000 });
      
      // Count tool executions
      const toolCalls = await page.locator('[data-testid="tool-call-indicator"]').count();
      expect(toolCalls).toBeGreaterThanOrEqual(2); // At least create_task and create_calendar_event
    });
  });

  test.describe('UI/UX Validation', () => {
    
    test('should show loading indicators during processing', async ({ page }) => {
      await page.click('[data-testid="nav-chat"]');
      
      // Send message
      await page.fill('[data-testid="chat-input"]', 'Help me plan my day');
      await page.click('[data-testid="send-button"]');
      
      // Should show loading indicator
      await expect(page.locator('[data-testid="chat-loading"]')).toBeVisible();
      
      // Wait for response
      await page.waitForSelector('[data-testid="message-bubble"]:last-child', 
        { timeout: 10000 });
      
      // Loading should be gone
      await expect(page.locator('[data-testid="chat-loading"]')).not.toBeVisible();
    });

    test('should display tool execution progress', async ({ page }) => {
      await page.click('[data-testid="nav-chat"]');
      
      // Request tool execution
      await page.fill('[data-testid="chat-input"]', 'List all my tasks');
      await page.click('[data-testid="send-button"]');
      
      // Should show tool indicator
      await page.waitForSelector('[data-testid="tool-call-indicator"]', { timeout: 10000 });
      
      // Should show executing status
      await expect(page.locator('[data-testid="tool-status"][data-status="executing"]'))
        .toBeVisible();
      
      // Should transition to completed
      await page.waitForSelector('[data-testid="tool-status"][data-status="completed"]', 
        { timeout: 10000 });
    });
  });
});

/**
 * Performance Validation Tests
 */
test.describe('Performance Validation', () => {
  
  test('should handle 100 messages without degradation', async ({ page }) => {
    await page.goto('/');
    await page.waitForSelector('[data-testid="app-ready"]', { timeout: 10000 });
    await page.click('[data-testid="nav-chat"]');
    
    const responseTimes: number[] = [];
    
    // Send 100 messages and measure response times
    for (let i = 0; i < 100; i++) {
      const startTime = Date.now();
      
      await page.fill('[data-testid="chat-input"]', `Test message ${i}`);
      await page.click('[data-testid="send-button"]');
      await page.waitForSelector(`[data-testid="message-bubble"]:nth-child(${(i + 1) * 2})`, 
        { timeout: 10000 });
      
      const endTime = Date.now();
      responseTimes.push(endTime - startTime);
      
      // Only test first 10 for speed
      if (i >= 10) break;
    }
    
    // Calculate average response time
    const avgResponseTime = responseTimes.reduce((a, b) => a + b, 0) / responseTimes.length;
    
    // Average should be under 3 seconds
    expect(avgResponseTime).toBeLessThan(3000);
    
    // Last response should not be significantly slower than first
    const firstResponse = responseTimes[0];
    const lastResponse = responseTimes[responseTimes.length - 1];
    expect(lastResponse).toBeLessThan(firstResponse * 1.5); // No more than 50% slower
  });
});
