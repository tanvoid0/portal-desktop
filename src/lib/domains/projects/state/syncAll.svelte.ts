import { listen } from '@tauri-apps/api/event';
import { queryClient } from '$lib/domains/shared/query/queryClient';
import { queryKeys } from '$lib/domains/shared/query/keys';
import { invalidateProjectsList } from '$lib/domains/shared/query/invalidateProjects';
import { vaultGate } from '$lib/domains/credentials';
import { backgroundJobs } from '$lib/domains/shared/jobs/backgroundJobs.svelte';
import { toast } from '$lib/utils/toast';
import { getErrorMessage } from '$lib/utils/errorMessage';
import {
  SYNC_ALL_PROGRESS_EVENT,
  syncAllProjects,
  cancelSyncAll,
  syncApplyResolution,
} from '../api/syncAllApi';
import { afterApply, emptyPlan } from '../utils/syncTriage';
import type { SyncResolution } from '../generated/SyncResolution';
import type { SyncResolveResult } from '../generated/SyncResolveResult';
import type { SyncAllPhase } from '../generated/SyncAllPhase';
import type { SyncAllProgress } from '../generated/SyncAllProgress';
import type { SyncAllReport } from '../generated/SyncAllReport';
import type { EnvItemResult } from '../generated/EnvItemResult';
import type { SyncProjectResult } from '../generated/SyncProjectResult';
import type { SyncStep } from '../generated/SyncStep';
import type { SyncStepResult } from '../generated/SyncStepResult';

export interface SyncRow {
  id: number;
  name: string;
  /** Only known once the final report lands. */
  path?: string;
  steps: Partial<Record<SyncStep, SyncStepResult>>;
}

export const PHASE_LABEL: Record<SyncAllPhase, string> = {
  env: 'Backing up env files',
  git: 'Syncing repositories',
  snapshot: 'Uploading snapshot',
  done: 'Done',
};

const REPORT_KEY = 'portal.syncAll.report';
const JOB_ID = 'sync-all';

/** The last run outlives an app restart. A bad or absent entry is just "never run". */
function readReport(): SyncAllReport | null {
  try {
    const saved = JSON.parse(localStorage.getItem(REPORT_KEY) ?? 'null');
    return Array.isArray(saved?.projects) && saved.snapshot ? saved : null;
  } catch {
    return null;
  }
}

/**
 * Module-level singleton: the run, its result and how the page was filtered all
 * survive navigating away from /projects/sync.
 */
class SyncAllState {
  running = $state(false);
  phase = $state<SyncAllPhase>('env');
  total = $state(0);
  done = $state(0);
  live = $state<Record<number, SyncRow>>({});
  report = $state<SyncAllReport | null>(readReport());
  /** Projects with a re-sync, push or rebase in flight. */
  busy = $state<number[]>([]);
  tab = $state<'repos' | 'env' | 'drive'>('repos');
  view = $state<'attention' | 'all'>('attention');
  search = $state('');
  /** Projects whose env files are expanded. */
  openEnv = $state<number[]>([]);
  envFilter = $state('');
  envLastRun = $state<EnvItemResult[]>([]);

  toggleEnv(id: number) {
    this.openEnv = this.openEnv.includes(id)
      ? this.openEnv.filter((x) => x !== id)
      : [...this.openEnv, id];
  }

  private save() {
    try {
      localStorage.setItem(REPORT_KEY, JSON.stringify(this.report));
    } catch {
      // Full or unavailable storage only costs the history.
    }
  }

  private apply(p: SyncAllProgress) {
    this.phase = p.phase;
    this.total = p.total;
    this.done = p.done;
    backgroundJobs.patch(JOB_ID, {
      detail: `${PHASE_LABEL[p.phase]} ${p.done}/${p.total}`,
      percent: p.total > 0 ? (p.done / p.total) * 100 : null,
    });
    const e = p.project;
    if (!e) return;
    // Read back after `??=`: it hands back the plain object, not the $state proxy.
    this.live[e.id] ??= { id: e.id, name: e.name, steps: {} };
    this.live[e.id].steps[e.step] = { step: e.step, status: e.status, detail: e.detail };
  }

  private refreshQueries() {
    invalidateProjectsList(queryClient);
    void queryClient.invalidateQueries({ queryKey: queryKeys.projects.envBackup });
    void queryClient.invalidateQueries({ queryKey: queryKeys.sync.all });
  }

