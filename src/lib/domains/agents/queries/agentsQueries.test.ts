import { describe, expect, it, vi } from 'vitest';

const { listEvents } = vi.hoisted(() => ({ listEvents: vi.fn().mockResolvedValue([]) }));
vi.mock('../api/agentsApi.js', () => ({ agentsApi: { listEvents } }));

import { agentEventsQuery, EVENTS_PAGE } from './agentsQueries';
import type { AgentFeedPage } from '../generated';

describe('agent events feed paging', () => {
  it('pages on the raw-row cursor Rust reports, stops when it is null', () => {
    const q = agentEventsQuery();
    expect(q.getNextPageParam({ events: [], next_cursor: 5 } as AgentFeedPage)).toBe(5);
    expect(q.getNextPageParam({ events: [], next_cursor: null } as AgentFeedPage)).toBeUndefined();
  });

  it('passes the page param as beforeSeq and the backend filter', async () => {
    await agentEventsQuery('claude-code').queryFn({ pageParam: 42 });
    expect(listEvents).toHaveBeenCalledWith({
      beforeSeq: 42,
      limit: EVENTS_PAGE,
      backendId: 'claude-code',
    });
  });
});
