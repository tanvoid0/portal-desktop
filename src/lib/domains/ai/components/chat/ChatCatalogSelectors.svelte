<script lang="ts">
  import { onMount } from "svelte";
  import Select from "$lib/components/ui/select.svelte";
  import CatalogModelSelect from "./CatalogModelSelect.svelte";
  import { Badge } from "$lib/components/ui/badge";
  import { Loader } from "@lucide/svelte";
  import type {
    ProviderType,
    CatalogProvider,
    PlatformCatalog,
    CatalogModel,
  } from "../../types/index.js";
  import { aiProviderService } from "../../services/aiProviderService.js";
  import {
    selectableCatalogProviders,
    modelsForCatalogProvider,
    defaultModelForCatalogProvider,
  } from "../../utils/catalog.js";
  import {
    loadChatCatalogPrefs,
    saveChatCatalogPrefs,
  } from "../../utils/chatCatalogPrefs.js";

  interface Props {
    selectedProvider?: ProviderType | null;
    selectedBackendProvider?: string | null;
    selectedModel?: string | null;
    onBackendProviderChange?: (providerId: string) => void;
    onModelChange?: (model: string) => void;
    disabled?: boolean;
    backendSelectClass?: string;
    modelSelectClass?: string;
    showPlatformLabel?: boolean;
  }

  let {
    selectedProvider = $bindable<ProviderType | null>("AgentPlatform"),
    selectedBackendProvider = $bindable<string | null>(null),
    selectedModel = $bindable<string | null>(null),
    onBackendProviderChange,
    onModelChange,
    disabled = false,
    backendSelectClass = "w-[140px]",
    modelSelectClass = "w-[280px]",
    showPlatformLabel = true,
  }: Props = $props();

  let catalog = $state<PlatformCatalog | null>(null);
  let catalogProviders = $state<CatalogProvider[]>([]);
  let isLoading = $state(false);
  let loadError = $state<string | null>(null);

  function pickInitialSelection(
    providers: CatalogProvider[],
    resolved: PlatformCatalog["resolved_defaults"],
  ) {
    const selectable = selectableCatalogProviders(providers);
    if (selectable.length === 0) return;

    const saved = loadChatCatalogPrefs();
    let backend =
      saved?.backendProvider &&
      selectable.some((entry) => entry.id === saved.backendProvider)
        ? saved.backendProvider
        : null;

    if (!backend) {
      backend =
        selectable.find((entry) => entry.id === resolved.provider)?.id ??
        selectable[0]?.id ??
        null;
    }
    if (!backend) return;

    const providerEntry = selectable.find((entry) => entry.id === backend);
    const models = modelsForCatalogProvider(providers, backend);
    let model =
      saved?.model && models.some((entry) => entry.id === saved.model)
        ? saved.model
        : null;
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

  async function loadCatalog() {
    isLoading = true;
    loadError = null;
    try {
      selectedProvider = selectedProvider ?? "AgentPlatform";
      const next = await aiProviderService.getCatalogLive();
      catalog = next;
      catalogProviders = selectableCatalogProviders(next.providers);

      if (!selectedBackendProvider || catalogProviders.length === 0) {
        pickInitialSelection(next.providers, next.resolved_defaults);
      } else if (selectedModel) {
        persistSelection();
      }
    } catch (error) {
      console.error("Failed to load live catalog:", error);
      loadError =
        error instanceof Error ? error.message : "Failed to load catalog";
      catalogProviders = [];
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
    if (!catalog) return;
    selectedBackendProvider = providerId;
    onBackendProviderChange?.(providerId);

    const providerEntry = catalogProviders.find(
      (entry) => entry.id === providerId,
    );
    const models = modelsForCatalogProvider(catalog.providers, providerId);
    const nextModel = defaultModelForCatalogProvider(
      providerEntry,
      catalog.resolved_defaults,
    );
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

  const backendOptions = $derived.by(() =>
    catalogProviders.map((provider) => ({
      value: provider.id,
      label: provider.label,
    })),
  );

  const catalogModels = $derived.by((): CatalogModel[] => {
    if (!catalog || !selectedBackendProvider) return [];
    const models = modelsForCatalogProvider(
      catalog.providers,
      selectedBackendProvider,
    );
    if (
      selectedModel &&
      !models.some((entry) => entry.id === selectedModel)
    ) {
      return [
        {
          id: selectedModel,
          provider: selectedBackendProvider,
          source: "alias",
        },
        ...models,
      ];
    }
    return models;
  });

  const isDefaultModel = $derived.by(() => {
    if (!catalog || !selectedBackendProvider || !selectedModel) return false;
    const provider = catalogProviders.find(
      (entry) => entry.id === selectedBackendProvider,
    );
    const resolved = catalog.resolved_defaults;
    return (
      selectedModel === provider?.default_model ||
      (resolved.provider === selectedBackendProvider &&
        selectedModel === resolved.model)
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
    placeholder={isLoading ? "Loading..." : "Provider"}
    disabled={disabled || isLoading || backendOptions.length === 0}
    class={backendSelectClass}
  />

  <CatalogModelSelect
    models={catalogModels}
    bind:value={selectedModel}
    onSelect={handleModelChange}
    placeholder={isLoading ? "Loading..." : "Select model"}
    disabled={disabled || isLoading || !selectedBackendProvider}
    class={modelSelectClass}
  />

  {#if isDefaultModel}
    <Badge variant="secondary" class="text-xs">Default</Badge>
  {/if}

  {#if isLoading}
    <Loader class="h-4 w-4 animate-spin text-muted-foreground" />
  {/if}

  {#if loadError && !isLoading}
    <span class="text-xs text-destructive" title={loadError}>Catalog error</span>
  {/if}
</div>