  async start(snapshotPassword: string | null): Promise<boolean> {
    if (this.running) return false;
    this.running = true;
    this.live = {};
    this.phase = 'env';
    this.total = 0;
    this.done = 0;
    const previous = this.report;
    this.report = null;
    backgroundJobs.upsert({
      id: JOB_ID,
      kind: 'sync',
      title: 'Syncing projects',
      detail: PHASE_LABEL.env,
      percent: null,
      href: '/projects/sync',
      cancel: () => this.cancel(),
    });
    let unlisten: (() => void) | undefined;
    try {
      // Listen before invoking so no early event is lost.
      unlisten = await listen<SyncAllProgress>(SYNC_ALL_PROGRESS_EVENT, (e) =>
        this.apply(e.payload)
      );
      this.report = await syncAllProjects({ snapshotPassword, projectIds: null });
      this.phase = 'done';
      this.save();
      return true;
    } catch (err) {
      this.report = previous;
      toast.error(`Sync failed: ${getErrorMessage(err)}`);
      return false;
    } finally {
      unlisten?.();
      this.running = false;
      backgroundJobs.remove(JOB_ID);
      this.refreshQueries();
    }
  }

  async cancel() {
    await cancelSyncAll();
  }

  private async whileBusy(id: number, work: () => Promise<void>) {
    if (this.running || this.busy.includes(id)) return;
    this.busy = [...this.busy, id];
    try {
      await work();
    } finally {
      this.busy = this.busy.filter((x) => x !== id);
    }
  }

  /** Apply a plan in one repository and fold the outcome into its row. */
  async runPlan(id: number, plan: SyncResolution): Promise<SyncResolveResult> {
    const result = await syncApplyResolution(id, plan);
    const row = this.project(id);
    if (row) this.replace(afterApply(row, plan, result));
    invalidateProjectsList(queryClient);
    return result;
  }

  /** The one-click fixes that need no review: push, rebase. */
  quick(id: number, action: Partial<Pick<SyncResolution, 'push' | 'rebase'>>) {
    return this.whileBusy(id, async () => {
      try {
        const result = await this.runPlan(id, { ...emptyPlan(), ...action });
        if (result.error) toast.error(result.error);
        else toast.success(result.done.join(', ') || 'Done');
      } catch (err) {
        toast.error(getErrorMessage(err));
      }
    });
  }

  /** Sync one project (env files and repository) and fold it into the report. */
  recheck(id: number) {
    return this.whileBusy(id, () => this.syncOne(id));
  }

  private async syncOne(id: number) {
    // A cancelled unlock still syncs the repository; the note says env was skipped.
    await vaultGate.ensureUnlocked('Unlock to back up env files.');
    try {
      const fresh = await syncAllProjects({ snapshotPassword: null, projectIds: [id] });
      if (fresh.envNote) toast.warning(fresh.envNote);
      this.merge(fresh, id);
    } catch (err) {
      toast.error(`Could not sync: ${getErrorMessage(err)}`);
    } finally {
      this.refreshQueries();
    }
  }

  /** A partial run updates its own rows and leaves the rest of the report alone. */
  private merge(fresh: SyncAllReport, id: number) {
    const rows = new Map(this.report?.projects.map((p) => [p.id, p]));
    for (const p of fresh.projects) {
      // Env could not run, or this row only shares `id`'s repository (env ran for
      // `id` alone): keep what an earlier run said about it.
      const env =
        fresh.envNote || p.id !== id
          ? (rows.get(p.id)?.steps.filter((s) => s.step === 'env') ?? [])
          : [];
      rows.set(p.id, { ...p, steps: [...env, ...p.steps] });
    }
    this.report = { ...(this.report ?? fresh), projects: [...rows.values()] };
    this.save();
  }

  project(id: number): SyncProjectResult | undefined {
    return this.report?.projects.find((p) => p.id === id);
  }

  replace(project: SyncProjectResult) {
    if (!this.report) return;
    this.report = {
      ...this.report,
      projects: this.report.projects.map((p) => (p.id === project.id ? project : p)),
    };
    this.save();
  }

  /** The final report once it lands; live rows until then. */
  rows = $derived<SyncRow[]>(
    this.report
      ? this.report.projects.map((p) => ({
          id: p.id,
          name: p.name,
          path: p.path,
          steps: Object.fromEntries(p.steps.map((s) => [s.step, s])),
        }))
      : Object.values(this.live)
  );
}

export const syncAll = new SyncAllState();
