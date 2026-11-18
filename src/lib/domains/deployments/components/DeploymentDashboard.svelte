<!--
	Deployment Dashboard - Main interface for managing deployments
-->

<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import {
    deploymentActions,
    isLoadingDeployments,
    deploymentError,
    isDockerOffline,
    dockerStatus,
  } from '../stores/deploymentStore';
  import { logger } from '$lib/domains/shared';
  import { toast } from '$lib/utils/toast';
  import DockerStatusBanner from './DockerStatusBanner.svelte';
  import ContainersTab from './ContainersTab.svelte';
  import DeploymentsTab from './DeploymentsTab.svelte';
  import { Button } from '$lib/components/ui/button';
  import { PageHeader } from '$lib/components/shell';
  import { Alert, AlertDescription, AlertTitle } from '$lib/components/ui/alert';
  import { Tabs, TabsContent, TabsList, TabsTrigger } from '$lib/components/ui/tabs';
  import Plus from '~icons/fa6-solid/plus';
  import Rocket from '~icons/fa6-solid/rocket';
  import AlertCircle from '~icons/fa6-solid/circle-exclamation';
  import Loader2 from '~icons/fa6-solid/spinner';
  import RefreshCw from '~icons/fa6-solid/rotate';
  import Box from '~icons/fa6-solid/box';

  type TabValue = 'deployments' | 'containers';

  let activeTab = $state<TabValue>('containers');
  let isLoadingContainers = $state(false);

  const loading = $derived($isLoadingDeployments);
  const errorMessage = $derived($deploymentError);
  const dockerOffline = $derived($isDockerOffline);
  // Docker health lives in the header badge; the banner only appears when it is down.
  const dockerBadge = $derived(
    $dockerStatus?.running
      ? $dockerStatus.version
        ? `Running · v${$dockerStatus.version}`
        : 'Running'
      : undefined
  );

  onMount(async () => {
    await loadDeployments();
    await loadContainers();
  });

  async function loadDeployments() {
    try {
      await deploymentActions.loadDeployments();
    } catch (err) {
      logger.error('Failed to load deployments', {
        context: 'DeploymentDashboard',
        error: err,
      });
      toast.error('Failed to load deployments');
    }
  }

  async function loadContainers() {
    isLoadingContainers = true;
    try {
      await deploymentActions.loadContainers();
    } catch (err) {
      logger.error('Failed to load containers', {
        context: 'DeploymentDashboard',
        error: err,
      });
      toast.error('Failed to load containers');
    } finally {
      isLoadingContainers = false;
    }
  }

  async function handleRefresh() {
    if (activeTab === 'containers') {
      await loadContainers();
      toast.success('Containers refreshed');
    } else {
      try {
        await deploymentActions.refreshDeploymentStatuses();
        toast.success('Deployment statuses refreshed');
      } catch (err) {
        logger.error('Failed to refresh deployment statuses', {
          context: 'DeploymentDashboard',
          error: err,
        });
        toast.error('Failed to refresh deployment statuses');
      }
    }
  }
</script>

<div class="space-y-6">
  <PageHeader
    title="Local Docker"
    description="Containers, resource usage, and Portal-managed deployments"
    badge={dockerBadge}
  >
    {#snippet actions()}
      <Button
        variant="outline"
        size="sm"
        onclick={handleRefresh}
        disabled={loading || isLoadingContainers}
      >
        {#if loading || isLoadingContainers}
          <Loader2 class="mr-2 h-4 w-4 animate-spin" />
        {:else}
          <RefreshCw class="mr-2 h-4 w-4" />
        {/if}
        Refresh
      </Button>
      {#if activeTab === 'deployments'}
        <Button size="sm" onclick={() => goto('/deployments/new')}>
          <Plus class="mr-2 h-4 w-4" />
          New Deployment
        </Button>
      {/if}
    {/snippet}
  </PageHeader>

  <DockerStatusBanner onReady={loadContainers} />

  {#if errorMessage && !dockerOffline}
    <Alert variant="destructive">
      <AlertCircle class="h-4 w-4" />
      <AlertTitle>Error</AlertTitle>
      <AlertDescription>
        {errorMessage}
      </AlertDescription>
    </Alert>
  {/if}

  <Tabs bind:value={activeTab} class="w-full">
    <TabsList>
      <TabsTrigger value="containers">
        <Box class="mr-2 h-4 w-4" />
        Containers
      </TabsTrigger>
      <TabsTrigger value="deployments">
        <Rocket class="mr-2 h-4 w-4" />
        Deployments
      </TabsTrigger>
    </TabsList>

    <TabsContent value="containers" class="mt-6 space-y-6">
      <ContainersTab isLoading={isLoadingContainers} onReload={loadContainers} />
    </TabsContent>

    <TabsContent value="deployments" class="mt-6 space-y-6">
      <DeploymentsTab />
    </TabsContent>
  </Tabs>
</div>
