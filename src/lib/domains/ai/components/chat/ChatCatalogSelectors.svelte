<script lang="ts">
  import { onMount } from 'svelte';
  import Select from '$lib/components/ui/select.svelte';
  import CatalogModelSelect from './CatalogModelSelect.svelte';
  import { Badge } from '$lib/components/ui/badge';
  import Loader from '~icons/fa6-solid/spinner';
  import type {
    ProviderType,
    CatalogProvider,
    PlatformCatalog,
    CatalogModel,
  } from '../../types/index.js';
  import { AI_PROVIDER_SETTINGS_PATH } from '$lib/config/ai-nav';
  import { aiProviderService } from '../../services/aiProviderService.js';
  import {
    selectableCatalogProviders,
    modelsForCatalogProvider,
    defaultModelForCatalogProvider,
  } from '../../utils/catalog.js';
  import { isAuthError } from '../../utils/catalog.js';
  import { loadChatCatalogPrefs, saveChatCatalogPrefs } from '../../utils/chatCatalogPrefs.js';

  import type { CatalogStatus } from '../../types/index.js';

  interface Props {
    selectedProvider?: ProviderType | null;
    selectedBackendProvider?: string | null;
    selectedModel?: string | null;
    onBackendProviderChange?: (providerId: string) => void;
    onModelChange?: (model: string) => void;
    /** Fires on every catalog load so callers can gate actions when offline. */
    onStatusChange?: (status: CatalogStatus) => void;
    disabled?: boolean;
    backendSelectClass?: string;
    modelSelectClass?: string;
    showPlatformLabel?: boolean;
    /** Hide the inline error label when the caller renders its own banner. */
    showInlineError?: boolean;
    /** Extra non-catalog providers to offer (e.g. a local CLI engine). */
    extraProviders?: { value: string; label: string }[];
    /** Hide the model picker (e.g. for an engine that ignores the model). */
    showModel?: boolean;
  }

  let {
    selectedProvider = $bindable<ProviderType | null>('AgentPlatform'),
    selectedBackendProvider = $bindable<string | null>(null),
    selectedModel = $bindable<string | null>(null),
    onBackendProviderChange,
    onModelChange,
    onStatusChange,
    disabled = false,
    backendSelectClass = 'w-[140px]',
    modelSelectClass = 'w-[280px]',
    showPlatformLabel = true,
    showInlineError = true,
    extraProviders = [],
    showModel = true,
  }: Props = $props();

  let catalog = $state<PlatformCatalog | null>(null);
  let catalogProviders = $state<CatalogProvider[]>([]);
  let isLoading = $state(false);
  let loadError = $state<string | null>(null);

  function pickInitialSelection(
    providers: CatalogProvider[],
    resolved: PlatformCatalog['resolved_defaults']
  ) {
    const selectable = selectableCatalogProviders(providers);
    if (selectable.length === 0) return;

    const saved = loadChatCatalogPrefs();
    let backend =
      saved?.backendProvider && selectable.some((entry) => entry.id === saved.backendProvider)
        ? saved.backendProvider
        : null;

    if (!backend) {
      backend =
        selectable.find((entry) => entry.id === resolved.provider)?.id ?? selectable[0]?.id ?? null;
    }
    if (!backend) return;

    const providerEntry = selectable.find((entry) => entry.id === backend);
    const models = modelsForCatalogProvider(providers, backend);
    let model =
      saved?.model && models.some((entry) => entry.id === saved.model) ? saved.model : null;
    if (!model) {
      model = defaultModelForCatalogProvider(providerEntry, resolved);
    }

    selectedBackendProvider = backend;
    if (model) {
      selectedModel = model;
      onModelChange?.(model);
    }
    onBackendProviderChange?.(backend);
  }

  /** Re-fetch the catalog. Doubles as the platform reachability check. */
  export async function reload() {
    await loadCatalog();
  }

  async function loadCatalog() {
    isLoading = true;
    loadError = null;
    onStatusChange?.({ checking: true, online: false, error: null });
    try {
      selectedProvider = selectedProvider ?? 'AgentPlatform';
      const next = await aiProviderService.getCatalogLive();
      catalog = next;
      catalogProviders = selectableCatalogProviders(next.providers);

      if (!selectedBackendProvider || catalogProviders.length === 0) {
        pickInitialSelection(next.providers, next.resolved_defaults);
      } else if (selectedModel) {
        persistSelection();
      }
      onStatusChange?.({ checking: false, online: true, error: null });
    } catch (error) {
      console.error('Failed to load live catalog:', error);
      loadError =
        error instanceof Error ? error.message : String(error || 'Failed to load catalog');
      // Deliberately keep the last known provider list. Blanking it leaves both
      // selectors inert — the engine cannot be switched and the model cannot be
      // changed back — which is how an unreachable platform turned into "the
      // model is stuck forever" (QA-087).
      onStatusChange?.({ checking: false, online: false, error: loadError });
    } finally {
      isLoading = false;
    }
  }

  function persistSelection() {
    if (!selectedBackendProvider || !selectedModel) return;
    saveChatCatalogPrefs({
      backendProvider: selectedBackendProvider,
      model: selectedModel,
    });
  }

  function handleBackendProviderChange(providerId: string) {
    // A non-catalog engine (e.g. the Claude CLI) has no catalog models; just
    // select it and clear the model — the engine ignores it.
    if (extraProviders.some((p) => p.value === providerId)) {
      selectedBackendProvider = providerId;
      selectedModel = null;
      onBackendProviderChange?.(providerId);
      return;
    }
    if (!catalog) return;
    selectedBackendProvider = providerId;
    onBackendProviderChange?.(providerId);

    const providerEntry = catalogProviders.find((entry) => entry.id === providerId);
    const models = modelsForCatalogProvider(catalog.providers, providerId);
    const nextModel = defaultModelForCatalogProvider(providerEntry, catalog.resolved_defaults);
    if (nextModel && models.some((entry) => entry.id === nextModel)) {
      selectedModel = nextModel;
      onModelChange?.(nextModel);
    } else if (models[0]) {
      selectedModel = models[0].id;
      onModelChange?.(models[0].id);
    } else {
      selectedModel = null;
    }
    persistSelection();
  }

  function handleModelChange(modelId: string) {
    selectedModel = modelId;
    onModelChange?.(modelId);
    persistSelection();
  }

  const backendOptions = $derived.by(() => [
    ...catalogProviders.map((provider) => ({
      value: provider.id,
      label: provider.label,
    })),
    ...extraProviders,
  ]);

  const catalogModels = $derived.by((): CatalogModel[] => {
    if (!selectedBackendProvider) return [];
    if (!catalog) {
      // No catalog (platform unreachable or refusing us): still offer the model
      // already in use, so the selector states the truth instead of showing an
      // empty box, and a later successful load can replace it.
      return selectedModel
        ? [{ id: selectedModel, provider: selectedBackendProvider, source: 'alias' }]
        : [];
    }
    const models = modelsForCatalogProvider(catalog.providers, selectedBackendProvider);
    if (selectedModel && !models.some((entry) => entry.id === selectedModel)) {
      return [
        {
          id: selectedModel,
          provider: selectedBackendProvider,
          source: 'alias',
        },
        ...models,
      ];
    }
    return models;
  });

  const isDefaultModel = $derived.by(() => {
    if (!catalog || !selectedBackendProvider || !selectedModel) return false;
    const provider = catalogProviders.find((entry) => entry.id === selectedBackendProvider);
    const resolved = catalog.resolved_defaults;
    return (
      selectedModel === provider?.default_model ||
      (resolved.provider === selectedBackendProvider && selectedModel === resolved.model)
    );
  });

  onMount(() => {
    void loadCatalog();
  });
