<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent } from '$lib/components/ui/card';
  import { toast } from '$lib/utils/toast';
  import Play from '~icons/fa6-solid/play';
  import RefreshCw from '~icons/fa6-solid/rotate';
  import type { Project } from '$lib/domains/projects/types';
  import {
    actions,
    type UnifiedAction,
    type UnifiedWorkflow,
    type ActionRunResult,
    type ActionSource,
  } from '$lib/domains/actions';
  import { GitHubProjectActionsPanel } from '$lib/domains/github';
  import ReleaseCard from './ReleaseCard.svelte';
  import ActionRunDetail from './ActionRunDetail.svelte';
  import ActionCatalogSidebar from './ActionCatalogSidebar.svelte';
  import PipelineGeneratorCard from './PipelineGeneratorCard.svelte';
  import DoctorFailureCard from './DoctorFailureCard.svelte';
  import DoctorProposalCard from './DoctorProposalCard.svelte';
  import DoctorReportCard from './DoctorReportCard.svelte';
  import LastRunCard from './LastRunCard.svelte';
  import RunHistoryCard from './RunHistoryCard.svelte';
  import { pickRunSelection } from '../stepGraph';
  import { queryClient } from '$lib/domains/shared/query';
  import { queryKeys } from '$lib/domains/shared/query/keys';
  import PipelineRunDetail from '$lib/domains/projects/pipelines/components/PipelineRunDetail.svelte';
  import { getRecentRuns, getRunningRuns, onRunUpdates } from '../runsService';
  import type { RunSummary } from '../types';
  import { untrack } from 'svelte';
  import { isActive } from '$lib/domains/actions/runActions';
  import { goto } from '$app/navigation';
  import { coderSession } from '$lib/domains/coder/state/coderSession.svelte.js';
  import {
    diagnose,
    generatePipeline,
    getDoctorRun,
    getGenerateRun,
    resetDoctorRun,
    dismissReviewPointer,
    verifyProposal,
  } from '../state/doctor.svelte';

  interface Props {
    project: Project;
    enabled?: boolean;
  }

  let { project, enabled = true }: Props = $props();

  type SourceFilter = 'all' | ActionSource;

  let loading = $state(true);
  let running = $state(false);
  let actionsList = $state<UnifiedAction[]>([]);
  let workflows = $state<UnifiedWorkflow[]>([]);
  let warnings = $state<string[]>([]);
  let filePath = $state<string | undefined>();
  let sourceFilter = $state<SourceFilter>('all');
  let selected = $state<Set<string>>(new Set());
  let lastResult = $state<ActionRunResult | null>(null);
  let seenProjectId: string | number | undefined;
  let runHistory = $state<RunSummary[]>([]);
  let historyLoading = $state(false);
  // The run shown in the detail card; it loads and live-polls itself (QA-130).
  let selectedRunId = $state<string | null>(null);
  const selectedKind = $derived(runHistory.find((r) => r.id === selectedRunId)?.kind);
  let runTerminal = $state(false);
  // The user clicked a tree node during the run: stop auto-following steps.
  let pinnedRun = false;

  const doctorRun = $derived(getDoctorRun(project.id));
  const doctorBusy = $derived(
    doctorRun.status === 'diagnosing' || doctorRun.status === 'verifying'
  );
  // Doctor state outlives this component (QA-156): back from Coder mid-run, the
  // card still shows the failure being worked on.
  const shownFailure = $derived(lastResult?.failure ?? doctorRun.failure);

  // Stage 4 — requirement-driven pipeline generation.
  let requirement = $state('');
  const genRun = $derived(getGenerateRun(project.id));
  const genRunning = $derived(
    genRun.status === 'generating' ||
      genRun.status === 'validating' ||
      genRun.status === 'previewing'
  );

  async function generate() {
    if (genRunning || !requirement.trim()) return;
    await generatePipeline(project, requirement);
    // Pick up the new/changed actions the agent wrote.
    await loadCatalog({ silent: true });
    await loadHistory();
    const run = getGenerateRun(project.id);
    if (run.status === 'done') toast.success('Pipeline generated — review the changes in Coder');
    else if (run.status === 'failed' && run.message) toast.error(run.message);
  }

  async function openThread(id: string) {
    await coderSession.selectThread(id);
    void goto('/ai/coder');
  }

  async function diagnoseFailure(f: NonNullable<typeof shownFailure>) {
    const verify = await diagnose(project, f);
    if (verify) {
      // Reflect the verify run as the latest result; on success the
      // "Run failed" card clears because verify.failure is undefined.
      lastResult = verify;
      const ls = verify.steps.find((s) => s.executionId);
      if (ls?.executionId) openExecution(ls.executionId);
      if (verify.success) toast.success('Doctor fixed it — review the changes in Coder');
    }
    const after = getDoctorRun(project.id);
    if (after.unavailable && after.message) toast.error(after.message);
    await loadHistory();
  }

  async function verifyFix() {
    const verify = await verifyProposal(project);
    if (verify) {
      lastResult = verify;
      const ls = verify.steps.find((s) => s.executionId);
      if (ls?.executionId) openExecution(ls.executionId);
      if (verify.success) toast.success('Verified — review the changes in Coder');
    }
    await loadHistory();
  }

  function normalizePath(path: string): string {
    return path.replace(/\\/g, '/').replace(/\/+$/, '').toLowerCase();
  }

  function isProjectExecution(exec: RunSummary): boolean {
    if (exec.projectId) return String(exec.projectId) === String(project.id);
    if (!exec.workingDirectory) return false;
    return normalizePath(exec.workingDirectory) === normalizePath(project.path);
  }

  const filteredActions = $derived(
    sourceFilter === 'all' ? actionsList : actionsList.filter((a) => a.source === sourceFilter)
  );

  const filteredWorkflows = $derived(
    sourceFilter === 'all' ? workflows : workflows.filter((w) => w.source === sourceFilter)
  );

  async function loadCatalog(opts?: { silent?: boolean }) {
    if (!enabled) return;
    const silent = opts?.silent ?? actionsList.length > 0;
    if (!silent) loading = true;
    try {
      const catalog = await actions.catalogForProject(project);
      actionsList = catalog.actions;
      workflows = catalog.workflows;
      warnings = catalog.warnings;
      filePath = catalog.filePath;
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to load actions catalog');
    } finally {
      loading = false;
    }
  }

  async function loadHistory(opts?: { silent?: boolean }) {
    if (!opts?.silent) historyLoading = true;
    try {
      const recent = await getRecentRuns(50);
      runHistory = recent.filter(isProjectExecution);
    } catch {
      // history is best-effort
    } finally {
      if (!opts?.silent) historyLoading = false;
    }
  }

  function openExecution(executionId: string, opts?: { terminal?: boolean }) {
    selectedRunId = executionId;
    runTerminal = !!opts?.terminal;
  }

  function toggleSelect(id: string) {
    const next = new Set(selected);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    selected = next;
  }

  async function runSelected() {
    const ids = [...selected];
    if (ids.length === 0) {
      toast.info('Select one or more actions to run');
      return;
    }
    await runTarget(ids.length === 1 ? ids[0]! : ids);
  }

  async function runTarget(target: string | string[], event?: Event) {
    event?.preventDefault();
    event?.stopPropagation();
    if (running) return;

    running = true;
    pinnedRun = false;
    lastResult = null;
    resetDoctorRun(project.id);
    try {
      const result = await actions.forProject(project).run(target, {
        // Surface the run the moment its process spawns — not after it finishes.
        onStepStart: ({ executionId, workflowRunId }) => {
          // Show the new step's row in the tree now, not on the next poll.
          if (workflowRunId)
            void queryClient.invalidateQueries({
              queryKey: queryKeys.actions.workflowRun(workflowRunId),
            });
          if (!pinnedRun) openExecution(executionId);
          void loadHistory();
        },
      });
      lastResult = result;
      // Prefer showing the script execution record (has persisted output)
      const pick = pickRunSelection(result.steps);
      if (pinnedRun) {
        // keep the user's own selection
      } else if (pick) {
        openExecution(pick);
      } else {
        selectedRunId = null;
      }
      if (result.steps.some((s) => s.status === 'cancelled')) {
        // Stopped by the user — stopRun already said so; not a failure.
      } else if (result.success) {
        toast.success(
          typeof target === 'string' ? `Ran ${target}` : `Ran ${target.length} actions`,
          result.remoteRunId ? `Remote: ${result.remoteRunId}` : undefined
        );
      } else {
        const failed =
          result.steps.find((s) => s.error) ?? result.steps.find((s) => s.status === 'failed');
        // Short + auto-dismissing: the failure card / run details carry the output.
        const err =
          (failed?.exitCode != null ? `Exit code ${failed.exitCode}` : null) ||
          failed?.error?.trim().split('\n')[0]?.slice(0, 120) ||
          'Action run finished with errors';
        toast.error(typeof target === 'string' ? `${target} failed` : 'Run failed', err, {
          duration: 5000,
        });
      }
      await loadHistory();
    } catch (err) {
      toast.error(err instanceof Error ? err.message : String(err));
    } finally {
      running = false;
    }
  }

  // Keep history live while the tab is open: poll the (cheap) running set and
  // reload silently only when a run starts or finishes — fast while runs are
  // active, slow when idle so runs started elsewhere still appear.
  $effect(() => {
    if (!enabled || !project?.id) return;
    return onRunUpdates(() => void loadHistory({ silent: true }));
  });

  $effect(() => {
    if (!enabled || !project?.id) return;
    const delay = runHistory.some(isActive) ? 2000 : 5000;
    const timer = setInterval(async () => {
      try {
        const running = new Set(
          (await getRunningRuns()).filter(isProjectExecution).map((e) => e.id)
        );
        const known = new Set(runHistory.filter(isActive).map((e) => e.id));
        const changed =
          [...known].some((id) => !running.has(id)) || [...running].some((id) => !known.has(id));
        if (changed) void loadHistory({ silent: true });
      } catch {
        // best-effort; Refresh still works
      }
    }, delay);
    return () => clearInterval(timer);
  });

  $effect(() => {
    const projectId = project?.id;
    const isEnabled = enabled;
    if (!isEnabled || !projectId) return;
    // Only re-fetch when the project id / tab enablement changes —
    // not when the project object identity updates from query refetches.
    untrack(() => {
      // The panel is reused across /projects/[id] navigations: drop the previous
      // project's failure/selection so Diagnose never runs A's failure against B.
      if (seenProjectId !== undefined && seenProjectId !== projectId) {
        lastResult = null;
        selectedRunId = null;
        selected = new Set();
        requirement = '';
      }
      seenProjectId = projectId;
      void loadCatalog();
      void loadHistory();
    });
  });
