<script lang="ts">
  import { Badge } from "$lib/components/ui/badge";
  import type { ProviderType } from "../../types/index.js";
  import { aiProviderService } from "../../services/aiProviderService.js";

  interface Props {
    selectedProvider?: ProviderType | null;
    onProviderChange?: (provider: ProviderType) => void;
  }

  let {
    selectedProvider = $bindable<ProviderType | null>(null),
    onProviderChange,
  }: Props = $props();

  let defaultProvider = $state<ProviderType | null>(null);
  let isLoading = $state(false);

  async function loadDefaultProvider() {
    isLoading = true;
    try {
      defaultProvider = await aiProviderService.getDefaultProvider();
      if (!selectedProvider) {
        selectedProvider = defaultProvider ?? "AgentPlatform";
      }
      onProviderChange?.(selectedProvider ?? "AgentPlatform");
    } catch {
      selectedProvider = "AgentPlatform";
    } finally {
      isLoading = false;
    }
  }

  $effect(() => {
    loadDefaultProvider();
  });
</script>

<div class="flex items-center gap-2 text-sm text-muted-foreground">
  {#if isLoading}
    <span>Loading provider…</span>
  {:else}
    <span>Agent Platform</span>
    {#if selectedProvider === defaultProvider || selectedProvider === "AgentPlatform"}
      <Badge variant="secondary" class="text-xs">Default</Badge>
    {/if}
  {/if}
</div>
