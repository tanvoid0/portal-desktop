import { describe, expect, it } from 'vitest';
import { buildStepGraph, pickRunSelection } from './stepGraph';
import type { ScriptExecutionInfo } from '$lib/domains/scripts/services/scriptExecutionService';

describe('buildStepGraph', () => {
  it('stages by longest path and marks unreached steps skipped once the run failed', () => {
    const plan = [
      { id: 'build', name: 'Build', needs: [] },
      { id: 'test', name: 'Test', needs: ['build'] },
      { id: 'lint', name: 'Lint', needs: ['build'] },
      { id: 'ship', name: 'Ship', needs: ['test', 'lint'] },
    ];
    const row = (stepId: string, status: ScriptExecutionInfo['status']) =>
      ({ id: `x-${stepId}`, stepId, status }) as ScriptExecutionInfo;
    const g = buildStepGraph(plan, [row('build', 'success'), row('test', 'failed')]);
    expect(g.columns.map((c) => c.map((n) => n.id))).toEqual([
      ['build'],
      ['test', 'lint'],
      ['ship'],
    ]);
    expect(g.edges).toHaveLength(4);
    expect(g.columns[2][0].status).toBe('skipped');
    expect(g.columns[1][1].status).toBe('skipped');
  });

  it('dedupes duplicate plan ids, merging needs', () => {
    const g = buildStepGraph([
      { id: 'a', name: 'a', needs: [] },
      { id: 'b', name: 'b', needs: [] },
      { id: 'a', name: 'a', needs: ['b'] },
    ]);
    expect(g.columns.map((c) => c.map((n) => n.id))).toEqual([['b'], ['a']]);
    expect(g.edges).toEqual([{ from: 'b', to: 'a' }]);
  });
});

describe('pickRunSelection', () => {
  it('prefers the failed step, else the last one run', () => {
    const s = (status: string, executionId?: string) => ({ status, executionId });
    expect(pickRunSelection([s('success', 'a'), s('failed', 'b'), s('skipped')])).toBe('b');
    expect(pickRunSelection([s('success', 'a'), s('success', 'b')])).toBe('b');
    expect(pickRunSelection([s('skipped')])).toBeUndefined();
  });
});
