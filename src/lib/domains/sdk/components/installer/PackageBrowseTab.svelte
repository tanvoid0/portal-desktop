<script lang="ts">
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import Search from '~icons/fa6-solid/magnifying-glass';
  import Download from '~icons/fa6-solid/download';
  import Info from '~icons/fa6-solid/circle-info';
  import Loader2 from '~icons/fa6-solid/spinner';
  import InlineSpinner from '$lib/components/shared/InlineSpinner.svelte';
  import InlineEmptyState from '$lib/components/shared/InlineEmptyState.svelte';
  import type { Package } from '$lib/domains/sdk/services/packageManagerService';

  interface Props {
    searching: boolean;
    query: string;
    results: Package[];
    installingId: string | null;
    oninstall: (pkg: Package) => void;
    ondetails: (pkg: Package) => void;
  }
  let { searching, query, results, installingId, oninstall, ondetails }: Props = $props();
</script>

{#if searching}
  <InlineSpinner />
{:else if results.length === 0 && query}
  <InlineEmptyState title="No packages found" />
{:else if results.length === 0}
  <InlineEmptyState
    icon={Search}
    title="Enter a search query to find packages"
    hint={'Try searching for popular packages like "chrome", "vscode", or "git"'}
  />
{:else}
  <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
    {#each results as pkg}
      <Card>
        <CardHeader>
          <div class="flex items-start justify-between">
            <div class="flex-1">
              <CardTitle class="text-lg">{pkg.name}</CardTitle>
              {#if pkg.version}
                <p class="mt-1 text-xs text-muted-foreground">v{pkg.version}</p>
              {/if}
            </div>
            <Badge variant="secondary" class="text-xs">{pkg.source}</Badge>
          </div>
        </CardHeader>
        <CardContent>
          {#if pkg.description}
            <p class="mb-4 line-clamp-2 text-sm text-muted-foreground">{pkg.description}</p>
          {/if}
          <div class="flex items-center gap-2">
            <Button
              variant="default"
              size="sm"
              onclick={() => oninstall(pkg)}
              disabled={installingId === pkg.id}
              class="flex-1"
            >
              {#if installingId === pkg.id}
                <Loader2 class="mr-1 h-4 w-4 animate-spin" />
                Installing...
              {:else}
                <Download class="mr-1 h-4 w-4" />
                Install
              {/if}
            </Button>
            <Button variant="outline" size="sm" onclick={() => ondetails(pkg)}>
              <Info class="h-4 w-4" />
            </Button>
          </div>
        </CardContent>
      </Card>
    {/each}
  </div>
{/if}
