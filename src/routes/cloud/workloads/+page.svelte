<!-- Cloud Overview Dashboard -->
<script lang="ts">
  import {
    cloudStore,
    loadResources,
  } from "$lib/domains/cloud/stores";
  import { ResourceType } from "$lib/domains/cloud/core/types";
  import {
    Card,
    CardContent,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { Button } from "$lib/components/ui/button";
  import { goto } from "$app/navigation";
  import { Badge } from "$lib/components/ui/badge";
  import {
    Boxes,
    Network,
    Rocket,
    Layers,
    Activity,
    Briefcase,
    Clock,
    FileText,
    Lock,
    Globe,
    Server,
  } from "@lucide/svelte";
  import { PageLoading, PageError } from "$lib/components/shell";
  import MetricsDisplay from "$lib/domains/cloud/components/MetricsDisplay.svelte";
  import { k8sResourceService } from "$lib/domains/cloud/services/k8sResourceService";
  import { toast } from "$lib/utils/toast";

  let isLoadingData = $state(false);
  let loadError = $state<string | null>(null);
  let hasLoadedOnce = $state(false);
  let clusterMetrics = $state<Record<string, any> | null>(null);
  let metricsLoading = $state(false);
  let wasConnected = $state(false);

  // Load once when cluster connection becomes available (not on every store refresh)
  $effect(() => {
    const connected =
      $cloudStore.connection.isConnected && !!$cloudStore.currentCluster;
    if (connected && !wasConnected) {
      void loadClusterData();
    }
    wasConnected = connected;
  });

  async function loadClusterData() {
    if (!$cloudStore.connection.isConnected || isLoadingData) return;

    isLoadingData = true;
    try {
      await Promise.all([
        loadResources(ResourceType.POD),
        loadResources(ResourceType.SERVICE),
        loadResources(ResourceType.DEPLOYMENT),
        loadResources(ResourceType.STATEFULSET),
        loadResources(ResourceType.DAEMONSET),
        loadResources(ResourceType.JOB),
        loadResources(ResourceType.CRONJOB),
        loadResources(ResourceType.CONFIGMAP),
        loadResources(ResourceType.SECRET),
        loadResources(ResourceType.INGRESS),
        loadResources(ResourceType.NAMESPACE),
      ]);

      // Load metrics
      await loadMetrics();
      loadError = null;
      hasLoadedOnce = true;
    } catch (error) {
      const message =
        error instanceof Error ? error.message : "Failed to load cluster data";
      if (!hasLoadedOnce) {
        loadError = message;
      } else {
        toast.error("Failed to refresh cluster data", message);
      }
    } finally {
      isLoadingData = false;
    }
  }

  async function loadMetrics() {
    if (!$cloudStore.connection.isConnected) return;

    try {
      metricsLoading = true;
      const namespace = $cloudStore.selectedNamespace || undefined;
      const metrics = await k8sResourceService.getAllPodsMetrics(namespace || null);
      clusterMetrics = metrics;
    } catch (error) {
      console.error("Failed to load metrics:", error);
    } finally {
      metricsLoading = false;
    }
  }

  function formatCPU(millicores: number | null): string {
    if (millicores === null || millicores === undefined) return "N/A";
    if (millicores >= 1000) {
      return `${(millicores / 1000).toFixed(2)} cores`;
    }
    return `${millicores.toFixed(0)}m`;
  }

  function formatMemory(bytes: number | null): string {
    if (bytes === null || bytes === undefined) return "N/A";
    const gb = bytes / (1024 * 1024 * 1024);
    if (gb >= 1) {
      return `${gb.toFixed(2)} Gi`;
    }
    const mb = bytes / (1024 * 1024);
    if (mb >= 1) {
      return `${mb.toFixed(2)} Mi`;
    }
    const kb = bytes / 1024;
    if (kb >= 1) {
      return `${kb.toFixed(2)} Ki`;
    }
    return `${bytes.toFixed(0)} B`;
  }

  // Statistics
  const stats = $derived.by(() => {
    const pods = $cloudStore.resources[ResourceType.POD];
    const services = $cloudStore.resources[ResourceType.SERVICE];
    const deployments = $cloudStore.resources[ResourceType.DEPLOYMENT];
    const statefulsets = $cloudStore.resources[ResourceType.STATEFULSET];
    const daemonsets = $cloudStore.resources[ResourceType.DAEMONSET];
    const jobs = $cloudStore.resources[ResourceType.JOB];
    const cronjobs = $cloudStore.resources[ResourceType.CRONJOB];
    const configmaps = $cloudStore.resources[ResourceType.CONFIGMAP];
    const secrets = $cloudStore.resources[ResourceType.SECRET];
    const ingresses = $cloudStore.resources[ResourceType.INGRESS];
    const namespaces = $cloudStore.resources[ResourceType.NAMESPACE];

    return {
      pods: {
        total: pods.length,
        running: pods.filter((p: any) => p.status === "running").length,
        pending: pods.filter((p: any) => p.status === "pending").length,
        failed: pods.filter((p: any) => p.status === "failed").length,
      },
      services: {
        total: services.length,
      },
      deployments: {
        total: deployments.length,
        running: deployments.filter((d: any) => d.status === "running").length,
      },
      statefulsets: {
        total: statefulsets.length,
        running: statefulsets.filter((ss: any) => ss.status === "running")
          .length,
      },
      daemonsets: {
        total: daemonsets.length,
        running: daemonsets.filter((ds: any) => ds.status === "running").length,
      },
      jobs: {
        total: jobs.length,
      },
      cronjobs: {
        total: cronjobs.length,
      },
      configmaps: {
        total: configmaps.length,
      },
      secrets: {
        total: secrets.length,
      },
      ingresses: {
        total: ingresses.length,
      },
      namespaces: {
        total: namespaces.length,
      },
    };
  });

  // Aggregate metrics
  const aggregateMetrics = $derived.by(() => {
    if (!clusterMetrics) return null;

    let totalCPU = 0;
    let totalMemory = 0;
    let podCount = 0;

    for (const [podName, metrics] of Object.entries(clusterMetrics)) {
      if (!metrics || typeof metrics !== "object") continue;
      if (metrics.cpu_usage !== null && metrics.cpu_usage !== undefined) {
        totalCPU += metrics.cpu_usage;
      }
      if (metrics.memory_usage !== null && metrics.memory_usage !== undefined) {
        totalMemory += metrics.memory_usage;
      }
      podCount++;
    }

    return {
      cpu_usage: podCount > 0 ? totalCPU : null,
      memory_usage: podCount > 0 ? totalMemory : null,
      pod_count: podCount,
    };
  });

  async function handleRefresh() {
    loadError = null;
    await loadClusterData();
  }
</script>

<div class="space-y-6">
  {#if !$cloudStore.connection.isConnected || !$cloudStore.currentCluster}
    <Card>
      <CardHeader>
        <CardTitle>Not Connected</CardTitle>
      </CardHeader>
      <CardContent>
        <p class="text-muted-foreground">
          Please connect to a cluster to view resources.
        </p>
      </CardContent>
    </Card>
  {:else if loadError && !hasLoadedOnce}
    <PageError
      title="Failed to load cluster data"
      message={loadError}
      onRetry={loadClusterData}
    />
  {:else if isLoadingData && !hasLoadedOnce}
    <PageLoading message="Loading cluster data..." />
  {:else}
    <!-- Connected Cluster View -->
    <div class="space-y-6">
      <!-- Current Cluster Info -->
      <Card>
        <CardHeader>
          <div class="flex items-center justify-between">
            <div>
              <CardTitle>Connected Cluster</CardTitle>
              <p class="mt-1 text-sm text-muted-foreground">
                {$cloudStore.currentCluster.name}
              </p>
            </div>
            <Badge variant="default" class="bg-green-500">Connected</Badge>
          </div>
        </CardHeader>
        <CardContent>
          <div class="grid grid-cols-2 gap-4 text-sm md:grid-cols-4">
            <div>
              <p class="text-muted-foreground">Context</p>
              <p class="font-medium">
                {$cloudStore.currentCluster.context || "N/A"}
              </p>
            </div>
            <div>
              <p class="text-muted-foreground">Namespace</p>
              <p class="font-medium">
                {$cloudStore.selectedNamespace || "default"}
              </p>
            </div>
            <div>
              <p class="text-muted-foreground">Server</p>
              <p class="truncate font-medium">
                {$cloudStore.currentCluster.server || "N/A"}
              </p>
            </div>
            <div>
              <p class="text-muted-foreground">Version</p>
              <p class="font-medium">
                {$cloudStore.currentCluster.version || "N/A"}
              </p>
            </div>
          </div>
        </CardContent>
      </Card>

      <!-- Metrics -->
      {#if aggregateMetrics}
        <MetricsDisplay
          metrics={aggregateMetrics}
          title="Cluster Resource Usage"
          showDetails={false}
        />
      {/if}

      <!-- Resource Statistics -->
      <div>
        <div class="mb-3 flex items-center justify-between">
          <h2 class="text-lg font-semibold">Resource Statistics</h2>
          <Button
            variant="outline"
            onclick={handleRefresh}
            disabled={isLoadingData}
          >
            Refresh
          </Button>
        </div>

        <div class="grid grid-cols-2 gap-3 md:grid-cols-4 lg:grid-cols-6">
          <Card>
            <CardHeader class="pb-1 px-4">
              <CardTitle class="flex items-center gap-2 text-sm font-medium">
                <Boxes class="h-4 w-4 text-muted-foreground" />
                Pods
              </CardTitle>
            </CardHeader>
            <CardContent class="px-4 py-3">
              <div class="text-xl font-bold leading-none">
                {stats.pods.total}
              </div>
              <p class="mt-1 text-xs text-muted-foreground">
                {stats.pods.running} running
              </p>
            </CardContent>
          </Card>

          <Card>
            <CardHeader class="pb-1 px-4">
              <CardTitle class="flex items-center gap-2 text-sm font-medium">
                <Network class="h-4 w-4 text-muted-foreground" />
                Services
              </CardTitle>
            </CardHeader>
            <CardContent class="px-4 py-3">
              <div class="text-xl font-bold leading-none">
                {stats.services.total}
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader class="pb-1 px-4">
              <CardTitle class="flex items-center gap-2 text-sm font-medium">
                <Rocket class="h-4 w-4 text-muted-foreground" />
                Deployments
              </CardTitle>
            </CardHeader>
            <CardContent class="px-4 py-3">
              <div class="text-xl font-bold leading-none">
                {stats.deployments.total}
              </div>
              <p class="mt-1 text-xs text-muted-foreground">
                {stats.deployments.running} running
              </p>
            </CardContent>
          </Card>

          <Card>
            <CardHeader class="pb-1 px-4">
              <CardTitle class="flex items-center gap-2 text-sm font-medium">
                <Layers class="h-4 w-4 text-muted-foreground" />
                StatefulSets
              </CardTitle>
            </CardHeader>
            <CardContent class="px-4 py-3">
              <div class="text-xl font-bold leading-none">
                {stats.statefulsets.total}
              </div>
              <p class="mt-1 text-xs text-muted-foreground">
                {stats.statefulsets.running} running
              </p>
            </CardContent>
          </Card>

          <Card>
            <CardHeader class="pb-1 px-4">
              <CardTitle class="flex items-center gap-2 text-sm font-medium">
                <Activity class="h-4 w-4 text-muted-foreground" />
                DaemonSets
              </CardTitle>
            </CardHeader>
            <CardContent class="px-4 py-3">
              <div class="text-xl font-bold leading-none">
                {stats.daemonsets.total}
              </div>
              <p class="mt-1 text-xs text-muted-foreground">
                {stats.daemonsets.running} running
              </p>
            </CardContent>
          </Card>

          <Card>
            <CardHeader class="pb-1 px-4">
              <CardTitle class="flex items-center gap-2 text-sm font-medium">
                <Briefcase class="h-4 w-4 text-muted-foreground" />
                Jobs
              </CardTitle>
            </CardHeader>
            <CardContent class="px-4 py-3">
              <div class="text-xl font-bold leading-none">
                {stats.jobs.total}
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader class="pb-1 px-4">
              <CardTitle class="flex items-center gap-2 text-sm font-medium">
                <Clock class="h-4 w-4 text-muted-foreground" />
                CronJobs
              </CardTitle>
            </CardHeader>
            <CardContent class="px-4 py-3">
              <div class="text-xl font-bold leading-none">
                {stats.cronjobs.total}
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader class="pb-1 px-4">
              <CardTitle class="flex items-center gap-2 text-sm font-medium">
                <FileText class="h-4 w-4 text-muted-foreground" />
                ConfigMaps
              </CardTitle>
            </CardHeader>
            <CardContent class="px-4 py-3">
              <div class="text-xl font-bold leading-none">
                {stats.configmaps.total}
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader class="pb-1 px-4">
              <CardTitle class="flex items-center gap-2 text-sm font-medium">
                <Lock class="h-4 w-4 text-muted-foreground" />
                Secrets
              </CardTitle>
            </CardHeader>
            <CardContent class="px-4 py-3">
              <div class="text-xl font-bold leading-none">
                {stats.secrets.total}
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader class="pb-1 px-4">
              <CardTitle class="flex items-center gap-2 text-sm font-medium">
                <Globe class="h-4 w-4 text-muted-foreground" />
                Ingress
              </CardTitle>
            </CardHeader>
            <CardContent class="px-4 py-3">
              <div class="text-xl font-bold leading-none">
                {stats.ingresses.total}
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader class="pb-1 px-4">
              <CardTitle class="flex items-center gap-2 text-sm font-medium">
                <Server class="h-4 w-4 text-muted-foreground" />
                Namespaces
              </CardTitle>
            </CardHeader>
            <CardContent class="px-4 py-3">
              <div class="text-xl font-bold leading-none">
                {stats.namespaces.total}
              </div>
            </CardContent>
          </Card>
        </div>
      </div>

      <!-- Quick Actions -->
      <Card>
        <CardHeader>
          <CardTitle>Quick Actions</CardTitle>
        </CardHeader>
        <CardContent>
          <div class="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-4">
            <Button
              variant="outline"
              class="w-full justify-start"
              onclick={() => goto("/cloud/workloads")}
            >
              <span class="mr-2 text-lg">☸️</span>
              View Workloads
            </Button>
            <Button
              variant="outline"
              class="w-full justify-start"
              onclick={() => goto("/cloud/workloads/pods")}
            >
              <span class="mr-2 text-lg">📦</span>
              View Pods
            </Button>
            <Button
              variant="outline"
              class="w-full justify-start"
              onclick={() => goto("/cloud/workloads/services")}
            >
              <span class="mr-2 text-lg">🔗</span>
              View Services
            </Button>
            <Button
              variant="outline"
              class="w-full justify-start"
              onclick={() => goto("/cloud/workloads/deployments")}
            >
              <span class="mr-2 text-lg">🚀</span>
              View Deployments
            </Button>
            <Button
              variant="outline"
              class="w-full justify-start"
              onclick={() => goto("/cloud/workloads/statefulsets")}
            >
              <span class="mr-2 text-lg">🗄️</span>
              View StatefulSets
            </Button>
            <Button
              variant="outline"
              class="w-full justify-start"
              onclick={() => goto("/cloud/workloads/daemonsets")}
            >
              <span class="mr-2 text-lg">👹</span>
              View DaemonSets
            </Button>
            <Button
              variant="outline"
              class="w-full justify-start"
              onclick={() => goto("/cloud/workloads/jobs")}
            >
              <span class="mr-2 text-lg">⚙️</span>
              View Jobs
            </Button>
            <Button
              variant="outline"
              class="w-full justify-start"
              onclick={() => goto("/cloud/workloads/cronjobs")}
            >
              <span class="mr-2 text-lg">⏰</span>
              View CronJobs
            </Button>
            <Button
              variant="outline"
              class="w-full justify-start"
              onclick={() => goto("/cloud/configmaps")}
            >
              <span class="mr-2 text-lg">⚙️</span>
              View ConfigMaps
            </Button>
            <Button
              variant="outline"
              class="w-full justify-start"
              onclick={() => goto("/cloud/secrets")}
            >
              <span class="mr-2 text-lg">🔐</span>
              View Secrets
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  {/if}
</div>
