/**
 * Live state for one Run launcher page instance. Not a module-level
 * singleton like coder's thread store (P1 has no sessions list to switch
 * between) — the page creates one via `createAgentRunState()` on mount and
 * tears it down on destroy.
 */

import type { UnlistenFn } from '@tauri-apps/api/event';
import { agentsApi } from '../api/agentsApi.js';
import type { AgentEvent } from '../generated';
import { eventsToMessages, type AgentRunView } from '../utils/eventsToMessages.js';
import { toast } from '$lib/utils/toast';

export function createAgentRunState() {
  let runId: string | null = $state(null);
  let events: AgentEvent[] = $state([]);
  let starting = $state(false);
  let error: string | null = $state(null);
  let unlisten: UnlistenFn | null = null;

  const view = $derived.by<AgentRunView>(() => eventsToMessages(events));

  async function ensureListening() {
    if (unlisten) return;
    unlisten = await agentsApi.onEvent(({ run_id, event }) => {
      if (run_id !== runId) return;
      events = [...events, event];
    });
  }

  async function start(backendId: string, cwd: string, prompt: string, model?: string | null) {
    starting = true;
    error = null;
    try {
      await ensureListening();
      events = [];
      runId = await agentsApi.startRun(backendId, cwd, prompt, model);
      // The pump can emit before this promise resolves; the listener drops
      // those because `runId` was still null when they arrived. Backfill from
      // the backend's own ring buffer now that we have an id. The buffer is
      // ordered and append-only, so if live events already landed while this
      // call was in flight, they're the tail past where the snapshot ends.
      // ponytail: no sequence id on AgentEvent to dedupe by value instead — fine
      // while the only gap is this one startup race.
      try {
        const backfill = await agentsApi.getEvents(runId);
        events =
          events.length > backfill.length
            ? [...backfill, ...events.slice(backfill.length)]
            : backfill;
      } catch {
        // Backfill is best-effort — a failed snapshot shouldn't fail the run.
      }
    } catch (e) {
      error = String(e);
      toast.error(error);
    } finally {
      starting = false;
    }
  }

  /** This page has no persisted run id of its own (unlike coder's thread
   *  list) — on mount, ask the backend if a run is still alive and, if so,
   *  reattach to it instead of showing a blank launcher while it sits
   *  blocked on a decision nobody can see anymore (QA-127). The ring buffer
   *  already carries any unresolved `permission_request`, so no separate
   *  "pending" fetch is needed — `eventsToMessages` derives it the same way
   *  it does for a live run. Picks the first if more than one is live; this
   *  page only ever starts one of its own.
   */
  async function rehydrate() {
    const running = await agentsApi.listRunning().catch(() => []);
    const liveRunId = running[0];
    if (!liveRunId) return;
    await ensureListening();
    runId = liveRunId;
    events = await agentsApi.getEvents(liveRunId).catch(() => []);
  }

  async function sendUser(text: string) {
    if (!runId) return;
    try {
      await agentsApi.sendUser(runId, text);
    } catch (e) {
      error = String(e);
      toast.error(error);
    }
  }

  async function decide(requestId: string, allow: boolean) {
    if (!runId) return;
    await agentsApi.decidePermission(runId, requestId, allow);
  }

  async function interrupt() {
    if (!runId) return;
    await agentsApi.interrupt(runId);
  }

  async function cancel() {
    if (!runId) return;
    await agentsApi.cancel(runId);
  }

  /** Back to a clean launcher — unlisten so the next `start()` re-attaches. */
  function reset() {
    unlisten?.();
    unlisten = null;
    runId = null;
    events = [];
    error = null;
  }

  function destroy() {
    unlisten?.();
    unlisten = null;
  }

  return {
    get runId() {
      return runId;
    },
    get starting() {
      return starting;
    },
    get error() {
      return error;
    },
    get view() {
      return view;
    },
    start,
    rehydrate,
    sendUser,
    decide,
    interrupt,
    cancel,
    reset,
    destroy,
  };
}
