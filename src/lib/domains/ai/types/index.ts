export type ProviderType = 'Ollama' | 'Gemini';

export interface ProviderConfig {
  provider_type: ProviderType;
  base_url: string | null;
  api_key: string | null;
  model: string;
  default_options: {
    temperature?: number;
    max_tokens?: number;
    timeout_ms?: number;
    model?: string | null;
    extra_options?: Record<string, unknown> | null;
  };
  enabled: boolean;
}

export interface ConfigurationStatus {
  is_configured: boolean;
  missing_fields: string[];
  warnings: string[];
}

export interface ChatMessage {
  role: 'user' | 'assistant';
  content: string;
  timestamp?: Date;
}

export interface Conversation {
  id: string;
  title: string;
  provider: ProviderType;
  created_at: string;
  updated_at: string;
  message_count?: number;
}

export interface ConversationMessage {
  id: string;
  conversation_id: string;
  role: 'user' | 'assistant';
  content: string;
  timestamp: string;
  sequence: number;
}

export interface TrainingData {
  id: string;
  name: string;
  type: string;
  content: string;
  metadata: Record<string, unknown>;
  created_at: string;
  updated_at: string;
}

export interface AILog {
  id: string;
  provider: ProviderType;
  log_type: 'request' | 'response' | 'error';
  request_data?: string;
  response_data?: string;
  error_message?: string;
  timestamp: string;
  conversation_id?: string;
}

export interface LogFilters {
  provider?: ProviderType;
  log_type?: 'request' | 'response' | 'error';
  date_from?: string;
  date_to?: string;
  search_query?: string;
}

