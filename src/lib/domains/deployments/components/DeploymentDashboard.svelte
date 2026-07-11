<!--
	Deployment Dashboard - Main interface for managing deployments
-->

<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import {
    deploymentActions,
    deployments,
    deploymentStats,
    isLoadingDeployments,
    deploymentError,
    containers,
    isDockerOffline,
  } from "../stores/deploymentStore";
  import { deploymentService } from "../services/deploymentService";
  import { logger } from "$lib/domains/shared";
  import { toast } from "$lib/utils/toast";
  import { confirmAction } from "$lib/utils/confirm";
  import DeploymentCard from "./DeploymentCard.svelte";
  import ContainerOverview from "./ContainerOverview.svelte";
  import DockerStatusBanner from "./DockerStatusBanner.svelte";
  import WorkloadList from "./WorkloadList.svelte";
  import { Button } from "$lib/components/ui/button";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { Input } from "$lib/components/ui/input";
  import Select from "$lib/components/ui/select.svelte";
  import { Badge } from "$lib/components/ui/badge";
  import {
    Alert,
    AlertDescription,
    AlertTitle,
  } from "$lib/components/ui/alert";
  import {
    Tabs,
    TabsContent,
    TabsList,
    TabsTrigger,
  } from "$lib/components/ui/tabs";
  import {
    Plus,
    Search,
    Rocket,
    Container as ContainerIcon,
    Play,
    Square,
    AlertCircle,
    Loader2,
    RefreshCw,
    Box,
  } from "@lucide/svelte";
  import type { DeploymentStatus, DockerContainer } from "../types";
  import type { GroupByMode } from "../utils/workloadGrouping";
  import { groupWorkloads } from "../utils/workloadGrouping";
  import { containerStatusGroup } from "../utils/format";

  type TabValue = "deployments" | "containers";
  type ContainerStatusTab = "running" | "stopped" | "other";

  let activeTab = $state<TabValue>("containers");
  let containerStatusTab = $state<ContainerStatusTab>("running");
  let searchQuery = $state("");
  let selectedStatus = $state<DeploymentStatus | null>(null);
  let containerSearchQuery = $state("");
  let groupByMode = $state<GroupByMode>("stack");
  let isLoadingContainers = $state(false);

  // Reactive stores
  let deploymentList = $derived($deployments);
  let stats = $derived($deploymentStats);
  let loading = $derived($isLoadingDeployments);
  let errorMessage = $derived($deploymentError);
  let dockerOffline = $derived($isDockerOffline);

  // Reactive stores
  let containerList = $derived($containers);

  onMount(async () => {
    await loadDeployments();
    await loadContainers();
  });

  async function loadDeployments() {
    try {
      await deploymentActions.loadDeployments();
    } catch (err) {
      logger.error("Failed to load deployments", {
        context: "DeploymentDashboard",
        error: err,
      });
      toast.error("Failed to load deployments");
    }
  }

  async function loadContainers() {
    isLoadingContainers = true;
    try {
      await deploymentActions.loadContainers();
    } catch (err) {
      logger.error("Failed to load containers", {
        context: "DeploymentDashboard",
        error: err,
      });
      toast.error("Failed to load containers");
    } finally {
      isLoadingContainers = false;
    }
  }

  function handleSearch() {
    // Filter logic would be implemented here
    // For now, we'll just log the search
    logger.info("Searching deployments", {
      context: "DeploymentDashboard",
      query: searchQuery,
    });
  }

  function handleStatusFilter(status: DeploymentStatus | null) {
    selectedStatus = status;
    // Filter logic would be implemented here
    logger.info("Filtering by status", {
      context: "DeploymentDashboard",
      status,
    });
  }

  function handleCreateDeployment() {
    goto("/deployments/new");
  }

  async function handleRefresh() {
    if (activeTab === "containers") {
      await loadContainers();
      toast.success("Containers refreshed");
    } else {
      try {
        await deploymentActions.refreshDeploymentStatuses();
        toast.success("Deployment statuses refreshed");
      } catch (err) {
        logger.error("Failed to refresh deployment statuses", {
          context: "DeploymentDashboard",
          error: err,
        });
        toast.error("Failed to refresh deployment statuses");
      }
    }
  }

  async function handleContainerStart(containerId: string) {
    try {
      await deploymentService.startContainer(containerId);
      toast.success("Container started");
      await loadContainers();
    } catch (err) {
      logger.error("Failed to start container", {
        context: "DeploymentDashboard",
        error: err,
      });
      toast.error(
        err instanceof Error ? err.message : "Failed to start container",
      );
    }
  }

  async function handleContainerStop(containerId: string) {
    try {
      await deploymentService.stopContainer(containerId);
      toast.success("Container stopped");
      await loadContainers();
    } catch (err) {
      logger.error("Failed to stop container", {
        context: "DeploymentDashboard",
        error: err,
      });
      toast.error(
        err instanceof Error ? err.message : "Failed to stop container",
      );
    }
  }

  async function handleContainerRemove(containerId: string) {
    const confirmed = await confirmAction(
      "Are you sure you want to remove this container? This action cannot be undone.",
      "Remove container",
    );
    if (!confirmed) return;
    try {
      await deploymentService.removeContainer(containerId);
      toast.success("Container removed");
      await loadContainers();
    } catch (err) {
      logger.error("Failed to remove container", {
        context: "DeploymentDashboard",
        error: err,
      });
      toast.error(
        err instanceof Error ? err.message : "Failed to remove container",
      );
    }
  }

  function filterContainers(list: DockerContainer[]): DockerContainer[] {
    if (!containerSearchQuery) return list;
    const query = containerSearchQuery.toLowerCase();
    return list.filter(
      (c) =>
        c.name?.toLowerCase().includes(query) ||
        c.image?.toLowerCase().includes(query) ||
        c.id?.toLowerCase().includes(query),
    );
  }

  function containersForStatus(
    list: DockerContainer[],
    status: ContainerStatusTab,
  ): DockerContainer[] {
    return filterContainers(list)
      .filter((c) => containerStatusGroup(c.status) === status)
      .sort((a, b) => a.name.localeCompare(b.name));
  }

  let containerStatusCounts = $derived({
    running: containersForStatus(containerList, "running").length,
    stopped: containersForStatus(containerList, "stopped").length,
    other: containersForStatus(containerList, "other").length,
  });

  let visibleContainers = $derived(
    containersForStatus(containerList, containerStatusTab),
  );

  let visibleWorkloadGroups = $derived(
    groupWorkloads(containerList, deploymentList, groupByMode, {
      statusFilter: containerStatusTab,
      searchQuery: containerSearchQuery,
    }),
  );

  let hasVisibleWorkloads = $derived(
    groupByMode === "flat"
      ? visibleContainers.length > 0
      : visibleWorkloadGroups.length > 0,
  );

  const containerStatusLabels: Record<ContainerStatusTab, string> = {
    running: "Running",
    stopped: "Stopped",
    other: "Other",
  };

  async function handleDeploymentStart(deploymentId: string) {
    try {
      await deploymentActions.startDeployment(deploymentId);
      toast.success("Deployment started");
      await loadContainers();
    } catch (err) {
      logger.error("Failed to start deployment", {
        context: "DeploymentDashboard",
        error: err,
      });
      toast.error(
        err instanceof Error ? err.message : "Failed to start deployment",
      );
    }
  }

  async function handleDeploymentStop(deploymentId: string) {
    try {
      await deploymentActions.stopDeployment(deploymentId);
      toast.success("Deployment stopped");
      await loadContainers();
    } catch (err) {
      logger.error("Failed to stop deployment", {
        context: "DeploymentDashboard",
        error: err,
      });
      toast.error(
        err instanceof Error ? err.message : "Failed to stop deployment",
      );
    }
  }

  function getStatusColor(status: DeploymentStatus): string {
    return deploymentService.getStatusColor(status);
  }

  function getStatusIcon(status: DeploymentStatus): string {
    return deploymentService.getStatusIcon(status);
  }
