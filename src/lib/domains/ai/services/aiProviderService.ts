import { invoke } from '@tauri-apps/api/core';
import type {
  ProviderType,
  ProviderConfig,
  ConfigurationStatus,
  PlatformCatalog,
  CatalogQuery,
  PlatformHealth,
  PlatformStartResult,
  PlatformPairResult,
} from '../types/index.js';

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
    return invoke<ConfigurationStatus>('get_ai_provider_config_status', {
      providerType,
    });
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
   * Get available models for a provider (model ids only).
   */
  async getAvailableModels(providerType?: ProviderType): Promise<string[]> {
    return invoke<string[]>('get_ai_provider_models', {
      providerType: providerType || null,
    });
  }

  /**
   * Fetch agent-platform provider/model catalog (`GET /v1/catalog`).
   */
  async getCatalog(query: CatalogQuery = {}): Promise<PlatformCatalog> {
    return invoke<PlatformCatalog>('get_ai_platform_catalog', {
      providers: query.providers ?? null,
      live: query.live ?? null,
      probeCapabilities: query.probe_capabilities ?? null,
    });
  }

  /** Reachability + auth probe of agent-platform. */
  async platformHealth(): Promise<PlatformHealth> {
    return invoke<PlatformHealth>('ai_platform_health');
  }

  /** Spawn agent-platform if it is not running; resolves once health answers (~15s max). */
  async startPlatform(): Promise<PlatformStartResult> {
    return invoke<PlatformStartResult>('start_agent_platform');
  }

  /** Mint a fresh workspace token with the master key and save it into the provider config. */
  async pairPlatform(masterKey: string): Promise<PlatformPairResult> {
    return invoke<PlatformPairResult>('pair_agent_platform', { masterKey });
  }

  /** Fast catalog load: all providers, YAML aliases only. */
  async getCatalogAliases(): Promise<PlatformCatalog> {
    return this.getCatalog({ providers: ['all'], live: false });
  }

  /** Full catalog with live upstream model lists and capability probes. */
  async getCatalogLive(): Promise<PlatformCatalog> {
    return this.getCatalog({ providers: ['all'], live: true, probe_capabilities: true });
  }
}

export const aiProviderService = new AIProviderService();
