import { queryKeys } from '$lib/domains/shared/query/keys';
import { agentsApi } from '../api/agentsApi.js';
import type { AgentFeedEvent } from '../generated';

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

/** Cursor of the next (older) feed page: the last row's cursor, or none once a page comes up short. */
export const nextEventsCursor = (page: AgentFeedEvent[]) =>
  page.length < EVENTS_PAGE ? undefined : page[page.length - 1].cursor;

/** Cross-session activity feed, newest first; each page is older than the last. */
export const agentEventsQuery = () => ({
  queryKey: queryKeys.agents.events,
  queryFn: ({ pageParam }: { pageParam: number | undefined }) =>
    agentsApi.listEvents({ beforeSeq: pageParam, limit: EVENTS_PAGE }),
  initialPageParam: undefined as number | undefined,
  getNextPageParam: nextEventsCursor,
});

/** One session's full transcript + event log. */
export const agentSessionQuery = (sessionId: string) => ({
  queryKey: queryKeys.agents.session(sessionId),
  queryFn: () => agentsApi.getSession(sessionId),
});
