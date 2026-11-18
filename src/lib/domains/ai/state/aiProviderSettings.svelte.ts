import { toastActions } from '$lib/utils/toast';
import { aiProviderService } from '../services/aiProviderService.js';
import { aiPlatform } from './aiPlatform.svelte.js';
import { flattenCatalogModels, isAuthError } from '../utils/catalog.js';
import type {
  ProviderConfig,
  ProviderType,
  ConfigurationStatus,
  PlatformCatalog,
  CatalogProvider,
  CatalogModel,
} from '../types/index.js';

const providerType: ProviderType = 'AgentPlatform';

/** State and handlers behind the Agent Platform provider settings card. */
export class AIProviderSettingsState {
  readonly providerType = providerType;
  providerConfig = $state<ProviderConfig | null>(null);
  defaultProvider = $state<ProviderType | null>(null);
  configStatus = $state<ConfigurationStatus | null>(null);
  isLoading = $state(false);
  testingProvider = $state(false);
  catalog = $state<PlatformCatalog | null>(null);
  catalogProviders = $state<CatalogProvider[]>([]);
  catalogModels = $state<CatalogModel[]>([]);
  loadingCatalog = $state(false);
  catalogLive = $state(false);

  showApiKey = $state(false);
  pairOpen = $state(false);
  pairing = $state(false);
  /** Real auth state of the saved token; `unknown` until a probe/test has run. */
  connection = $state<'unknown' | 'ok' | 'unauthorized' | 'unreachable'>('unknown');

  private markConnection = (error?: unknown) => {
    this.connection =
      error === undefined ? 'ok' : isAuthError(error) ? 'unauthorized' : 'unreachable';
  };

  init = () => {
    void aiPlatform.refresh();
    void this.loadProvider();
  };

  pair = async (key: string) => {
    this.pairing = true;
    try {
      const res = await aiProviderService.pairPlatform(key);
      if (res.ok) {
        toastActions.success('Paired', 'The API token was replaced with a fresh one.');
        this.pairOpen = false;
        await this.loadProvider();
      } else {
        toastActions.error('Pairing failed', res.detail);
      }
    } catch (error) {
      toastActions.error('Pairing failed', error);
    } finally {
      this.pairing = false;
      await aiPlatform.refresh();
    }
  };

  private async loadProvider() {
    this.isLoading = true;
    try {
      const [config, defaultProviderType, status] = await Promise.all([
        aiProviderService.getProviderConfig(providerType).catch(() => null),
        aiProviderService.getDefaultProvider(),
        aiProviderService.getConfigStatus(providerType).catch(() => null),
      ]);

      this.providerConfig = config;
      this.defaultProvider = defaultProviderType;
      this.configStatus = status;
      if (status?.is_configured) {
        aiProviderService
          .testProvider(providerType)
          .then(() => this.markConnection())
          .catch(this.markConnection);
      }
    } catch (error) {
      console.error('Failed to load Agent Platform configuration:', error);
      toastActions.error('Failed to load configuration', error);
    } finally {
      this.isLoading = false;
    }
  }

  updateConfig = async (updates: Partial<ProviderConfig>) => {
    if (!this.providerConfig) return;
    if ('api_key' in updates || 'base_url' in updates) this.connection = 'unknown';

    const updated: ProviderConfig = {
      ...this.providerConfig,
      ...updates,
    };

    try {
      await aiProviderService.saveProviderConfig(updated);
      this.providerConfig = updated;

      const status = await aiProviderService.getConfigStatus(providerType);
      this.configStatus = status;

      if (updated.enabled && status.is_configured && !this.defaultProvider) {
        await aiProviderService.setDefaultProvider(providerType);
        this.defaultProvider = providerType;
      }

      toastActions.success('Configuration saved');
    } catch (error) {
      console.error('Failed to save config:', error);
      toastActions.error('Failed to save configuration', error);
    }
  };

  setAsDefault = async () => {
    try {
      await aiProviderService.setDefaultProvider(providerType);
      this.defaultProvider = providerType;
      toastActions.success('Agent Platform set as default provider');
    } catch (error) {
      toastActions.error('Failed to set default provider', error);
    }
  };

  testConnection = async () => {
    if (!this.providerConfig) return;

    this.testingProvider = true;
    try {
      const configToSave = { ...this.providerConfig, enabled: true };
      await aiProviderService.saveProviderConfig(configToSave);
      this.providerConfig = configToSave;

      if (!this.defaultProvider) {
        await aiProviderService.setDefaultProvider(providerType);
        this.defaultProvider = providerType;
      }

      await new Promise((resolve) => setTimeout(resolve, 100));
      await aiProviderService.testProvider(providerType);
      this.markConnection();
      toastActions.success('Connection successful');

      this.configStatus = await aiProviderService.getConfigStatus(providerType);
    } catch (error) {
      this.markConnection(error);
      toastActions.error('Connection test failed', error);
    } finally {
      this.testingProvider = false;
    }
  };

  loadModels = async (live = false) => {
    if (!this.providerConfig) return;

    this.loadingCatalog = true;
    this.catalogLive = live;
    try {
      await aiProviderService.saveProviderConfig({
        ...this.providerConfig,
        enabled: true,
      });

      const catalog = live
        ? await aiProviderService.getCatalogLive()
        : await aiProviderService.getCatalogAliases();
      this.catalog = catalog;

      this.catalogProviders = catalog.providers;
      this.catalogModels = flattenCatalogModels(catalog.providers);
      if (live) this.markConnection();

      if (this.catalogModels.length === 0) {
        toastActions.info(
          'No models found',
          'Check that agent-platform is running and has models configured in /config'
        );
      } else {
        const liveNote = live ? ' (live upstream)' : ' (aliases)';
        toastActions.success(
          `Loaded ${this.catalogModels.length} model(s) from ${this.catalogProviders.length} provider(s)${liveNote}`
        );

        if (!this.providerConfig.model && catalog.resolved_defaults?.model) {
          await this.updateConfig({ model: catalog.resolved_defaults.model });
        }
      }
    } catch (error) {
      this.markConnection(error);
      toastActions.error('Failed to load catalog', error);
    } finally {
      this.loadingCatalog = false;
    }
  };
}
