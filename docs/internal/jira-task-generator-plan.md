# Jira Story to Task Generator Enhancement

## Overview
Add an AI-powered feature to the task manager that takes a Jira story text input and generates structured tasks with subtasks, descriptions, labels, and project links. After user approval, creates developer-friendly tasks with clean formatting while preserving the original Jira text as reference.

**Key Architecture Decision:**
Use a centralized, abstract AI service that supports multiple providers (Ollama, Gemini, OpenAI, Anthropic) configurable from the UI. This service will be reusable across the entire application.

**DX & UX Focus:**
- Clean, maintainable code with clear separation of concerns
- Type-safe interfaces with comprehensive TypeScript types
- Reusable, composable components
- Excellent error handling with actionable feedback
- Smooth, intuitive user interactions
- Keyboard shortcuts and accessibility support
- Smart defaults and auto-completion
- Provider-agnostic AI abstraction layer

## Architecture

### Centralized AI Service (Backend)

1. **AI Provider Trait** (`src-tauri/src/domains/shared/services/ai_provider.rs`)
   - Abstract trait for all AI providers
   - Unified interface: `generate(prompt, options) -> Result<String>`
   - Common error types and configuration structures

2. **Provider Implementations** (`src-tauri/src/domains/shared/services/providers/`)
   - `ollama_provider.rs` - Ollama (local) implementation
   - `gemini_provider.rs` - Google Gemini implementation
   - `openai_provider.rs` - OpenAI implementation
   - `anthropic_provider.rs` - Anthropic/Claude implementation
   - **DX:** Each provider implements the same trait
   - **DX:** Consistent error handling across providers

3. **AI Service Coordinator** (`src-tauri/src/domains/shared/services/ai_service.rs`)
   - Provider selection based on settings
   - Configuration management
   - Request routing to appropriate provider
   - Retry logic with exponential backoff
   - Response normalization
   - Logging with request/response details
   - Streaming support for progress feedback

4. **AI Settings Management** (`src-tauri/src/domains/settings/`)
   - Add AI configuration to settings
   - Provider selection (Ollama, Gemini, OpenAI, etc.)
   - API keys and configuration per provider
   - Model selection per provider
   - Default provider selection

### Task Generation Services

1. **Jira Parser** (`src-tauri/src/domains/tasks/services/jira_parser.rs`)
   - Extract key information from Jira text
   - Identify story title, description, acceptance criteria
   - Detect mentioned projects, labels, priorities
   - **DX:** Well-documented parsing functions
   - **DX:** Error types with context
   - **DX:** Unit tests for edge cases

2. **AI Task Generator** (`src-tauri/src/domains/tasks/services/ai_task_generator.rs`)
   - Uses centralized AI service (not tied to specific provider)
   - Modular prompt builders
   - Configurable prompt templates
   - Structured response parsing with validation
   - Logging for debugging
   - Generate structured JSON response:
     - Main task (title, description, priority, type, tags, estimated_time)
     - Subtasks (title, description, estimated_time, dependencies)
     - Suggested project link (with confidence score)
     - Labels/tags (with relevance scores)

### Frontend Services

1. **Centralized AI Service** (`src/lib/domains/shared/services/aiService.ts`)
   - Provider-agnostic service class
   - Type-safe provider configurations
   - Comprehensive JSDoc with usage examples
   - Error handling with typed exceptions
   - Request/response logging (dev mode)
   - Retry logic with exponential backoff
   - Methods:
     - `generate(prompt, options): Promise<string>`
     - `getAvailableProviders(): Promise<AIProvider[]>`
     - `getCurrentProvider(): Promise<AIProvider>`
     - `setProvider(providerId): Promise<void>`
     - `testConnection(providerId): Promise<boolean>`

2. **AI Settings Service** (`src/lib/domains/settings/services/aiSettingsService.ts`)
   - Type-safe provider configuration management
   - Validation for API keys and settings
   - Methods:
     - `getProviders(): Promise<AIProvider[]>`
     - `saveProviderConfig(provider, config): Promise<void>`
     - `getDefaultProvider(): Promise<AIProvider>`
     - `setDefaultProvider(providerId): Promise<void>`

