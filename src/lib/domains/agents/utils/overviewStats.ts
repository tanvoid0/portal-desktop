import type { AgentSessionSummary } from '../generated';

export const RECENT_LIMIT = 20;

/** `backendId` '' = all backends. Input order (most recent first) is kept. */
export function overviewSessions(sessions: AgentSessionSummary[], backendId: string) {
  const filtered = backendId ? sessions.filter((s) => s.backend_id === backendId) : sessions;
  const running = filtered.filter((s) => s.status === 'running');
  const recent = filtered.filter((s) => s.status !== 'running').slice(0, RECENT_LIMIT);
  const stats = {
    total: filtered.length,
    running: running.length,
    interrupted: filtered.filter((s) => s.status === 'interrupted').length,
    failed: filtered.filter((s) => s.status === 'exited' && !!s.exit_code).length,
  };
  return { running, recent, stats };
}
