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
  pending: PendingApproval | null;
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
  let pending: PendingApproval | null = null;
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
      case 'permission_request':
        pending = {
          call_id: e.request_id,
          tool: e.tool_name,
          arguments: e.tool_input,
          suggested_rule: '',
          summary: summarizePermission(e.tool_name, e.tool_input),
        };
        break;
      case 'permission_resolved':
        if (pending?.call_id === e.request_id) pending = null;
        break;
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
  return { messages, streamingText, pending, running, ended, sessionId };
}
