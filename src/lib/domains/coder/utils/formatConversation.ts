import { coderSession } from '../state/coderSession.svelte.js';
import type { ChatMessage, CoderThread, PendingApproval } from '../types.js';

interface ConversationSnapshot {
  thread: CoderThread | null | undefined;
  messages: ChatMessage[];
  streamingText: string;
  pending: PendingApproval | null;
  error: string | null | undefined;
}

/** Plain-text transcript of the active session, for pasting into a bug report. */
export function formatConversationForCopy(s: ConversationSnapshot): string {
  const { thread, messages, streamingText, pending, error } = s;

  const toolNameFor = (callId: string): string => {
    for (const m of messages) {
      for (const tc of m.tool_calls ?? []) {
        if (tc.id === callId) return tc.function.name;
      }
    }
    return callId;
  };

  const lines: string[] = ['=== Coder Session ==='];
  if (thread?.title) lines.push(`Title: ${thread.title}`);
  if (coderSession.workspaceRoot) lines.push(`Workspace: ${coderSession.workspaceRoot}`);
  if (coderSession.selectedModel) lines.push(`Model: ${coderSession.selectedModel}`);
  lines.push(`Agent mode: ${coderSession.agentMode}`);
  lines.push(`Permission: ${coderSession.permissionMode}`);
  if (thread?.id) lines.push(`Thread ID: ${thread.id}`);
  lines.push('');

  for (const m of messages) {
    if (m.role === 'system') continue;

    const roleLabel =
      m.role === 'user'
        ? 'User'
        : m.role === 'assistant'
          ? 'Assistant'
          : m.role === 'tool'
            ? `Tool (${toolNameFor(m.tool_call_id ?? '')})`
            : m.role;

    lines.push(`--- ${roleLabel} ---`);
    if (m.content) lines.push(m.content);

    for (const tc of m.tool_calls ?? []) {
      lines.push('');
      lines.push(`[Tool Call: ${tc.function.name}]`);
      lines.push(tc.function.arguments);
    }
    lines.push('');
  }

  if (streamingText) {
    lines.push('--- Assistant (streaming) ---');
    lines.push(streamingText);
    lines.push('');
  }

  if (pending) {
    lines.push('--- Pending Approval ---');
    lines.push(`Tool: ${pending.tool}`);
    lines.push(`Summary: ${pending.summary}`);
    lines.push(JSON.stringify(pending.arguments, null, 2));
    lines.push('');
  }

  if (error) {
    lines.push('--- Error ---');
    lines.push(error);
  }

  return lines.join('\n').trim();
}
