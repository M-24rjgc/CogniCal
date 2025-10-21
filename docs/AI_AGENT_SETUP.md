# AI Agent Setup Guide

This guide walks you through setting up the AI Agent with memory and tool-calling capabilities in CogniCal.

## Overview

The AI Agent consists of two main components:

1. **Core AI Service**: Handles communication with DeepSeek API (required)
2. **Memory Service**: Provides long-term conversation memory via MCP server (optional)

## Prerequisites

### Required
- CogniCal application installed
- DeepSeek API key ([Get one here](https://platform.deepseek.com))

### Optional (for memory features)
- Python 3.8 or higher
- uv package manager (recommended) or pip
- ~200MB disk space for MCP server and knowledge base

## Step 1: Basic AI Setup

### 1.1 Get DeepSeek API Key

1. Visit [DeepSeek Platform](https://platform.deepseek.com)
2. Sign up or log in
3. Navigate to API Keys section
4. Create a new API key
5. Copy the key (you won't be able to see it again)

### 1.2 Configure API Key in CogniCal

**Option A: Through UI (Recommended)**
1. Launch CogniCal
2. Go to **Settings** > **AI Configuration**
3. Paste your API key in the "DeepSeek API Key" field
4. Click **Save**
5. Test the connection with a simple chat message

**Option B: Environment Variable**
```bash
# Linux/macOS
export DEEPSEEK_API_KEY=your_api_key_here

# Windows (PowerShell)
$env:DEEPSEEK_API_KEY="your_api_key_here"

# Windows (CMD)
set DEEPSEEK_API_KEY=your_api_key_here
```

### 1.3 Verify Basic AI Functionality

1. Open the **Chat** tab
2. Send a test message: "Hello, can you help me?"
3. You should receive a response from the AI

✅ **Basic AI is now working!** You can use the chat feature without memory.

## Step 2: Memory Service Setup (Optional)

The memory service enables the AI to remember past conversations and provide contextual responses.

### 2.1 Install uv Package Manager

**Why uv?** It's faster than pip and handles dependencies better.

**macOS/Linux:**
```bash
curl -LsSf https://astral.sh/uv/install.sh | sh
```

**Windows (PowerShell):**
```powershell
powershell -c "irm https://astral.sh/uv/install.ps1 | iex"
```

**Verify installation:**
```bash
uv --version
```

### 2.2 Install kb-mcp-server

**Using uv (recommended):**
```bash
uv pip install kb-mcp-server
```

**Using pip:**
```bash
pip install kb-mcp-server
```

**Verify installation:**
```bash
kb-mcp-server --version
```

### 2.3 Initialize Knowledge Base

The knowledge base stores your conversation history.

**Default location:**
- Linux/macOS: `~/.cognical/knowledge_base`
- Windows: `%USERPROFILE%\.cognical\knowledge_base`

**Create knowledge base:**
```bash
# Create directory
mkdir -p ~/.cognical/knowledge_base

# Initialize (automatic on first use)
# CogniCal will initialize the knowledge base when you first use memory features
```

**Custom location (optional):**
```bash
# Set custom path
export COGNICAL_KB_PATH=/path/to/your/knowledge_base

# Windows
set COGNICAL_KB_PATH=C:\path\to\your\knowledge_base
```

### 2.4 Configure Memory Service in CogniCal

1. Launch CogniCal
2. Go to **Settings** > **AI Agent**
3. Enable "Memory Features"
4. Set knowledge base path (or use default)
5. Click **Save**
6. Click **Start Memory Service**

### 2.5 Verify Memory Service

1. Check the **Memory Status** indicator in Settings
   - 🟢 Green: Memory service is running
   - 🔴 Red: Memory service is offline
   - 🟡 Yellow: Memory service is starting

2. Test memory functionality:
   ```
   You: My name is Alice and I work on the marketing team
   AI: Nice to meet you, Alice! I'll remember that you're on the marketing team.
   
   [Start a new chat session]
   
   You: What team do I work on?
   AI: You work on the marketing team, Alice!
   ```

✅ **Memory features are now active!**

## Step 3: Enable Tool Calling (Automatic)

Tool calling is enabled by default and allows the AI to:
- Create and manage tasks
- Schedule calendar events
- Search your data

**Verify tool calling:**
```
You: Create a task to review the budget report by Friday
AI: I've created a high-priority task "Review budget report" with a due date of Friday, [date].
```

**Configure tool settings:**
1. Go to **Settings** > **AI Agent** > **Tool Configuration**
2. Enable/disable specific tools
3. Set execution timeout (default: 5 seconds)
4. Configure permissions

## Troubleshooting

### Issue: "DeepSeek API key is invalid"

**Solutions:**
1. Verify the API key is correct (no extra spaces)
2. Check if the key has been revoked
3. Ensure you have API credits remaining
4. Try generating a new API key

### Issue: "Memory service failed to start"

**Solutions:**

1. **Check if kb-mcp-server is installed:**
   ```bash
   kb-mcp-server --version
   ```
   If not found, reinstall using uv or pip.

2. **Check Python version:**
   ```bash
   python --version  # Should be 3.8+
   ```

3. **Check knowledge base permissions:**
   ```bash
   # Linux/macOS
   ls -la ~/.cognical/knowledge_base
   
   # Ensure you have write permissions
   chmod -R u+w ~/.cognical/knowledge_base
   ```

4. **Check logs:**
   - Go to Settings > Advanced > View Logs
   - Look for MCP server errors
   - Common issues: port conflicts, permission errors

5. **Restart the service:**
   - Settings > AI Agent > Restart Memory Service

### Issue: "Memory service is slow"

**Solutions:**

1. **Check knowledge base size:**
   ```bash
   du -sh ~/.cognical/knowledge_base
   ```
   If > 500MB, consider archiving old conversations.

2. **Reduce context size:**
   - Settings > AI Agent > Max Context Entries
   - Lower from 5 to 3

3. **Optimize knowledge base:**
   - Settings > AI Agent > Optimize Knowledge Base
   - This rebuilds indexes for faster search

### Issue: "Tools are not executing"

**Solutions:**

1. **Check if tools are enabled:**
   - Settings > AI Agent > Enable Tool Calling

2. **Check tool permissions:**
   - Some tools may require confirmation
   - Check for permission prompts

3. **Verify tool syntax:**
   - Be specific in your requests
   - Example: "Create a task titled 'Review report' due tomorrow"

4. **Check logs:**
   - Settings > Advanced > View Logs
   - Look for tool execution errors

### Issue: "AI doesn't remember conversations"

**Solutions:**

1. **Verify memory service is running:**
   - Check Memory Status indicator (should be green)

2. **Check if conversations are being stored:**
   - Settings > AI Agent > Export Conversation
   - If export is empty, storage is failing

3. **Check disk space:**
   ```bash
   df -h  # Linux/macOS
   ```
   Ensure you have at least 500MB free.

4. **Restart memory service:**
   - Settings > AI Agent > Restart Memory Service

5. **Check knowledge base path:**
   - Settings > AI Agent > Knowledge Base Path
   - Ensure the path exists and is writable

## Advanced Configuration

### Custom Knowledge Base Location

**Why?** You might want to:
- Store on a different drive with more space
- Use a network location for team sharing
- Separate personal and work knowledge bases

**How to configure:**

1. **Create the directory:**
   ```bash
   mkdir -p /path/to/custom/kb
   ```

2. **Set environment variable:**
   ```bash
   export COGNICAL_KB_PATH=/path/to/custom/kb
   ```

3. **Or configure in UI:**
   - Settings > AI Agent > Knowledge Base Path
   - Browse to your custom location
   - Click Save

### Memory Configuration Options

Configure in Settings > AI Agent > Advanced:

- **Max Context Entries** (default: 5)
  - How many past conversations to include in context
  - Higher = more context but slower responses
  - Range: 1-10

- **Search Limit** (default: 10)
  - Maximum search results when querying memory
  - Higher = more thorough but slower
  - Range: 5-50

- **Enable Knowledge Graph** (default: false)
  - Experimental feature for relationship mapping
  - Requires additional setup
  - Increases memory usage

### Tool Configuration

Configure in Settings > AI Agent > Tools:

- **Execution Timeout** (default: 5000ms)
  - Maximum time for tool execution
  - Increase for slow operations
  - Range: 1000-30000ms

- **Enable Specific Tools**
  - Task Management: Create, update, delete tasks
  - Calendar: View, create, update events
  - Search: Search tasks and calendar

- **Require Confirmation**
  - Prompt before destructive actions
  - Recommended for delete operations

### Performance Tuning

**For faster responses:**
1. Reduce Max Context Entries to 3
2. Reduce Search Limit to 5
3. Disable Knowledge Graph
4. Archive old conversations

**For better context:**
1. Increase Max Context Entries to 8
2. Increase Search Limit to 20
3. Enable Knowledge Graph
4. Keep all conversations

**Balanced (recommended):**
- Max Context Entries: 5
- Search Limit: 10
- Knowledge Graph: Disabled

## Data Management

### Exporting Conversations

**Why export?**
- Backup your conversation history
- Transfer to another device
- Share with team members (anonymized)

**How to export:**
1. Go to Chat tab
2. Click menu (⋮) in header
3. Select "Export Conversation"
4. Choose location and filename
5. Save as `.tar.gz` file

**What's included:**
- All conversation turns
- Metadata (timestamps, tools used)
- Semantic embeddings
- Configuration settings

### Importing Conversations

**How to import:**
1. Go to Settings > AI Agent
2. Click "Import Knowledge Base"
3. Select your `.tar.gz` export file
4. Choose merge or replace
5. Restart memory service

### Clearing History

**Caution:** This archives conversations but doesn't delete them.

**How to clear:**
1. Go to Chat tab
2. Click menu (⋮) in header
3. Select "Clear History"
4. Confirm action

**To permanently delete:**
1. Go to Settings > AI Agent
2. Click "Delete Knowledge Base"
3. Confirm permanent deletion
4. ⚠️ This cannot be undone!

### Archiving Old Conversations

**Recommended:** Archive conversations older than 6 months.

**How to archive:**
1. Export conversations (see above)
2. Go to Settings > AI Agent
3. Click "Archive Old Conversations"
4. Set date threshold (e.g., 6 months)
5. Archived conversations are removed from active search

## Security and Privacy

### Data Storage

- **Local only**: All data stored on your device
- **No cloud sync**: Conversations never leave your machine
- **Encrypted**: Knowledge base can be encrypted (optional)

### API Communication

- **HTTPS only**: All API calls use secure connections
- **No logging**: DeepSeek doesn't log conversation content
- **API key security**: Stored encrypted in system keychain

### Privacy Controls

**Settings > AI Agent > Privacy:**

- **Disable Memory**: Use AI without storing conversations
- **Auto-delete**: Automatically delete conversations after X days
- **Anonymize Exports**: Remove personal information from exports
- **Opt-out Telemetry**: Disable usage analytics

## Getting Help

### Check Logs

**View logs:**
1. Settings > Advanced > View Logs
2. Look for errors related to:
   - MCP server
   - Memory service
   - Tool execution
   - API calls

**Log locations:**
- Linux/macOS: `~/.cognical/logs/`
- Windows: `%APPDATA%\cognical\logs\`

### Common Log Messages

**"MCP server connection refused"**
- Solution: Restart memory service

**"Knowledge base locked"**
- Solution: Close other CogniCal instances

**"API rate limit exceeded"**
- Solution: Wait a few minutes, check API quota

### Support Resources

- **User Guide**: [AI_AGENT_USER_GUIDE.md](./AI_AGENT_USER_GUIDE.md)
- **Developer Guide**: [AI_AGENT_DEVELOPER_GUIDE.md](./AI_AGENT_DEVELOPER_GUIDE.md)
- **GitHub Issues**: Report bugs and request features
- **Community Forum**: Ask questions and share tips

## Next Steps

Now that your AI Agent is set up:

1. **Explore features**: Try creating tasks and scheduling events through chat
2. **Build memory**: Have conversations to build up context
3. **Customize**: Adjust settings to match your workflow
4. **Provide feedback**: Help us improve the AI Agent

**Recommended first conversations:**
```
"My name is [name] and I work as a [role] at [company]"
"I usually work from 9am to 5pm on weekdays"
"My priorities this week are [list priorities]"
"I prefer high-priority tasks to be highlighted"
```

This helps the AI learn your preferences and provide better assistance!

---

**Version**: 1.0  
**Last Updated**: January 2025  
**Questions?** Check the [User Guide](./AI_AGENT_USER_GUIDE.md) or open an issue.
