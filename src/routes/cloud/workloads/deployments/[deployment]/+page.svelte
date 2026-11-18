<!-- Deployment Detail Page -->
<script lang="ts">
  import FaSpinner from '~icons/fa6-solid/spinner';
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import { ResourceDetail } from '$lib/domains/cloud/state/resourceDetail.svelte';
  import { ResourceType } from '$lib/domains/cloud/core/types';
  import { Tabs, TabsContent, TabsList, TabsTrigger } from '$lib/components/ui/tabs';
  import { Button } from '$lib/components/ui/button';
  import RotateCcw from '~icons/fa6-solid/rotate-left';
  import { k8sResourceService } from '$lib/domains/cloud/services/k8sResourceService';
  import { PageLoading, PageError } from '$lib/components/shell';
  import ResourceYamlTab from '$lib/domains/cloud/components/ResourceYamlTab.svelte';
  import ResourceDetailHeader from '$lib/domains/cloud/components/ResourceDetailHeader.svelte';
  import { toastActions } from '$lib/utils/toast';
  import { confirmAction } from '$lib/utils/confirm';
  import DeploymentOverviewTab from '$lib/domains/cloud/components/deployment/DeploymentOverviewTab.svelte';
  import DeploymentScaleDialog from '$lib/domains/cloud/components/deployment/DeploymentScaleDialog.svelte';

  const r = new ResourceDetail(ResourceType.DEPLOYMENT, 'Deployment', () => page.params.deployment);
  const deployment = $derived(r.resource);

  // Scale dialog state
  let showScaleDialog = $state(false);
  let scaleReplicas = $state(1);
  let isScaling = $state(false);

  // Rollback state
  let isRollingBack = $state(false);

  onMount(r.load);

  function handleScale() {
    if (!deployment) return;
    scaleReplicas = Number(deployment.metadata?.desired) || 1;
    showScaleDialog = true;
  }

  async function confirmScale() {
    if (!deployment) return;

    try {
      isScaling = true;
      await k8sResourceService.scaleDeployment(
        deployment.namespace,
        deployment.name,
        scaleReplicas
      );

      toastActions.success(`Scaled ${deployment.name} to ${scaleReplicas} replicas`);
      showScaleDialog = false;
      await r.load();
    } catch (error) {
      toastActions.error(
        `Failed to scale deployment: ${error instanceof Error ? error.message : 'Unknown error'}`
      );
    } finally {
      isScaling = false;
    }
  }

  async function handleRollback() {
    if (!deployment) return;

    const confirmed = await confirmAction(
      `Are you sure you want to rollback deployment "${deployment.name}" to the previous revision?`,
      'Rollback deployment'
    );
    if (!confirmed) return;

    try {
      isRollingBack = true;
      const result = await k8sResourceService.rollbackDeployment(
        deployment.namespace,
        deployment.name
      );

      toastActions.success(result);
      await r.load();
    } catch (error) {
      toastActions.error(
        `Failed to rollback deployment: ${error instanceof Error ? error.message : 'Unknown error'}`
      );
    } finally {
      isRollingBack = false;
    }
  }
</script>

<div class="space-y-6 p-6">
  {#if r.isLoading}
    <PageLoading message="Loading deployment..." />
  {:else if r.error}
    <PageError title="Failed to load deployment" message={r.error} onRetry={r.load} />
  {:else if deployment}
    <ResourceDetailHeader
      kind="Deployment"
      name={deployment.name}
      namespace={deployment.namespace}
      backHref="/cloud/deployments"
      backLabel="Back to Deployments"
      onRefresh={r.load}
    >
      <Button variant="outline" size="sm" onclick={handleScale}>Scale</Button>
      <Button variant="outline" size="sm" onclick={handleRollback} disabled={isRollingBack}>
        {#if isRollingBack}
          <FaSpinner class="mr-2 h-4 w-4 animate-spin" />
        {:else}
          <RotateCcw class="mr-2 h-4 w-4" />
        {/if}
        Rollback
      </Button>
    </ResourceDetailHeader>

    <!-- Tabs -->
    <Tabs value={r.activeTab} onValueChange={r.setTab}>
      <TabsList>
        <TabsTrigger value="overview">Overview</TabsTrigger>
        <TabsTrigger value="yaml">YAML</TabsTrigger>
      </TabsList>

      <!-- Overview Tab -->
      <TabsContent value="overview" class="space-y-4">
        <DeploymentOverviewTab {deployment} />
      </TabsContent>

      <!-- YAML Tab -->
      <TabsContent value="yaml" class="h-[600px]">
        <ResourceYamlTab
          resource={deployment}
          kind="Deployment"
          active={r.activeTab === 'yaml'}
          onApplied={r.load}
        />
      </TabsContent>
    </Tabs>

    <DeploymentScaleDialog
      bind:open={showScaleDialog}
      name={deployment.name}
      bind:replicas={scaleReplicas}
      scaling={isScaling}
      onConfirm={confirmScale}
    />
  {/if}
</div>
