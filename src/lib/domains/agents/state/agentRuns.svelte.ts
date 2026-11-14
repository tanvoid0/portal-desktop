/**
 * Module-level agent run/session state — survives page navigation, one
 * `agents://event` listener for the app's lifetime. Same shape as coder's
 * `coderSession.svelte.ts` (lazy `ensureInit`, module singleton), scaled down
 * to what the agents domain needs: per-session event logs rather than a
 * chat-message runtime per thread.
 */

import { agentsApi } from '../api/agentsApi.js';
import { eventsToMessages, type AgentRunView } from '../utils/eventsToMessages.js';
import type { AgentEvent, AgentEventSeq } from '../generated';
import { queryClient } from '$lib/domains/shared/query/queryClient';
import { queryKeys } from '$lib/domains/shared/query/keys';
import type { UnlistenFn } from '@tauri-apps/api/event';

type DeltaEvent = Extract<AgentEvent, { type: 'text_delta' } | { type: 'thinking_delta' }>;

function isDeltaEvent(e: AgentEvent): e is DeltaEvent {
  return e.type === 'text_delta' || e.type === 'thinking_delta';
}

function parentOf(e: DeltaEvent): string | null {
  return e.parent_tool_use_id ?? null;
}

/** Appends `item` to `log`, folding it into the last entry when both are the
 *  same delta type from the same parent — otherwise a token-by-token stream
 *  grows the log by one tiny event per token for as long as the run lives. */
export function appendFolded(log: AgentEventSeq[], item: AgentEventSeq): AgentEventSeq[] {
  const last = log[log.length - 1];
  if (
    last &&
    isDeltaEvent(last.event) &&
    isDeltaEvent(item.event) &&
    last.event.type === item.event.type &&
    parentOf(last.event) === parentOf(item.event)
  ) {
    const merged = { ...last.event, text: last.event.text + item.event.text };
    return [...log.slice(0, -1), { seq: item.seq, event: merged }];
  }
  return [...log, item];
}

class AgentRunsState {
  private initPromise: Promise<void> | null = null;
  private unlisten: UnlistenFn | null = null;
  /** Per-session live+persisted log, seq-ordered, delta-folded. Absence of a
   *  key means "not attached" — `append` drops events for those. */
  private logs = $state<Record<string, AgentEventSeq[]>>({});
  /** Session ids with a live backend process right now. */
  runningIds = $state<Set<string>>(new Set());
  /** Handoff slot for "open Agents and pre-fill a new run" from another page
   *  — consumed once by the run page on mount, same pattern as coder's
   *  `pendingPrompt`. */
  pendingLaunch = $state<{ cwd?: string; prompt?: string; taskId?: string } | null>(null);

  /** Idempotent — guards double-init from multiple mounts/HMR. */
  async ensureInit(): Promise<void> {
    if (!this.initPromise) {
      this.initPromise = this.runInit().catch((err) => {
        this.initPromise = null;
        throw err;
      });
    }
    await this.initPromise;
  }

  private async runInit(): Promise<void> {
    try {
      this.runningIds = new Set(await agentsApi.listRunning());
    } catch (e) {
      console.error('agents: listRunning failed', e);
    }

    this.unlisten = await agentsApi.onEvent(({ run_id, seq, event }) => {
      this.append(run_id, { seq, event });
      if (event.type === 'session_started') {
        this.markRunning(run_id, true);
        this.invalidateSessions();
      } else if (event.type === 'exited') {
        this.markRunning(run_id, false);
        this.invalidateSessions();
      }
    });
  }

  private append(sessionId: string, item: AgentEventSeq) {
    const log = this.logs[sessionId];
    if (!log) return; // not attached — attach() will pick this up from the ring/persisted base later
    this.logs[sessionId] = appendFolded(log, item);
  }

  private markRunning(id: string, running: boolean) {
    const next = new Set(this.runningIds);
    if (running) next.add(id);
    else next.delete(id);
    this.runningIds = next;
  }

  private invalidateSessions() {
    void queryClient.invalidateQueries({ queryKey: queryKeys.agents.sessions });
  }

  /** Attaches to a session's log: persisted base (`getSession`) + whatever's
   *  already in the ring past that base's max seq (`getEvents`), then further
   *  live events arrive via the shared listener. A no-op if already tracked. */
  async attach(sessionId: string): Promise<AgentEventSeq[]> {
    await this.ensureInit();
    const cached = this.logs[sessionId];
    if (cached) return cached;

    let log: AgentEventSeq[] = [];
    try {
      const detail = await agentsApi.getSession(sessionId);
      const rows = [...(detail?.events ?? [])].sort((a, b) => a.seq - b.seq);
      for (const row of rows) log = appendFolded(log, { seq: row.seq, event: row.event });
    } catch (e) {
      console.error('agents: getSession failed', e);
    }

    const maxSeq = log.length ? log[log.length - 1].seq : -1;
    try {
      const ring = await agentsApi.getEvents(sessionId);
      for (const item of ring) {
        if (item.seq > maxSeq) log = appendFolded(log, item);
      }
    } catch (e) {
      console.error('agents: getEvents failed', e);
    }

    this.logs[sessionId] = log;
    return log;
  }

  /** Reduces a session's full log through `eventsToMessages`. Empty view for an untracked session. */
  viewFor(sessionId: string): AgentRunView {
    const log = this.logs[sessionId];
    return eventsToMessages(log ? log.map((item) => item.event) : []);
  }

  pendingCountFor(sessionId: string): number {
    return this.viewFor(sessionId).pending.length;
  }

  async start(
    backendId: string,
    cwd: string,
    prompt: string,
    model?: string | null,
    taskId?: string | null
  ): Promise<string> {
    await this.ensureInit();
    const id = await agentsApi.startRun(
      backendId,
      cwd,
      prompt,
      model ?? null,
      null,
      taskId ?? null
    );
    this.markRunning(id, true);
    await this.attach(id);
    this.invalidateSessions();
    return id;
  }

  async resume(sessionId: string, text: string): Promise<string> {
    await this.ensureInit();
    const id = await agentsApi.resumeSession(sessionId, text);
    this.markRunning(id, true);
    // Resume reuses the same row/id — drop the cached (possibly "ended") log
    // so `attach` rebuilds it from the fresh persisted base + ring.
    delete this.logs[sessionId];
    await this.attach(id);
    this.invalidateSessions();
    return id;
  }

  sendUser(sessionId: string, text: string): Promise<void> {
    return agentsApi.sendUser(sessionId, text);
  }

  decide(sessionId: string, requestId: string, allow: boolean): Promise<void> {
    return agentsApi.decidePermission(sessionId, requestId, allow);
  }

  interrupt(sessionId: string): Promise<void> {
    return agentsApi.interrupt(sessionId);
  }

  cancel(sessionId: string): Promise<void> {
    return agentsApi.cancel(sessionId);
  }
}

export const agentRuns = new AgentRunsState();
