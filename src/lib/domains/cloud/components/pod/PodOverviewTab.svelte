<!-- Pod overview: metrics, info, containers, raw metadata -->
<script lang="ts">
  import type { ICloudResource } from '$lib/domains/cloud/core/types';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import MetricsDisplay from '$lib/domains/cloud/components/MetricsDisplay.svelte';

  interface Props {
    pod: ICloudResource;
    containers: any[];
    podMetrics: any | null;
    metricsLoading: boolean;
  }
  let { pod, containers, podMetrics, metricsLoading }: Props = $props();
</script>

{#if podMetrics || metricsLoading}
  <MetricsDisplay metrics={podMetrics} title="Pod Resource Usage" showDetails={true} />
{/if}

<div class="grid grid-cols-1 gap-4 md:grid-cols-2">
  <Card>
    <CardHeader>
      <CardTitle>Pod Information</CardTitle>
    </CardHeader>
    <CardContent class="space-y-2">
      <div class="flex justify-between">
        <span class="text-muted-foreground">Name:</span>
        <span class="font-medium">{pod.name}</span>
      </div>
      <div class="flex justify-between">
        <span class="text-muted-foreground">Namespace:</span>
        <span class="font-medium">{pod.namespace}</span>
      </div>
      <div class="flex justify-between">
        <span class="text-muted-foreground">Status:</span>
        <Badge variant={pod.status === 'running' ? 'default' : 'secondary'}>
          {pod.status}
        </Badge>
      </div>
      <div class="flex justify-between">
        <span class="text-muted-foreground">Age:</span>
        <span class="font-medium">{pod.metadata?.age || 'N/A'}</span>
      </div>
      <div class="flex justify-between">
        <span class="text-muted-foreground">Restarts:</span>
        <span class="font-medium">{pod.metadata?.restarts || 0}</span>
      </div>
      {#if pod.metadata?.ip}
        <div class="flex justify-between">
          <span class="text-muted-foreground">Pod IP:</span>
          <span class="font-medium">{pod.metadata.ip}</span>
        </div>
      {/if}
      {#if pod.metadata?.node}
        <div class="flex justify-between">
          <span class="text-muted-foreground">Node:</span>
          <span class="font-medium">{pod.metadata.node}</span>
        </div>
      {/if}
    </CardContent>
  </Card>

  <Card>
    <CardHeader>
      <CardTitle>Containers</CardTitle>
    </CardHeader>
    <CardContent>
      {#if containers.length > 0}
        <div class="space-y-3">
          {#each containers as container}
            <div class="rounded-lg border p-3">
              <div class="mb-2 flex items-center justify-between">
                <span class="font-medium">{container.name || container}</span>
                {#if container.ready !== undefined}
                  <Badge variant={container.ready ? 'default' : 'secondary'}>
                    {container.ready ? 'Ready' : 'Not Ready'}
                  </Badge>
                {/if}
              </div>
              {#if container.image}
                <p class="text-sm text-muted-foreground">
                  Image: {container.image}
                </p>
              {/if}
              {#if container.restartCount !== undefined}
                <p class="text-sm text-muted-foreground">
                  Restarts: {container.restartCount}
                </p>
              {/if}
            </div>
          {/each}
        </div>
      {:else}
        <p class="text-muted-foreground">No container information available</p>
      {/if}
    </CardContent>
  </Card>
</div>

{#if pod.metadata && Object.keys(pod.metadata).length > 0}
  <Card>
    <CardHeader>
      <CardTitle>Metadata</CardTitle>
    </CardHeader>
    <CardContent>
      <pre class="overflow-auto rounded-lg bg-muted p-4 text-xs">{JSON.stringify(
          pod.metadata,
          null,
          2
        )}</pre>
    </CardContent>
  </Card>
{/if}
