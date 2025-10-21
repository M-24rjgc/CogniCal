# AI Agent User Guide

## Overview

CogniCal's AI Agent is an intelligent assistant that can remember your conversations, learn from your interactions, and perform actions on your behalf. Unlike traditional chatbots, the AI Agent has:

- **Long-term Memory**: Remembers past conversations and learns your preferences
- **Tool Capabilities**: Can create tasks, schedule events, and manage your productivity
- **Natural Conversation**: Understands context and responds naturally

## Getting Started

### Accessing the AI Agent

1. Click on the **Chat** tab in the navigation menu
2. Type your message in the input field at the bottom
3. Press Enter or click the Send button

The AI Agent will respond with helpful information and can perform actions when needed.

## Core Features

### 1. Conversational Memory

The AI Agent remembers your conversations and uses this context to provide better responses.

**How it works:**
- Every conversation is stored with semantic embeddings
- When you send a message, the AI retrieves relevant past conversations
- The AI uses this context to understand your preferences and history

**Example conversations:**

```
You: I need to prepare for the quarterly review meeting
AI: I remember you mentioned the quarterly review is on March 15th. 
    Would you like me to create a task to prepare the presentation?

You: Yes, make it high priority
AI: I've created a high-priority task "Prepare quarterly review presentation" 
    with a due date of March 14th (one day before the meeting).
```

**Benefits:**
- No need to repeat context in every conversation
- The AI learns your work patterns and preferences
- Seamless continuation of previous discussions

### 2. Task Management

The AI Agent can create, update, and manage your tasks through natural conversation.

#### Creating Tasks

Simply tell the AI what you need to do:

```
You: I need to finish the budget report by Friday
AI: I've created a task "Finish budget report" with a due date of 
    Friday, January 26th. Would you like me to set a priority level?

You: Make it high priority and add a tag for finance
AI: Updated the task to high priority and added the "finance" tag.
```

**What you can specify:**
- Task title and description
- Priority (low, medium, high)
- Due date (natural language like "tomorrow", "next Monday", "in 3 days")
- Tags for organization

#### Updating Tasks

```
You: Change the budget report deadline to next Monday
AI: I've updated the "Finish budget report" task with a new due date 
    of Monday, January 29th.

You: Actually, mark it as completed
AI: Great! I've marked the "Finish budget report" task as completed.
```

#### Listing and Searching Tasks

```
You: What tasks do I have due this week?
AI: You have 3 tasks due this week:
    1. Finish budget report (High priority) - Due Friday
    2. Review team proposals (Medium priority) - Due Thursday
    3. Update project timeline (Low priority) - Due Saturday

You: Show me all high priority tasks
AI: Here are your high priority tasks:
    1. Finish budget report - Due Friday, January 26th
    2. Prepare quarterly review presentation - Due March 14th
```

### 3. Calendar Management

The AI Agent can check your schedule and create calendar events.

#### Checking Your Schedule

```
You: What's on my calendar today?
AI: You have 2 events today:
    - 10:00 AM: Team standup (30 minutes)
    - 2:00 PM: Client meeting (1 hour)

You: Am I free tomorrow afternoon?
AI: Tomorrow afternoon you have:
    - 3:00 PM: Project review meeting (1 hour)
    You're free before 3:00 PM.
```

#### Scheduling Events

```
You: Schedule a meeting with the design team for next Tuesday at 2pm
AI: I've created a calendar event "Meeting with design team" for 
    Tuesday, January 30th at 2:00 PM. How long should the meeting be?

You: Make it 1 hour
AI: Updated the event duration to 1 hour (2:00 PM - 3:00 PM).
```

#### Conflict Detection

```
You: Schedule a call with Sarah tomorrow at 10am
AI: I notice you already have "Team standup" scheduled at 10:00 AM tomorrow. 
    Would you like me to schedule the call at a different time?

You: How about 11am?
AI: Perfect! I've scheduled "Call with Sarah" for tomorrow at 11:00 AM.
```

