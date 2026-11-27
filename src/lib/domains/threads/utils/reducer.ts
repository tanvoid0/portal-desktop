/**
 * Pure fold of the thread log, the TS twin of `portal-core/.../threads/apply.rs`.
 * Same events in, same state out; `reducer.test.ts` checks it against fixtures the
 * Rust side writes. Copies on every event (O(messages)), fine at human rates.
 */
import type { ChatMessage, CoderSubAgent, PendingApproval } from '$lib/domains/coder/generated';
import type {
  HostRequest,
  QueuedMessage,
  ResolvedCall,
  ThreadDeltaFrame,
  ThreadEventFrame,
  ThreadMeta,
  ThreadSnapshot,
  ThreadStatus,
  ThreadUsage,
} from '../generated';

/** Mirrors `RECENTLY_RESOLVED` in types.rs. */
const RECENTLY_RESOLVED = 32;

export interface ThreadView {
  meta: ThreadMeta;
  messages: ChatMessage[];
  /** Live-only partials of the turn in flight (deltas are never persisted). */
  streamingText: string;
  streamingThinking: string;
  status: ThreadStatus;
  approvals: PendingApproval[];
  queue: QueuedMessage[];
  hostRequests: HostRequest[];
  subAgents: CoderSubAgent[];
  subagentsRunning: number;
  recentlyResolved: ResolvedCall[];
  usage: ThreadUsage;
  todos: unknown;
  error: string | null;
  stopping: boolean;
  backendSessionId: string | null;
  /** Live-only, from heartbeat frames. */
  waitingSeconds: number;
  canRetry: boolean;
  /** The cursor: last committed seq folded in. */
  seq: number;
}

/** Idle or errored, with a failure to retry or a user turn nobody answered. */
function withRetry(v: ThreadView): ThreadView {
  const open = v.status === 'running' || v.status === 'waiting';
  v.canRetry = !open && (v.error != null || v.messages[v.messages.length - 1]?.role === 'user');
  return v;
}

export function emptyView(): ThreadView {
  return {
    meta: {
      provider: '',
      model: null,
      llm_provider: null,
      workspace_root: '',
      project_id: null,
      kind: '',
      title: '',
    },
    messages: [],
    streamingText: '',
    streamingThinking: '',
    status: 'idle',
    approvals: [],
    queue: [],
    hostRequests: [],
    subAgents: [],
    subagentsRunning: 0,
    recentlyResolved: [],
    usage: { total_cost_usd: null, model_usage: null },
    todos: null,
    error: null,
    stopping: false,
    backendSessionId: null,
    waitingSeconds: 0,
    canRetry: false,
    seq: 0,
  };
}

export function fromSnapshot(s: ThreadSnapshot): ThreadView {
  const st = s.state;
  return withRetry({
    meta: st.meta,
    messages: s.messages,
    streamingText: s.partial_text,
    streamingThinking: s.partial_thinking,
    status: st.status,
    approvals: st.approvals,
    queue: st.queue,
    hostRequests: st.host_requests,
    subAgents: st.sub_agents,
    subagentsRunning: st.subagents_running,
    recentlyResolved: st.recently_resolved,
    usage: st.usage,
    todos: st.todos,
    error: st.error,
    stopping: st.stopping,
    backendSessionId: st.backend_session_id,
    waitingSeconds: s.waited_seconds,
    canRetry: false,
    seq: s.seq,
  });
}

function upsert<T>(list: T[], item: T, same: (a: T) => boolean): T[] {
  return list.some(same) ? list.map((a) => (same(a) ? item : a)) : [...list, item];
}

function resolved(v: ThreadView, call_id: string, allow: boolean, by: string): ResolvedCall[] {
  return [...v.recentlyResolved, { call_id, allow, by }].slice(-RECENTLY_RESOLVED);
}

/** The partials and the heartbeat die with the message/turn that supersedes them (hub.rs `clears`). */
function clearStream(n: ThreadView) {
  n.streamingText = '';
  n.streamingThinking = '';
  n.waitingSeconds = 0;
}

/**
 * Folds one committed frame. `null` = a gap (`frame.seq > seq + 1`): the caller buffers it
 * and catches up. A frame at or below the cursor is a duplicate and returns `v` untouched.
 */
