<!-- Local Docker containers: search, status tabs and grouped workload list -->
<script lang="ts">
  import {
    containers,
    deployments,
    isDockerOffline,
    deploymentActions,
  } from '../stores/deploymentStore';
  import { deploymentService } from '../services/deploymentService';
  import { logger } from '$lib/domains/shared';
  import { toast } from '$lib/utils/toast';
  import { confirmAction } from '$lib/utils/confirm';
  import ContainerOverview from './ContainerOverview.svelte';
  import WorkloadList from './WorkloadList.svelte';
  import { Card, CardContent } from '$lib/components/ui/card';
  import { Input } from '$lib/components/ui/input';
  import { Badge } from '$lib/components/ui/badge';
  import { Tabs, TabsList, TabsTrigger } from '$lib/components/ui/tabs';
  import CardSkeletonGrid from '$lib/components/shared/CardSkeletonGrid.svelte';
  import Search from '~icons/fa6-solid/magnifying-glass';
  import ContainerIcon from '~icons/fa6-solid/cube';
  import Play from '~icons/fa6-solid/play';
  import Square from '~icons/fa6-solid/square';
  import Box from '~icons/fa6-solid/box';
  import type { DockerContainer } from '../types';
  import type { GroupByMode } from '../utils/workloadGrouping';
  import { groupWorkloads } from '../utils/workloadGrouping';
  import { containerStatusGroup } from '../utils/format';

  type ContainerStatusTab = 'running' | 'stopped' | 'other';

  interface Props {
    isLoading: boolean;
    onReload: () => Promise<void>;
  }
  let { isLoading, onReload }: Props = $props();

  let containerStatusTab = $state<ContainerStatusTab>('running');
  let containerSearchQuery = $state('');
  let groupByMode = $state<GroupByMode>('stack');

  const containerList = $derived($containers);
  const deploymentList = $derived($deployments);
  const dockerOffline = $derived($isDockerOffline);

  /** Run a container/deployment action, toast the result, then reload containers. */
  async function run(
    action: () => Promise<unknown>,
    verb: string,
    noun: 'container' | 'deployment'
  ) {
    try {
      await action();
      toast.success(`${noun[0].toUpperCase()}${noun.slice(1)} ${verb}`);
      await onReload();
    } catch (err) {
      const op = verb === 'started' ? 'start' : verb === 'stopped' ? 'stop' : 'remove';
      logger.error(`Failed to ${op} ${noun}`, {
        context: 'DeploymentDashboard',
        error: err,
      });
      toast.error(err instanceof Error ? err.message : `Failed to ${op} ${noun}`);
    }
  }

  const handleContainerStart = (id: string) =>
    run(() => deploymentService.startContainer(id), 'started', 'container');
  const handleContainerStop = (id: string) =>
    run(() => deploymentService.stopContainer(id), 'stopped', 'container');
  const handleDeploymentStart = (id: string) =>
    run(() => deploymentActions.startDeployment(id), 'started', 'deployment');
  const handleDeploymentStop = (id: string) =>
    run(() => deploymentActions.stopDeployment(id), 'stopped', 'deployment');

  async function handleContainerRemove(containerId: string) {
    const confirmed = await confirmAction(
      'Are you sure you want to remove this container? This action cannot be undone.',
      'Remove container'
    );
    if (!confirmed) return;
    await run(() => deploymentService.removeContainer(containerId), 'removed', 'container');
  }

  function filterContainers(list: DockerContainer[]): DockerContainer[] {
    if (!containerSearchQuery) return list;
    const query = containerSearchQuery.toLowerCase();
    return list.filter(
      (c) =>
        c.name?.toLowerCase().includes(query) ||
        c.image?.toLowerCase().includes(query) ||
        c.id?.toLowerCase().includes(query)
    );
  }

  function containersForStatus(
    list: DockerContainer[],
    status: ContainerStatusTab
  ): DockerContainer[] {
    return filterContainers(list)
      .filter((c) => containerStatusGroup(c.status) === status)
      .sort((a, b) => a.name.localeCompare(b.name));
  }

  const containerStatusCounts = $derived({
    running: containersForStatus(containerList, 'running').length,
    stopped: containersForStatus(containerList, 'stopped').length,
    other: containersForStatus(containerList, 'other').length,
  });

  const visibleContainers = $derived(containersForStatus(containerList, containerStatusTab));

  const visibleWorkloadGroups = $derived(
    groupWorkloads(containerList, deploymentList, groupByMode, {
      statusFilter: containerStatusTab,
      searchQuery: containerSearchQuery,
    })
  );

  const hasVisibleWorkloads = $derived(
    groupByMode === 'flat' ? visibleContainers.length > 0 : visibleWorkloadGroups.length > 0
  );

  const containerStatusLabels: Record<ContainerStatusTab, string> = {
    running: 'Running',
    stopped: 'Stopped',
    other: 'Other',
  };
