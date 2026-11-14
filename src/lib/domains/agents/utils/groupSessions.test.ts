import { describe, expect, it } from 'vitest';
import type { AgentSessionSummary } from '../generated';
import type { Project } from '$lib/domains/projects/types';
import { groupSessionsByProject } from './groupSessions';

const session = (id: string, cwd: string): AgentSessionSummary => ({
  id,
  backend_id: 'claude-code',
  model: null,
  cwd,
  status: 'exited',
  started_at: '2026-01-01T00:00:00Z',
  ended_at: null,
  exit_code: null,
  title: null,
  task_id: null,
  backend_session_id: null,
});

const project = (id: string, name: string, path: string): Project =>
  ({ id, name, path, status: 'active' }) as unknown as Project;

describe('groupSessionsByProject', () => {
  it('matches a project path case- and separator-insensitively', () => {
    const groups = groupSessionsByProject(
      [session('a', 'D:\\Foo\\'), session('b', 'd:/foo')],
      [project('p1', 'Foo', 'D:\\Foo')]
    );
    expect(groups).toHaveLength(1);
    expect(groups[0].sessions.map((s) => s.id)).toEqual(['a', 'b']);
  });

  it('puts a cwd under a project when it is a subdirectory, picking the most specific match', () => {
    const groups = groupSessionsByProject(
      [session('sub', '/code/mono/apps/web/src')],
      [project('root', 'mono', '/code/mono'), project('web', 'web', '/code/mono/apps/web')]
    );
    expect(groups).toHaveLength(1);
    expect(groups[0].projectId).toBe('web');
  });

  it('gives a cwd with no matching project its own group, keyed and labeled by its folder', () => {
    const groups = groupSessionsByProject(
      [session('x', '/elsewhere/scratch')],
      [project('p1', 'Foo', '/code/foo')]
    );
    expect(groups).toHaveLength(1);
    expect(groups[0].projectId).toBeUndefined();
    expect(groups[0].label).toBe('scratch');
  });

  it('orders groups by first-seen session, most-recent-first input wins', () => {
    const groups = groupSessionsByProject(
      [session('recent', '/code/b'), session('older', '/code/a'), session('b2', '/code/b')],
      [project('a', 'A', '/code/a'), project('b', 'B', '/code/b')]
    );
    expect(groups.map((g) => g.projectId)).toEqual(['b', 'a']);
    expect(groups[0].sessions.map((s) => s.id)).toEqual(['recent', 'b2']);
  });
});
