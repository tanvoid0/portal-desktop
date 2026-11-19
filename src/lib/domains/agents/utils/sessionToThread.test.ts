import { describe, expect, it } from 'vitest';
import type { AgentSessionSummary } from '../generated';
import { sessionToThread } from './sessionToThread';

const session: AgentSessionSummary = {
  id: 's1',
  backend_id: 'claude-code',
  model: 'opus',
  cwd: 'D:/work/app',
  status: 'exited',
  started_at: '2026-01-01T10:00:00Z',
  ended_at: '2026-01-01T11:00:00Z',
  exit_code: 0,
  title: null,
  task_id: null,
  backend_session_id: null,
};

describe('sessionToThread', () => {
  it('maps cwd, backend and end time onto the thread', () => {
    const t = sessionToThread(session);
    expect(t.workspace_root).toBe('D:/work/app');
    expect(t.llm_provider).toBe('claude-code');
    expect(t.updated_at).toBe('2026-01-01T11:00:00Z');
    expect(t.title).toBe('claude-code');
  });

  it('falls back to started_at while the run is live', () => {
    expect(sessionToThread({ ...session, ended_at: null, title: 'Fix it' }).updated_at).toBe(
      session.started_at
    );
  });
});
