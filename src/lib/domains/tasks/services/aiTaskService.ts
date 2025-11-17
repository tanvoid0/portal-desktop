import { invoke } from '@tauri-apps/api/core';

export type ProviderType = 'Ollama' | 'Gemini' | 'OpenAI' | 'Anthropic';

export interface GeneratedTask {
  title: string;
  description: string;
  priority: string; // low, medium, high
  type_: string; // Story, Bug, Feature, etc.
  estimated_time: number | null; // minutes
  tags: string[];
}

export interface GeneratedSubtask {
  title: string;
  description: string;
  estimated_time: number | null; // minutes
  dependencies: number[]; // indices of other subtasks this depends on
  order: number; // suggested order
}

export interface ProjectSuggestion {
  name: string;
  confidence: number;
  reason: string;
}

export interface GeneratedTaskStructure {
  main_task: GeneratedTask;
  subtasks: GeneratedSubtask[];
  suggested_project: ProjectSuggestion | null;
  suggested_labels: string[];
  confidence: number;
  model_used: string;
}

export interface ConversationMessage {
  role: 'user' | 'assistant';
  content: string;
}

export interface TaskContext {
  parentTask?: {
    id: string;
    title: string;
    description?: string;
    priority?: string;
    type?: string;
    tags?: string[];
  };
  existingChildren?: Array<{
    id: string;
    title: string;
    description?: string;
    status?: string;
  }>;
  existingSiblings?: Array<{
    id: string;
    title: string;
    description?: string;
  }>;
}

export interface GenerateTasksFromStoryRequest {
  story_text: string;
  provider_type?: ProviderType;
  history?: ConversationMessage[];
  context?: TaskContext;
  developer_note?: string;
  instruction?: string;
}

export interface AIErrorInfo {
  message: string;
  type: 'configuration' | 'network' | 'provider_unavailable' | 'model_not_found' | 'unknown';
  actionable: boolean;
  settingsPath?: string;
}

/**
 * Parse error message to extract actionable information
 * Exported for use in AI dialog components
 */
export function parseError(error: unknown): AIErrorInfo {
    const errorMessage =
      error instanceof Error
        ? error.message
        : typeof error === 'string'
        ? error
        : 'Failed to generate tasks from story';

    // Check for configuration issues (including wrapped error messages)
    if (
      errorMessage.includes('No default provider') ||
      errorMessage.includes('No default provider set') ||
      errorMessage.includes('Configuration incomplete') ||
      errorMessage.includes('provider configuration') ||
      errorMessage.includes('Missing fields') ||
      errorMessage.includes('not configured') ||
      errorMessage.includes('Provider not found') ||
      errorMessage.includes('Provider not registered')
    ) {
      return {
        message: 'AI provider is not configured. Please set up an AI provider (Ollama, Gemini, OpenAI, or Anthropic) in Settings.',
        type: 'configuration',
        actionable: true,
        settingsPath: '/settings/learning',
      };
    }

    // Check for model not found errors
    if (
      errorMessage.includes("model") &&
      (errorMessage.includes("not found") ||
        errorMessage.includes("404") ||
        errorMessage.includes("does not exist"))
    ) {
      // Extract model name if possible - handle both JSON and plain text formats
      // Try multiple patterns to extract the model name
      let modelMatch = errorMessage.match(/model\s+['"]([^'"]+)['"]/);
      
      if (!modelMatch) {
        // Try pattern: model 'name' (with single quotes)
        modelMatch = errorMessage.match(/model\s+['"]([^'"]+?)['"]/);
      }
      
      if (!modelMatch) {
        // Try to extract from JSON error field: {"error":"model 'name' not found"}
        // Handle both escaped and unescaped JSON
        modelMatch = errorMessage.match(/"error":\s*"model\s+['"]([^'"]+?)['"]/);
      }
      
      if (!modelMatch) {
        // Try unescaped JSON format
        modelMatch = errorMessage.match(/\{"error":\s*"model\s+['"]([^'"]+?)['"]/);
      }
      
      const modelName = modelMatch ? modelMatch[1] : 'the specified model';
      
      return {
        message: `Model "${modelName}" is not installed in Ollama. Please install it using: ollama pull ${modelName}`,
        type: 'model_not_found',
        actionable: true,
        settingsPath: '/settings/learning',
      };
    }

    // Check for provider unavailable
    if (
      errorMessage.includes('Provider not available') ||
      errorMessage.includes('service is not running') ||
      errorMessage.includes('failed to connect')
    ) {
      return {
        message: 'AI provider is not available. Please check that your AI service is running and properly configured.',
        type: 'provider_unavailable',
        actionable: true,
        settingsPath: '/settings/learning',
      };
    }

    // Check for network issues
    if (
      errorMessage.includes('Network error') ||
      errorMessage.includes('timeout') ||
      errorMessage.includes('connection')
    ) {
      return {
        message: 'Network error connecting to AI provider. Please check your connection and try again.',
        type: 'network',
        actionable: false,
      };
    }

    // Default unknown error
    return {
      message: errorMessage,
      type: 'unknown',
      actionable: false,
    };
}

export class AITaskService {
  /**
   * Generate tasks from story/description text using AI
   * @param request - Story text and optional provider type
   * @returns Generated task structure with main task, subtasks, and suggestions
   */
  async generateTasksFromStory(
    request: GenerateTasksFromStoryRequest
  ): Promise<GeneratedTaskStructure> {
    try {
      const command = {
        story_text: request.story_text,
        provider_type: request.provider_type || null,
        history: request.history || null,
        context: request.context || null,
        developer_note: request.developer_note || null,
        instruction: request.instruction || null,
      };

      const response = await invoke<GeneratedTaskStructure>(
        'generate_tasks_from_story',
        { command }
      );

      return response;
    } catch (error) {
      const errorInfo = parseError(error);
      console.error('AI task generation error:', error);
      
      // Create a custom error with the parsed information
      const customError = new Error(errorInfo.message) as Error & { errorInfo: AIErrorInfo };
      customError.errorInfo = errorInfo;
      throw customError;
    }
  }
}

export const aiTaskService = new AITaskService();

