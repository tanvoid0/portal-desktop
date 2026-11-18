<!-- ConfigMap Detail Page -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { ResourceDetail } from '$lib/domains/cloud/state/resourceDetail.svelte';
  import { ResourceType } from '$lib/domains/cloud/core/types';
  import { Tabs, TabsContent, TabsList, TabsTrigger } from '$lib/components/ui/tabs';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import Trash2 from '~icons/fa6-solid/trash';
  import { k8sResourceService } from '$lib/domains/cloud/services/k8sResourceService';
  import Loading from '$lib/components/ui/loading.svelte';
  import { PageLoading, PageError } from '$lib/components/shell';
  import { toastActions } from '$lib/utils/toast';
  import { confirmAction } from '$lib/utils/confirm';
  import InfoCard from '$lib/domains/cloud/components/InfoCard.svelte';
  import ResourceYamlTab from '$lib/domains/cloud/components/ResourceYamlTab.svelte';
  import ResourceDetailHeader from '$lib/domains/cloud/components/ResourceDetailHeader.svelte';
  import LabelsCard from '$lib/domains/cloud/components/LabelsCard.svelte';

  const r = new ResourceDetail(ResourceType.CONFIGMAP, 'ConfigMap', () => page.params.configmap);
  const configMap = $derived(r.resource);

  onMount(r.load);

  async function handleDelete() {
    if (!configMap) return;

    const confirmed = await confirmAction(
      `Are you sure you want to delete ConfigMap "${configMap.name}"? This action cannot be undone.`,
      'Delete config map'
    );
    if (!confirmed) return;

    try {
      await k8sResourceService.deleteConfigmap(configMap.namespace, configMap.name);

      toastActions.success(`ConfigMap "${configMap.name}" deleted successfully`);
      goto('/cloud/configmaps');
    } catch (err) {
      toastActions.error(err instanceof Error ? err.message : 'Failed to delete ConfigMap');
    }
  }

  const dataEntries = $derived.by(() => {
    if (!configMap || !configMap.metadata?.data) return [];
    return Object.entries(configMap.metadata.data);
  });
</script>

<div class="space-y-6 p-6">
  {#if r.isLoading}
    <PageLoading message="Loading configmap..." />
  {:else if r.error}
    <PageError title="Failed to load configmap" message={r.error} onRetry={r.load} />
  {:else if configMap}
    <ResourceDetailHeader
      kind="ConfigMap"
      name={configMap.name}
      namespace={configMap.namespace}
      backHref="/cloud/configmaps"
      backLabel="Back to ConfigMaps"
      onRefresh={r.load}
    >
      <Button variant="destructive" size="sm" onclick={handleDelete}>
        <Trash2 class="mr-2 h-4 w-4" />
        Delete
      </Button>
    </ResourceDetailHeader>

    <!-- Tabs -->
    <Tabs value={r.activeTab} onValueChange={r.setTab}>
      <TabsList>
        <TabsTrigger value="overview">Overview</TabsTrigger>
        <TabsTrigger value="data">Data</TabsTrigger>
        <TabsTrigger value="yaml">YAML</TabsTrigger>
      </TabsList>

      <!-- Overview Tab -->
      <TabsContent value="overview" class="space-y-4">
        <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
          <InfoCard
            title="ConfigMap Information"
            fields={[
              { label: 'Name', value: configMap.name },
              { label: 'Namespace', value: configMap.namespace },
              { label: 'Data Keys', value: configMap.metadata?.dataCount || 0 },
              { label: 'Age', value: configMap.metadata?.age || 'N/A' },
            ]}
          />

          {#if configMap.metadata?.labels && Object.keys(configMap.metadata.labels).length > 0}
            <LabelsCard items={configMap.metadata.labels} />
          {/if}
        </div>
      </TabsContent>

      <!-- Data Tab -->
      <TabsContent value="data" class="space-y-4">
        <Card>
          <CardHeader>
            <CardTitle>Configuration Data</CardTitle>
          </CardHeader>
          <CardContent>
            {#if dataEntries.length === 0}
              <div class="py-8 text-center text-muted-foreground">
                <p>No data entries</p>
              </div>
            {:else}
              <div class="space-y-4">
                {#each dataEntries as [key, value]}
                  <div class="rounded-lg border p-4">
                    <div class="mb-2 flex items-center justify-between">
                      <span class="font-medium">{key}</span>
                      <Button
                        variant="ghost"
                        size="sm"
                        onclick={() => {
                          navigator.clipboard.writeText(String(value));
                          toastActions.success('Value copied to clipboard');
                        }}
                      >
                        Copy
                      </Button>
                    </div>
                    <pre
                      class="overflow-auto rounded bg-muted p-3 text-sm break-words whitespace-pre-wrap">{value}</pre>
                  </div>
                {/each}
              </div>
            {/if}
          </CardContent>
        </Card>
      </TabsContent>

      <!-- YAML Tab -->
      <TabsContent value="yaml" class="space-y-4">
        <div class="h-[600px]">
          <ResourceYamlTab
            resource={configMap}
            kind="ConfigMap"
            active={r.activeTab === 'yaml'}
            onApplied={r.load}
          />
        </div>
      </TabsContent>
    </Tabs>
  {/if}
</div>
