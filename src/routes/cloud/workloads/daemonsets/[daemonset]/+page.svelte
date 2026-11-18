<!-- DaemonSet Detail Page -->
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

  const daemonSetName = $derived(page.params.daemonset);
  const r = new ResourceDetail(ResourceType.DAEMONSET, 'DaemonSet', () => page.params.daemonset);
  const daemonSet = $derived(r.resource);

  onMount(r.load);

  function handleTabChange(newTab: string) {
    r.activeTab = newTab;
    goto(`/cloud/workloads/daemonsets/${daemonSetName}?namespace=${r.namespace}&tab=${newTab}`);
  }
</script>

<div class="space-y-6">
  <div class="flex items-center gap-4">
    <Button variant="ghost" size="sm" onclick={() => goto('/cloud/daemonsets')}>
      <ArrowLeft class="mr-2 h-4 w-4" />
      Back
    </Button>
    <Button variant="outline" size="sm" onclick={r.load}>
      <RefreshCw class="mr-2 h-4 w-4" />
      Refresh
    </Button>
  </div>

  {#if r.isLoading}
    <PageLoading message="Loading daemonset..." />
  {:else if r.error}
    <PageError title="Failed to load daemonset" message={r.error} onRetry={r.load} />
  {:else if daemonSet}
    <div class="space-y-6">
      <PageHeader
        title={daemonSet.name}
        description={`Namespace: ${daemonSet.namespace}`}
        badge={daemonSet.status}
        badgeVariant={daemonSet.status === 'running' ? 'default' : 'secondary'}
      />

      <Tabs value={r.activeTab} onValueChange={handleTabChange}>
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
              active={r.activeTab === 'yaml'}
              onApplied={r.load}
            />
          </div>
        </TabsContent>
      </Tabs>
    </div>
  {/if}
</div>
