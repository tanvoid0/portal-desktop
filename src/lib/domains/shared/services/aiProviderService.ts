import { invoke } from '@tauri-apps/api/core';

export type ProviderType = 'Ollama' | 'Gemini' | 'OpenAI' | 'Anthropic';

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

export class AIProviderService {
  /**
   * Get all configured AI providers
   */
  async getProviders(): Promise<ProviderConfig[]> {
    return invoke<ProviderConfig[]>('get_ai_providers');
  }

  /**
   * Get configuration for a specific provider
   */
  async getProviderConfig(providerType: ProviderType): Promise<ProviderConfig> {
    return invoke<ProviderConfig>('get_ai_provider_config', { providerType });
  }

  /**
   * Get configuration status for a provider
   */
  async getConfigStatus(providerType: ProviderType): Promise<ConfigurationStatus> {
    return invoke<ConfigurationStatus>('get_ai_provider_config_status', { providerType });
  }

  /**
   * Save provider configuration
   */
  async saveProviderConfig(config: ProviderConfig): Promise<void> {
    return invoke('save_ai_provider_config', { config });
  }

  /**
   * Get default AI provider
   */
  async getDefaultProvider(): Promise<ProviderType | null> {
    return invoke<ProviderType | null>('get_default_ai_provider');
  }

  /**
   * Set default AI provider
   */
  async setDefaultProvider(providerType: ProviderType): Promise<void> {
    return invoke('set_default_ai_provider', { providerType });
  }

  /**
   * Test provider connection
   */
  async testProvider(providerType: ProviderType): Promise<void> {
    return invoke('test_ai_provider', { providerType });
  }

  /**
   * Get available models for a provider
   */
  async getAvailableModels(providerType?: ProviderType): Promise<string[]> {
    return invoke<string[]>('get_ai_provider_models', { providerType: providerType || null });
  }

  /**
   * Get available Ollama models (downloadable models organized by family)
   */
  async getAvailableOllamaModels(): Promise<Record<string, Array<{ name: string; size?: string }>>> {
    return invoke<Record<string, Array<{ name: string; size?: string }>>>('get_available_ollama_models');
  }
}

export const aiProviderService = new AIProviderService();