</script>

<ContainerOverview containers={containerList} />

<div class="flex flex-col gap-4">
  <div class="relative">
    <Search
      class="absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2 transform text-muted-foreground"
    />
    <Input
      placeholder="Search containers by name, image, or ID..."
      bind:value={containerSearchQuery}
      class="pl-10"
    />
  </div>

  {#if !isLoading && containerList.length > 0}
    <Tabs bind:value={containerStatusTab} class="w-full">
      <TabsList>
        <TabsTrigger value="running" class="gap-2">
          <Play class="h-4 w-4" />
          Running
          <Badge variant="outline" class="tabular-nums">{containerStatusCounts.running}</Badge>
        </TabsTrigger>
        <TabsTrigger value="stopped" class="gap-2">
          <Square class="h-4 w-4" />
          Stopped
          <Badge variant="outline" class="tabular-nums">{containerStatusCounts.stopped}</Badge>
        </TabsTrigger>
        <TabsTrigger value="other" class="gap-2">
          <Box class="h-4 w-4" />
          Other
          <Badge variant="outline" class="tabular-nums">{containerStatusCounts.other}</Badge>
        </TabsTrigger>
      </TabsList>
    </Tabs>
  {/if}
</div>

{#if isLoading}
  <CardSkeletonGrid count={4} />
{/if}

{#if !isLoading && hasVisibleWorkloads}
  <WorkloadList
    containers={containerList}
    deployments={deploymentList}
    groupBy={groupByMode}
    statusFilter={containerStatusTab}
    searchQuery={containerSearchQuery}
    onGroupByChange={(mode) => (groupByMode = mode)}
    onStart={handleContainerStart}
    onStop={handleContainerStop}
    onRemove={handleContainerRemove}
    onStartDeployment={handleDeploymentStart}
    onStopDeployment={handleDeploymentStop}
  />
{/if}

{#if !isLoading && containerList.length === 0}
  <Card>
    <CardContent class="flex flex-col items-center justify-center py-12">
      <ContainerIcon class="mb-4 h-12 w-12 text-muted-foreground" />
      <h3 class="mb-2 text-lg font-semibold">
        {dockerOffline ? 'Docker Is Offline' : 'No Containers Found'}
      </h3>
      <p class="mb-4 text-center text-muted-foreground">
        {#if dockerOffline}
          Start Docker Desktop using the banner above, then retry.
        {:else if containerSearchQuery}
          Try adjusting your search
        {:else}
          No Docker containers are running or stopped
        {/if}
      </p>
    </CardContent>
  </Card>
{:else if !isLoading && containerList.length > 0 && !hasVisibleWorkloads}
  <Card>
    <CardContent class="flex flex-col items-center justify-center py-8">
      <p class="text-muted-foreground">
        {#if containerSearchQuery}
          No {containerStatusLabels[containerStatusTab].toLowerCase()} containers match "{containerSearchQuery}"
        {:else}
          No {containerStatusLabels[containerStatusTab].toLowerCase()} containers
        {/if}
      </p>
    </CardContent>
  </Card>
{/if}
