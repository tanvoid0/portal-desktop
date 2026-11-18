<!-- Job Detail Page -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import { ResourceDetail } from '$lib/domains/cloud/state/resourceDetail.svelte';
  import { ResourceType } from '$lib/domains/cloud/core/types';
  import { Tabs, TabsContent, TabsList, TabsTrigger } from '$lib/components/ui/tabs';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import { k8sResourceService } from '$lib/domains/cloud/services/k8sResourceService';
  import Loading from '$lib/components/ui/loading.svelte';
  import { PageLoading, PageError } from '$lib/components/shell';
  import InfoCard from '$lib/domains/cloud/components/InfoCard.svelte';
  import ResourceYamlTab from '$lib/domains/cloud/components/ResourceYamlTab.svelte';
  import ResourceDetailHeader from '$lib/domains/cloud/components/ResourceDetailHeader.svelte';

  const r = new ResourceDetail(ResourceType.JOB, 'Job', () => page.params.job);
  const job = $derived(r.resource);

  onMount(r.load);
</script>

<div class="space-y-6 p-6">
  {#if r.isLoading}
    <PageLoading message="Loading job..." />
  {:else if r.error}
    <PageError title="Failed to load job" message={r.error} onRetry={r.load} />
  {:else if job}
    <ResourceDetailHeader
      kind="Job"
      name={job.name}
      namespace={job.namespace}
      backHref="/cloud/jobs"
      backLabel="Back to Jobs"
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
          active={r.activeTab === 'yaml'}
          onApplied={r.load}
        />
      </TabsContent>
    </Tabs>
  {/if}
</div>
