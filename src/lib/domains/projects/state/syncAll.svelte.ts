import { listen } from '@tauri-apps/api/event';
import { queryClient } from '$lib/domains/shared/query/queryClient';
import { queryKeys } from '$lib/domains/shared/query/keys';
import { invalidateProjectsList } from '$lib/domains/shared/query/invalidateProjects';
import { toast } from '$lib/utils/toast';
import { getErrorMessage } from '$lib/utils/errorMessage';
import { SYNC_ALL_PROGRESS_EVENT, syncAllProjects, cancelSyncAll } from '../api/syncAllApi';
import type { SyncAllOptions } from '../generated/SyncAllOptions';
import type { SyncAllPhase } from '../generated/SyncAllPhase';
import type { SyncAllProgress } from '../generated/SyncAllProgress';
import type { SyncAllReport } from '../generated/SyncAllReport';
import type { SyncStep } from '../generated/SyncStep';
import type { SyncStepResult } from '../generated/SyncStepResult';

export interface SyncRow {
  id: number;
  name: string;
  steps: Partial<Record<SyncStep, SyncStepResult>>;
}

/**
 * Module-level singleton so a "Sync all" run survives leaving /projects;
 * the panel is mounted by the page and just reads this.
 */
class SyncAllState {
  running = $state(false);
  open = $state(false);
  phase = $state<SyncAllPhase>('env');
  total = $state(0);
  done = $state(0);
  rows = $state<Record<number, SyncRow>>({});
  report = $state<SyncAllReport | null>(null);

  private apply(p: SyncAllProgress) {
    this.phase = p.phase;
    this.total = p.total;
    this.done = p.done;
    const e = p.project;
    if (!e) return;
    const row = (this.rows[e.id] ??= { id: e.id, name: e.name, steps: {} });
    row.steps[e.step] = { step: e.step, status: e.status, detail: e.detail };
  }

  async start(options: SyncAllOptions) {
    if (this.running) return;
    this.running = true;
    this.open = true;
    this.report = null;
    this.rows = {};
    this.phase = 'env';
    this.total = 0;
    this.done = 0;
    // Listen before invoking so no early event is lost.
    const unlisten = await listen<SyncAllProgress>(SYNC_ALL_PROGRESS_EVENT, (e) =>
      this.apply(e.payload)
    );
    try {
      this.report = await syncAllProjects(options);
      this.phase = 'done';
    } catch (err) {
      toast.error(`Sync all failed: ${getErrorMessage(err)}`);
    } finally {
      unlisten();
      this.running = false;
      invalidateProjectsList(queryClient);
      void queryClient.invalidateQueries({ queryKey: queryKeys.projects.envBackup });
      void queryClient.invalidateQueries({ queryKey: queryKeys.sync.all });
    }
  }

  async cancel() {
    await cancelSyncAll();
  }
}

export const syncAll = new SyncAllState();
