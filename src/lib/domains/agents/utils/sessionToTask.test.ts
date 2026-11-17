import { describe, expect, it } from 'vitest';
import type { AgentEvent, AgentSessionSummary } from '../generated';
import { sessionsToTasks, type SessionTaskContext } from './sessionToTask';

const s = (id: string, status: string, exit_code: number | null = null) =>
  ({
    id,
    backend_id: 'b',
    status,
    exit_code,
    cwd: 'C:/x/proj',
    title: null,
    started_at: '2026-01-01T00:00:00Z',
    ended_at: null,
  }) as AgentSessionSummary;

const ctx = (
  waiting: string[] = [],
  events: Record<string, AgentEvent[]> = {}
): SessionTaskContext => ({
  isWaiting: (id) => waiting.includes(id),
  events: (id) => events[id] ?? [],
  backendName: () => 'Backend',
});

describe('sessionsToTasks', () => {
  it('maps session state onto task statuses', () => {
    const tasks = sessionsToTasks(
      [
        s('w', 'running'),
        s('p', 'running'),
        s('ok', 'exited', 0),
        s('bad', 'exited', 1),
        s('i', 'interrupted'),
      ],
      ctx(['w'])
    );
    expect(tasks.map((t) => [t.id, t.status])).toEqual([
      ['w', 'pending'],
      ['p', 'in-progress'],
      ['ok', 'completed'],
      ['bad', 'cancelled'],
      ['i', 'cancelled'],
    ]);
    expect(tasks[0].title).toBe('b');
    expect(tasks[0].description).toBe('proj');
  });

  it('emits subagents as subtasks of their session', () => {
    const events: AgentEvent[] = [
      { type: 'subagent_started', tool_use_id: 't1', name: 'Explore' },
      { type: 'subagent_started', tool_use_id: 't2' },
      { type: 'subagent_finished', tool_use_id: 't1' },
    ];
    const tasks = sessionsToTasks([s('a', 'running')], ctx([], { a: events }));
    expect(tasks.slice(1).map((t) => [t.id, t.parentId, t.title, t.status])).toEqual([
      ['a:t1', 'a', 'Explore', 'completed'],
      ['a:t2', 'a', 'Subagent', 'in-progress'],
    ]);
  });
});
