import { untrack } from 'svelte';
import { goto } from '$app/navigation';
import { toast } from '$lib/utils/toast';
import type { Project } from '$lib/domains/projects/types';
import {
  actions,
  type UnifiedAction,
  type UnifiedWorkflow,
  type ActionRunResult,
  type ActionSource,
} from '$lib/domains/actions';
import { queryClient } from '$lib/domains/shared/query';
import { queryKeys } from '$lib/domains/shared/query/keys';
import { coderSession } from '$lib/domains/coder/state/coderSession.svelte.js';
import { isActive } from '../runActions';
import { pickRunSelection } from '../stepGraph';
import { getRecentRuns, getRunningRuns, onRunUpdates } from '../runsService';
import type { RunSummary } from '../types';
import {
  diagnose,
  generatePipeline,
  getDoctorRun,
  getGenerateRun,
  resetDoctorRun,
  verifyProposal,
} from './doctor.svelte';

export type SourceFilter = 'all' | ActionSource;

function normalizePath(path: string): string {
  return path.replace(/\\/g, '/').replace(/\/+$/, '').toLowerCase();
}

/**
 * Catalog / run / history state of ProjectActionsPanel. Construct during
 * component init (registers $effects); pass a getter so props stay reactive.
 */
export class ProjectActionsState {
  loading = $state(true);
  running = $state(false);
  actionsList = $state<UnifiedAction[]>([]);
  workflows = $state<UnifiedWorkflow[]>([]);
  warnings = $state<string[]>([]);
  filePath = $state<string | undefined>();
  sourceFilter = $state<SourceFilter>('all');
  selected = $state<Set<string>>(new Set());
  lastResult = $state<ActionRunResult | null>(null);
  runHistory = $state<RunSummary[]>([]);
  historyLoading = $state(false);
  // The run shown in the detail card; it loads and live-polls itself (QA-130).
  selectedRunId = $state<string | null>(null);
  runTerminal = $state(false);
  // Stage 4 — requirement-driven pipeline generation.
  requirement = $state('');

  private seenProjectId: string | number | undefined;
  // The user clicked a tree node during the run: stop auto-following steps.
  pinnedRun = false;

  private get project() {
    return this.props().project;
  }

  selectedKind = $derived(this.runHistory.find((r) => r.id === this.selectedRunId)?.kind);
  doctorRun = $derived(getDoctorRun(this.project.id));
  doctorBusy = $derived(
    this.doctorRun.status === 'diagnosing' || this.doctorRun.status === 'verifying'
  );
  // Doctor state outlives this component (QA-156): back from Coder mid-run, the
  // card still shows the failure being worked on.
  shownFailure = $derived(this.lastResult?.failure ?? this.doctorRun.failure);
  genRun = $derived(getGenerateRun(this.project.id));
  genRunning = $derived(
    this.genRun.status === 'generating' ||
      this.genRun.status === 'validating' ||
      this.genRun.status === 'previewing'
  );
  filteredActions = $derived(
    this.sourceFilter === 'all'
      ? this.actionsList
      : this.actionsList.filter((a) => a.source === this.sourceFilter)
  );
  filteredWorkflows = $derived(
    this.sourceFilter === 'all'
      ? this.workflows
      : this.workflows.filter((w) => w.source === this.sourceFilter)
  );

  constructor(private props: () => { project: Project; enabled: boolean }) {
    // Keep history live while the tab is open: poll the (cheap) running set and
    // reload silently only when a run starts or finishes — fast while runs are
    // active, slow when idle so runs started elsewhere still appear.
    $effect(() => {
      if (!this.props().enabled || !this.project?.id) return;
      return onRunUpdates(() => void this.loadHistory({ silent: true }));
    });

    $effect(() => {
      if (!this.props().enabled || !this.project?.id) return;
      const delay = this.runHistory.some(isActive) ? 2000 : 5000;
      const timer = setInterval(async () => {
        try {
          const running = new Set(
            (await getRunningRuns()).filter(this.isProjectExecution).map((e) => e.id)
          );
          const known = new Set(this.runHistory.filter(isActive).map((e) => e.id));
          const changed =
            [...known].some((id) => !running.has(id)) || [...running].some((id) => !known.has(id));
          if (changed) void this.loadHistory({ silent: true });
        } catch {
          // best-effort; Refresh still works
        }
      }, delay);
      return () => clearInterval(timer);
    });

    $effect(() => {
      const projectId = this.project?.id;
      const isEnabled = this.props().enabled;
      if (!isEnabled || !projectId) return;
      // Only re-fetch when the project id / tab enablement changes —
      // not when the project object identity updates from query refetches.
      untrack(() => {
        // The panel is reused across /projects/[id] navigations: drop the previous
        // project's failure/selection so Diagnose never runs A's failure against B.
        if (this.seenProjectId !== undefined && this.seenProjectId !== projectId) {
          this.lastResult = null;
          this.selectedRunId = null;
          this.selected = new Set();
          this.requirement = '';
        }
        this.seenProjectId = projectId;
        void this.loadCatalog();
        void this.loadHistory();
      });
    });
  }

