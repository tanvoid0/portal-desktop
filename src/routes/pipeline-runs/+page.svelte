<!--
  Global Pipeline Runs Page
-->
<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import Select from "$lib/components/ui/select.svelte";
  import {
    PageHeader,
    PageLoading,
    PageError,
  } from "$lib/components/shell";
  import PipelineRunHistoryList from "$lib/domains/projects/pipelines/components/PipelineRunHistoryList.svelte";
  import {
    executionService,
    type PipelineExecutionListItem,
    type ExecutionStatus,
  } from "$lib/domains/projects/pipelines";
  import { logger } from "$lib/domains/shared";
  import { RefreshCw } from "@lucide/svelte";

  const log = logger.createScoped("PipelineRunsPage");

  let executions = $state<PipelineExecutionListItem[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let search = $state("");
  let statusFilter = $state<string>("all");

  const statusOptions = [
    { value: "all", label: "All statuses" },
    { value: "running", label: "Running" },
    { value: "success", label: "Success" },
    { value: "failed", label: "Failed" },
    { value: "cancelled", label: "Cancelled" },
    { value: "pending", label: "Pending" },
  ];

  const filteredExecutions = $derived(
    executions.filter((execution) => {
      const matchesStatus =
        statusFilter === "all" || execution.status === statusFilter;
      const query = search.trim().toLowerCase();
      const matchesSearch =
        !query ||
        execution.pipelineName?.toLowerCase().includes(query) ||
        execution.projectName?.toLowerCase().includes(query) ||
        execution.status.toLowerCase().includes(query);
      return matchesStatus && matchesSearch;
    }),
  );

  async function loadExecutions() {
    loading = true;
    error = null;
    try {
      executions = await executionService.getAllExecutions(100);
    } catch (err) {
      log.error("Failed to load pipeline runs", err);
      error = err instanceof Error ? err.message : "Failed to load pipeline runs";
    } finally {
      loading = false;
    }
  }

  function handleSelect(execution: PipelineExecutionListItem) {
    goto(`/projects/${execution.projectId}/pipelines/run/${execution.id}`);
  }

  onMount(() => {
    void loadExecutions();
  });
</script>

<svelte:head>
  <title>Pipeline Runs - Portal Desktop</title>
</svelte:head>

<div class="space-y-6">
  <PageHeader
    title="Pipeline Runs"
    description="Global history of pipeline executions across all projects."
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
    <PageError
      title="Failed to load pipeline runs"
      message={error}
      onRetry={loadExecutions}
    />
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
      emptyTitle="No matching pipeline runs"
      emptyDescription="Run a pipeline from a project or adjust your filters."
      onRefresh={loadExecutions}
      onSelect={handleSelect}
    />
  {/if}
</div>