### 4. Natural Language Understanding

The AI Agent understands natural language, so you can communicate casually.

**Examples of natural commands:**

```
"Remind me to call John next week"
→ Creates a task with due date next week

"What did we discuss about the marketing campaign?"
→ Searches memory for relevant conversations

"I'm free on Thursday, schedule the team meeting then"
→ Creates a calendar event on Thursday

"Show me everything due before the end of the month"
→ Lists all tasks with due dates before month end

"Move all my Friday meetings to Monday"
→ Updates multiple calendar events
```

## Advanced Features

### Memory Search

You can search through your conversation history to find specific information.

**How to search:**
1. Click the **Search History** button in the chat header
2. Enter your search query
3. Browse through relevant past conversations

**Example searches:**
- "budget discussions" - Find all conversations about budgets
- "meeting with Sarah" - Find conversations mentioning Sarah
- "project deadlines" - Find discussions about deadlines

### Exporting Conversations

You can export your conversation history for backup or reference.

**How to export:**
1. Click the **Export Conversation** button in the chat header
2. Choose a location to save the file
3. The system exports a `.tar.gz` file containing your knowledge base

### Performance Metrics

The AI Agent displays performance information for transparency:

- **Token Usage**: How many tokens were used in the conversation
- **Response Time**: How long the AI took to respond
- **Memory Entries**: How many past conversations were used for context
- **Tools Used**: Which tools were executed (tasks, calendar, etc.)

**Viewing metrics:**
- Click the info icon on any message to see detailed metrics
- Toggle metrics display in Settings > AI Agent

## Tips for Best Results

### 1. Be Specific When Needed

While the AI understands natural language, being specific helps:

**Good:**
```
"Create a high-priority task to review the Q1 report by next Friday"
```

**Also works:**
```
"I need to review the Q1 report"
AI: When would you like to complete this?
You: Next Friday, and make it high priority
```

### 2. Use Context from Memory

The AI remembers your conversations, so reference past discussions:

```
"Remember that project we discussed last week? Create a task for the next phase"
```

### 3. Combine Multiple Actions

You can request multiple actions in one message:

```
"Create a task to prepare the presentation and schedule a review meeting 
for next Thursday at 3pm"
```

### 4. Ask for Clarification

If the AI's response isn't what you expected, ask for clarification:

```
You: Schedule a meeting tomorrow
AI: I've scheduled a meeting for tomorrow. What time would you like?
You: 2pm for 30 minutes with the title "Budget Review"
AI: Perfect! Created "Budget Review" for tomorrow at 2:00 PM (30 minutes).
```

## Troubleshooting

### Memory Features Not Working

**Symptom**: The AI doesn't remember past conversations

**Solutions:**
1. Check if the memory service is running:
   - Go to Settings > AI Agent
   - Look for "Memory Status" indicator
   - If offline, restart the application

2. Verify memory service configuration:
   - Check that the memory service is properly configured
   - See the setup guide for configuration instructions

3. Check knowledge base path:
   - Go to Settings > AI Agent
   - Verify the knowledge base path is correct
   - Default: `~/.cognical/knowledge_base`

### Tool Execution Failures

**Symptom**: The AI says it can't create tasks or events

**Solutions:**
1. Check tool permissions:
   - Go to Settings > AI Agent
   - Ensure "Enable Tool Calling" is turned on

2. Verify the action is supported:
   - Currently supported: create, update, list, search, delete tasks
   - Currently supported: get, create, update calendar events

3. Check for error messages:
   - The AI will explain why a tool failed
   - Common issues: invalid dates, missing required fields

### Slow Response Times

**Symptom**: The AI takes a long time to respond

**Possible causes:**
1. **Large memory context**: The AI is searching through many conversations
   - Solution: Clear old conversations or archive them

2. **Multiple tool executions**: The AI is performing several actions
   - This is normal for complex requests