  generate = async () => {
    if (this.genRunning || !this.requirement.trim()) return;
    await generatePipeline(this.project, this.requirement);
    // Pick up the new/changed actions the agent wrote.
    await this.loadCatalog({ silent: true });
    await this.loadHistory();
    const run = getGenerateRun(this.project.id);
    if (run.status === 'done') toast.success('Pipeline generated — review the changes in Coder');
    else if (run.status === 'failed' && run.message) toast.error(run.message);
  };

  openThread = async (id: string) => {
    await coderSession.selectThread(id);
    void goto('/ai/coder');
  };

  diagnoseFailure = async (f: NonNullable<typeof this.shownFailure>) => {
    const verify = await diagnose(this.project, f);
    if (verify) {
      // Reflect the verify run as the latest result; on success the
      // "Run failed" card clears because verify.failure is undefined.
      this.lastResult = verify;
      const ls = verify.steps.find((s) => s.executionId);
      if (ls?.executionId) this.openExecution(ls.executionId);
      if (verify.success) toast.success('Doctor fixed it — review the changes in Coder');
    }
    const after = getDoctorRun(this.project.id);
    if (after.unavailable && after.message) toast.error(after.message);
    await this.loadHistory();
  };

  verifyFix = async () => {
    const verify = await verifyProposal(this.project);
    if (verify) {
      this.lastResult = verify;
      const ls = verify.steps.find((s) => s.executionId);
      if (ls?.executionId) this.openExecution(ls.executionId);
      if (verify.success) toast.success('Verified — review the changes in Coder');
    }
    await this.loadHistory();
  };

  isProjectExecution = (exec: RunSummary): boolean => {
    if (exec.projectId) return String(exec.projectId) === String(this.project.id);
    if (!exec.workingDirectory) return false;
    return normalizePath(exec.workingDirectory) === normalizePath(this.project.path);
  };

  loadCatalog = async (opts?: { silent?: boolean }) => {
    if (!this.props().enabled) return;
    const silent = opts?.silent ?? this.actionsList.length > 0;
    if (!silent) this.loading = true;
    try {
      const catalog = await actions.catalogForProject(this.project);
      this.actionsList = catalog.actions;
      this.workflows = catalog.workflows;
      this.warnings = catalog.warnings;
      this.filePath = catalog.filePath;
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to load actions catalog');
    } finally {
      this.loading = false;
    }
  };

  loadHistory = async (opts?: { silent?: boolean }) => {
    if (!opts?.silent) this.historyLoading = true;
    try {
      const recent = await getRecentRuns(50);
      this.runHistory = recent.filter(this.isProjectExecution);
    } catch {
      // history is best-effort
    } finally {
      if (!opts?.silent) this.historyLoading = false;
    }
  };

  openExecution = (executionId: string, opts?: { terminal?: boolean }) => {
    this.selectedRunId = executionId;
    this.runTerminal = !!opts?.terminal;
  };

  toggleSelect = (id: string) => {
    const next = new Set(this.selected);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    this.selected = next;
  };

  runSelected = async () => {
    const ids = [...this.selected];
    if (ids.length === 0) {
      toast.info('Select one or more actions to run');
      return;
    }
    await this.runTarget(ids.length === 1 ? ids[0]! : ids);
  };

  runTarget = async (target: string | string[], event?: Event) => {
    event?.preventDefault();
    event?.stopPropagation();
    if (this.running) return;

    this.running = true;
    this.pinnedRun = false;
    this.lastResult = null;
    resetDoctorRun(this.project.id);
    try {
      const result = await actions.forProject(this.project).run(target, {
        // Surface the run the moment its process spawns — not after it finishes.
        onStepStart: ({ executionId, workflowRunId }) => {
          // Show the new step's row in the tree now, not on the next poll.
          if (workflowRunId)
            void queryClient.invalidateQueries({
              queryKey: queryKeys.actions.workflowRun(workflowRunId),
            });
          if (!this.pinnedRun) this.openExecution(executionId);
          void this.loadHistory();
        },
      });
      this.lastResult = result;
      // Prefer showing the script execution record (has persisted output)
      const pick = pickRunSelection(result.steps);
      if (this.pinnedRun) {
        // keep the user's own selection
      } else if (pick) {
        this.openExecution(pick);
      } else {
        this.selectedRunId = null;
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
      await this.loadHistory();
    } catch (err) {
      toast.error(err instanceof Error ? err.message : String(err));
    } finally {
      this.running = false;
    }
  };
}
