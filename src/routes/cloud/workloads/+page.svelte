<!-- Cloud Overview Dashboard -->
<script lang="ts">
  import { cloudStore, loadResources } from '$lib/domains/cloud/stores';
  import { ResourceType } from '$lib/domains/cloud/core/types';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import { goto } from '$app/navigation';
  import { Badge } from '$lib/components/ui/badge';
  import { PageError, PageLoading, PageStats } from '$lib/components/shell';
  import MetricsDisplay from '$lib/domains/cloud/components/MetricsDisplay.svelte';
  import { k8sResourceService } from '$lib/domains/cloud/services/k8sResourceService';
  import { toast } from '$lib/utils/toast';

  let isLoadingData = $state(false);
  let loadError = $state<string | null>(null);
  let hasLoadedOnce = $state(false);
  let clusterMetrics = $state<Record<string, any> | null>(null);
  let metricsLoading = $state(false);
  let wasConnected = $state(false);

  // Load once when cluster connection becomes available (not on every store refresh)
  $effect(() => {
    const connected = $cloudStore.connection.isConnected && !!$cloudStore.currentCluster;
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
      const message = error instanceof Error ? error.message : 'Failed to load cluster data';
      if (!hasLoadedOnce) {
        loadError = message;
      } else {
        toast.error('Failed to refresh cluster data', message);
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
      console.error('Failed to load metrics:', error);
    } finally {
      metricsLoading = false;
    }
  }

  function formatCPU(millicores: number | null): string {
    if (millicores === null || millicores === undefined) return 'N/A';
    if (millicores >= 1000) {
      return `${(millicores / 1000).toFixed(2)} cores`;
    }
    return `${millicores.toFixed(0)}m`;
  }

  function formatMemory(bytes: number | null): string {
    if (bytes === null || bytes === undefined) return 'N/A';
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
        running: pods.filter((p: any) => p.status === 'running').length,
        pending: pods.filter((p: any) => p.status === 'pending').length,
        failed: pods.filter((p: any) => p.status === 'failed').length,
      },
      services: {
        total: services.length,
      },
      deployments: {
        total: deployments.length,
        running: deployments.filter((d: any) => d.status === 'running').length,
      },
      statefulsets: {
        total: statefulsets.length,
        running: statefulsets.filter((ss: any) => ss.status === 'running').length,
      },
      daemonsets: {
        total: daemonsets.length,
        running: daemonsets.filter((ds: any) => ds.status === 'running').length,
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
      if (!metrics || typeof metrics !== 'object') continue;
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
        <p class="text-muted-foreground">Please connect to a cluster to view resources.</p>
      </CardContent>
    </Card>
  {:else if loadError && !hasLoadedOnce}
    <PageError title="Failed to load cluster data" message={loadError} onRetry={loadClusterData} />
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
                {$cloudStore.currentCluster.context || 'N/A'}
              </p>
            </div>
            <div>
              <p class="text-muted-foreground">Namespace</p>
              <p class="font-medium">
                {$cloudStore.selectedNamespace || 'default'}
              </p>
            </div>
            <div>
              <p class="text-muted-foreground">Server</p>
              <p class="truncate font-medium">
                {$cloudStore.currentCluster.server || 'N/A'}
              </p>
            </div>
            <div>
              <p class="text-muted-foreground">Version</p>
              <p class="font-medium">
                {$cloudStore.currentCluster.version || 'N/A'}
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
          <Button variant="outline" onclick={handleRefresh} disabled={isLoadingData}>
            Refresh
          </Button>
        </div>

        <PageStats
          stats={[
            {
              label: 'Pods',
              value: stats.pods.total,
              description: `${stats.pods.running} running`,
            },
            { label: 'Services', value: stats.services.total },
            {
              label: 'Deployments',
              value: stats.deployments.total,
              description: `${stats.deployments.running} running`,
            },
            {
              label: 'StatefulSets',
              value: stats.statefulsets.total,
              description: `${stats.statefulsets.running} running`,
            },
            {
              label: 'DaemonSets',
              value: stats.daemonsets.total,
              description: `${stats.daemonsets.running} running`,
            },
            { label: 'Jobs', value: stats.jobs.total },
            { label: 'CronJobs', value: stats.cronjobs.total },
            { label: 'ConfigMaps', value: stats.configmaps.total },
            { label: 'Secrets', value: stats.secrets.total },
            { label: 'Ingress', value: stats.ingresses.total },
            { label: 'Namespaces', value: stats.namespaces.total },
          ]}
        />
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
              onclick={() => goto('/cloud/workloads')}
            >
              <span class="mr-2 text-lg">☸️</span>
              View Workloads
            </Button>
            <Button
              variant="outline"
              class="w-full justify-start"
              onclick={() => goto('/cloud/pods')}
            >
              <span class="mr-2 text-lg">📦</span>
              View Pods
            </Button>
            <Button
              variant="outline"
              class="w-full justify-start"
              onclick={() => goto('/cloud/services')}
            >
              <span class="mr-2 text-lg">🔗</span>
              View Services
            </Button>
            <Button
              variant="outline"
              class="w-full justify-start"
              onclick={() => goto('/cloud/deployments')}
            >
              <span class="mr-2 text-lg">🚀</span>
              View Deployments
            </Button>
            <Button
              variant="outline"
              class="w-full justify-start"
              onclick={() => goto('/cloud/statefulsets')}
            >
              <span class="mr-2 text-lg">🗄️</span>
              View StatefulSets
            </Button>
            <Button
              variant="outline"
              class="w-full justify-start"
              onclick={() => goto('/cloud/daemonsets')}
            >
              <span class="mr-2 text-lg">👹</span>
              View DaemonSets
            </Button>
            <Button
              variant="outline"
              class="w-full justify-start"
              onclick={() => goto('/cloud/jobs')}
            >
              <span class="mr-2 text-lg">⚙️</span>
              View Jobs
            </Button>
            <Button
              variant="outline"
              class="w-full justify-start"
              onclick={() => goto('/cloud/cronjobs')}
            >
              <span class="mr-2 text-lg">⏰</span>
              View CronJobs
            </Button>
            <Button
              variant="outline"
              class="w-full justify-start"
              onclick={() => goto('/cloud/configmaps')}
            >
              <span class="mr-2 text-lg">⚙️</span>
              View ConfigMaps
            </Button>
            <Button
              variant="outline"
              class="w-full justify-start"
              onclick={() => goto('/cloud/secrets')}
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