3. **Network issues**: Connection to DeepSeek API is slow
   - Check your internet connection

**Performance targets:**
- Simple queries: < 2 seconds
- With memory search: < 3 seconds
- With tool execution: < 5 seconds

### Incorrect Tool Parameters

**Symptom**: The AI creates tasks or events with wrong information

**Solutions:**
1. Be more specific in your request:
   ```
   Instead of: "Schedule a meeting"
   Try: "Schedule a meeting titled 'Team Sync' for tomorrow at 2pm for 1 hour"
   ```

2. Correct the AI immediately:
   ```
   You: That's not quite right, change the time to 3pm
   AI: I've updated the meeting time to 3:00 PM.
   ```

3. Review before confirming:
   - The AI will often ask for confirmation on important actions
   - Review the details before saying "yes"

### Memory Storage Issues

**Symptom**: Conversations aren't being saved

**Solutions:**
1. Check disk space:
   - The knowledge base requires storage space
   - Each 10,000 conversations uses ~50MB

2. Verify write permissions:
   - Ensure the app can write to the knowledge base directory
   - Check file permissions on the knowledge base path

3. Check logs:
   - Go to Settings > Advanced > View Logs
   - Look for memory service errors

## Privacy and Data

### What Gets Stored

The AI Agent stores:
- Your messages and the AI's responses
- Metadata: timestamps, conversation IDs, tools used
- Semantic embeddings for search

### What Doesn't Get Stored

- Your API keys or credentials
- File contents (unless explicitly shared in chat)
- System information

### Data Control

You have full control over your data:

**Clear Conversation History:**
1. Click the menu in the chat header
2. Select "Clear History"
3. Confirm the action
4. Note: This archives conversations, not deletes them

**Export Your Data:**
1. Click "Export Conversation" in the chat header
2. Save the `.tar.gz` file
3. This file contains your complete knowledge base

**Delete Your Data:**
1. Go to Settings > AI Agent
2. Click "Delete Knowledge Base"
3. Confirm the permanent deletion
4. Warning: This cannot be undone

### Data Location

Your conversation data is stored locally:
- Default location: `~/.cognical/knowledge_base`
- Can be changed in Settings > AI Agent
- Never sent to external servers (except AI API calls)

## Keyboard Shortcuts

- **Send Message**: `Enter`
- **New Line**: `Shift + Enter`
- **Clear Input**: `Escape`
- **Focus Input**: `Ctrl/Cmd + K`
- **Search History**: `Ctrl/Cmd + F`

## Best Practices

### 1. Regular Exports

Export your conversation history regularly for backup:
- Weekly for active users
- Monthly for occasional users

### 2. Organize with Tags

Use consistent tags for tasks:
- Project names: `#project-alpha`
- Categories: `#finance`, `#marketing`
- Priorities: `#urgent`, `#review`

### 3. Clear Old Conversations

Archive conversations older than 6 months:
- Improves search performance
- Reduces memory usage
- Keeps context relevant

### 4. Provide Feedback

Help the AI learn:
- Correct mistakes immediately
- Be specific about what went wrong
- The AI will remember your preferences

## Getting Help

If you encounter issues not covered in this guide:

1. **Check the logs**: Settings > Advanced > View Logs
2. **Restart the memory service**: Settings > AI Agent > Restart
3. **Review the developer documentation**: For technical details
4. **Report bugs**: Use the feedback form in Settings

## What's Next

The AI Agent is continuously improving. Upcoming features:

- **Multi-language support**: Conversations in your preferred language
- **Voice input**: Talk to the AI instead of typing
- **Custom tools**: Define your own actions for the AI
- **Collaborative memory**: Share knowledge bases with your team
- **Smart suggestions**: Proactive recommendations based on your patterns

---

**Version**: 1.0  
**Last Updated**: January 2025  
**Feedback**: We'd love to hear your thoughts on the AI Agent!
