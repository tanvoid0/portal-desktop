import { describe, expect, it } from 'vitest';
import type { AgentEvent, AgentEventRow, AgentUsageCell } from '../generated';
import { buildSubagentTree, familyOf, pivotUsage } from './analytics';

const cell = (day: string, model: string, tokens: number, cost_usd = 0) =>
  ({ day, model, tokens, cache_tokens: 0, cost_usd }) as AgentUsageCell;

describe('pivotUsage', () => {
  it('groups models by family, fills gap days, keeps fixed family order', () => {
    const { rows, families } = pivotUsage(
      [
        cell('2026-11-01', 'claude-sonnet-4-5', 10),
        cell('2026-11-01', 'claude-opus-4-5', 5),
        cell('2026-11-01', 'claude-opus-4-1', 1),
        cell('2026-11-03', 'mystery', 7),
      ],
      'tokens'
    );
    expect(families).toEqual(['opus', 'sonnet', 'other']);
    expect(rows.map((r) => r.day)).toEqual(['2026-11-01', '2026-11-02', '2026-11-03']);
    expect(rows[0]).toMatchObject({ opus: 6, sonnet: 10, other: 0 });
    expect(rows[1]).toMatchObject({ opus: 0, sonnet: 0, other: 0 });
    expect(familyOf('mystery')).toBe('other');
  });

  it('is empty without data', () => {
    expect(pivotUsage([], 'cost_usd')).toEqual({ rows: [], families: [] });
  });
});

describe('buildSubagentTree', () => {
  const row = (event: AgentEvent) => ({ event }) as AgentEventRow;
  const call = (id: string, parent?: string) =>
    row({ type: 'tool_call', id, name: 'x', input: {}, parent_tool_use_id: parent });

  it('nests a subagent under the subagent whose call spawned it and counts own tools', () => {
    const tree = buildSubagentTree([
      call('A'),
      row({ type: 'subagent_started', tool_use_id: 'A', name: 'planner' }),
      call('t1', 'A'),
      call('B', 'A'),
      row({ type: 'subagent_started', tool_use_id: 'B' }),
      call('t2', 'B'),
      call('t3', 'B'),
      row({ type: 'subagent_finished', tool_use_id: 'B' }),
    ]);
    expect(tree).toHaveLength(1);
    expect(tree[0]).toMatchObject({ name: 'planner', tools: 2, done: false });
    expect(tree[0].children[0]).toMatchObject({ name: 'subagent', tools: 2, done: true });
  });
});