</script>

<div class="flex flex-wrap items-center gap-2">
  {#if showPlatformLabel}
    <span class="text-sm text-muted-foreground">Agent Platform</span>
  {/if}

  <Select
    options={backendOptions}
    value={selectedBackendProvider || undefined}
    onSelect={handleBackendProviderChange}
    placeholder={isLoading ? 'Loading...' : 'Provider'}
    disabled={disabled || isLoading || backendOptions.length === 0}
    class={backendSelectClass}
  />

  {#if showModel}
    <CatalogModelSelect
      models={catalogModels}
      bind:value={selectedModel}
      onSelect={handleModelChange}
      placeholder={isLoading ? 'Loading...' : 'Select model'}
      disabled={disabled || isLoading || !selectedBackendProvider}
      class={modelSelectClass}
    />
  {/if}

  {#if showModel && isDefaultModel}
    <Badge variant="secondary" class="text-xs">Default</Badge>
  {/if}

  {#if isLoading}
    <Loader class="h-4 w-4 animate-spin text-muted-foreground" />
  {/if}

  {#if loadError && !isLoading && showInlineError}
    <span class="text-xs text-destructive" title={loadError}>
      {isAuthError(loadError) ? 'Agent Platform token rejected.' : 'Agent Platform unreachable.'}
      <a href={AI_PROVIDER_SETTINGS_PATH} class="underline">Settings &gt; AI</a>
    </span>
  {/if}
</div>
