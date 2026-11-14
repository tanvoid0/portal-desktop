import { describe, expect, it } from 'vitest';
import type { AgentEvent, AgentEventRow } from '../generated';
import { eventRowsToEvents, eventsToMessages } from './eventsToMessages';

const fixture: AgentEvent[] = [
  { type: 'session_started', session_id: 's1' },
  { type: 'text_delta', text: 'Lis' },
  { type: 'text_delta', text: 'ting files' },
  { type: 'message', role: 'assistant', text: 'Listing files' },
  { type: 'tool_call', id: 'call1', name: 'Bash', input: { command: 'ls' } },
  {
    type: 'permission_request',
    request_id: 'call1',
    tool_name: 'Bash',
    tool_input: { command: 'ls' },
    suggestions: [],
  },
  { type: 'permission_resolved', request_id: 'call1', allowed: true },
  { type: 'tool_result', tool_use_id: 'call1', content: 'command not found', is_error: true },
  { type: 'message', role: 'assistant', text: 'subagent note', parent_tool_use_id: 'call1' },
  { type: 'exited', code: 0 },
];

describe('eventsToMessages', () => {
  it('maps a native Bash tool call onto the synthetic run_command shape', () => {
    const view = eventsToMessages(fixture);
    const toolCallMessage = view.messages.find((m) => m.tool_calls?.length);
    expect(toolCallMessage).toBeDefined();
    const call = toolCallMessage!.tool_calls![0];
    expect(call.function.name).toBe('run_command');
    expect(JSON.parse(call.function.arguments)).toMatchObject({ command: 'ls' });
  });

  it('prefixes a failed tool result with "Error: "', () => {
    const view = eventsToMessages(fixture);
    const result = view.messages.find((m) => m.role === 'tool');
    expect(result?.content).toBe('Error: command not found');
  });

  it('clears pending approvals and ends the run after exit', () => {
    const view = eventsToMessages(fixture);
    expect(view.pending).toHaveLength(0);
    expect(view.ended).toBe(true);
  });

  it('keeps subagent messages (non-null parent_tool_use_id) out of the main feed', () => {
    const view = eventsToMessages(fixture);
    expect(view.messages.some((m) => m.content === 'subagent note')).toBe(false);
  });

  it('eventRowsToEvents reduces to the same view as the live event stream', () => {
    const rows: AgentEventRow[] = [...fixture]
      .map((event, i) => ({ id: `r${i}`, seq: i, created_at: '2026-01-01T00:00:00Z', event }))
      // shuffle to prove seq ordering, not array order, drives the result
      .reverse();
    const liveView = eventsToMessages(fixture);
    const historicalView = eventsToMessages(eventRowsToEvents(rows));
    expect(historicalView).toEqual(liveView);
  });
});