</script>

<div class="space-y-6">
  <!-- Header -->
  <div class="flex items-center justify-between">
    <div>
      <h1 class="flex items-center gap-2 text-3xl font-bold tracking-tight">
        <ContainerIcon class="h-8 w-8" />
        Local Docker
      </h1>
      <p class="text-muted-foreground">
        Containers, resource usage, and Portal-managed deployments
      </p>
    </div>
    <div class="flex gap-2">
      <Button
        variant="outline"
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
      {#if activeTab === "deployments"}
        <Button onclick={handleCreateDeployment}>
          <Plus class="mr-2 h-4 w-4" />
          New Deployment
        </Button>
      {/if}
    </div>
  </div>

  <DockerStatusBanner onReady={loadContainers} />

  <!-- Error Alert -->
  {#if errorMessage && !dockerOffline}
    <Alert variant="destructive">
      <AlertCircle class="h-4 w-4" />
      <AlertTitle>Error</AlertTitle>
      <AlertDescription>
        {errorMessage}
      </AlertDescription>
    </Alert>
  {/if}

  <!-- Tabs -->
  <Tabs bind:value={activeTab} class="w-full">
    <TabsList class="grid w-full grid-cols-2">
      <TabsTrigger value="containers">
        <Box class="mr-2 h-4 w-4" />
        Containers
      </TabsTrigger>
      <TabsTrigger value="deployments">
        <Rocket class="mr-2 h-4 w-4" />
        Deployments
      </TabsTrigger>
    </TabsList>

    <!-- Containers Tab -->
    <TabsContent value="containers" class="mt-6 space-y-6">
      <ContainerOverview containers={containerList} />

      <!-- Container Search + Status Tabs -->
      <div class="flex flex-col gap-4">
        <div class="relative">
          <Search
            class="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 transform text-muted-foreground"
          />
          <Input
            placeholder="Search containers by name, image, or ID..."
            bind:value={containerSearchQuery}
            class="pl-10"
          />
        </div>

        {#if !isLoadingContainers && containerList.length > 0}
          <Tabs bind:value={containerStatusTab} class="w-full">
            <TabsList class="grid w-full grid-cols-3 sm:w-fit">
              <TabsTrigger value="running" class="gap-2">
                <Play class="h-4 w-4" />
                Running
                <Badge variant="outline" class="tabular-nums">
                  {containerStatusCounts.running}
                </Badge>
              </TabsTrigger>
              <TabsTrigger value="stopped" class="gap-2">
                <Square class="h-4 w-4" />
                Stopped
                <Badge variant="outline" class="tabular-nums">
                  {containerStatusCounts.stopped}
                </Badge>
              </TabsTrigger>
              <TabsTrigger value="other" class="gap-2">
                <Box class="h-4 w-4" />
                Other
                <Badge variant="outline" class="tabular-nums">
                  {containerStatusCounts.other}
                </Badge>
              </TabsTrigger>
            </TabsList>
          </Tabs>
        {/if}
      </div>

      <!-- Loading State -->
      {#if isLoadingContainers}
        <div class="grid gap-4 md:grid-cols-2">
          {#each Array(4) as _}
            <Card>
              <CardHeader>
                <div class="h-4 animate-pulse rounded bg-muted"></div>
                <div class="h-3 w-2/3 animate-pulse rounded bg-muted"></div>
              </CardHeader>
              <CardContent>
                <div class="space-y-2">
                  <div class="h-3 animate-pulse rounded bg-muted"></div>
                  <div class="h-3 w-1/2 animate-pulse rounded bg-muted"></div>
                </div>
              </CardContent>
            </Card>
          {/each}
        </div>
      {/if}

      <!-- Container List -->
      {#if !isLoadingContainers && hasVisibleWorkloads}
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

      <!-- Empty State -->
      {#if !isLoadingContainers && containerList.length === 0}
        <Card>
          <CardContent class="flex flex-col items-center justify-center py-12">
            <ContainerIcon class="mb-4 h-12 w-12 text-muted-foreground" />
            <h3 class="mb-2 text-lg font-semibold">
              {dockerOffline ? "Docker Is Offline" : "No Containers Found"}
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
      {:else if !isLoadingContainers && containerList.length > 0 && !hasVisibleWorkloads}
        <Card>
          <CardContent class="flex flex-col items-center justify-center py-8">
            <p class="text-muted-foreground">
              {#if containerSearchQuery}
                No {containerStatusLabels[
                  containerStatusTab
                ].toLowerCase()} containers match "{containerSearchQuery}"
              {:else}
                No {containerStatusLabels[
                  containerStatusTab
                ].toLowerCase()} containers
              {/if}
            </p>
          </CardContent>
        </Card>
      {/if}
    </TabsContent>

    <!-- Deployments Tab -->
    <TabsContent value="deployments" class="mt-6 space-y-6">
      <!-- Stats Cards -->
      <div class="grid gap-4 md:grid-cols-4">
        <Card>
          <CardHeader
            class="flex flex-row items-center justify-between space-y-0 pb-2"
          >
            <CardTitle class="text-sm font-medium">Total Deployments</CardTitle>
            <ContainerIcon class="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div class="text-2xl font-bold">{stats.total}</div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader
            class="flex flex-row items-center justify-between space-y-0 pb-2"
          >
            <CardTitle class="text-sm font-medium">Running</CardTitle>
            <Badge variant="default" class="bg-green-100 text-green-800">
              <Play class="mr-1 h-3 w-3" />
              {stats.running}
            </Badge>
          </CardHeader>
          <CardContent>
            <div class="text-2xl font-bold text-green-600">{stats.running}</div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader
            class="flex flex-row items-center justify-between space-y-0 pb-2"
          >
            <CardTitle class="text-sm font-medium">Stopped</CardTitle>
            <Badge variant="outline">
              <Square class="mr-1 h-3 w-3" />
              {stats.stopped}
            </Badge>
          </CardHeader>
          <CardContent>
            <div class="text-2xl font-bold text-gray-600">{stats.stopped}</div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader
            class="flex flex-row items-center justify-between space-y-0 pb-2"
          >
            <CardTitle class="text-sm font-medium">Building</CardTitle>
            <Badge variant="outline" class="bg-yellow-100 text-yellow-800">
              🔨 {stats.building}
            </Badge>
          </CardHeader>
          <CardContent>
            <div class="text-2xl font-bold text-yellow-600">
              {stats.building}
            </div>
          </CardContent>
        </Card>
      </div>

      <!-- Filters -->
      <div class="flex flex-col gap-4 sm:flex-row">
        <div class="flex-1">
          <div class="relative">
            <Search
              class="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 transform text-muted-foreground"
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
            { value: "", label: "All Statuses" },
            { value: "Running", label: "Running" },
            { value: "Stopped", label: "Stopped" },
            { value: "Building", label: "Building" },
            { value: "Error", label: "Error" },
          ]}
          defaultValue={selectedStatus || ""}
          placeholder="Filter by status"
          onSelect={(value) =>
            handleStatusFilter(value ? (value as DeploymentStatus) : null)}
          class="w-[200px]"
        />
      </div>

      <!-- Loading State -->
      {#if loading}
        <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
          {#each Array(6) as _}
            <Card>
              <CardHeader>
                <div class="h-4 animate-pulse rounded bg-muted"></div>
                <div class="h-3 w-2/3 animate-pulse rounded bg-muted"></div>
              </CardHeader>
              <CardContent>
                <div class="space-y-2">
                  <div class="h-3 animate-pulse rounded bg-muted"></div>
                  <div class="h-3 w-1/2 animate-pulse rounded bg-muted"></div>
                </div>
              </CardContent>
            </Card>
          {/each}
        </div>
      {/if}

      <!-- Deployments Grid -->
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

      <!-- Empty State -->
      {#if !loading && deploymentList.length === 0}
        <Card>
          <CardContent class="flex flex-col items-center justify-center py-12">
            <ContainerIcon class="mb-4 h-12 w-12 text-muted-foreground" />
            <h3 class="mb-2 text-lg font-semibold">No Deployments Found</h3>
            <p class="mb-4 text-center text-muted-foreground">
              {searchQuery || selectedStatus
                ? "Try adjusting your filters"
                : "Create your first deployment to get started"}
            </p>
            {#if !searchQuery && !selectedStatus}
              <Button onclick={handleCreateDeployment}>
                <Plus class="mr-2 h-4 w-4" />
                New Deployment
              </Button>
            {/if}
          </CardContent>
        </Card>
      {/if}
    </TabsContent>
  </Tabs>
</div>
