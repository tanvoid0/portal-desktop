import { describe, expect, it } from 'vitest';
import type { AgentSessionSummary } from '../generated';
import { overviewSessions } from './overviewStats';

const s = (id: string, backend_id: string, status: string, exit_code: number | null = null) =>
  ({ id, backend_id, status, exit_code }) as AgentSessionSummary;

describe('overviewSessions', () => {
  const all = [
    s('1', 'a', 'running'),
    s('2', 'a', 'exited', 1),
    s('3', 'b', 'exited', 0),
    s('4', 'b', 'interrupted'),
  ];

  it('counts across all backends', () => {
    const r = overviewSessions(all, '');
    expect(r.stats).toEqual({ total: 4, running: 1, interrupted: 1, failed: 1 });
    expect(r.running.map((x) => x.id)).toEqual(['1']);
    expect(r.recent.map((x) => x.id)).toEqual(['2', '3', '4']);
  });

  it('filters by backend', () => {
    expect(overviewSessions(all, 'b').stats.total).toBe(2);
  });
});
