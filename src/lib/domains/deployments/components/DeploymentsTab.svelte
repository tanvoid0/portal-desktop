<!-- Portal-managed deployments: stats, filters and card grid -->
<script lang="ts">
  import {
    deploymentActions,
    deployments,
    deploymentStats,
    isLoadingDeployments,
  } from '../stores/deploymentStore';
  import { logger } from '$lib/domains/shared';
  import DeploymentCard from './DeploymentCard.svelte';
  import { Card, CardContent } from '$lib/components/ui/card';
  import { Input } from '$lib/components/ui/input';
  import { PageStats } from '$lib/components/shell';
  import Select from '$lib/components/ui/select.svelte';
  import CardSkeletonGrid from '$lib/components/shared/CardSkeletonGrid.svelte';
  import Search from '~icons/fa6-solid/magnifying-glass';
  import ContainerIcon from '~icons/fa6-solid/cube';
  import Play from '~icons/fa6-solid/play';
  import Square from '~icons/fa6-solid/square';
  import Hammer from '~icons/fa6-solid/hammer';
  import type { DeploymentStatus } from '../types';

  let searchQuery = $state('');
  let selectedStatus = $state<DeploymentStatus | null>(null);

  const deploymentList = $derived($deployments);
  const stats = $derived($deploymentStats);
  const loading = $derived($isLoadingDeployments);

  function handleSearch() {
    // Filter logic would be implemented here
    // For now, we'll just log the search
    logger.info('Searching deployments', {
      context: 'DeploymentDashboard',
      query: searchQuery,
    });
  }

  function handleStatusFilter(status: DeploymentStatus | null) {
    selectedStatus = status;
    // Filter logic would be implemented here
    logger.info('Filtering by status', {
      context: 'DeploymentDashboard',
      status,
    });
  }
</script>

<PageStats
  stats={[
    { label: 'Running', value: stats.running, icon: Play },
    { label: 'Stopped', value: stats.stopped, icon: Square },
    { label: 'Building', value: stats.building, icon: Hammer },
  ]}
/>

<div class="flex flex-col gap-4 sm:flex-row">
  <div class="flex-1">
    <div class="relative">
      <Search
        class="absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2 transform text-muted-foreground"
      />
      <Input
        placeholder="Search deployments..."
        bind:value={searchQuery}
        oninput={handleSearch}
        class="pl-10"
      />
    </div>
  </div>
  <Select
    options={[
      { value: '', label: 'All Statuses' },
      { value: 'Running', label: 'Running' },
      { value: 'Stopped', label: 'Stopped' },
      { value: 'Building', label: 'Building' },
      { value: 'Error', label: 'Error' },
    ]}
    defaultValue={selectedStatus || ''}
    placeholder="Filter by status"
    onSelect={(value) => handleStatusFilter(value ? (value as DeploymentStatus) : null)}
    class="w-[200px]"
  />
</div>

{#if loading}
  <CardSkeletonGrid count={6} class="md:grid-cols-2 lg:grid-cols-3" />
{/if}

{#if !loading && deploymentList.length > 0}
  <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
    {#each deploymentList as deployment}
      <DeploymentCard
        {deployment}
        onStart={() => deploymentActions.startDeployment(deployment.id)}
        onStop={() => deploymentActions.stopDeployment(deployment.id)}
        onDelete={() => deploymentActions.deleteDeployment(deployment.id)}
      />
    {/each}
  </div>
{/if}

{#if !loading && deploymentList.length === 0}
  <Card>
    <CardContent class="flex flex-col items-center justify-center py-12">
      <ContainerIcon class="mb-4 h-12 w-12 text-muted-foreground" />
      <h3 class="mb-2 text-lg font-semibold">No Deployments Found</h3>
      <p class="mb-4 text-center text-muted-foreground">
        {searchQuery || selectedStatus
          ? 'Try adjusting your filters'
          : 'Create your first deployment to get started'}
      </p>
    </CardContent>
  </Card>
{/if}
