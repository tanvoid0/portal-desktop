import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { notify, initNotify, dismissNotification } from '$lib/utils/notify';
import { permissionFact, pipelineFacts } from '$lib/domains/notifications/utils/facts';
import { agentsApi } from '$lib/domains/agents/api/agentsApi';
import { agentRuns } from '$lib/domains/agents/state/agentRuns.svelte.js';
import { threadApi } from '$lib/domains/threads/api/threadApi';
import { PORTAL_ENGINE_TAB, agentTabPath } from '$lib/config/agents-tabs';
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
  // app-wide: `PairRequestGate` pops the Allow/Deny modal in-window, so this
  // is only the native banner for when the window isn't focused (focusing the
  // window then shows the modal). Desktop-only: approval happens on the PC.
  if (!isMobileShell)
    listen<{ device_id: string; device_name: string }>('device://pending', (event) => {
      if (document.hasFocus()) return;
      notify({
        title: 'A phone wants to pair',
        body: `${event.payload.device_name} is waiting for approval.`,
        duration: 0,
        route: '/devices',
        actionLabel: 'Review',
        source: 'device',
        facts: [{ kind: 'device', value: event.payload.device_name }],
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
      const about = { route, source: 'agent' as const, runId: run_id };
      if (event.type === 'turn_done' && !event.is_error) {
        // Errored turns are reported by the `exited` branch below instead —
        // skip here so the user doesn't see "finished" then "failed".
        notify({
          ...about,
          title: 'Agent finished',
          body: event.final_text?.slice(0, 120),
          type: 'success',
        });
      } else if (event.type === 'exited' && event.code) {
        notify({
          ...about,
          title: 'Agent run failed',
          body: `Exited with code ${event.code}`,
          type: 'error',
          facts: [{ kind: 'exit', value: String(event.code) }],
        });
      }
    })
    .then(keep);

  // One alert per approval, for the Portal coder and every CLI agent alike: the thread log's
  // `approval_requested` is the single source (the legacy `agents://event` one is not used).
  // Keyed per thread (call ids restart per thread); a re-request of the same call is not a new
  // ask, and the entry goes (with its toast) once the call is resolved. Value = the route.
  const openApprovals = new Map<string, string>();
  threadApi
    .onEvent((f) => {
      const e = f.event;
      if (e.type !== 'approval_requested' && e.type !== 'approval_resolved') return;
      const callId = e.type === 'approval_requested' ? e.approval.call_id : e.call_id;
      const key = `${f.thread_id}:${callId}`;
      if (e.type === 'approval_resolved') {
        const route = openApprovals.get(key);
        if (route) dismissNotification(route);
        openApprovals.delete(key);
        return;
      }
      if (openApprovals.has(key)) return;
      const a = e.approval;
      const portal = f.provider === 'portal';
      const route = portal
        ? `${PORTAL_ENGINE_TAB.url}?id=${f.thread_id}`
        : `${agentTabPath(f.provider)}?session=${f.thread_id}`;
      openApprovals.set(key, route);
      notify({
        route,
        source: 'agent',
        runId: f.thread_id,
        title: portal ? 'Portal needs your approval' : 'Agent needs your approval',
        body: `Wants to run ${a.tool}`,
        type: 'warning',
        duration: 0,
        requestId: a.call_id,
        facts: permissionFact(a.tool, a.arguments),
      });
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
      type: execution.status === 'failed' ? 'error' : 'success',
      route: `/pipeline-runs?run=${execution.id}`,
      source: 'pipeline',
      projectId: execution.projectId,
      facts: pipelineFacts(execution),
    });
  }).then(keep);

  return () => {
    disposed = true;
    unlisteners.forEach((fn) => fn());
  };
}
