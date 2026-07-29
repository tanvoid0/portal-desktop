<script lang="ts">
  import ChatCatalogSelectors from "./chat/ChatCatalogSelectors.svelte";
  import type { CatalogStatus, ProviderType } from "../types/index.js";

  interface Props {
    selectedProvider?: ProviderType | null;
    selectedBackendProvider?: string | null;
    selectedModel?: string | null;
    disabled?: boolean;
    onBackendProviderChange?: (providerId: string) => void;
    onModelChange?: (model: string) => void;
    onStatusChange?: (status: CatalogStatus) => void;
    backendSelectClass?: string;
    modelSelectClass?: string;
    showPlatformLabel?: boolean;
    showInlineError?: boolean;
  }

  let {
    selectedProvider = $bindable<ProviderType | null>(null),
    selectedBackendProvider = $bindable<string | null>(null),
    selectedModel = $bindable<string | null>(null),
    disabled = false,
    onBackendProviderChange,
    onModelChange,
    onStatusChange,
    backendSelectClass = "w-[140px]",
    modelSelectClass = "w-[280px]",
    showPlatformLabel = true,
    showInlineError = true,
  }: Props = $props();

  let selectors = $state<ChatCatalogSelectors | null>(null);

  /** Re-check the platform by re-fetching the catalog. */
  export async function reload() {
    await selectors?.reload();
  }
</script>

<ChatCatalogSelectors
  bind:this={selectors}
  bind:selectedProvider
  bind:selectedBackendProvider
  bind:selectedModel
  {disabled}
  {onBackendProviderChange}
  {onModelChange}
  {onStatusChange}
  {backendSelectClass}
  {modelSelectClass}
  {showPlatformLabel}
  {showInlineError}
/>
