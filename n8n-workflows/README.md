# n8n Workflows for Portal Desktop

This directory contains n8n automation workflows that integrate with Portal Desktop to provide intelligent project automation based on detected frameworks and package managers.

## Quick Start

### 1. Install and Start Ollama (Local AI)

```bash
# Install Ollama
curl -fsSL https://ollama.ai/install.sh | sh

# Start Ollama service
ollama serve

# In another terminal, pull recommended models
ollama pull llama3.2:3b
ollama pull codellama:7b
ollama pull mistral:7b
```

### 2. Start n8n Locally

```bash
# From the project root
npm run n8n:start

# Or manually
cd n8n-workflows
npx n8n start
```

### 3. Access n8n UI

- **URL**: http://localhost:5678
- **Username**: admin
- **Password**: portal123

### 4. Import Workflow Templates

1. Open n8n UI
2. Go to "Workflows" → "Import from File"
3. Import templates from `templates/` directory
4. Update webhook URLs to match your setup

## Available Workflows

### Project Setup Workflows

- **Node.js Setup** (`node-project-setup.json`)
  - Detects package.json
  - Installs dependencies (npm/yarn/pnpm)
  - Runs initial build
  - Creates .gitignore if missing

- **Rust Setup** (`rust-project-setup.json`)
  - Detects Cargo.toml
  - Runs cargo build
  - Runs cargo test
  - Sets up git if needed

- **Python Setup** (`python-project-setup.json`)
  - Detects requirements.txt
  - Creates virtual environment
  - Installs dependencies
  - Sets up .gitignore

### Development Workflows

- **Dependency Update** (`dependency-update.json`)
  - Checks for outdated packages
  - Shows update report
  - Optionally updates dependencies

- **Health Check** (`health-check.json`)
  - Scans project for issues
  - Checks git status
  - Validates dependencies
  - Reports findings

### AI-Powered Workflows (Ollama)

- **AI Code Analysis** (`ai-code-analysis.json`)
  - Analyzes project code using Ollama
  - Provides code quality assessment
  - Suggests improvements and security fixes
  - Uses CodeLlama model for code understanding

- **AI Project Suggestions** (`ai-suggestions.json`)
  - Generates project-specific recommendations
  - Suggests development setup improvements
  - Recommends tools and configurations
  - Uses Llama3.2 model for general suggestions

### Batch Operations

- **Multi-Project Update** (`batch-update.json`)
  - Processes multiple projects
  - Runs commands in parallel
  - Aggregates results

## Ollama Configuration

### Recommended Models

- **llama3.2:3b** - Fast, general-purpose model for suggestions
- **codellama:7b** - Specialized for code analysis and generation
- **mistral:7b** - Alternative model for different perspectives

### Model Management

```bash
# List installed models
ollama list

# Pull a specific model
ollama pull llama3.2:3b

# Remove a model
ollama rm codellama:7b

# Run a model interactively
ollama run llama3.2:3b
```

### Performance Tips

- **GPU Acceleration**: Ollama automatically uses GPU if available
- **Memory Requirements**: 
  - llama3.2:3b needs ~2GB RAM
  - codellama:7b needs ~4GB RAM
  - mistral:7b needs ~4GB RAM
- **CPU Fallback**: Works on CPU if no GPU available (slower)

## Integration with Portal Desktop

### Webhook Configuration

Each workflow exposes a webhook endpoint:
- **Node Setup**: `/webhook/node-setup`
- **Rust Setup**: `/webhook/rust-setup`
- **Python Setup**: `/webhook/python-setup`
- **Health Check**: `/webhook/health-check`

### Input Data Format

Portal Desktop sends project data in this format:

```json
{
  "project_id": 123,
  "name": "My Project",
  "path": "/path/to/project",
  "framework": "Node.js",
  "package_manager": "npm",
  "build_command": "npm run build",
  "start_command": "npm start",
  "test_command": "npm test",
  "output_directory": "dist",
  "dev_port": 3000
}
```

### Response Format

Workflows return results in this format:

```json
{
  "success": true,
  "execution_id": "abc123",
  "results": {
    "commands_executed": ["npm install", "npm run build"],
    "output": "Build completed successfully",
    "duration": 45.2,
    "files_created": ["dist/", ".gitignore"]
  },
  "errors": [],
  "suggestions": ["Consider adding TypeScript", "Set up ESLint"]
}
```

## Creating Custom Workflows

### 1. Use n8n Visual Editor

1. Open n8n UI
2. Create new workflow
3. Add nodes for your automation
4. Set up webhook trigger
5. Export as JSON template

### 2. Common Node Types

- **Webhook**: Receives data from Portal Desktop
- **Execute Command**: Runs shell commands
- **HTTP Request**: Makes API calls
- **Set**: Manipulates data
- **IF**: Conditional logic
- **Switch**: Route data based on conditions

### 3. Best Practices

- Always handle errors gracefully
- Use environment variables for paths
- Include progress feedback
- Validate input data
- Return structured responses

## Troubleshooting

### n8n Won't Start

```bash
# Check if port 5678 is in use
lsof -i :5678

# Kill existing process
pkill -f n8n

# Start fresh
npm run n8n:start
```

### Workflows Not Triggering

1. Check webhook URLs in Portal Desktop config
2. Verify n8n is running on correct port
3. Check n8n logs for errors
4. Ensure workflows are active (not paused)

### Database Issues

```bash
# Reset n8n database
rm n8n-workflows/database.sqlite
npm run n8n:start
```

## Advanced Configuration

### Custom Ports

Edit `.env` file:
```bash
N8N_PORT=5679
```

### Authentication

Update `.env`:
```bash
N8N_BASIC_AUTH_USER=your_username
N8N_BASIC_AUTH_PASSWORD=your_password
```

### External Integrations

n8n can connect to:
- GitHub (for repository operations)
- Slack (for notifications)
- Discord (for team updates)
- Email (for reports)
- CI/CD systems (for deployment)

## Examples

### Simple Node.js Project Setup

```json
{
  "nodes": [
    {
      "name": "Webhook",
      "type": "n8n-nodes-base.webhook",
      "parameters": {
        "path": "node-setup",
        "httpMethod": "POST"
      }
    },
    {
      "name": "Install Dependencies",
      "type": "n8n-nodes-base.executeCommand",
      "parameters": {
        "command": "cd {{ $json.path }} && npm install"
      }
    },
    {
      "name": "Run Build",
      "type": "n8n-nodes-base.executeCommand",
      "parameters": {
        "command": "cd {{ $json.path }} && npm run build"
      }
    }
  ]
}
```

This setup provides a foundation for intelligent project automation that grows with your development needs.
