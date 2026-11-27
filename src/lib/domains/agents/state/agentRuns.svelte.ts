/**
 * Module-level agent run/session state — survives page navigation, one
 * `agents://event` listener for the app's lifetime. Same shape as coder's
 * `coderSession.svelte.ts` (lazy `ensureInit`, module singleton), scaled down
 * to what the agents domain needs: per-session event logs (Timeline, Board) plus a
 * `threadStore` subscription per attached run, which is what the transcript, approvals
 * and queue read.
 */

import { agentsApi } from '../api/agentsApi.js';
import { threadStore } from '$lib/domains/threads/state/threadStore.svelte';
import type { AgentEvent, AgentEventSeq, AgentSessionDetail } from '../generated';
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

/** Persisted base log of a session. Imported transcripts only have message
 *  rows and no events — synthesize `message` events from them so they render. */
export function detailToLog(
  detail: Pick<AgentSessionDetail, 'events' | 'messages'> | null | undefined
): AgentEventSeq[] {
  // Messages and events share one seq counter (service.rs `next_seq`). An
  // assistant `Message` is persisted to both tables at the same seq; user turns
  // (the seq-0 prompt, follow-ups) and imported transcripts exist only as
  // messages. So: every event, plus every message with no event at its seq.
  const events = detail?.events ?? [];
  const eventSeqs = new Set(events.map((e) => e.seq));
  const rows: AgentEventSeq[] = [
    ...events.map((e) => ({ seq: e.seq, event: e.event })),
    ...(detail?.messages ?? [])
      .filter((m) => !eventSeqs.has(m.seq))
      .map((m) => ({
        seq: m.seq,
        event: {
          type: 'message' as const,
          role: m.role,
          text: m.text,
          parent_tool_use_id: m.parent_tool_use_id,
        },
      })),
  ].sort((a, b) => a.seq - b.seq);
  let log: AgentEventSeq[] = [];
  for (const row of rows) log = appendFolded(log, row);
  return log;
}

class AgentRunsState {
  private initPromise: Promise<void> | null = null;
  private unlisten: UnlistenFn | null = null;
  /** Per-session live+persisted log, seq-ordered, delta-folded. Absence of a
   *  key means "not attached" — `append` drops events for those. */
  private logs = $state<Record<string, AgentEventSeq[]>>({});
  // ponytail: thread subscriptions are never released, like `logs`; unsubscribe on delete if it costs.
  private subscribed = new Map<string, () => void>();
  /** Session ids with a live backend process right now. */
  runningIds = $state<Set<string>>(new Set());
  /** True once `listRunning` answered — before that `runningIds` is empty, not authoritative. */
  private runningKnown = $state(false);
  /** Handoff slot for "open Agents and pre-fill a new run" from another page
   *  — consumed once by the run page on mount, same pattern as coder's
   *  `pendingPrompt`. */
  pendingLaunch = $state<{ cwd?: string; prompt?: string; taskId?: string } | null>(null);
  /** Last focused session per backend tab. Engine-tab links (#722) carry no
   *  `?session=`, so switching Portal ↔ a backend and back restores from here. */
  lastFocused: Record<string, string | null> = {};

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

  /** A failed first answer is retried in the background (2s, 4s, 8s): until one lands
   *  `runningKnown` is false and no session reads as ended. */
  private async loadRunning(attempt = 0): Promise<void> {
    try {
      this.runningIds = new Set(await agentsApi.listRunning());
      this.runningKnown = true;
    } catch (e) {
      console.error('agents: listRunning failed', e);
      if (attempt < 3) setTimeout(() => void this.loadRunning(attempt + 1), 2000 * 2 ** attempt);
    }
  }

  private async runInit(): Promise<void> {
    await this.loadRunning();

    this.unlisten = await agentsApi.onEvent(({ run_id, seq, event }) => {
      this.append(run_id, { seq, event });
      if (event.type === 'session_started') {
        this.markRunning(run_id, true);
        void this.attach(run_id);
        this.invalidateSessions();
      } else if (event.type === 'exited') {
        this.markRunning(run_id, false);
        this.invalidateSessions();
      }
    });

    // The global approval stack reads pending approvals off every running
    // session, but `append` drops events for unattached ones — so after a
    // reload off the agents page it showed nothing (#769). Attach them all;
    // not awaited (attach awaits this init).
    for (const id of this.runningIds) void this.attach(id);
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
    void queryClient.invalidateQueries({ queryKey: queryKeys.agents.events });
  }

  /** Attaches to a session's log: persisted base (`getSession`) + whatever's
   *  already in the ring past that base's max seq (`getEvents`), then further
   *  live events arrive via the shared listener. A no-op if already tracked. */
  async attach(sessionId: string): Promise<AgentEventSeq[]> {
    await this.ensureInit();
    // A subscription whose `listen` failed (or whose thread is unknown) has no view: re-subscribe
    // on the next attach rather than trust it forever.
    if (!threadStore.view(sessionId)) {
      this.subscribed.get(sessionId)?.();
      this.subscribed.set(sessionId, threadStore.subscribe(sessionId));
    }
    const cached = this.logs[sessionId];
    if (cached) return cached;

    let log: AgentEventSeq[] = [];
    try {
      const detail = await agentsApi.getSession(sessionId);
      log = detailToLog(detail);
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

  /** Raw event log of an attached session ([] if untracked). */
  eventsFor(sessionId: string): AgentEvent[] {
    return (this.logs[sessionId] ?? []).map((item) => item.event);
  }

  /** No live backend process. Imported sessions never emit `exited`, so the
   *  event view alone can't say; the running ids can. */
  isEnded(sessionId: string): boolean {
    return this.runningKnown && !this.runningIds.has(sessionId);
  }

  pendingCountFor(sessionId: string): number {
    return threadStore.view(sessionId)?.approvals.length ?? 0;
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
