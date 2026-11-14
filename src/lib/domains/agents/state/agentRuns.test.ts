import { describe, expect, it } from 'vitest';
import type { AgentEventSeq } from '../generated';
import { appendFolded } from './agentRuns.svelte.js';

function seq(n: number, event: AgentEventSeq['event']): AgentEventSeq {
  return { seq: n, event };
}

describe('appendFolded', () => {
  it('folds consecutive text_delta events from the same parent into one', () => {
    let log: AgentEventSeq[] = [];
    log = appendFolded(log, seq(0, { type: 'text_delta', text: 'Lis' }));
    log = appendFolded(log, seq(1, { type: 'text_delta', text: 'ting' }));
    log = appendFolded(log, seq(2, { type: 'text_delta', text: ' files' }));
    expect(log).toHaveLength(1);
    expect(log[0]).toEqual(seq(2, { type: 'text_delta', text: 'Listing files' }));
  });

  it('does not fold across a different event type, or thinking into text', () => {
    let log: AgentEventSeq[] = [];
    log = appendFolded(log, seq(0, { type: 'text_delta', text: 'a' }));
    log = appendFolded(log, seq(1, { type: 'thinking_delta', text: 'b' }));
    log = appendFolded(log, seq(2, { type: 'message', role: 'assistant', text: 'c' }));
    expect(log).toHaveLength(3);
  });

  it('does not fold deltas from different parents (main turn vs. a subagent)', () => {
    let log: AgentEventSeq[] = [];
    log = appendFolded(log, seq(0, { type: 'text_delta', text: 'main' }));
    log = appendFolded(
      log,
      seq(1, { type: 'text_delta', text: 'sub', parent_tool_use_id: 'call1' })
    );
    expect(log).toHaveLength(2);
  });

  it('keeps the merged entry at the later seq', () => {
    let log: AgentEventSeq[] = [];
    log = appendFolded(log, seq(5, { type: 'text_delta', text: 'a' }));
    log = appendFolded(log, seq(9, { type: 'text_delta', text: 'b' }));
    expect(log[0].seq).toBe(9);
  });
});
