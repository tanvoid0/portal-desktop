<!-- StatefulSet Detail Page -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { ResourceDetail } from '$lib/domains/cloud/state/resourceDetail.svelte';
  import { ResourceType } from '$lib/domains/cloud/core/types';
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

  const statefulSetName = $derived(page.params.statefulset);
  const r = new ResourceDetail(
    ResourceType.STATEFULSET,
    'StatefulSet',
    () => page.params.statefulset
  );
  const statefulSet = $derived(r.resource);

  onMount(r.load);

  function handleTabChange(newTab: string) {
    r.activeTab = newTab;
    goto(`/cloud/workloads/statefulsets/${statefulSetName}?namespace=${r.namespace}&tab=${newTab}`);
  }
</script>

<div class="space-y-6">
  <div class="flex items-center gap-4">
    <Button variant="ghost" size="sm" onclick={() => goto('/cloud/statefulsets')}>
      <ArrowLeft class="mr-2 h-4 w-4" />
      Back
    </Button>
    <Button variant="outline" size="sm" onclick={r.load}>
      <RefreshCw class="mr-2 h-4 w-4" />
      Refresh
    </Button>
  </div>

  {#if r.isLoading}
    <PageLoading message="Loading statefulset..." />
  {:else if r.error}
    <PageError title="Failed to load statefulset" message={r.error} onRetry={r.load} />
  {:else if statefulSet}
    <div class="space-y-6">
      <PageHeader
        title={statefulSet.name}
        description={`Namespace: ${statefulSet.namespace}`}
        badge={statefulSet.status}
        badgeVariant={statefulSet.status === 'running' ? 'default' : 'secondary'}
      />

      <Tabs value={r.activeTab} onValueChange={handleTabChange}>
        <TabsList>
          <TabsTrigger value="overview">Overview</TabsTrigger>
          <TabsTrigger value="yaml">YAML</TabsTrigger>
        </TabsList>

        <TabsContent value="overview" class="space-y-4">
          <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
            <InfoCard
              title="StatefulSet Information"
              fields={[
                { label: 'Desired Replicas', value: statefulSet.metadata?.desired || 0 },
                { label: 'Current Replicas', value: statefulSet.metadata?.current || 0 },
                { label: 'Ready Replicas', value: statefulSet.metadata?.ready || 0 },
                { label: 'Age', value: statefulSet.metadata?.age || 'N/A' },
              ]}
              valueClass="text-lg font-semibold"
            />

            {#if statefulSet.metadata?.labels && Object.keys(statefulSet.metadata.labels).length > 0}
              <LabelsCard items={statefulSet.metadata.labels} />
            {/if}
          </div>
        </TabsContent>

        <TabsContent value="yaml">
          <div class="h-[600px]">
            <ResourceYamlTab
              resource={statefulSet}
              kind="StatefulSet"
              active={r.activeTab === 'yaml'}
              onApplied={r.load}
            />
          </div>
        </TabsContent>
      </Tabs>
    </div>
  {/if}
</div>