3. **AI Task Service** (`src/lib/domains/tasks/services/aiTaskService.ts`)
   - Uses centralized AI service (not provider-specific)
   - Type-safe service class
   - Methods:
     - `generateTasksFromJira(options): Promise<GeneratedTaskStructure>`
     - `parseJiraText(text): ParsedJiraStory` (client-side quick parse)
     - `validateJiraText(text): ValidationResult`
     - `getSuggestedProjects(context): Promise<Project[]>`

### Frontend Components (UX-Optimized)

1. **AISettings.svelte** (`src/lib/domains/settings/components/`)
   - Provider selection dropdown (Ollama, Gemini, OpenAI, etc.)
   - Provider-specific configuration forms
   - API key input with secure storage
   - Model selection per provider
   - Test connection button
   - Default provider selection
   - Visual indicators for active/configured providers

2. **JiraImportDialog.svelte** (`src/lib/domains/tasks/components/jira/`)
   - Large, comfortable textarea with Jira formatting hints
   - Auto-detect Jira format (story, epic, bug) with visual indicators
   - Paste detection with instant format recognition
   - Provider selection dropdown (uses current default)
   - Smart suggestions for missing fields
   - Real-time character count and validation
   - Keyboard shortcut: `Cmd/Ctrl + Shift + J` to open
   - Loading states with progress indicators
   - Clear error messages with recovery suggestions

3. **TaskGenerationPreview.svelte** (`src/lib/domains/tasks/components/jira/`)
   - Split-view: editable preview on left, original Jira on right (collapsible)
   - Inline editing with visual feedback
   - Drag-to-reorder subtasks
   - Bulk actions (select all, delete multiple)
   - Smart field suggestions (project auto-complete)
   - Diff view showing changes from original
   - One-click approve or "Edit All" mode
   - Undo/redo support

## Implementation Phases

### Phase 1: Centralized AI Service (Backend)
1. Create AI provider trait and common types
2. Implement provider backends (Ollama, Gemini, OpenAI, Anthropic)
3. Create AI service coordinator with provider selection
4. Add AI configuration to settings backend with Tauri commands

### Phase 2: Frontend AI Service & Types
1. Create centralized AI service frontend
2. Create AI settings service and component
3. Create TypeScript types for AI providers
4. Create task-specific types

### Phase 3: Task Generation Backend
1. Create Jira parser service
2. Create AI task generator using centralized AI service
3. Add Tauri command for task generation

### Phase 4: Frontend UI Components
1. Create JiraImportDialog with provider selection
2. Create TaskGenerationPreview component
3. Create supporting components (TaskPreviewCard, SubtaskList, etc.)
4. Enhance TaskForm with Jira import button

### Phase 5: Task Creation Logic
1. Implement task creation sequence with original text preservation
2. Format generated content (clean, concise, action-oriented)
3. Store original Jira text in collapsed section

### Phase 6: Error Handling & Feedback
1. Comprehensive error handling for all scenarios
2. User feedback (toasts, loading states, progress indicators)
3. Error recovery suggestions

### Phase 7: Testing & Documentation
1. Unit tests for parsing logic
2. Integration tests for AI generation
3. E2E tests for full workflow
4. Component usage examples and API documentation

## File Structure

