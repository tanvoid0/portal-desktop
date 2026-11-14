// Projects the backend-agnostic `AgentEvent` stream onto the `ChatMessage[]` /
// `PendingApproval` shapes the existing coder UI (`CoderFeed`, `ApprovalBanner`)
// already renders, so the Run launcher reuses those components instead of a
// parallel chat UI.
//
// `AgentEvent::ToolCall` carries Claude Code's *native* tool names/args
// (`Bash` + `command`, `Read` + `file_path`, ...) — `mapToolCall` below
// translates those onto the synthetic shape `ToolCallCard.svelte` /
// `toolCallDisplay.ts` already render for Coder's own tool calls
// (`run_command` + `command`, `read_file` + `path`, ...).

import type { AgentEvent, AgentEventRow } from '../generated';
import type { ChatMessage, PendingApproval } from '$lib/domains/coder/types.js';

export interface AgentRunView {
  messages: ChatMessage[];
  /** In-flight assistant text not yet folded into `messages` (a `message` event does that). */
  streamingText: string;
  /** Every approval still awaiting a decision — a subagent can request its own
   *  concurrently with the main turn's, so this is a list, not one slot. */
  pending: PendingApproval[];
  /** A turn is in flight (no tool call/final text yet to show instead). */
  running: boolean;
  /** The backend process itself has exited — the run is over, not just the turn. */
  ended: boolean;
  sessionId: string | null;
  /** Last `Todos` event's raw item list, shape owned by the backend/CLI. */
  todos: unknown;
  /** Running total from the latest `Usage` event, if the backend reports one. */
  totalCostUsd: number | null;
  /** Subagents currently dispatched (`SubagentStarted` seen, no matching `SubagentFinished` yet). */
  subagentsRunning: number;
}

function summarizePermission(toolName: string, input: unknown): string {
  if (input && typeof input === 'object') {
    const obj = input as Record<string, unknown>;
    if (typeof obj.command === 'string') return obj.command;
    if (typeof obj.file_path === 'string') return obj.file_path;
  }
  return toolName;
}

/** Native Claude Code tool name/input -> Coder's synthetic tool-call shape. */
function mapToolCall(
  name: string,
  input: unknown
): { name: string; args: Record<string, unknown> } {
  const obj = (input && typeof input === 'object' ? input : {}) as Record<string, unknown>;
  switch (name) {
    case 'Bash':
      return { name: 'run_command', args: { command: obj.command } };
    case 'Read':
      return { name: 'read_file', args: { path: obj.file_path } };
    case 'Write':
      return { name: 'write_file', args: { path: obj.file_path, content: obj.content } };
    case 'Edit':
      return {
        name: 'edit_file',
        args: {
          path: obj.file_path,
          old_string: obj.old_string,
          new_string: obj.new_string,
          replace_all: obj.replace_all,
        },
      };
    case 'Grep':
    case 'Glob':
      return { name: 'search_files', args: { query: obj.pattern, path: obj.path } };
    case 'LS':
      return { name: 'list_dir', args: { path: obj.path } };
    default:
      // MultiEdit and anything unrecognized (incl. future tools) — Coder has
      // no matching synthetic branch, so pass the native shape straight
      // through; ToolCallCard's default branch dumps raw args rather than
      // rendering nothing.
      return { name, args: obj };
  }
}

/** `parent_tool_use_id` lives only on the event types a subagent turn streams. */
function parentToolUseId(e: AgentEvent): string | null {
  switch (e.type) {
    case 'text_delta':
    case 'thinking_delta':
    case 'message':
    case 'tool_call':
      return e.parent_tool_use_id ?? null;
    default:
      return null;
  }
}

/** Pure reducer over the full event history — cheap enough to rerun on every append (ring-capped at 500). */
export function eventsToMessages(events: AgentEvent[]): AgentRunView {
  const messages: ChatMessage[] = [];
  let streamingText = '';
  let thinkingText = '';
  const pending: PendingApproval[] = [];
  let running = false;
  let ended = false;
  let sessionId: string | null = null;
  let todos: unknown = null;
  let totalCostUsd: number | null = null;
  let subagentsRunning = 0;

  for (const e of events) {
    // A subagent streams the same event types as the main turn, tagged by
    // `parent_tool_use_id` — still visible in the Timeline view
    // (`eventTimeline.ts`), just not folded into this feed; no nested
    // subagent transcript rendering in this pass.
    if (parentToolUseId(e)) continue;

    switch (e.type) {
      case 'session_started':
        sessionId = e.session_id;
        running = true;
        break;
      case 'text_delta':
        streamingText += e.text;
        running = true;
        break;
      case 'thinking_delta':
        thinkingText += e.text;
        running = true;
        break;
      case 'message':
        messages.push({ role: e.role, content: e.text });
        streamingText = '';
        thinkingText = '';
        break;
      case 'tool_call': {
        const { name, args } = mapToolCall(e.name, e.input);
        messages.push({
          role: 'assistant',
          // Thinking that led straight into this call rides along as the
          // message's content — `buildFeedBlocks` renders content +
          // tool_calls together as a 'thought' block. Thinking with no
          // following tool call has nowhere to go yet and is dropped, same
          // ceiling Coder's own feed has today.
          content: thinkingText || null,
          tool_calls: [
            {
              id: e.id,
              type: 'function',
              function: { name, arguments: JSON.stringify(args) },
            },
          ],
        });
        streamingText = '';
        thinkingText = '';
        break;
      }
      case 'tool_result':
        messages.push({
          role: 'tool',
          // `isToolResultFailure` detects a failure by this same "Error: "
          // prefix for Coder's own tool calls — match it so a failed native
          // call renders the same way.
          content: e.is_error ? `Error: ${e.content}` : e.content,
          tool_call_id: e.tool_use_id,
        });
        break;
      case 'subagent_started':
        subagentsRunning++;
        break;
      case 'subagent_finished':
        subagentsRunning = Math.max(0, subagentsRunning - 1);
        break;
      case 'todos':
        todos = e.items;
        break;
      case 'usage':
        if (e.total_cost_usd != null) totalCostUsd = e.total_cost_usd;
        break;
      case 'permission_request': {
        const existing = pending.findIndex((p) => p.call_id === e.request_id);
        const approval: PendingApproval = {
          call_id: e.request_id,
          tool: e.tool_name,
          arguments: e.tool_input,
          suggested_rule: '',
          summary: summarizePermission(e.tool_name, e.tool_input),
        };
        if (existing >= 0) pending[existing] = approval;
        else pending.push(approval);
        break;
      }
      case 'permission_resolved': {
        const idx = pending.findIndex((p) => p.call_id === e.request_id);
        if (idx >= 0) pending.splice(idx, 1);
        break;
      }
      case 'turn_done':
        running = false;
        break;
      case 'exited':
        running = false;
        ended = true;
        break;
      default:
        break;
    }
  }
  // A pending approval pauses the turn — `ApprovalBanner`'s buttons are
  // disabled via `busy={running}` (shared with the coder domain, where
  // `running` already means "not safe to submit a decision right now"),
  // so `running` must go false for as long as anything is unresolved or the
  // decision buttons can never be clicked. See QA-110.
  if (pending.length > 0) running = false;
  return {
    messages,
    streamingText,
    pending,
    running,
    ended,
    sessionId,
    todos,
    totalCostUsd,
    subagentsRunning,
  };
}

/** Persisted rows -> the same `AgentEvent[]` the live stream reduces, seq-ordered. */
export function eventRowsToEvents(rows: AgentEventRow[]): AgentEvent[] {
  return [...rows].sort((a, b) => a.seq - b.seq).map((row) => row.event);
}
