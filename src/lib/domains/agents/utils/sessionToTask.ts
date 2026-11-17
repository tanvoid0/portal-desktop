import type { Task } from '$lib/domains/tasks/types';
import type { KanbanColumnDef } from '$lib/domains/tasks/state/taskUi.svelte';
import type { AgentEvent, AgentSessionSummary } from '../generated';
import { basename } from './groupSessions.js';

/** Agents board columns: session state mapped onto the task statuses the shared board filters on. */
export const AGENT_COLUMNS: KanbanColumnDef[] = [
  {
    id: 'pending',
    title: 'Waiting',
    color: 'bg-amber-100 dark:bg-amber-900/20',
    hint: 'Running, blocked on a permission decision from you',
    match: (t) => t.status === 'pending',
  },
  {
    id: 'in-progress',
    title: 'Working',
    color: 'bg-blue-100 dark:bg-blue-900/20',
    hint: 'Running and making progress',
    match: (t) => t.status === 'in-progress',
  },
  {
    id: 'completed',
    title: 'Completed',
    color: 'bg-green-100 dark:bg-green-900/20',
    hint: 'Exited cleanly',
    match: (t) => t.status === 'completed',
    newestFirst: true,
  },
  {
    id: 'cancelled',
    title: 'Failed',
    color: 'bg-red-100 dark:bg-red-900/20',
    hint: 'Exited with an error, or interrupted before finishing',
    match: (t) => t.status === 'cancelled',
    newestFirst: true,
  },
];

export interface SessionTaskContext {
  /** Does this running session have an approval awaiting a decision? */
  isWaiting: (sessionId: string) => boolean;
  /** Live event log; only attached (running) sessions have one. */
  events: (sessionId: string) => AgentEvent[];
  backendName: (backendId: string) => string;
}

function status(s: AgentSessionSummary, waiting: boolean): Task['status'] {
  if (s.status === 'running') return waiting ? 'pending' : 'in-progress';
  return s.status === 'exited' && !s.exit_code ? 'completed' : 'cancelled';
}

/** Subagent dispatches in an event log, as `[tool_use_id, name, finished]`. */
function subagents(events: AgentEvent[]) {
  const subs = new Map<string, { name: string; done: boolean }>();
  for (const e of events) {
    if (e.type === 'subagent_started') {
      subs.set(e.tool_use_id, { name: e.name ?? 'Subagent', done: false });
    } else if (e.type === 'subagent_finished') {
      const sub = subs.get(e.tool_use_id);
      if (sub) sub.done = true;
    }
  }
  return subs;
}

/** Sessions -> board cards, each followed by its subagents as subtask cards. */
export function sessionsToTasks(sessions: AgentSessionSummary[], ctx: SessionTaskContext): Task[] {
  return sessions.flatMap((s) => {
    const started = new Date(s.started_at);
    const ended = s.ended_at ? new Date(s.ended_at) : undefined;
    const st = status(s, s.status === 'running' && ctx.isWaiting(s.id));
    const parent: Task = {
      id: s.id,
      title: s.title || s.backend_id,
      description: basename(s.cwd),
      status: st,
      priority: 'medium',
      assignee: ctx.backendName(s.backend_id),
      createdAt: started,
      updatedAt: ended ?? started,
      completedAt: ended,
    };
    const subs = [...subagents(ctx.events(s.id))].map(([id, sub]): Task => ({
      id: `${s.id}:${id}`,
      parentId: s.id,
      title: sub.name,
      // A subagent can't outlive its parent: interrupted/crashed parent => no more finish events.
      status: sub.done || s.status !== 'running' ? 'completed' : 'in-progress',
      priority: 'medium',
      type: 'Subtask',
      createdAt: started,
      updatedAt: started,
    }));
    return [parent, ...subs];
  });
}
