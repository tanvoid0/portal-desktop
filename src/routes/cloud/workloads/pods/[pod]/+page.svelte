<!-- Pod Detail Page -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { cloudStore } from '$lib/domains/cloud/stores';
  import { ResourceDetail } from '$lib/domains/cloud/state/resourceDetail.svelte';
  import { ResourceType } from '$lib/domains/cloud/core/types';
  import { Tabs, TabsContent, TabsList, TabsTrigger } from '$lib/components/ui/tabs';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import ArrowLeft from '~icons/fa6-solid/arrow-left';
  import RefreshCw from '~icons/fa6-solid/rotate';
  import Terminal from '~icons/fa6-solid/terminal';
  import Network from '~icons/fa6-solid/network-wired';
  import { podApi } from '$lib/domains/cloud/api/podApi';
  import { PageError, PageHeader, PageLoading } from '$lib/components/shell';
  import { toastActions } from '$lib/utils/toast';
  import { confirmAction } from '$lib/utils/confirm';
  import type { PortForwardInfo } from '$lib/domains/cloud/providers/gcp/GCPTypes';
  import PodOverviewTab from '$lib/domains/cloud/components/pod/PodOverviewTab.svelte';
  import PodLogsTab from '$lib/domains/cloud/components/pod/PodLogsTab.svelte';
  import PodYamlTab from '$lib/domains/cloud/components/pod/PodYamlTab.svelte';
  import PodPortForwardsTab from '$lib/domains/cloud/components/pod/PodPortForwardsTab.svelte';
  import PodExecTab from '$lib/domains/cloud/components/pod/PodExecTab.svelte';
  import PodPortForwardDialog from '$lib/domains/cloud/components/pod/PodPortForwardDialog.svelte';

  const r = new ResourceDetail(ResourceType.POD, 'Pod', () => page.params.pod);
  const pod = $derived(r.resource);

  let showPortForwardDialog = $state(false);
  let activePortForwards = $state<any[]>([]);
  let loadingPortForwards = $state(false);

  let podMetrics = $state<any | null>(null);
  let metricsLoading = $state(false);
  let metricsError = $state<string | null>(null);

  onMount(async () => {
    await r.load();
    await loadPortForwards();
    await loadMetrics();
  });

  async function loadPortForwards() {
    if (!pod || !$cloudStore.connection.isConnected) return;

    loadingPortForwards = true;
    try {
      const forwards = (await podApi.listPortForwards()) as PortForwardInfo[];
      // Filter to only show forwards for this pod
      activePortForwards = forwards.filter(
        (f) => pod && f.pod_name === pod.name && f.namespace === pod.namespace
      );
    } catch (error) {
      console.error('Failed to load port forwards:', error);
    } finally {
      loadingPortForwards = false;
    }
  }

  async function stopPortForward(id: string) {
    try {
      await podApi.stopPortForward(id);
      toastActions.success('Port forward stopped');
      await loadPortForwards();
    } catch (error) {
      toastActions.error(
        `Failed to stop port forward: ${error instanceof Error ? error.message : 'Unknown error'}`
      );
    }
  }

  function handleExec() {
    // Navigate to exec tab or open terminal
    r.setTab('exec');
  }

  async function handleDelete() {
    if (!pod) return;

    const confirmed = await confirmAction(
      `Are you sure you want to delete pod "${pod.name}"?`,
      'Delete pod'
    );
    if (!confirmed) return;

    try {
      await podApi.deletePod(pod.namespace, pod.name);
      goto('/cloud/pods');
    } catch (err) {
      toastActions.error(
        'Failed to delete pod',
        err instanceof Error ? err.message : 'Unknown error'
      );
    }
  }

  async function loadMetrics() {
    if (!pod || !$cloudStore.connection.isConnected) return;

    try {
      metricsLoading = true;
      metricsError = null;
      const metrics = await podApi.getMetrics(pod.namespace, pod.name);
      podMetrics = metrics;
    } catch (err) {
      metricsError = err instanceof Error ? err.message : 'Failed to load metrics';
      console.error('Failed to load metrics:', err);
      podMetrics = null;
    } finally {
      metricsLoading = false;
    }
  }

  const containers = $derived.by(() => {
    if (!pod || !pod.metadata?.containers) return [];
    return Array.isArray(pod.metadata.containers)
      ? pod.metadata.containers
      : [pod.metadata.containers];
  });
</script>

<div class="space-y-6 p-6">
  {#if r.isLoading}
    <PageLoading message="Loading pod..." />
  {:else if r.error}
    <PageError title="Failed to load pod" message={r.error} onRetry={r.load} />
  {:else if pod}
    <div class="flex items-center gap-2">
      <Button variant="ghost" size="sm" onclick={() => goto('/cloud/pods')}>
        <ArrowLeft class="mr-2 h-4 w-4" />
        Back
      </Button>
      <div class="min-w-0 flex-1">
        <PageHeader
          title={pod.name}
          description={`Namespace: ${pod.namespace}`}
          badge={pod.status}
          badgeVariant={pod.status === 'running' ? 'default' : 'secondary'}
        >
          {#snippet actions()}
            <Button
              variant="outline"
              size="sm"
              onclick={async () => {
                await r.load();
                await loadMetrics();
              }}
            >
              <RefreshCw class="mr-2 h-4 w-4" />
              Refresh
            </Button>
            <Button variant="outline" size="sm" onclick={() => (showPortForwardDialog = true)}>
              <Network class="mr-2 h-4 w-4" />
              Port Forward
              {#if activePortForwards.length > 0}
                <Badge variant="secondary" class="ml-2">{activePortForwards.length}</Badge>
              {/if}
            </Button>
            <Button variant="outline" size="sm" onclick={handleExec}>
              <Terminal class="mr-2 h-4 w-4" />
              Exec
            </Button>
            <Button variant="destructive" size="sm" onclick={handleDelete}>Delete</Button>
          {/snippet}
        </PageHeader>
      </div>
    </div>

    <!-- Tabs -->
    <Tabs value={r.activeTab} onValueChange={r.setTab}>
      <TabsList>
        <TabsTrigger value="overview">Overview</TabsTrigger>
        <TabsTrigger value="logs">Logs</TabsTrigger>
        <TabsTrigger value="yaml">YAML</TabsTrigger>
        <TabsTrigger value="port-forwards">
          Port Forwards
          {#if activePortForwards.length > 0}
            <Badge variant="secondary" class="ml-2">{activePortForwards.length}</Badge>
          {/if}
        </TabsTrigger>
        <TabsTrigger value="exec">Exec</TabsTrigger>
      </TabsList>

      <TabsContent value="overview" class="space-y-4">
        <PodOverviewTab {pod} {containers} {podMetrics} {metricsLoading} />
      </TabsContent>

      <TabsContent value="logs" class="space-y-4">
        <PodLogsTab {pod} active={r.activeTab === 'logs'} />
      </TabsContent>

      <TabsContent value="yaml" class="space-y-4">
        <PodYamlTab {pod} active={r.activeTab === 'yaml'} />
      </TabsContent>

      <TabsContent value="port-forwards" class="space-y-4">
        <PodPortForwardsTab
          forwards={activePortForwards}
          loading={loadingPortForwards}
          onNew={() => (showPortForwardDialog = true)}
          onStop={stopPortForward}
        />
      </TabsContent>

      <TabsContent value="exec" class="space-y-4">
        <PodExecTab {pod} {containers} />
      </TabsContent>
    </Tabs>

    <PodPortForwardDialog bind:open={showPortForwardDialog} {pod} onStarted={loadPortForwards} />
  {/if}
</div>
