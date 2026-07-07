<script lang="ts">
  import ChatProviderSelector from "./chat/ChatProviderSelector.svelte";
  import ChatModelSelector from "./chat/ChatModelSelector.svelte";
  import type { ProviderType } from "../types/index.js";

  interface Props {
    selectedProvider?: ProviderType | null;
    selectedModel?: string | null;
    disabled?: boolean;
    onModelChange?: (model: string) => void;
    modelSelectClass?: string;
  }

  let {
    selectedProvider = $bindable<ProviderType | null>(null),
    selectedModel = $bindable<string | null>(null),
    disabled = false,
    onModelChange,
    modelSelectClass = "w-[240px]",
  }: Props = $props();
</script>

<div class="flex flex-wrap items-center gap-2">
  <ChatProviderSelector bind:selectedProvider />
  {#if selectedProvider}
    <ChatModelSelector
      bind:selectedProvider
      bind:selectedModel
      {disabled}
      {onModelChange}
      selectClass={modelSelectClass}
    />
  {/if}
</div>
