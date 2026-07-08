<script lang="ts">
  import CatalogModelSelect from "./CatalogModelSelect.svelte";
  import { Badge } from "$lib/components/ui/badge";
  import { Loader } from "@lucide/svelte";
  import type { ProviderType, CatalogModel } from "../../types/index.js";
  import { aiProviderService } from "../../services/aiProviderService.js";
  import {
    flattenCatalogModels,
  } from "../../utils/catalog.js";

  interface Props {
    selectedProvider?: ProviderType | null;
    selectedModel?: string | null;
    onModelChange?: (model: string) => void;
    disabled?: boolean;
    selectClass?: string;
  }

  let {
    selectedProvider = $bindable<ProviderType | null>(null),
    selectedModel = $bindable<string | null>(null),
    onModelChange,
    disabled = false,
    selectClass = "w-[280px]",
  }: Props = $props();

  let catalogModels = $state<CatalogModel[]>([]);
  let isLoading = $state(false);
  let defaultModel = $state<string | null>(null);
  let resolvedDefaultModel = $state<string | null>(null);

  async function loadFallbackModels() {
    try {
      const fallbackIds = await aiProviderService.getAvailableModels(
        selectedProvider ?? undefined,
      );
      catalogModels = fallbackIds.map((id) => ({
        id,
        provider: "unknown",
        source: "alias",
      }));
    } catch {
      catalogModels = [];
    }
  }

  async function loadModels() {
    if (!selectedProvider || selectedProvider !== "AgentPlatform") {
      catalogModels = [];
      return;
    }

    isLoading = true;
    try {
      const catalog = await aiProviderService.getCatalogAliases();
      catalogModels = flattenCatalogModels(catalog.providers);
      if (catalogModels.length === 0) {
        await loadFallbackModels();
      }
      resolvedDefaultModel = catalog.resolved_defaults?.model ?? null;

      const config =
        await aiProviderService.getProviderConfig(selectedProvider);
      defaultModel = config.model || resolvedDefaultModel;

      if (!selectedModel && defaultModel) {
        selectedModel = defaultModel;
        onModelChange?.(defaultModel);
      }
    } catch (error) {
      console.error("Failed to load catalog:", error);
      await loadFallbackModels();
    } finally {
      isLoading = false;
    }
  }

  $effect(() => {
    if (selectedProvider) {
      loadModels();
    } else {
      catalogModels = [];
      selectedModel = null;
    }
  });

  function handleModelChange(value: string) {
    selectedModel = value;
    onModelChange?.(value);
  }

  const catalogModelList = $derived.by((): CatalogModel[] => {
    if (
      selectedModel &&
      !catalogModels.some((entry) => entry.id === selectedModel)
    ) {
      return [
        { id: selectedModel, provider: "unknown", source: "alias" },
        ...catalogModels,
      ];
    }
    return catalogModels;
  });
</script>

<div class="flex items-center gap-2">
  <CatalogModelSelect
    models={catalogModelList}
    bind:value={selectedModel}
    onSelect={handleModelChange}
    placeholder={isLoading ? "Loading..." : "Select model"}
    disabled={disabled || isLoading || !selectedProvider}
    class={selectClass}
  />
  {#if selectedModel && (selectedModel === defaultModel || selectedModel === resolvedDefaultModel)}
    <Badge variant="secondary" class="text-xs">Default</Badge>
  {/if}
  {#if isLoading}
    <Loader class="h-4 w-4 animate-spin text-muted-foreground" />
  {/if}
</div>
