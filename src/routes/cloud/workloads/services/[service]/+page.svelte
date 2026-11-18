<!-- Service Detail Page -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import { replaceState } from '$app/navigation';
  import { cloudStore, loadResources } from '$lib/domains/cloud/stores';
  import { ResourceType, type ICloudResource } from '$lib/domains/cloud/core/types';
  import { Tabs, TabsContent, TabsList, TabsTrigger } from '$lib/components/ui/tabs';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import { k8sResourceService } from '$lib/domains/cloud/services/k8sResourceService';
  import Loading from '$lib/components/ui/loading.svelte';
  import { PageLoading, PageError } from '$lib/components/shell';
  import InfoCard from '$lib/domains/cloud/components/InfoCard.svelte';
  import ResourceYamlTab from '$lib/domains/cloud/components/ResourceYamlTab.svelte';
  import ResourceDetailHeader from '$lib/domains/cloud/components/ResourceDetailHeader.svelte';

  const serviceName = $derived(page.params.service);
  const namespace = $derived(
    page.url.searchParams.get('namespace') || $cloudStore.selectedNamespace || 'default'
  );
  const tabParam = $derived(page.url.searchParams.get('tab') || 'overview');

  let activeTab = $state('overview');

  // Sync activeTab with tabParam when it changes
  $effect(() => {
    activeTab = tabParam;
  });
  let service = $state<ICloudResource | null>(null);
  let isLoading = $state(true);
  let error = $state<string | null>(null);

  onMount(async () => {
    await loadService();
  });

  async function loadService() {
    if (!serviceName || !$cloudStore.connection.isConnected) {
      error = 'Service name or connection required';
      isLoading = false;
      return;
    }

    try {
      isLoading = true;
      error = null;

      await loadResources(ResourceType.SERVICE, namespace);
      const resources = $cloudStore.resources[ResourceType.SERVICE] || [];
      service = resources.find((s) => s.name === serviceName) || null;

      if (!service) {
        error = `Service "${serviceName}" not found in namespace "${namespace}".`;
      }
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to load service';
      console.error('Failed to load service:', err);
    } finally {
      isLoading = false;
    }
  }

  function handleTabChange(tab: string) {
    activeTab = tab;
    const url = new URL(page.url.href);
    url.searchParams.set('tab', tab);
    replaceState(url, {});
  }

  const ports = $derived.by(() => {
    if (!service || !service.metadata?.ports) return [];
    const portArray = Array.isArray(service.metadata.ports) ? service.metadata.ports : [];
    return portArray.map((p: any) => ({
      name: typeof p === 'object' ? p.name || 'Unnamed' : 'Unnamed',
      port: typeof p === 'object' ? p.port || p.hostPort || 0 : p,
      targetPort: typeof p === 'object' ? p.targetPort || p.containerPort || p.port || 0 : p,
      protocol: typeof p === 'object' ? p.protocol || 'TCP' : 'TCP',
    }));
  });

  const selector = $derived.by(() => {
    if (!service || !service.metadata?.selector) return {};
    return service.metadata.selector || {};
  });
</script>

<div class="space-y-6 p-6">
  {#if isLoading}
    <PageLoading message="Loading service..." />
  {:else if error}
    <PageError title="Failed to load service" message={error} onRetry={loadService} />
  {:else if service}
    <ResourceDetailHeader
      kind="Service"
      name={service.name}
      namespace={service.namespace}
      backHref="/cloud/services"
      backLabel="Back to Services"
      onRefresh={loadService}
    />

    <!-- Tabs -->
    <Tabs value={activeTab} onValueChange={handleTabChange}>
      <TabsList>
        <TabsTrigger value="overview">Overview</TabsTrigger>
        <TabsTrigger value="yaml">YAML</TabsTrigger>
      </TabsList>

      <!-- Overview Tab -->
      <TabsContent value="overview" class="space-y-4">
        <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
          <InfoCard
            title="Service Information"
            fields={[
              { label: 'Cluster IP', value: service.metadata?.cluster_ip || 'None' },
              { label: 'External IP', value: service.metadata?.external_ip || 'None' },
              { label: 'Age', value: service.metadata?.age || 'N/A' },
            ]}
            status={{ text: service.status }}
          />

          <Card>
            <CardHeader>
              <CardTitle>Ports</CardTitle>
            </CardHeader>
            <CardContent>
              {#if ports.length > 0}
                <div class="space-y-2">
                  {#each ports as port (port.port)}
                    <div class="rounded-md border p-3">
                      <div class="flex items-center justify-between">
                        <div>
                          <p class="font-medium">{port.name || 'Unnamed'}</p>
                          <p class="text-sm text-muted-foreground">
                            Port: {port.port} → {port.targetPort || port.port}
                          </p>
                          <p class="text-xs text-muted-foreground">
                            Protocol: {port.protocol || 'TCP'}
                          </p>
                        </div>
                      </div>
                    </div>
                  {/each}
                </div>
              {:else}
                <p class="text-muted-foreground">No ports configured</p>
              {/if}
            </CardContent>
          </Card>
        </div>

        <Card>
          <CardHeader>
            <CardTitle>Selector</CardTitle>
          </CardHeader>
          <CardContent>
            {#if Object.keys(selector).length > 0}
              <div class="flex flex-wrap gap-2">
                {#each Object.entries(selector) as [key, value]}
                  <Badge variant="outline">{key}={value}</Badge>
                {/each}
              </div>
            {:else}
              <p class="text-muted-foreground">No selector configured</p>
            {/if}
          </CardContent>
        </Card>
      </TabsContent>

      <!-- YAML Tab -->
      <TabsContent value="yaml" class="h-[600px]">
        <ResourceYamlTab
          resource={service}
          kind="Service"
          active={activeTab === 'yaml'}
          onApplied={loadService}
        />
      </TabsContent>
    </Tabs>
  {/if}
</div>
