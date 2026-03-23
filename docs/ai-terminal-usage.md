# AI Terminal Usage Guide

## Overview

The AI Terminal supports two modes of operation:
1. **Regular Terminal Mode**: Execute shell commands normally
2. **AI Query Mode**: Ask natural language questions and get AI responses

## How to Use AI Mode

### Method 1: Using the `/ai` Prefix

Simply prefix your query with `/ai`:

```
/ai tell me a joke
/ai how do I list files in linux?
/ai what is the difference between git merge and git rebase?
```

### Method 2: Using the AI Mode Toggle Button

1. Click the purple "Sparkles" button in the command input area
2. Type your question (the `/ai` prefix is added automatically)
3. Submit to get an AI response

### Method 3: Keyboard Shortcut (Ctrl+Space)

1. Press `Ctrl+Space` to toggle AI mode on/off
2. When AI mode is active, you'll see:
   - Purple border around the input field
   - "AI Mode" badge on the right
   - Purple sparkles icon on the left
3. Type your question and press Enter

## How It Works

When you submit a query in AI mode:

1. A command block is created with your query (prefixed with `/ai`)
2. The query is sent to the configured AI provider (Ollama or Gemini)
3. The AI response streams in real-time, appearing in the command block
4. Once complete, the block shows a success status (green checkmark)

## Features

- **Streaming Responses**: AI responses appear in real-time as they're generated
- **Error Handling**: If the AI provider is unavailable or encounters an error, you'll see a clear error message
- **Command History**: AI queries are saved to your command history just like regular commands
- **No Context Mixing**: AI queries don't execute in the shell - they're pure conversations

## Example Workflow

```bash
# Regular command
ls -la

# AI query (using /ai prefix)
/ai explain what the ls -la command does

# Regular command
git status

# AI query (using keyboard shortcut - Ctrl+Space)
what files should I commit?
```

## Configuration

Make sure you have an AI provider configured:

1. Go to **AI** > **Providers** in the sidebar
2. Configure either:
   - **Ollama**: Local AI (requires Ollama installed)
   - **Gemini**: Google's AI (requires API key)
3. Enable the provider and test the connection

## Troubleshooting

### "Error: Provider not configured"
- Go to AI > Providers and configure at least one AI provider
- Make sure the provider is enabled

### "Error: Connection failed"
- For Ollama: Ensure Ollama is running (`ollama serve`)
- For Gemini: Check your API key is valid

### Input is disabled after query
- Wait for the AI response to complete
- The input is intentionally disabled while processing to prevent multiple concurrent queries

## Advanced: Conversation Context (Coming Soon)

Future versions will support maintaining conversation context across multiple queries, allowing follow-up questions like:

```
/ai what is Docker?
/ai can you explain that in simpler terms?
/ai show me an example
```