export function applyEvent(v: ThreadView, f: ThreadEventFrame): ThreadView | null {
  if (f.seq <= v.seq) return v;
  if (f.seq > v.seq + 1) return null;
  const e = f.event;
  const n: ThreadView = { ...v, seq: f.seq };
  switch (e.type) {
    case 'created':
      n.meta = {
        provider: e.provider,
        model: e.model,
        llm_provider: e.llm_provider,
        workspace_root: e.workspace_root,
        project_id: e.project_id,
        kind: e.kind,
        title: e.title,
      };
      n.status = 'idle';
      break;
    case 'message_appended':
      n.messages = [...v.messages, e.message];
      // A user message or tool result mid-turn leaves the stream alone.
      if (e.message.role === 'assistant') clearStream(n);
      break;
    case 'messages_rewritten':
      n.messages = [...v.messages.slice(0, e.keep), ...e.tail];
      clearStream(n);
      break;
    case 'turn_started':
      n.status = 'running';
      n.error = null;
      break;
    case 'turn_ended':
      n.error = e.error;
      n.stopping = false;
      n.approvals = [];
      n.hostRequests = [];
      n.status = e.outcome === 'error' ? 'error' : 'idle';
      clearStream(n);
      break;
    case 'stop_requested':
      n.stopping = true;
      break;
    case 'approval_requested':
      n.approvals = upsert(v.approvals, e.approval, (a) => a.call_id === e.approval.call_id);
      n.status = 'waiting';
      break;
    case 'approval_resolved':
      n.approvals = v.approvals.filter((a) => a.call_id !== e.call_id);
      n.recentlyResolved = resolved(v, e.call_id, e.allow, f.actor);
      if (n.approvals.length === 0 && v.status === 'waiting') n.status = 'running';
      break;
    case 'host_requested': {
      const r: HostRequest = { call_id: e.call_id, kind: e.kind, payload: e.payload };
      n.hostRequests = upsert(v.hostRequests, r, (h) => h.call_id === e.call_id);
      break;
    }
    case 'host_resolved':
      n.hostRequests = v.hostRequests.filter((h) => h.call_id !== e.call_id);
      n.recentlyResolved = resolved(v, e.call_id, true, f.actor);
      break;
    case 'enqueued':
      n.queue = [...v.queue, { queue_id: e.queue_id, text: e.text }];
      break;
    case 'dequeued':
      n.queue = v.queue.filter((q) => q.queue_id !== e.queue_id);
      break;
    case 'title_set':
      n.meta = { ...v.meta, title: e.title };
      break;
    case 'model_set':
      n.meta = { ...v.meta, model: e.model, llm_provider: e.llm_provider };
      break;
    case 'change_recorded':
      break; // the body lives in `coder_file_changes`
    case 'sub_agent_updated':
      n.subAgents = upsert(v.subAgents, e.sub_agent, (a) => a.id === e.sub_agent.id);
      break;
    case 'usage_updated':
      n.usage = {
        total_cost_usd: e.total_cost_usd ?? v.usage.total_cost_usd,
        model_usage: e.model_usage ?? v.usage.model_usage,
      };
      break;
    case 'todos_updated':
      n.todos = e.items;
      break;
    case 'subagents_running':
      n.subagentsRunning = e.count;
      break;
    case 'session_linked':
      n.backendSessionId = e.backend_session_id;
      break;
    case 'process_exited':
      n.approvals = [];
      clearStream(n);
      break;
  }
  return withRetry(n);
}

/**
 * Folds one live streaming chunk. `null` = re-snapshot: the chunk starts past our
 * partial, or names another seq. A chunk at `seq == cursor` that overlaps what we
 * hold (the flush just before a snapshot is already in its partial) adds only its tail.
 */
export function applyDelta(v: ThreadView, d: ThreadDeltaFrame): ThreadView | null {
  if (d.seq !== v.seq) return null;
  const thinking = d.thinking !== '';
  const have = thinking ? v.streamingThinking : v.streamingText;
  const chunk = thinking ? d.thinking : d.text;
  if (d.offset > have.length) return null;
  const fresh = chunk.slice(have.length - d.offset); // offsets are UTF-16 units, like .length
  if (fresh === '') return v;
  return thinking
    ? { ...v, streamingThinking: have + fresh }
    : { ...v, streamingText: have + fresh };
}
