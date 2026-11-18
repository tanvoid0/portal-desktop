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
  import { scriptExecutionService } from '$lib/domains/scripts/services/scriptExecutionService';
  import { createProjectsQuery } from '$lib/domains/projects/queries/projectQueries';
  import ArrowLeft from '~icons/fa6-solid/arrow-left';
  import FolderOpen from '~icons/fa6-solid/folder-open';

  const runId = $derived(page.params.id ?? '');
  let terminal = $state(false);
  let workingDirectory = $state<string | null>(null);

  const projectsQuery = createProjectsQuery();
  const norm = (p: string) => p.replace(/\\/g, '/').replace(/\/+$/, '').toLowerCase();
  // Same exact-folder rule as the project Actions tab.
  const project = $derived(
    workingDirectory
      ? ((projectsQuery.data ?? []).find((p) => norm(p.path) === norm(workingDirectory!)) ?? null)
      : null
  );

  $effect(() => {
    const id = runId;
    terminal = false;
    workingDirectory = null;
    let cancelled = false;
    scriptExecutionService
      .getExecution(id)
      .then((e) => {
        if (!cancelled) workingDirectory = e?.workingDirectory ?? null;
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

  {#if runId}
    <ActionRunDetail
      executionId={runId}
      bind:terminal
      showPageLink={false}
      onStarted={(id) => void goto(`/pipeline-runs/${id}`)}
      onSelectExecution={(id) => void goto(`/pipeline-runs/${id}`)}
    />
  {/if}
</div>
