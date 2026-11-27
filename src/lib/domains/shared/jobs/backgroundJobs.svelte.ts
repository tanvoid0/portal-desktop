/**
 * App-wide registry of long-running work (disk scans, action/pipeline runs,
 * downloads). Features register what they start; `BackgroundJobsTray` renders it
 * above every page so progress stays visible while you navigate.
 */
import { listen } from '@tauri-apps/api/event';
import { getRunningRuns, onRunUpdates } from '$lib/domains/actions/runsService';
import { stopRun } from '$lib/domains/actions/runActions';
import type { ScanProgress } from '$lib/domains/disk/types';

export interface BackgroundJob {
  id: string;
  kind: 'scan' | 'run' | 'download' | 'sync';
  title: string;
  detail?: string;
  /** 0-100; null/undefined = indeterminate */
  percent?: number | null;
  href?: string;
  cancel?: () => void | Promise<void>;
}

class BackgroundJobs {
  items = $state<BackgroundJob[]>([]);

  upsert = (job: BackgroundJob) => {
    const i = this.items.findIndex((j) => j.id === job.id);
    if (i < 0) this.items = [...this.items, job];
    else this.items = this.items.map((j) => (j.id === job.id ? { ...j, ...job } : j));
  };

  patch = (id: string, p: Partial<BackgroundJob>) => {
    if (this.items.some((j) => j.id === id))
      this.items = this.items.map((j) => (j.id === id ? { ...j, ...p } : j));
  };

  remove = (id: string) => {
    if (this.items.some((j) => j.id === id)) this.items = this.items.filter((j) => j.id !== id);
  };

  /** Mirror the backend's running runs; returns teardown. Call once from the app shell. */
  start = (): (() => void) => {
    const sync = async () => {
      try {
        const runs = await getRunningRuns();
        const ids = new Set(runs.map((r) => `run:${r.id}`));
        for (const j of this.items) if (j.kind === 'run' && !ids.has(j.id)) this.remove(j.id);
        for (const r of runs) {
          this.upsert({
            id: `run:${r.id}`,
            kind: 'run',
            title: r.title,
            detail: r.projectName ?? r.kind,
            percent: r.progress?.percent ?? null,
            href: r.kind === 'action' ? '/automation' : `/pipeline-runs/${r.id}`,
            cancel: () => void stopRun(r).then(sync),
          });
        }
      } catch {
        /* backend not ready / mobile shell without the command — try again next tick */
      }
    };
    void sync();
    const stopUpdates = onRunUpdates(sync);
    // Action runs don't emit pipeline events, so poll as well.
    const timer = setInterval(sync, 4000);

    // Scan progress arrives on a global event; the scan job itself is registered by its starter.
    const unScan = listen<ScanProgress>('scan://progress', ({ payload: p }) => {
      const counting = p.phase === 'counting';
      this.patch('scan', {
        detail: counting
          ? `Counting files… ${p.scannedFiles.toLocaleString()} found`
          : `${p.scannedFiles.toLocaleString()} / ${p.totalFiles.toLocaleString()} files`,
        percent:
          counting || p.totalFiles <= 0
            ? null
            : Math.min(100, (p.scannedFiles / p.totalFiles) * 100),
      });
    });

    return () => {
      stopUpdates();
      clearInterval(timer);
      void unScan.then((off) => off());
    };
  };
}

export const backgroundJobs = new BackgroundJobs();
