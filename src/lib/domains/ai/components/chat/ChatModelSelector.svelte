<script lang="ts">
  import Select from "$lib/components/ui/select.svelte";
  import { Badge } from "$lib/components/ui/badge";
  import { Loader } from "@lucide/svelte";
  import type { ProviderType, CatalogModel } from "../../types/index.js";
  import { aiProviderService } from "../../services/aiProviderService.js";
  import {
    formatModelLabel,
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

  async function loadModels() {
    if (!selectedProvider || selectedProvider !== "AgentPlatform") {
      catalogModels = [];
      return;
    }

    isLoading = true;
    try {
      const catalog = await aiProviderService.getCatalogAliases();
      catalogModels = flattenCatalogModels(catalog.providers);
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
      try {
        const fallbackIds = await aiProviderService.getAvailableModels(
          selectedProvider,
        );
        catalogModels = fallbackIds.map((id) => ({
          id,
          provider: "unknown",
          source: "alias",
        }));
      } catch {
        catalogModels = [];
      }
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

  const modelOptions = $derived(
    catalogModels.map((model) => ({
      value: model.id,
      label: formatModelLabel(model),
    })),
  );
</script>

<div class="flex items-center gap-2">
  <Select
    options={modelOptions}
    value={selectedModel || undefined}
    onSelect={handleModelChange}
    placeholder={isLoading ? "Loading..." : "Select model"}
    disabled={disabled || isLoading || !selectedProvider || catalogModels.length === 0}
    class={selectClass}
  />
  {#if selectedModel && (selectedModel === defaultModel || selectedModel === resolvedDefaultModel)}
    <Badge variant="secondary" class="text-xs">Default</Badge>
  {/if}
  {#if isLoading}
    <Loader class="h-4 w-4 animate-spin text-muted-foreground" />
  {/if}
</div>
