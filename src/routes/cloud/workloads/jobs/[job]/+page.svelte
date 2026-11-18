<!-- Job Detail Page -->
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

  const jobName = $derived(page.params.job);
  const namespace = $derived(
    page.url.searchParams.get('namespace') || $cloudStore.selectedNamespace || 'default'
  );
  const tabParam = $derived(page.url.searchParams.get('tab') || 'overview');

  let activeTab = $state('overview');

  // Sync activeTab with tabParam when it changes
  $effect(() => {
    activeTab = tabParam;
  });
  let job = $state<ICloudResource | null>(null);
  let isLoading = $state(true);
  let error = $state<string | null>(null);

  onMount(async () => {
    await loadJob();
  });

  async function loadJob() {
    if (!jobName || !$cloudStore.connection.isConnected) {
      error = 'Job name or connection required';
      isLoading = false;
      return;
    }

    try {
      isLoading = true;
      error = null;

      await loadResources(ResourceType.JOB, namespace);
      const resources = $cloudStore.resources[ResourceType.JOB] || [];
      job = resources.find((j) => j.name === jobName) || null;

      if (!job) {
        error = `Job "${jobName}" not found in namespace "${namespace}".`;
      }
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to load job';
      console.error('Failed to load job:', err);
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
</script>

<div class="space-y-6 p-6">
  {#if isLoading}
    <PageLoading message="Loading job..." />
  {:else if error}
    <PageError title="Failed to load job" message={error} onRetry={loadJob} />
  {:else if job}
    <ResourceDetailHeader
      kind="Job"
      name={job.name}
      namespace={job.namespace}
      backHref="/cloud/jobs"
      backLabel="Back to Jobs"
      onRefresh={loadJob}
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
            title="Job Information"
            fields={[
              { label: 'Completions', value: job.metadata?.completions || 'N/A' },
              { label: 'Succeeded', value: job.metadata?.succeeded || 0 },
              { label: 'Failed', value: job.metadata?.failed || 0 },
              { label: 'Active', value: job.metadata?.active || 0 },
              { label: 'Age', value: job.metadata?.age || 'N/A' },
            ]}
            status={{ text: job.status }}
          />

          <Card>
            <CardHeader>
              <CardTitle>Configuration</CardTitle>
            </CardHeader>
            <CardContent class="space-y-3">
              <div>
                <p class="text-sm text-muted-foreground">Parallelism</p>
                <p class="font-medium">{job.metadata?.parallelism || 'N/A'}</p>
              </div>
              <div>
                <p class="text-sm text-muted-foreground">Backoff Limit</p>
                <p class="font-medium">
                  {job.metadata?.backoff_limit || 'N/A'}
                </p>
              </div>
              {#if job.metadata?.image}
                <div>
                  <p class="text-sm text-muted-foreground">Image</p>
                  <p class="font-medium">{job.metadata.image}</p>
                </div>
              {/if}
            </CardContent>
          </Card>
        </div>

        {#if job.metadata && Object.keys(job.metadata).length > 0}
          <Card>
            <CardHeader>
              <CardTitle>Metadata</CardTitle>
            </CardHeader>
            <CardContent>
              <pre class="overflow-auto rounded-lg bg-muted p-4 text-xs">{JSON.stringify(
                  job.metadata,
                  null,
                  2
                )}</pre>
            </CardContent>
          </Card>
        {/if}
      </TabsContent>

      <!-- YAML Tab -->
      <TabsContent value="yaml" class="h-[600px]">
        <ResourceYamlTab
          resource={job}
          kind="Job"
          active={activeTab === 'yaml'}
          onApplied={loadJob}
        />
      </TabsContent>
    </Tabs>
  {/if}
</div>
