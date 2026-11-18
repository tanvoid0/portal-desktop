<!-- Pod exec: one card per container, opens a kubectl exec terminal -->
<script lang="ts">
  import { goto } from '$app/navigation';
  import type { ICloudResource } from '$lib/domains/cloud/core/types';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import Terminal from '~icons/fa6-solid/terminal';

  interface Props {
    pod: ICloudResource;
    containers: any[];
  }
  let { pod, containers }: Props = $props();

  function exec(container: any) {
    const containerFlag = container.name ? `-c ${container.name}` : '';
    const execCommand = `kubectl exec -it ${pod.name} ${containerFlag} -n ${pod.namespace} -- sh`;
    goto(`/terminal?command=${encodeURIComponent(execCommand)}`);
  }
</script>

<Card>
  <CardHeader>
    <CardTitle>Exec into Pod</CardTitle>
  </CardHeader>
  <CardContent>
    <p class="mb-4 text-muted-foreground">
      Execute commands in a container within this pod. This will open a terminal session.
    </p>
    {#if containers.length > 0}
      <div class="space-y-4">
        {#each containers as container}
          <div class="rounded-lg border p-4">
            <div class="flex items-center justify-between">
              <div>
                <p class="font-medium">{container.name || container}</p>
                {#if container.image}
                  <p class="text-sm text-muted-foreground">
                    {container.image}
                  </p>
                {/if}
              </div>
              <Button onclick={() => exec(container)}>
                <Terminal class="mr-2 h-4 w-4" />
                Exec
              </Button>
            </div>
          </div>
        {/each}
      </div>
    {:else}
      <p class="text-muted-foreground">No containers available</p>
    {/if}
  </CardContent>
</Card>
