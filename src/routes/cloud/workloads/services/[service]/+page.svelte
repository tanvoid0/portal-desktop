<!-- Service Detail Page -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import { ResourceDetail } from '$lib/domains/cloud/state/resourceDetail.svelte';
  import { ResourceType } from '$lib/domains/cloud/core/types';
  import { Tabs, TabsContent, TabsList, TabsTrigger } from '$lib/components/ui/tabs';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { k8sResourceService } from '$lib/domains/cloud/services/k8sResourceService';
  import Loading from '$lib/components/ui/loading.svelte';
  import { PageLoading, PageError } from '$lib/components/shell';
  import InfoCard from '$lib/domains/cloud/components/InfoCard.svelte';
  import LabelsCard from '$lib/domains/cloud/components/LabelsCard.svelte';
  import ResourceYamlTab from '$lib/domains/cloud/components/ResourceYamlTab.svelte';
  import ResourceDetailHeader from '$lib/domains/cloud/components/ResourceDetailHeader.svelte';

  const r = new ResourceDetail(ResourceType.SERVICE, 'Service', () => page.params.service);
  const service = $derived(r.resource);

  onMount(r.load);

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
  {#if r.isLoading}
    <PageLoading message="Loading service..." />
  {:else if r.error}
    <PageError title="Failed to load service" message={r.error} onRetry={r.load} />
  {:else if service}
    <ResourceDetailHeader
      kind="Service"
      name={service.name}
      namespace={service.namespace}
      backHref="/cloud/services"
      backLabel="Back to Services"
      onRefresh={r.load}
    />

    <!-- Tabs -->
    <Tabs value={r.activeTab} onValueChange={r.setTab}>
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

        <LabelsCard title="Selector" items={selector} emptyText="No selector configured" />
      </TabsContent>

      <!-- YAML Tab -->
      <TabsContent value="yaml" class="h-[600px]">
        <ResourceYamlTab
          resource={service}
          kind="Service"
          active={r.activeTab === 'yaml'}
          onApplied={r.load}
        />
      </TabsContent>
    </Tabs>
  {/if}
</div>
