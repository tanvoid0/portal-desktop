<!-- Deployment overview: info, replica progress bars, labels -->
<script lang="ts">
  import type { ICloudResource } from '$lib/domains/cloud/core/types';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';

  interface Props {
    deployment: ICloudResource;
  }
  let { deployment }: Props = $props();

  const labels = $derived(deployment.metadata?.labels || {});
  const desired = $derived(deployment.metadata?.desired);
  const bars = $derived([
    { label: 'Ready', value: deployment.metadata?.available, color: 'bg-green-600' },
    { label: 'Up-to-date', value: deployment.metadata?.up_to_date, color: 'bg-blue-600' },
  ]);
</script>

<div class="grid grid-cols-1 gap-4 md:grid-cols-2">
  <Card>
    <CardHeader>
      <CardTitle>Deployment Information</CardTitle>
    </CardHeader>
    <CardContent class="space-y-3">
      <div>
        <p class="text-sm text-muted-foreground">Status</p>
        <Badge class="mt-1" variant="default">{deployment.status}</Badge>
      </div>
      <div>
        <p class="text-sm text-muted-foreground">Desired Replicas</p>
        <p class="font-medium">{desired || 0}</p>
      </div>
      <div>
        <p class="text-sm text-muted-foreground">Current Replicas</p>
        <p class="font-medium">{deployment.metadata?.current || 0}</p>
      </div>
      <div>
        <p class="text-sm text-muted-foreground">Available Replicas</p>
        <p class="font-medium">{deployment.metadata?.available || 0}/{desired || 0}</p>
      </div>
      <div>
        <p class="text-sm text-muted-foreground">Up-to-date Replicas</p>
        <p class="font-medium">{deployment.metadata?.up_to_date || 0}</p>
      </div>
      <div>
        <p class="text-sm text-muted-foreground">Age</p>
        <p class="font-medium">{deployment.metadata?.age || 'N/A'}</p>
      </div>
    </CardContent>
  </Card>

  <Card>
    <CardHeader>
      <CardTitle>Replica Status</CardTitle>
    </CardHeader>
    <CardContent>
      <div class="space-y-4">
        {#each bars as bar (bar.label)}
          <div>
            <div class="mb-2 flex justify-between">
              <span class="text-sm">{bar.label}</span>
              <span class="font-medium">{bar.value || 0}/{desired || 0}</span>
            </div>
            <div class="h-2 w-full rounded-full bg-muted">
              <div
                class="h-2 rounded-full {bar.color} transition-all"
                style="width: {desired ? (bar.value / desired) * 100 : 0}%"
              ></div>
            </div>
          </div>
        {/each}
      </div>
    </CardContent>
  </Card>
</div>

<Card>
  <CardHeader>
    <CardTitle>Labels</CardTitle>
  </CardHeader>
  <CardContent>
    {#if Object.keys(labels).length > 0}
      <div class="flex flex-wrap gap-2">
        {#each Object.entries(labels) as [key, value]}
          <Badge variant="outline">{key}={value}</Badge>
        {/each}
      </div>
    {:else}
      <p class="text-muted-foreground">No labels configured</p>
    {/if}
  </CardContent>
</Card>
