<!--
  Global Action Runs page — every action run (script_executions) across projects,
  list on the left, selected run in a right rail (?run=<id>). Right-click a run
  for its menu.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import Select from '$lib/components/ui/select.svelte';
  import {
    PageHeader,
    PageLoading,
    PageError,
    PageEmpty,
    ResponsivePanel,
  } from '$lib/components/shell';
  import ActionRunList from '$lib/domains/actions/components/ActionRunList.svelte';
  import ActionRunDetail from '$lib/domains/actions/components/ActionRunDetail.svelte';
  import { isActive } from '$lib/domains/actions/runActions';
  import { runRequester } from '$lib/domains/actions/runOutput';
  import {
    scriptExecutionService,
    type ScriptExecutionInfo,
  } from '$lib/domains/scripts/services/scriptExecutionService';
  import { createProjectsQuery } from '$lib/domains/projects/queries/projectQueries';
  import RefreshCw from '~icons/fa6-solid/rotate';
  import History from '~icons/fa6-solid/clock-rotate-left';

  let executions = $state<ScriptExecutionInfo[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let search = $state(page.url.searchParams.get('q') ?? '');
  let statusFilter = $state<string>('all');
  // Run id whose detail shows the interactive terminal — tied to the id so
  // back/forward to another run never auto-runs that run's command.
  let terminalFor = $state<string | null>(null);

  const projectsQuery = createProjectsQuery();

  // The URL is the single source of truth for the selection, so reload and
  // back/forward restore it without replaying any list state.
  const selectedRunId = $derived(page.url.searchParams.get('run'));

  const statusOptions = [
    { value: 'all', label: 'All statuses' },
    { value: 'running', label: 'Running' },
    { value: 'success', label: 'Success' },
    { value: 'failed', label: 'Failed' },
    { value: 'cancelled', label: 'Stopped' },
  ];

  const initialStatus = page.url.searchParams.get('status');
  if (statusOptions.some((o) => o.value === initialStatus)) statusFilter = initialStatus!;

  const norm = (p: string) => p.replace(/\\/g, '/').replace(/\/+$/, '').toLowerCase();

  /** Project the run belongs to — same exact-folder rule as the project Actions tab. */
  function projectOf(exec: ScriptExecutionInfo) {
    if (!exec.workingDirectory) return null;
    const dir = norm(exec.workingDirectory);
    return (projectsQuery.data ?? []).find((p) => norm(p.path) === dir) ?? null;
  }

  const filteredExecutions = $derived.by(() => {
    const query = search.trim().toLowerCase();
    return executions.filter((e) => {
      if (statusFilter !== 'all' && e.status !== statusFilter) return false;
      if (!query) return true;
      return [
        e.command,
        e.status,
        e.workingDirectory,
        projectOf(e)?.name,
        runRequester(e.triggeredBy),
      ]
        .filter(Boolean)
        .some((v) => v!.toLowerCase().includes(query));
    });
  });

  async function loadExecutions(opts?: { silent?: boolean }) {
    if (!opts?.silent) loading = true;
    error = null;
    try {
      executions = await scriptExecutionService.getRecentExecutions(200);
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to load action runs';
    } finally {
      loading = false;
    }
  }

  function selectRun(id: string | null, opts?: { terminal?: boolean }) {
    terminalFor = opts?.terminal ? id : null;
    const url = new URL(page.url.href);
    if (id) url.searchParams.set('run', id);
    else url.searchParams.delete('run');
    // A real navigation, not pushState: shallow routing leaves page.url as it
    // was, so the ?run= selection never reached selectedRunId (QA-158).
    void goto(url);
  }

  // Poll just the running set (no output payload) and reload the full list only
  // when a run starts or finishes. Fast while runs are active; slow when idle so
  // runs started elsewhere (CLI, schedule, another window) still appear.
  $effect(() => {
    const delay = executions.some(isActive) ? 2000 : 5000;
    const timer = setInterval(async () => {
      try {
        const running = new Set(
          (await scriptExecutionService.getRunningExecutions()).map((e) => e.id)
        );
        const known = new Set(executions.filter(isActive).map((e) => e.id));
        const changed =
          [...known].some((id) => !running.has(id)) || [...running].some((id) => !known.has(id));
        if (changed) void loadExecutions({ silent: true });
      } catch {
        // best-effort; Refresh still works
      }
    }, delay);
    return () => clearInterval(timer);
  });

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
      description="Every action run across your projects. Right-click a run to stop, re-run or open it in a terminal."
    >
      {#snippet actions()}
        <Button variant="outline" onclick={() => void loadExecutions()} disabled={loading}>
          <RefreshCw class="mr-2 h-4 w-4 {loading ? 'animate-spin' : ''}" />
          Refresh
        </Button>
      {/snippet}
    </PageHeader>

    <div class="flex flex-col gap-3 sm:flex-row">
      <Input
        bind:value={search}
        placeholder="Search by project, command, status or requester..."
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
      <PageError title="Failed to load action runs" message={error} onRetry={loadExecutions} />
    {:else if loading && executions.length === 0}
      <PageLoading message="Loading action runs..." />
    {:else if filteredExecutions.length === 0}
      <PageEmpty
        title={executions.length ? 'No matching runs' : 'No action runs yet'}
        description={executions.length
          ? 'Adjust the search or status filter.'
          : "Run an action from a project's Actions tab."}
        icon={History}
      />
    {:else}
      <p class="text-sm text-muted-foreground">
        {filteredExecutions.length} run{filteredExecutions.length === 1 ? '' : 's'} shown
      </p>
      <ActionRunList
        executions={filteredExecutions}
        selectedId={selectedRunId}
        projectName={(e) => projectOf(e)?.name ?? null}
        onSelect={(e, opts) => selectRun(e.id, opts)}
        onStarted={(id) => {
          selectRun(id);
          void loadExecutions({ silent: true });
        }}
        onChanged={(deletedId) => {
          if (deletedId && deletedId === selectedRunId) selectRun(null);
          void loadExecutions({ silent: true });
        }}
        onOpenProject={(e) => {
          const p = projectOf(e);
          if (p) void goto(`/projects/${p.id}?tab=actions`);
        }}
      />
    {/if}
  </div>

  {#if selectedRunId}
    <ResponsivePanel
      bind:open={() => true, (v) => !v && selectRun(null)}
      side="right"
      desktopClass="sticky top-0 max-h-[calc(100vh-7rem)] w-[clamp(28rem,42%,72rem)] self-start"
    >
      <div class="min-h-0 flex-1 overflow-y-auto p-4">
        <ActionRunDetail
          executionId={selectedRunId}
          bind:terminal={
            () => terminalFor === selectedRunId, (v) => (terminalFor = v ? selectedRunId : null)
          }
          onStarted={(id) => {
            selectRun(id);
            void loadExecutions({ silent: true });
          }}
          onSelectExecution={(id) => selectRun(id)}
          onChanged={() => void loadExecutions({ silent: true })}
          onClose={() => selectRun(null)}
        />
      </div>
    </ResponsivePanel>
  {/if}
</div>
