import { describe, expect, it } from 'vitest';
import { foldTimelineEvents } from './eventTimeline';
import type { AgentEventRow } from '../generated';

function row(id: string, event: AgentEventRow['event']): AgentEventRow {
  return { id, seq: Number(id), created_at: `2026-01-01T00:00:0${id}Z`, event };
}

describe('foldTimelineEvents', () => {
  it('renders a text/thinking delta row as a safety net, though the pump loop no longer persists them', () => {
    const items = foldTimelineEvents([
      row('1', { type: 'thinking_delta', text: 'hmm' }),
      row('2', { type: 'text_delta', text: 'ok' }),
      row('3', { type: 'turn_done', is_error: false }),
    ]);
    expect(items).toEqual([
      { id: '1', created_at: '2026-01-01T00:00:01Z', label: 'Thinking', detail: 'hmm' },
      { id: '2', created_at: '2026-01-01T00:00:02Z', label: 'Assistant text', detail: 'ok' },
      { id: '3', created_at: '2026-01-01T00:00:03Z', label: 'Turn finished' },
    ]);
  });

  it('includes the tool input as detail on a permission request', () => {
    const items = foldTimelineEvents([
      row('1', {
        type: 'permission_request',
        request_id: 'r1',
        tool_name: 'Write',
        tool_input: { file_path: 'a.txt', content: 'hi' },
        suggestions: [],
      }),
    ]);
    expect(items[0]).toMatchObject({
      label: 'Permission requested: Write',
      detail: '{"file_path":"a.txt","content":"hi"}',
    });
  });

  it('flags a failed tool result and a non-zero exit', () => {
    const items = foldTimelineEvents([
      row('1', { type: 'tool_result', tool_use_id: 't1', content: 'boom', is_error: true }),
      row('2', { type: 'exited', code: 1 }),
    ]);
    expect(items[0]).toMatchObject({ label: 'Tool result (error)', isError: true });
    expect(items[1]).toMatchObject({ label: 'Exited (code 1)', isError: true });
  });
});
