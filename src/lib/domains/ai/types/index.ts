export type ProviderType = 'AgentPlatform';

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

/**
 * Per-turn generation stats shown under an assistant reply. In-memory only —
 * they are not persisted with the conversation.
 */
export interface MessageStats {
  tokensPerSecond?: number | null;
  completionTokens?: number | null;
  durationMs?: number | null;
  /** Time spent inside a `<think>` block, when the model emits one. */
  thinkingMs?: number | null;
  stopReason?: string | null;
}

export interface ChatMessage {
  role: 'user' | 'assistant';
  content: string;
  timestamp?: Date | string;
  stats?: MessageStats | null;
}

export interface Conversation {
  id: string;
  title: string;
  provider: ProviderType;
  model?: string | null;
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

/** Resolved defaults from agent-platform `GET /v1/catalog`. */
export interface CatalogResolvedDefaults {
  provider: string;
  model: string;
}

/** Per-model capability flags from catalog probing. */
export interface CatalogModelCapabilities {
  chat?: boolean;
  tools?: boolean;
  vision_input?: boolean;
  embeddings?: boolean;
  image_generation?: boolean;
  streaming?: boolean;
  /** ollama_show | heuristic | provider_default */
  probe_source?: string;
}

/** Provider-level capability summary from catalog. */
export interface CatalogProviderCapabilities {
  streaming?: boolean;
  tools?: boolean;
  json_mode?: boolean;
  modalities?: Record<string, boolean>;
  model_discovery?: Record<string, unknown>;
}

/** A model entry from agent-platform catalog. */
export interface CatalogModel {
  id: string;
  /** Parent provider id when present; v1 catalog omits this on model rows. */
  provider?: string | null;
  source: 'alias' | 'live' | string;
  backend_id?: string | null;
  metadata?: Record<string, unknown>;
  capabilities?: CatalogModelCapabilities;
}

/** A provider entry from agent-platform catalog. */
export interface CatalogProvider {
  id: string;
  label: string;
  configured: boolean;
  reachable: boolean | null;
  default_model?: string | null;
  capabilities?: CatalogProviderCapabilities;
  models: CatalogModel[];
}

/** Full catalog response from agent-platform `GET /v1/catalog`. */
export interface PlatformCatalog {
  object: string;
  resolved_defaults: CatalogResolvedDefaults;
  providers: CatalogProvider[];
}

/** Reachability of agent-platform, derived from the catalog fetch. */
export interface CatalogStatus {
  checking: boolean;
  online: boolean;
  error: string | null;
}

export interface CatalogQuery {
  /** `["all"]` for all providers; omit for effective default only. */
  providers?: string[] | null;
  /** `false` = YAML aliases only (no upstream fetches). */
  live?: boolean | null;
  /** When true with live catalog, probe Ollama models for tools/vision flags. */
  probe_capabilities?: boolean | null;
}

/** Estimated input context breakdown from agent-platform chat APIs. */
export interface ContextUsage {
  context_window: number;
  total_estimated: number;
  percent_used: number;
  prompt_budget: number;
  reserved_output: number;
  categories: Record<string, number>;
}

/** Aggregated LLM token usage for one or more completion steps. */
export interface LlmUsage {
  prompt_tokens: number;
  completion_tokens: number;
  total_tokens: number;
  cost_usd: number;
}
