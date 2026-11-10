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
    } catch (e) {
      error = String(e);
      toast.error(error);
    } finally {
      starting = false;
    }
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

  async function decide(allow: boolean) {
    if (!runId || !view.pending) return;
    await agentsApi.decidePermission(runId, view.pending.call_id, allow);
  }

  async function interrupt() {
    if (!runId) return;
    await agentsApi.interrupt(runId);
  }

  async function cancel() {
    if (!runId) return;
    await agentsApi.cancel(runId);
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
    sendUser,
    decide,
    interrupt,
    cancel,
    destroy,
  };
}
