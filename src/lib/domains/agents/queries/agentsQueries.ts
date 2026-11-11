import { queryKeys } from '$lib/domains/shared/query/keys';
import { agentsApi } from '../api/agentsApi.js';

/** Detected agent backends (Claude Code today) and their reported models. */
export const agentBackendsQuery = () => ({
  queryKey: queryKeys.agents.backends,
  queryFn: () => agentsApi.listBackends(),
});

/** Persisted sessions, most recent first — historical, so no polling. */
export const agentSessionsQuery = () => ({
  queryKey: queryKeys.agents.sessions,
  queryFn: () => agentsApi.listSessions(),
});

/** One session's full transcript + event log. */
export const agentSessionQuery = (sessionId: string) => ({
  queryKey: queryKeys.agents.session(sessionId),
  queryFn: () => agentsApi.getSession(sessionId),
});
