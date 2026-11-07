<script lang="ts">
  import ModelCard from './ModelCard.svelte';
  import ProgressIndicator from './ProgressIndicator.svelte';
  import { Button } from './ui/button';

  interface Props {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    models?: any[];
    isInstalled?: boolean;
    loading?: boolean;
    error?: string | null;
    installingModel?: string | null;
    installationProgress?: number;
    installationStatus?: string;
    onInstall?: (modelName: string) => void;
    onRemove?: (modelName: string) => void;
    onView?: (modelName: string) => void;
    onRetry?: () => void;
    onBrowseAvailable?: () => void;
  }

  let {
    models = [],
    isInstalled = false,
    loading = false,
    error = null,
    installingModel = null,
    installationProgress = 0,
    installationStatus = '',
    onInstall,
    onRemove,
    onView,
    onRetry,
    onBrowseAvailable,
  }: Props = $props();
</script>

{#if loading}
  <div class="flex items-center justify-center p-8">
    <div class="h-8 w-8 animate-spin rounded-full border-b-2 border-primary"></div>
    <span class="ml-2">Loading models...</span>
  </div>
{:else if error}
  <div class="p-8 text-center">
    <p class="text-destructive">Error: {error}</p>
    {#if onRetry}
      <Button onclick={onRetry} class="mt-4">Retry</Button>
    {/if}
  </div>
{:else if models.length > 0 || installingModel}
  <div class="space-y-4">
    {#if installingModel}
      <ProgressIndicator
        title="Installing {installingModel}"
        status={installationStatus}
        progress={installationProgress}
      />
    {/if}

    <div class="space-y-2">
      {#each models as model}
        <ModelCard {model} {isInstalled} {onInstall} {onRemove} {onView} />
      {/each}
    </div>

    {#if models.length === 0 && !installingModel}
      <div class="p-8 text-center">
        <p class="text-muted-foreground">
          {isInstalled ? 'No models installed yet.' : 'No models available.'}
        </p>
        {#if isInstalled && onBrowseAvailable}
          <Button onclick={onBrowseAvailable} class="mt-4">Browse Available Models</Button>
        {/if}
      </div>
    {/if}
  </div>
{:else}
  <div class="p-8 text-center">
    <p class="text-muted-foreground">No models available.</p>
    {#if onRetry}
      <Button onclick={onRetry} class="mt-4">Load Models</Button>
    {/if}
  </div>
{/if}
