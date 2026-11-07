<!--
  Global Action Runs Page - list on the left, selected run in a right rail (?run=<id>)
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { pushState } from '$app/navigation';
  import { page } from '$app/state';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import Select from '$lib/components/ui/select.svelte';
  import { PageHeader, PageLoading, PageError, ResponsivePanel } from '$lib/components/shell';
  import PipelineRunHistoryList from '$lib/domains/projects/pipelines/components/PipelineRunHistoryList.svelte';
  import PipelineRunDetail from '$lib/domains/projects/pipelines/components/PipelineRunDetail.svelte';
  import {
    executionService,
    type PipelineExecutionListItem,
  } from '$lib/domains/projects/pipelines';
  import { logger } from '$lib/domains/shared';
  import RefreshCw from '~icons/fa6-solid/rotate';

  const log = logger.createScoped('ActionRunsPage');

  let executions = $state<PipelineExecutionListItem[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let search = $state(page.url.searchParams.get('q') ?? '');
  let statusFilter = $state<string>('all');

  // The URL is the single source of truth for the selection, so reload and
  // back/forward restore it without replaying any list state.
  const selectedRunId = $derived(page.url.searchParams.get('run'));

  const statusOptions = [
    { value: 'all', label: 'All statuses' },
    { value: 'running', label: 'Running' },
    { value: 'success', label: 'Success' },
    { value: 'failed', label: 'Failed' },
    { value: 'cancelled', label: 'Cancelled' },
    { value: 'pending', label: 'Pending' },
  ];

  const initialStatus = page.url.searchParams.get('status');
  if (statusOptions.some((o) => o.value === initialStatus)) statusFilter = initialStatus!;

  const filteredExecutions = $derived(
    executions.filter((execution) => {
      const matchesStatus = statusFilter === 'all' || execution.status === statusFilter;
      const query = search.trim().toLowerCase();
      const matchesSearch =
        !query ||
        execution.pipelineName?.toLowerCase().includes(query) ||
        execution.projectName?.toLowerCase().includes(query) ||
        execution.status.toLowerCase().includes(query);
      return matchesStatus && matchesSearch;
    })
  );

  async function loadExecutions() {
    loading = true;
    error = null;
    try {
      executions = await executionService.getAllExecutions(100);
    } catch (err) {
      log.error('Failed to load pipeline runs', err);
      error = err instanceof Error ? err.message : 'Failed to load pipeline runs';
    } finally {
      loading = false;
    }
  }

  function selectRun(id: string | null) {
    const url = new URL(page.url.href);
    if (id) url.searchParams.set('run', id);
    else url.searchParams.delete('run');
    pushState(url, {});
  }

  function handleSelect(execution: PipelineExecutionListItem) {
    selectRun(execution.id);
  }

  onMount(() => {
    void loadExecutions();
  });
</script>

<svelte:head>
  <title>Action Runs - Portal Desktop</title>
</svelte:head>

<div class="flex min-h-0 gap-6">
  <div class="min-w-0 flex-1 space-y-6">
    <PageHeader
      title="Action Runs"
      description="Global history of local action and pipeline executions."
    >
      {#snippet actions()}
        <Button variant="outline" onclick={loadExecutions} disabled={loading}>
          <RefreshCw class="mr-2 h-4 w-4 {loading ? 'animate-spin' : ''}" />
          Refresh
        </Button>
      {/snippet}
    </PageHeader>

    <div class="flex flex-col gap-3 sm:flex-row">
      <Input
        bind:value={search}
        placeholder="Search by project, pipeline, or status..."
        class="sm:max-w-sm"
      />
      <Select
        options={statusOptions}
        value={statusFilter}
        onSelect={(value) => (statusFilter = value)}
        placeholder="Filter by status"
        class="sm:w-48"
      />
    </div>

    {#if error}
      <PageError title="Failed to load pipeline runs" message={error} onRetry={loadExecutions} />
    {:else if loading && executions.length === 0}
      <PageLoading message="Loading pipeline runs..." />
    {:else}
      <PipelineRunHistoryList
        executions={filteredExecutions}
        {loading}
        title="All runs"
        description="{filteredExecutions.length} run{filteredExecutions.length === 1
          ? ''
          : 's'} shown"
        showProject
        showPipeline
        selectedId={selectedRunId}
        emptyTitle="No matching pipeline runs"
        emptyDescription="Run a pipeline from a project or adjust your filters."
        onRefresh={loadExecutions}
        onSelect={handleSelect}
      />
    {/if}
  </div>

  {#if selectedRunId}
    <ResponsivePanel
      bind:open={() => true, (v) => !v && selectRun(null)}
      side="right"
      desktopClass="sticky top-0 max-h-[calc(100vh-7rem)] w-[28rem] self-start"
    >
      <div class="min-h-0 flex-1 overflow-y-auto p-4">
        <PipelineRunDetail executionId={selectedRunId} onClose={() => selectRun(null)} />
      </div>
    </ResponsivePanel>
  {/if}
</div>
