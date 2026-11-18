<!-- Loading spinner / error + Retry / content, shared by the Ollama tabs. -->
<script lang="ts">
  import type { Snippet } from 'svelte';
  import { Button } from '$lib/components/ui/button';

  interface Props {
    loading: boolean;
    error: string | null;
    loadingText: string;
    onretry: () => void;
    children: Snippet;
  }
  let { loading, error, loadingText, onretry, children }: Props = $props();
</script>

{#if loading}
  <div class="flex items-center justify-center p-8">
    <div class="h-8 w-8 animate-spin rounded-full border-b-2 border-primary"></div>
    <span class="ml-2">{loadingText}</span>
  </div>
{:else if error}
  <div class="p-8 text-center">
    <p class="text-destructive">Error: {error}</p>
    <Button onclick={onretry} class="mt-4">Retry</Button>
  </div>
{:else}
  {@render children()}
{/if}
