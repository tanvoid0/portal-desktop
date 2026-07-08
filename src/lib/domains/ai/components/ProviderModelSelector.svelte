<script lang="ts">
  import ChatCatalogSelectors from "./chat/ChatCatalogSelectors.svelte";
  import type { ProviderType } from "../types/index.js";

  interface Props {
    selectedProvider?: ProviderType | null;
    selectedBackendProvider?: string | null;
    selectedModel?: string | null;
    disabled?: boolean;
    onBackendProviderChange?: (providerId: string) => void;
    onModelChange?: (model: string) => void;
    backendSelectClass?: string;
    modelSelectClass?: string;
    showPlatformLabel?: boolean;
  }

  let {
    selectedProvider = $bindable<ProviderType | null>(null),
    selectedBackendProvider = $bindable<string | null>(null),
    selectedModel = $bindable<string | null>(null),
    disabled = false,
    onBackendProviderChange,
    onModelChange,
    backendSelectClass = "w-[140px]",
    modelSelectClass = "w-[280px]",
    showPlatformLabel = true,
  }: Props = $props();
</script>

<ChatCatalogSelectors
  bind:selectedProvider
  bind:selectedBackendProvider
  bind:selectedModel
  {disabled}
  {onBackendProviderChange}
  {onModelChange}
  {backendSelectClass}
  {modelSelectClass}
  {showPlatformLabel}
/>
