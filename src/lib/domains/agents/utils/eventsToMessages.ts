// Projects the backend-agnostic `AgentEvent` stream onto the `ChatMessage[]` /
// `PendingApproval` shapes the existing coder UI (`CoderFeed`, `ApprovalBanner`)
// already renders, so the Run launcher reuses those components instead of a
// parallel chat UI.

import type { AgentEvent } from '../generated';
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
}

function summarizePermission(toolName: string, input: unknown): string {
  if (input && typeof input === 'object') {
    const obj = input as Record<string, unknown>;
    if (typeof obj.command === 'string') return obj.command;
    if (typeof obj.file_path === 'string') return obj.file_path;
  }
  return toolName;
}

/** Pure reducer over the full event history — cheap enough to rerun on every append (ring-capped at 500). */
export function eventsToMessages(events: AgentEvent[]): AgentRunView {
  const messages: ChatMessage[] = [];
  let streamingText = '';
  const pending: PendingApproval[] = [];
  let running = false;
  let ended = false;
  let sessionId: string | null = null;

  for (const e of events) {
    switch (e.type) {
      case 'session_started':
        sessionId = e.session_id;
        running = true;
        break;
      case 'text_delta':
        streamingText += e.text;
        running = true;
        break;
      case 'message':
        messages.push({ role: e.role, content: e.text });
        streamingText = '';
        break;
      case 'tool_call':
        messages.push({
          role: 'assistant',
          content: null,
          tool_calls: [
            {
              id: e.id,
              type: 'function',
              function: { name: e.name, arguments: JSON.stringify(e.input) },
            },
          ],
        });
        streamingText = '';
        break;
      case 'tool_result':
        messages.push({ role: 'tool', content: e.content, tool_call_id: e.tool_use_id });
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
  return { messages, streamingText, pending, running, ended, sessionId };
}