</script>

<div class="space-y-6">
  <div class="flex flex-wrap items-start justify-between gap-3">
    <div>
      <h2 class="text-xl font-semibold">Actions</h2>
      <p class="text-sm text-muted-foreground">
        Smart defaults from project metadata
        {#if filePath}
          · overrides from <code class="text-xs">{filePath}</code>
        {/if}
        · GitHub & n8n when available
      </p>
    </div>
    <div class="flex flex-wrap gap-2">
      <Button
        variant="outline"
        size="sm"
        class="gap-2"
        onclick={() => {
          void loadCatalog();
          void loadHistory();
        }}
        disabled={loading || running}
      >
        <RefreshCw class="h-4 w-4" />
        Refresh
      </Button>
      <Button
        type="button"
        size="sm"
        class="gap-2"
        onclick={() => void runSelected()}
        disabled={running || selected.size === 0}
      >
        <Play class="h-4 w-4" />
        {running ? 'Running…' : `Run selected (${selected.size})`}
      </Button>
    </div>
  </div>

  <div class="flex flex-col gap-6 xl:flex-row xl:items-start">
    <ActionCatalogSidebar
      actions={filteredActions}
      workflows={filteredWorkflows}
      {warnings}
      {loading}
      {running}
      {selected}
      bind:sourceFilter
      onToggleSelect={toggleSelect}
      onRun={(id, e) => void runTarget(id, e)}
    />

    <!-- Run details + history -->
    <div class="min-w-0 flex-1 space-y-4">
      <PipelineGeneratorCard
        bind:requirement
        run={genRun}
        running={genRunning}
        blocked={running || doctorBusy}
        onGenerate={() => void generate()}
        onOpenThread={(id) => void openThread(id)}
      />

      {#if shownFailure}
        <DoctorFailureCard
          failure={shownFailure}
          run={doctorRun}
          busy={doctorBusy || genRunning}
          onDiagnose={() => void diagnoseFailure(shownFailure)}
          onOpenThread={(id) => void openThread(id)}
        />
      {/if}

      {#if doctorRun.proposal}
        <DoctorProposalCard
          proposal={doctorRun.proposal}
          reviewThreadId={doctorRun.reviewThreadId}
          onVerify={() => void verifyFix()}
          onOpenThread={(id) => void openThread(id)}
        />
      {/if}

      {#if doctorRun.reviewThreadId && !shownFailure && !doctorRun.proposal}
        <Card class="border-primary/30">
          <CardContent class="flex flex-wrap items-center gap-2 py-3">
            <span class="text-sm">
              {doctorRun.status === 'fixed' ? 'Fixed' : 'Agent changes'} · pending your review in Coder
            </span>
            <Button
              type="button"
              size="sm"
              variant="link"
              onclick={() => void openThread(doctorRun.reviewThreadId!)}
            >
              Review changes in Coder
            </Button>
            <Button
              type="button"
              size="sm"
              variant="ghost"
              class="ml-auto"
              onclick={() => dismissReviewPointer(project.id)}
            >
              Dismiss
            </Button>
          </CardContent>
        </Card>
      {/if}

      {#if doctorRun.report}
        <DoctorReportCard report={doctorRun.report} />
      {/if}

      <div class="grid gap-4 min-[2200px]:grid-cols-2 min-[2200px]:items-start">
        {#if selectedRunId && selectedKind && selectedKind !== 'action'}
          <PipelineRunDetail executionId={selectedRunId} onClose={() => (selectedRunId = null)} />
        {:else if selectedRunId}
          <ActionRunDetail
            executionId={selectedRunId}
            bind:terminal={runTerminal}
            onStarted={(id) => {
              openExecution(id);
              void loadHistory();
            }}
            onSelectExecution={(id) => {
              pinnedRun = true;
              openExecution(id);
            }}
            onChanged={() => void loadHistory()}
            onClose={() => (selectedRunId = null)}
          />
        {:else if lastResult}
          <LastRunCard result={lastResult} onOpenExecution={(id) => openExecution(id)} />
        {/if}

        <RunHistoryCard
          runs={runHistory}
          loading={historyLoading}
          selectedId={selectedRunId}
          onRefresh={() => void loadHistory()}
          onSelect={(e, opts) => openExecution(e.id, opts)}
          onStarted={(id) => {
            openExecution(id);
            void loadHistory();
          }}
          onChanged={(deletedId) => {
            if (deletedId && deletedId === selectedRunId) selectedRunId = null;
            void loadHistory();
          }}
        />
      </div>
    </div>
  </div>

  <ReleaseCard {project} />

  {#if sourceFilter === 'all' || sourceFilter === 'github'}
    <div class="space-y-3 border-t pt-6">
      <h3 class="text-sm font-medium text-muted-foreground">GitHub Actions runs</h3>
      <GitHubProjectActionsPanel {project} {enabled} />
    </div>
  {/if}
</div>
