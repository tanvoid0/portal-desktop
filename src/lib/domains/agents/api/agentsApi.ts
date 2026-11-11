import { invokeClient } from '$lib/utils/invokeClient';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type {
  AgentEvent,
  AgentSessionDetail,
  AgentSessionSummary,
  BackendInfo,
} from '../generated';

/** Thin wrapper over the `agents_*` Tauri commands — the backend trait, the
 * process spawn, and the permission-hook relay all live in Rust. */
export const agentsApi = {
  listBackends(): Promise<BackendInfo[]> {
    return invokeClient.post<BackendInfo[]>('agents_list_backends');
  },

  startRun(
    backendId: string,
    cwd: string,
    prompt: string,
    model?: string | null,
    resumeSessionId?: string | null
  ): Promise<string> {
    return invokeClient.post<string>('agents_start_run', {
      backendId,
      cwd,
      prompt,
      model: model ?? null,
      resumeSessionId: resumeSessionId ?? null,
    });
  },

  sendUser(runId: string, text: string): Promise<void> {
    return invokeClient.post<void>('agents_send_user', { runId, text });
  },

  decidePermission(runId: string, requestId: string, allow: boolean): Promise<void> {
    return invokeClient.post<void>('agents_decide_permission', { runId, requestId, allow });
  },

  interrupt(runId: string): Promise<void> {
    return invokeClient.post<void>('agents_interrupt', { runId });
  },

  cancel(runId: string): Promise<void> {
    return invokeClient.post<void>('agents_cancel', { runId });
  },

  /** Ring-buffer snapshot, for a UI that attaches after the run already started. */
  getEvents(runId: string): Promise<AgentEvent[]> {
    return invokeClient.post<AgentEvent[]>('agents_get_events', { runId });
  },

  /** Run ids with a live backend process right now, so a reloaded
   *  `/agents/run` can reattach instead of showing a blank launcher (QA-127). */
  listRunning(): Promise<string[]> {
    return invokeClient.post<string[]>('agents_list_running');
  },

  /** Live event stream for every run — filter by `run_id` in the callback. */
  onEvent(cb: (p: { run_id: string; event: AgentEvent }) => void): Promise<UnlistenFn> {
    return listen<{ run_id: string; event: AgentEvent }>('agents://event', (e) => cb(e.payload));
  },

  /** Persisted sessions, most recent first. */
  listSessions(): Promise<AgentSessionSummary[]> {
    return invokeClient.post<AgentSessionSummary[]>('agents_list_sessions');
  },

  /** One session's full persisted transcript + event log, in order. */
  getSession(sessionId: string): Promise<AgentSessionDetail | null> {
    return invokeClient.post<AgentSessionDetail | null>('agents_get_session', { sessionId });
  },
};
