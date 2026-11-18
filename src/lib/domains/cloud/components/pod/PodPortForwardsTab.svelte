<!-- Active port forwards for a pod -->
<script lang="ts">
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import Network from '~icons/fa6-solid/network-wired';
  import Loading from '$lib/components/ui/loading.svelte';

  interface Props {
    forwards: any[];
    loading: boolean;
    onNew: () => void;
    onStop: (id: string) => void;
  }
  let { forwards, loading, onNew, onStop }: Props = $props();
</script>

<Card>
  <CardHeader>
    <div class="flex items-center justify-between">
      <CardTitle>Active Port Forwards</CardTitle>
      <Button variant="outline" size="sm" onclick={onNew}>
        <Network class="mr-2 h-4 w-4" />
        New Port Forward
      </Button>
    </div>
  </CardHeader>
  <CardContent>
    {#if loading}
      <div class="flex items-center justify-center py-8">
        <Loading text="Loading port forwards..." />
      </div>
    {:else if forwards.length === 0}
      <p class="py-8 text-center text-muted-foreground">No active port forwards for this pod.</p>
    {:else}
      <div class="space-y-3">
        {#each forwards as forward (forward.id)}
          <div class="flex items-center justify-between rounded-lg border p-4">
            <div class="flex-1">
              <div class="mb-1 flex items-center gap-2">
                <Badge variant="default">Active</Badge>
                <span class="font-medium">{forward.url}</span>
              </div>
              <p class="text-sm text-muted-foreground">
                localhost:{forward.local_port} → {forward.pod_name}:{forward.remote_port}
              </p>
              <p class="mt-1 text-xs text-muted-foreground">
                Started: {new Date(forward.created_at).toLocaleString()}
              </p>
            </div>
            <Button variant="destructive" size="sm" onclick={() => onStop(forward.id)}>Stop</Button>
          </div>
        {/each}
      </div>
    {/if}
  </CardContent>
</Card>
