<!-- Platform catalog summary and clickable model chips (grouped by provider when available). -->
<script lang="ts">
  import { Badge } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import { formatModelMetadata } from '../../utils/catalog.js';
  import type { PlatformCatalog, CatalogProvider, CatalogModel } from '../../types/index.js';

  interface Props {
    catalog: PlatformCatalog | null;
    providers: CatalogProvider[];
    models: CatalogModel[];
    selectedModel: string;
    onSelect: (id: string) => void;
  }

  let {
    catalog,
    providers: catalogProviders,
    models: catalogModels,
    selectedModel,
    onSelect,
  }: Props = $props();
</script>

{#if catalog}
  <p class="text-xs text-muted-foreground">
    Platform defaults:
    <code class="rounded bg-muted px-1">{catalog.resolved_defaults.provider}</code>
    /
    <code class="rounded bg-muted px-1">{catalog.resolved_defaults.model}</code>
  </p>
{/if}

{#if catalogProviders.length > 0}
  <div class="space-y-4 rounded-md border p-3">
    {#each catalogProviders as provider (provider.id)}
      <div class="space-y-2">
        <div class="flex flex-wrap items-center gap-2">
          <span class="text-sm font-medium">{provider.label}</span>
          <Badge variant="outline" class="text-xs">{provider.id}</Badge>
          {#if provider.configured}
            <Badge class="bg-green-500 text-xs hover:bg-green-600">configured</Badge>
          {:else}
            <Badge variant="secondary" class="text-xs">not configured</Badge>
          {/if}
          {#if provider.reachable}
            <Badge variant="secondary" class="text-xs">reachable</Badge>
          {/if}
          {#if provider.default_model}
            <span class="text-xs text-muted-foreground">
              default: {provider.default_model}
            </span>
          {/if}
        </div>
        {#if provider.models.length > 0}
          <div class="flex flex-wrap gap-2">
            {#each provider.models as model (model.id)}
              <Button
                variant="outline"
                size="sm"
                onclick={() => onSelect(model.id)}
                class={selectedModel === model.id ? 'bg-primary text-primary-foreground' : ''}
                title={formatModelMetadata(model)}
              >
                {model.id}
                {#if model.source === 'alias'}
                  <Badge variant="secondary" class="ml-1 text-[10px]">alias</Badge>
                {/if}
              </Button>
            {/each}
          </div>
        {:else}
          <p class="text-xs text-muted-foreground">No models listed for this provider.</p>
        {/if}
      </div>
    {/each}
  </div>
{:else if catalogModels.length > 0}
  <div class="mt-2 flex flex-wrap gap-2">
    {#each catalogModels as model (model.id)}
      <Button
        variant="outline"
        size="sm"
        onclick={() => onSelect(model.id)}
        class={selectedModel === model.id ? 'bg-primary text-primary-foreground' : ''}
        title={formatModelMetadata(model)}
      >
        {model.id}
      </Button>
    {/each}
  </div>
{/if}
