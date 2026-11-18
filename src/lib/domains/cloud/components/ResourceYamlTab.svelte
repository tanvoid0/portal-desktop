<!-- Generic resource YAML editor card; loads lazily once its tab is active -->
<script lang="ts">
  import type { ICloudResource } from '$lib/domains/cloud/core/types';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import Loading from '$lib/components/ui/loading.svelte';
  import { k8sResourceService } from '$lib/domains/cloud/services/k8sResourceService';
  import { toastActions } from '$lib/utils/toast';
  import YamlEditor from './YamlEditor.svelte';

  interface Props {
    resource: ICloudResource;
    kind: string;
    active: boolean;
    /** Reload the resource itself after a successful apply. */
    onApplied?: () => Promise<void> | void;
  }
  let { resource, kind, active, onApplied = () => {} }: Props = $props();

  let yaml = $state('');
  let yamlLoading = $state(false);
  let yamlError = $state<string | null>(null);

  $effect(() => {
    if (active && !yaml && !yamlLoading) loadYAML();
  });

  async function loadYAML() {
    try {
      yamlLoading = true;
      yamlError = null;
      yaml = await k8sResourceService.getResourceYaml(kind, resource.namespace, resource.name);
    } catch (err) {
      yamlError = err instanceof Error ? err.message : 'Failed to load YAML';
      console.error('Failed to load YAML:', err);
    } finally {
      yamlLoading = false;
    }
  }

  async function handleSaveYAML(yamlContent: string) {
    try {
      const result = await k8sResourceService.applyResourceYaml(resource.namespace, yamlContent);
      toastActions.success(result);
      await onApplied();
      await loadYAML();
    } catch (err) {
      toastActions.error(err instanceof Error ? err.message : 'Failed to apply YAML');
      throw err;
    }
  }
</script>

<Card class="flex h-full flex-col">
  <CardHeader>
    <CardTitle>{kind} YAML</CardTitle>
  </CardHeader>
  <CardContent class="flex-1 overflow-hidden">
    {#if yamlLoading}
      <div class="flex h-full items-center justify-center">
        <Loading text="Loading YAML..." />
      </div>
    {:else if yamlError}
      <div class="flex h-full items-center justify-center text-center text-destructive">
        <p>{yamlError}</p>
      </div>
    {:else if yaml}
      <YamlEditor
        value={yaml}
        onSave={handleSaveYAML}
        resourceName={resource.name}
        resourceKind={kind}
        namespace={resource.namespace}
      />
    {:else}
      <div class="flex h-full items-center justify-center text-center text-muted-foreground">
        <p>No YAML available.</p>
      </div>
    {/if}
  </CardContent>
</Card>
