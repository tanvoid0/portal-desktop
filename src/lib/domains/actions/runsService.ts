/** The unified run list: action runs, pipeline runs and releases as one RunSummary[]. */
import { listen } from '@tauri-apps/api/event';
import { invokeClient } from '$lib/utils/invokeClient';
import type { RunSummary } from './types';

export const getRecentRuns = (limit: number) =>
  invokeClient.post<RunSummary[]>('get_recent_runs', { limit });

export const getRunningRuns = () => invokeClient.post<RunSummary[]>('get_running_runs');

/** Call `cb` (at most ~1/s, trailing) whenever a pipeline run reports progress.
 *  Returns the unsubscribe function. */
export function onRunUpdates(cb: () => void): () => void {
  let timer: ReturnType<typeof setTimeout> | null = null;
  const unlisten = listen('pipeline-execution-update', () => {
    timer ??= setTimeout(() => {
      timer = null;
      cb();
    }, 1000);
  });
  return () => {
    if (timer) clearTimeout(timer);
    void unlisten.then((fn) => fn());
  };
}
