import { queryKeys } from '$lib/domains/shared/query/keys';
import type { AgentFeedPage } from '../generated';
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

export const EVENTS_PAGE = 30;

/** `''` = all backends. Filtered in SQL before paging, so pages are never empty by filtering. */
export const agentEventsQuery = (backendId = '', refetchInterval: number | false = false) => ({
  queryKey: queryKeys.agents.feed(backendId),
  queryFn: ({ pageParam }: { pageParam: number | undefined }) =>
    agentsApi.listEvents({
      beforeSeq: pageParam,
      limit: EVENTS_PAGE,
      backendId: backendId || undefined,
    }),
  initialPageParam: undefined as number | undefined,
  // Rust reports the raw-row cursor (unparseable rows still count toward a full page).
  getNextPageParam: (page: AgentFeedPage) => page.next_cursor ?? undefined,
  refetchInterval,
});

/** `''` = all backends. Historical aggregates, so no polling. */
export const agentAnalyticsQuery = (backendId = '') => ({
  queryKey: queryKeys.agents.analytics(backendId),
  queryFn: () => agentsApi.analytics(backendId || undefined),
});

/** One session's full transcript + event log. */
export const agentSessionQuery = (sessionId: string) => ({
  queryKey: queryKeys.agents.session(sessionId),
  queryFn: () => agentsApi.getSession(sessionId),
});
