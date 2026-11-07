<!--
	SDK Services Page
	Lists SDKs that run as services, from the same shared query as the sidebar and overview.
-->

<script lang="ts">
  import { createQuery, useQueryClient } from '@tanstack/svelte-query';
  import { invoke } from '@tauri-apps/api/core';
  import { goto } from '$app/navigation';
  import { Card, CardContent } from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import { PageError, PageHeader, PageLoading } from '$lib/components/shell';
  import { sdkConfigsQuery } from '$lib/domains/sdk/queries/sdkQueries';
  import { queryKeys } from '$lib/domains/shared/query/keys';
  import { toast } from '$lib/utils/toast';
  import Play from '~icons/fa6-solid/play';
  import Square from '~icons/fa6-solid/square';
  import ExternalLink from '~icons/fa6-solid/arrow-up-right-from-square';

  const queryClient = useQueryClient();
  const configsQuery = createQuery(sdkConfigsQuery);
  const services = $derived((configsQuery.data ?? []).filter((c) => c.service_running != null));
  let busy = $state<string | null>(null);

  async function toggleService(id: string, running: boolean) {
    busy = id;
    try {
      await invoke(running ? 'stop_service' : 'start_service', { sdkType: id });
      await queryClient.invalidateQueries({ queryKey: queryKeys.sdk.configs });
    } catch (err) {
      toast.error('Service action failed', err instanceof Error ? err.message : String(err));
    } finally {
      busy = null;
    }
  }
</script>

<svelte:head>
  <title>SDK Services - Portal Desktop</title>
</svelte:head>

<div class="space-y-6">
  <PageHeader title="SDK Services" description="Manage running SDK services">
    {#snippet actions()}
      <Button
        variant="outline"
        size="sm"
        onclick={() => configsQuery.refetch()}
        disabled={configsQuery.isFetching}
      >
        Refresh
      </Button>
    {/snippet}
  </PageHeader>

  {#if configsQuery.isPending}
    <PageLoading message="Loading SDK services..." />
  {:else if configsQuery.isError}
    <PageError
      title="Failed to load SDK services"
      message={configsQuery.error instanceof Error
        ? configsQuery.error.message
        : String(configsQuery.error)}
      onRetry={() => configsQuery.refetch()}
    />
  {:else if services.length === 0}
    <Card>
      <CardContent class="p-6 text-center">
        <p class="text-muted-foreground">No SDK services found</p>
      </CardContent>
    </Card>
  {:else}
    <div class="space-y-3">
      {#each services as c (c.id)}
        <Card>
          <CardContent class="flex flex-wrap items-center justify-between gap-4 p-4">
            <div class="flex min-w-0 items-center gap-3">
              <h3 class="text-lg font-semibold">{c.display_name}</h3>
              {#if c.service_running}
                <Badge variant="default" class="bg-green-100 text-green-800">
                  <Play class="mr-1 h-3 w-3" />
                  Running
                </Badge>
              {:else}
                <Badge variant="outline" class="text-gray-500">
                  <Square class="mr-1 h-3 w-3" />
                  Stopped
                </Badge>
              {/if}
              {#if c.service_port}
                <span class="text-sm text-muted-foreground">Port: {c.service_port}</span>
              {/if}
            </div>
            <div class="flex flex-wrap items-center gap-2">
              <Button
                variant={c.service_running ? 'outline' : 'default'}
                size="sm"
                disabled={busy === c.id}
                onclick={() => toggleService(c.id, !!c.service_running)}
              >
                {#if c.service_running}
                  <Square class="mr-1 h-4 w-4" />
                  Stop
                {:else}
                  <Play class="mr-1 h-4 w-4" />
                  Start
                {/if}
              </Button>
              {#if c.service_running && c.service_port}
                <Button
                  variant="outline"
                  size="sm"
                  onclick={() => window.open(`http://localhost:${c.service_port}`, '_blank')}
                >
                  <ExternalLink class="mr-1 h-4 w-4" />
                  Open
                </Button>
              {/if}
              <Button variant="outline" size="sm" onclick={() => goto(`/sdk/${c.id}`)}>
                Manage
              </Button>
            </div>
          </CardContent>
        </Card>
      {/each}
    </div>
  {/if}
</div>
