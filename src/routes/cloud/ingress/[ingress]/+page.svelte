<!-- Ingress Detail Page -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import { replaceState } from '$app/navigation';
  import { cloudStore, loadResources } from '$lib/domains/cloud/stores';
  import { ResourceType, type ICloudResource } from '$lib/domains/cloud/core/types';
  import { Tabs, TabsContent, TabsList, TabsTrigger } from '$lib/components/ui/tabs';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import Loading from '$lib/components/ui/loading.svelte';
  import { PageLoading, PageError } from '$lib/components/shell';
  import InfoCard from '$lib/domains/cloud/components/InfoCard.svelte';
  import ResourceYamlTab from '$lib/domains/cloud/components/ResourceYamlTab.svelte';
  import ResourceDetailHeader from '$lib/domains/cloud/components/ResourceDetailHeader.svelte';
  import LabelsCard from '$lib/domains/cloud/components/LabelsCard.svelte';
  import { k8sResourceService } from '$lib/domains/cloud/services/k8sResourceService';

  const ingressName = $derived(page.params.ingress);
  const namespace = $derived(
    page.url.searchParams.get('namespace') || $cloudStore.selectedNamespace || 'default'
  );
  const tabParam = $derived(page.url.searchParams.get('tab') || 'overview');

  let activeTab = $state('overview');

  // Sync activeTab with tabParam when it changes
  $effect(() => {
    activeTab = tabParam;
  });

  let ingress = $state<ICloudResource | null>(null);
  let isLoading = $state(true);
  let error = $state<string | null>(null);

  onMount(async () => {
    await loadIngress();
  });

  async function loadIngress() {
    if (!ingressName || !$cloudStore.connection.isConnected) {
      error = 'Ingress name or connection required';
      isLoading = false;
      return;
    }

    try {
      isLoading = true;
      error = null;

      await loadResources(ResourceType.INGRESS, namespace);
      const resources = $cloudStore.resources[ResourceType.INGRESS] || [];
      ingress = resources.find((i) => i.name === ingressName) || null;

      if (!ingress) {
        error = `Ingress "${ingressName}" not found in namespace "${namespace}".`;
      }
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to load Ingress';
      console.error('Failed to load Ingress:', err);
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

  const addresses = $derived.by(() => {
    if (!ingress || !ingress.metadata?.addresses) return [];
    return ingress.metadata.addresses;
  });

  const hosts = $derived.by(() => {
    if (!ingress || !ingress.metadata?.ports) return [];
    return ingress.metadata.ports;
  });
</script>

<div class="space-y-6 p-6">
  {#if isLoading}
    <PageLoading message="Loading ingress..." />
  {:else if error}
    <PageError title="Failed to load ingress" message={error} onRetry={loadIngress} />
  {:else if ingress}
    <ResourceDetailHeader
      kind="Ingress"
      name={ingress.name}
      namespace={ingress.namespace}
      backHref="/cloud/ingress"
      backLabel="Back to Ingress"
      onRefresh={loadIngress}
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
            title="Ingress Information"
            fields={[
              { label: 'Name', value: ingress.name },
              { label: 'Namespace', value: ingress.namespace },
              { label: 'Ingress Class', value: ingress.metadata?.class || 'N/A' },
              { label: 'Age', value: ingress.metadata?.age || 'N/A' },
            ]}
          />

          {#if addresses.length > 0 || hosts.length > 0}
            <Card>
              <CardHeader>
                <CardTitle>Network Information</CardTitle>
              </CardHeader>
              <CardContent class="space-y-3">
                {#if addresses.length > 0}
                  <div>
                    <p class="text-sm text-muted-foreground">Addresses</p>
                    <div class="mt-1 flex flex-wrap gap-2">
                      {#each addresses as addr}
                        <Badge variant="outline">{addr}</Badge>
                      {/each}
                    </div>
                  </div>
                {/if}
                {#if hosts.length > 0}
                  <div>
                    <p class="text-sm text-muted-foreground">Hosts</p>
                    <div class="mt-1 flex flex-wrap gap-2">
                      {#each hosts as host}
                        <Badge variant="secondary">{host}</Badge>
                      {/each}
                    </div>
                  </div>
                {/if}
              </CardContent>
            </Card>
          {/if}

          {#if ingress.metadata?.labels && Object.keys(ingress.metadata.labels).length > 0}
            <LabelsCard items={ingress.metadata.labels} />
          {/if}
        </div>
      </TabsContent>

      <!-- YAML Tab -->
      <TabsContent value="yaml" class="h-[600px]">
        <ResourceYamlTab
          resource={ingress}
          kind="Ingress"
          active={activeTab === 'yaml'}
          onApplied={loadIngress}
        />
      </TabsContent>
    </Tabs>
  {/if}
</div>
