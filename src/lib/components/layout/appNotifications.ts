import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { notify, initNotify } from '$lib/utils/notify';
import { agentsApi } from '$lib/domains/agents/api/agentsApi';
import { agentRuns } from '$lib/domains/agents/state/agentRuns.svelte.js';
import { agentTabPath } from '$lib/config/agents-tabs';
import type { PipelineExecution } from '$lib/domains/projects/pipelines';
import { isMobileShell } from '$lib/domains/mobile/shell';

/** Wire the app-wide notification listeners; returns a teardown that unlistens whatever resolved. */
export function startAppNotifications(): () => void {
  const unlisteners: (UnlistenFn | (() => void))[] = [];
  let disposed = false;
  // Late-resolving listeners are released at once if the shell is already gone.
  const keep = (fn: UnlistenFn | (() => void)) => {
    if (disposed) fn();
    else unlisteners.push(fn);
  };
  // One notification per execution per terminal status — the backend emits
  // this event on every step tick, not just on success/failed.
  const notifiedExecutions = new Set<string>();

  // A phone pairing over discovery/manual-IP blocks server-side for up to
  // ~170s waiting for a human to click Allow on /devices — but that page
  // is the only place that heard this event before, so a request nobody
  // is currently looking at timed out unnoticed (QA-121). Surface it
  // app-wide; the Devices page keeps its own listener for the refetch.
  // Desktop-only: pairing approval happens on the PC.
  if (!isMobileShell)
    listen<{ device_id: string; device_name: string }>('device://pending', (event) => {
      notify({
        title: 'A phone wants to pair',
        body: `${event.payload.device_name} is waiting for approval.`,
        route: '/devices',
        actionLabel: 'Review',
        duration: 0,
      });
    }).then(keep);

  // Native banner when unfocused + always-on toast for turn/permission/exit
  // events from any running agent — see domains/agents/service.rs. On
  // mobile the backend posts the native notification itself
  // (Mobile::notify in mobile/src-tauri/src/lib.rs) — initNotify() here
  // only wires tap-to-navigate, via the plugin's cross-platform onAction
  // event (#641).
  initNotify().then(keep);
  void agentRuns.ensureInit();
  agentsApi
    .onEvent(({ run_id, event }) => {
      const route = `${agentTabPath()}?session=${run_id}`;
      if (event.type === 'turn_done' && !event.is_error) {
        // Errored turns are reported by the `exited` branch below instead —
        // skip here so the user doesn't see "finished" then "failed".
        notify({ title: 'Agent finished', body: event.final_text?.slice(0, 120), route });
      } else if (event.type === 'permission_request') {
        notify({
          title: 'Agent needs your approval',
          body: `Wants to run ${event.tool_name}`,
          route,
          duration: 0,
        });
      } else if (event.type === 'exited' && event.code) {
        notify({
          title: 'Agent run failed',
          body: `Exited with code ${event.code}`,
          type: 'error',
          route,
        });
      }
    })
    .then(keep);

  // Toast when a pipeline run finishes — same event the execution monitor
  // listens to, so this reaches desktop and a phone that's in the foreground.
  // Action runs (and every workflow step, which is one) share this event but
  // have their own UI; toasting them would announce each as a pipeline.
  listen<PipelineExecution>('pipeline-execution-update', (event) => {
    const execution = event.payload;
    if (execution.kind === 'action') return;
    if (execution.status !== 'success' && execution.status !== 'failed') return;
    if (notifiedExecutions.has(execution.id)) return;
    notifiedExecutions.add(execution.id);
    notify({
      title: execution.status === 'success' ? 'Pipeline run finished' : 'Pipeline run failed',
      body: execution.error,
      type: execution.status === 'failed' ? 'error' : undefined,
      route: `/pipeline-runs?run=${execution.id}`,
    });
  }).then(keep);

  return () => {
    disposed = true;
    unlisteners.forEach((fn) => fn());
  };
}
