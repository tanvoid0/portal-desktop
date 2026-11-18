<!-- Pod YAML viewer; loads lazily once its tab is active -->
<script lang="ts">
  import type { ICloudResource } from '$lib/domains/cloud/core/types';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import RefreshCw from '~icons/fa6-solid/rotate';
  import Download from '~icons/fa6-solid/download';
  import Loading from '$lib/components/ui/loading.svelte';
  import { podApi } from '$lib/domains/cloud/api/podApi';
  import { downloadTextFile } from '$lib/utils/downloadTextFile';

  interface Props {
    pod: ICloudResource;
    active: boolean;
  }
  let { pod, active }: Props = $props();

  let yaml = $state('');
  let yamlLoading = $state(false);
  let yamlError = $state<string | null>(null);

  $effect(() => {
    if (active && !yaml && !yamlLoading) loadYAML();
  });

  async function loadYAML() {
    if (yamlLoading) return;

    try {
      yamlLoading = true;
      yamlError = null;

      yaml = await podApi.getYaml(pod.namespace, pod.name);
    } catch (err) {
      yamlError = err instanceof Error ? err.message : 'Failed to load YAML';
      console.error('Failed to load YAML:', err);
    } finally {
      yamlLoading = false;
    }
  }
</script>

<Card>
  <CardHeader>
    <div class="flex items-center justify-between">
      <CardTitle>Pod YAML</CardTitle>
      <div class="flex items-center gap-2">
        <Button variant="outline" size="sm" onclick={loadYAML} disabled={yamlLoading}>
          <RefreshCw class="mr-2 h-4 w-4 {yamlLoading ? 'animate-spin' : ''}" />
          Refresh
        </Button>
        {#if yaml}
          <Button
            variant="outline"
            size="sm"
            onclick={() => downloadTextFile(yaml, `${pod.name}.yaml`, 'text/yaml')}
          >
            <Download class="mr-2 h-4 w-4" />
            Download
          </Button>
        {/if}
      </div>
    </div>
  </CardHeader>
  <CardContent>
    {#if yamlLoading}
      <div class="flex items-center justify-center py-8">
        <Loading text="Loading YAML..." />
      </div>
    {:else if yamlError}
      <div class="text-destructive">{yamlError}</div>
    {:else if yaml}
      <pre
        class="max-h-[600px] overflow-auto rounded-lg bg-muted p-4 font-mono text-xs whitespace-pre-wrap">{yaml}</pre>
    {:else}
      <p class="text-muted-foreground">No YAML available</p>
    {/if}
  </CardContent>
</Card>
