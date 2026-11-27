import { describe, expect, it } from 'vitest';
import type { PipelineExecution } from '$lib/domains/projects/pipelines';
import { permissionFact, pipelineFacts } from './facts';

describe('notification facts', () => {
  it('names the tool and what it touches', () => {
    expect(permissionFact('Bash', { command: 'ls' })).toEqual([
      { kind: 'tool', value: 'Bash' },
      { kind: 'command', value: 'ls' },
    ]);
    expect(permissionFact('Write', { file_path: '/a' })[1]).toEqual({ kind: 'file', value: '/a' });
    expect(permissionFact('Other', null)).toHaveLength(1);
  });

  it('reports the failed step, and no duration for an unfinished run', () => {
    const run = {
      startedAt: '2026-01-01T00:00:00Z',
      stepExecutions: [{ status: 'failed', stepName: 'build', exitCode: 0 }],
    } as unknown as PipelineExecution;
    expect(pipelineFacts(run).map((f) => f.kind)).toEqual(['step', 'exit']);
    const done = { ...run, finishedAt: '2026-01-01T00:00:05Z' as unknown as Date };
    expect(pipelineFacts(done).at(-1)!.kind).toBe('duration');
  });
});
