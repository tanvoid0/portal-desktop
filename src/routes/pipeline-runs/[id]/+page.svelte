<!--
  One action run on its own page — a stable link (/pipeline-runs/<id>) to open
  from a project, a session, or an MCP tool result. The /pipeline-runs rail
  shows the same ActionRunDetail.
-->
<script lang="ts">
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import { Button } from '$lib/components/ui/button';
  import ActionRunDetail from '$lib/domains/actions/components/ActionRunDetail.svelte';
  import PipelineRunDetail from '$lib/domains/projects/pipelines/components/PipelineRunDetail.svelte';
  import { getRecentRuns } from '$lib/domains/actions/runsService';
  import { scriptExecutionService } from '$lib/domains/scripts/services/scriptExecutionService';
  import { createProjectsQuery } from '$lib/domains/projects/queries/projectQueries';
  import type { RunSummary } from '$lib/domains/actions/types';
  import ArrowLeft from '~icons/fa6-solid/arrow-left';
  import FolderOpen from '~icons/fa6-solid/folder-open';

  const runId = $derived(page.params.id ?? '');
  let terminal = $state(false);
  let workingDirectory = $state<string | null>(null);
  let kind = $state<RunSummary['kind']>('action');
  let projectId = $state<string | null>(null);

  const projectsQuery = createProjectsQuery();
  const norm = (p: string) => p.replace(/\\/g, '/').replace(/\/+$/, '').toLowerCase();
  // A pipeline/release names its project; an action matches on its exact folder
  // (same rule as the project Actions tab).
  const project = $derived(
    (projectsQuery.data ?? []).find((p) =>
      projectId
        ? String(p.id) === projectId
        : workingDirectory && norm(p.path) === norm(workingDirectory)
    ) ?? null
  );

  $effect(() => {
    const id = runId;
    terminal = false;
    workingDirectory = null;
    kind = 'action';
    projectId = null;
    let cancelled = false;
    // ponytail: kind comes from the 200 most recent runs; an older id is shown as an action.
    getRecentRuns(200)
      .then(async (runs) => {
        const run = runs.find((r) => r.id === id);
        if (cancelled) return;
        if (run) {
          kind = run.kind;
          projectId = run.projectId ?? null;
          workingDirectory = run.workingDirectory ?? null;
        } else {
          const e = await scriptExecutionService.getExecution(id);
          if (!cancelled) workingDirectory = e?.workingDirectory ?? null;
        }
      })
      .catch(() => {});
    return () => {
      cancelled = true;
    };
  });
</script>

<svelte:head>
  <title>Action Run - Portal Desktop</title>
</svelte:head>

<div class="space-y-4">
  <div class="flex flex-wrap items-center gap-2">
    <Button variant="ghost" size="sm" class="gap-2" href="/pipeline-runs">
      <ArrowLeft class="h-4 w-4" />
      All action runs
    </Button>
    {#if project}
      <Button variant="outline" size="sm" class="gap-2" href="/projects/{project.id}?tab=actions">
        <FolderOpen class="h-4 w-4" />
        {project.name}
      </Button>
    {/if}
  </div>

  {#if runId && kind !== 'action'}
    <PipelineRunDetail executionId={runId} onClose={() => void goto('/pipeline-runs')} />
  {:else if runId}
    <ActionRunDetail
      executionId={runId}
      bind:terminal
      showPageLink={false}
      onStarted={(id) => void goto(`/pipeline-runs/${id}`)}
      onSelectExecution={(id) => void goto(`/pipeline-runs/${id}`)}
    />
  {/if}
</div>
