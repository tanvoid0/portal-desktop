import { describe, expect, it } from 'vitest';
import { buildStepGraph } from './stepGraph';
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
});
