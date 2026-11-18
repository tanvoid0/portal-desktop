<!-- DaemonSet Detail Page -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { cloudStore, loadResources } from '$lib/domains/cloud/stores';
  import { ResourceType, type ICloudResource } from '$lib/domains/cloud/core/types';
  import { Tabs, TabsContent, TabsList, TabsTrigger } from '$lib/components/ui/tabs';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import ArrowLeft from '~icons/fa6-solid/arrow-left';
  import RefreshCw from '~icons/fa6-solid/rotate';
  import { k8sResourceService } from '$lib/domains/cloud/services/k8sResourceService';
  import Loading from '$lib/components/ui/loading.svelte';
  import { PageError, PageHeader, PageLoading } from '$lib/components/shell';
  import InfoCard from '$lib/domains/cloud/components/InfoCard.svelte';
  import ResourceYamlTab from '$lib/domains/cloud/components/ResourceYamlTab.svelte';
  import LabelsCard from '$lib/domains/cloud/components/LabelsCard.svelte';

  const daemonSetName = $derived(page.params.daemonset);
  const namespace = $derived(
    page.url.searchParams.get('namespace') || $cloudStore.selectedNamespace || 'default'
  );
  const tabParam = $derived(page.url.searchParams.get('tab') || 'overview');

  let activeTab = $state('overview');

  // Sync activeTab with tabParam when it changes
  $effect(() => {
    activeTab = tabParam;
  });
  let daemonSet = $state<ICloudResource | null>(null);
  let isLoading = $state(true);
  let error = $state<string | null>(null);

  onMount(async () => {
    await loadDaemonSet();
  });

  async function loadDaemonSet() {
    if (!daemonSetName || !$cloudStore.connection.isConnected) {
      error = 'DaemonSet name or connection required';
      isLoading = false;
      return;
    }

    try {
      isLoading = true;
      error = null;

      await loadResources(ResourceType.DAEMONSET, namespace);
      const resources = $cloudStore.resources[ResourceType.DAEMONSET] || [];
      daemonSet = resources.find((ds) => ds.name === daemonSetName) || null;

      if (!daemonSet) {
        error = `DaemonSet "${daemonSetName}" not found in namespace "${namespace}".`;
      }
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to load DaemonSet';
      console.error('Failed to load DaemonSet:', err);
    } finally {
      isLoading = false;
    }
  }

  function handleTabChange(newTab: string) {
    activeTab = newTab;
    goto(`/cloud/workloads/daemonsets/${daemonSetName}?namespace=${namespace}&tab=${newTab}`);
  }
</script>

<div class="space-y-6">
  <div class="flex items-center gap-4">
    <Button variant="ghost" size="sm" onclick={() => goto('/cloud/daemonsets')}>
      <ArrowLeft class="mr-2 h-4 w-4" />
      Back
    </Button>
    <Button variant="outline" size="sm" onclick={loadDaemonSet}>
      <RefreshCw class="mr-2 h-4 w-4" />
      Refresh
    </Button>
  </div>

  {#if isLoading}
    <PageLoading message="Loading daemonset..." />
  {:else if error}
    <PageError title="Failed to load daemonset" message={error} onRetry={loadDaemonSet} />
  {:else if daemonSet}
    <div class="space-y-6">
      <PageHeader
        title={daemonSet.name}
        description={`Namespace: ${daemonSet.namespace}`}
        badge={daemonSet.status}
        badgeVariant={daemonSet.status === 'running' ? 'default' : 'secondary'}
      />

      <Tabs value={activeTab} onValueChange={handleTabChange}>
        <TabsList>
          <TabsTrigger value="overview">Overview</TabsTrigger>
          <TabsTrigger value="yaml">YAML</TabsTrigger>
        </TabsList>

        <TabsContent value="overview" class="space-y-4">
          <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
            <InfoCard
              title="DaemonSet Information"
              fields={[
                { label: 'Desired', value: daemonSet.metadata?.desired || 0 },
                { label: 'Current', value: daemonSet.metadata?.current || 0 },
                { label: 'Ready', value: daemonSet.metadata?.ready || 0 },
                { label: 'Up to Date', value: daemonSet.metadata?.up_to_date || 0 },
                { label: 'Available', value: daemonSet.metadata?.available || 0 },
                { label: 'Age', value: daemonSet.metadata?.age || 'N/A' },
              ]}
              valueClass="text-lg font-semibold"
            />

            {#if daemonSet.metadata?.labels && Object.keys(daemonSet.metadata.labels).length > 0}
              <LabelsCard items={daemonSet.metadata.labels} />
            {/if}
          </div>
        </TabsContent>

        <TabsContent value="yaml">
          <div class="h-[600px]">
            <ResourceYamlTab
              resource={daemonSet}
              kind="DaemonSet"
              active={activeTab === 'yaml'}
              onApplied={loadDaemonSet}
            />
          </div>
        </TabsContent>
      </Tabs>
    </div>
  {/if}
</div>
