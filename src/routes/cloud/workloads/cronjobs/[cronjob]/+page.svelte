<!-- CronJob Detail Page -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import { ResourceDetail } from '$lib/domains/cloud/state/resourceDetail.svelte';
  import { ResourceType } from '$lib/domains/cloud/core/types';
  import { Tabs, TabsContent, TabsList, TabsTrigger } from '$lib/components/ui/tabs';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import Loading from '$lib/components/ui/loading.svelte';
  import { PageLoading, PageError } from '$lib/components/shell';
  import ResourceYamlTab from '$lib/domains/cloud/components/ResourceYamlTab.svelte';
  import ResourceDetailHeader from '$lib/domains/cloud/components/ResourceDetailHeader.svelte';
  import { k8sResourceService } from '$lib/domains/cloud/services/k8sResourceService';

  const r = new ResourceDetail(ResourceType.CRONJOB, 'CronJob', () => page.params.cronjob);
  const cronJob = $derived(r.resource);

  onMount(r.load);
</script>

<div class="space-y-6 p-6">
  {#if r.isLoading}
    <PageLoading message="Loading cronjob..." />
  {:else if r.error}
    <PageError title="Failed to load cronjob" message={r.error} onRetry={r.load} />
  {:else if cronJob}
    <ResourceDetailHeader
      kind="CronJob"
      name={cronJob.name}
      namespace={cronJob.namespace}
      backHref="/cloud/cronjobs"
      backLabel="Back to CronJobs"
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
          <Card>
            <CardHeader>
              <CardTitle>CronJob Information</CardTitle>
            </CardHeader>
            <CardContent class="space-y-3">
              <div>
                <p class="text-sm text-muted-foreground">Status</p>
                <Badge class="mt-1" variant={cronJob.metadata?.suspend ? 'secondary' : 'default'}>
                  {cronJob.metadata?.suspend ? 'Suspended' : 'Active'}
                </Badge>
              </div>
              <div>
                <p class="text-sm text-muted-foreground">Schedule</p>
                <p class="font-mono font-medium">
                  {cronJob.metadata?.schedule || 'N/A'}
                </p>
              </div>
              <div>
                <p class="text-sm text-muted-foreground">Active Jobs</p>
                <p class="font-medium">{cronJob.metadata?.active || 0}</p>
              </div>
              <div>
                <p class="text-sm text-muted-foreground">Last Schedule Time</p>
                <p class="font-medium">
                  {cronJob.metadata?.last_schedule_time || 'Never'}
                </p>
              </div>
              <div>
                <p class="text-sm text-muted-foreground">Last Successful Time</p>
                <p class="font-medium">
                  {cronJob.metadata?.last_successful_time || 'Never'}
                </p>
              </div>
              <div>
                <p class="text-sm text-muted-foreground">Age</p>
                <p class="font-medium">{cronJob.metadata?.age || 'N/A'}</p>
              </div>
              {#if cronJob.metadata?.image}
                <div>
                  <p class="text-sm text-muted-foreground">Image</p>
                  <p class="font-medium">{cronJob.metadata.image}</p>
                </div>
              {/if}
            </CardContent>
          </Card>
        </div>
      </TabsContent>

      <!-- YAML Tab -->
      <TabsContent value="yaml" class="h-[600px]">
        <ResourceYamlTab
          resource={cronJob}
          kind="CronJob"
          active={r.activeTab === 'yaml'}
          onApplied={r.load}
        />
      </TabsContent>
    </Tabs>
  {/if}
</div>