```
src-tauri/src/domains/shared/
  services/
    ai_provider.rs (new - abstract trait)
    ai_service.rs (new - coordinator)
    providers/
      ollama_provider.rs (new)
      gemini_provider.rs (new)
      openai_provider.rs (new)
      anthropic_provider.rs (new)

src-tauri/src/domains/settings/
  services/
    ai_settings_service.rs (new - AI config management)
  commands.rs (enhance - add AI settings commands)

src-tauri/src/domains/tasks/
  services/
    jira_parser.rs (new)
    ai_task_generator.rs (new - uses centralized AI service)
  commands.rs (enhance)
  errors.rs (new - custom error types)

src/lib/domains/shared/
  services/
    aiService.ts (new - centralized AI service)
  types/
    ai.ts (new - AI provider types)
  composables/
    useJiraParser.ts (new - reusable parsing logic)
    useKeyboardShortcuts.ts (enhance - add Jira shortcuts)

src/lib/domains/settings/
  services/
    aiSettingsService.ts (new - AI settings management)
  components/
    AISettings.svelte (new - AI provider configuration UI)

src/lib/domains/tasks/
  components/
    jira/
      JiraImportDialog.svelte (new)
      TaskGenerationPreview.svelte (new)
      TaskPreviewCard.svelte (new)
      SubtaskList.svelte (new)
      JiraOriginalView.svelte (new)
      ProjectAutocomplete.svelte (new)
    TaskForm.svelte (enhance)
  services/
    aiTaskService.ts (new - uses centralized AI service)
  types/
    jira.ts (new - Jira-related types)
    index.ts (enhance - add GeneratedTaskStructure type)
  utils/
    jiraValidation.ts (new - validation helpers)
    taskFormatting.ts (new - formatting utilities)
```

## Key Features

### Developer Experience (DX)
- **Type Safety:** Comprehensive TypeScript types with strict checking
- **Modular Architecture:** Clear separation of concerns, reusable components
- **Error Handling:** Typed errors with context and recovery suggestions
- **Documentation:** JSDoc comments, inline examples, architecture docs
- **Testing:** Unit tests, integration tests, test fixtures
- **Logging:** Structured logging for debugging and monitoring
- **Maintainability:** Clean code, consistent patterns, easy to extend
- **Provider-Agnostic:** Easy to add new AI providers without changing task generation logic

### User Experience (UX)
- **Intuitive Interface:** Clear visual hierarchy, familiar patterns
- **Keyboard Shortcuts:** Power user shortcuts (Cmd/Ctrl+Shift+J, Cmd/Ctrl+Enter)
- **Smart Defaults:** Auto-detection of format, projects, priorities
- **Real-time Feedback:** Loading states, progress indicators, validation
- **Error Recovery:** Clear error messages with actionable recovery steps
- **Accessibility:** ARIA labels, keyboard navigation, screen reader support
- **Visual Polish:** Smooth animations, clear visual feedback, responsive design
- **Efficiency:** Bulk actions, inline editing, drag-and-drop reordering
- **Provider Selection:** Easy switching between AI providers from UI

### Core Functionality
- **Provider-Agnostic AI:** Support for Ollama, Gemini, OpenAI, Anthropic (configurable from UI)
- **Centralized AI Service:** Reusable across entire application
- **AI-powered parsing of Jira stories** using configured provider
- **Automatic task and subtask generation** with smart splitting
- **Project auto-detection** with fuzzy matching and manual override
- **Preview and edit interface** with diff view
- **Developer-friendly formatting** (concise, action-oriented)
- **Original text preservation** as collapsible reference
- **Smart field suggestions** (projects, labels, priorities)
- **Batch operations** for creating multiple tasks
- **AI Provider Configuration** from settings UI

## Implementation Todos

1. **backend-ai-provider-trait** - Create AI provider trait and common types
2. **backend-provider-impls** - Implement provider backends (Ollama, Gemini, OpenAI, Anthropic)
3. **backend-ai-service** - Create centralized AI service coordinator
4. **backend-ai-settings** - Add AI configuration to settings backend
5. **backend-jira-parser** - Create jira_parser.rs service
6. **backend-ai-generator** - Create ai_task_generator.rs using centralized AI service
7. **backend-tauri-command** - Add generate_tasks_from_jira_story Tauri command
8. **frontend-ai-service** - Create centralized aiService.ts frontend service
9. **frontend-ai-settings** - Create aiSettingsService.ts and AISettings.svelte component
10. **frontend-ai-task-service** - Create aiTaskService.ts using centralized AI service
11. **frontend-jira-dialog** - Create JiraImportDialog.svelte with provider selection UI
12. **frontend-preview** - Create TaskGenerationPreview.svelte for reviewing/editing
13. **frontend-integration** - Integrate JiraImportDialog into TaskForm.svelte
14. **task-creation-logic** - Implement task creation sequence with original text preservation
15. **error-handling** - Add comprehensive error handling and user feedback

