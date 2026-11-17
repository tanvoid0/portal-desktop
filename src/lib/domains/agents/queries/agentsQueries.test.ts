import { describe, expect, it, vi } from 'vitest';

const { listEvents } = vi.hoisted(() => ({ listEvents: vi.fn().mockResolvedValue([]) }));
vi.mock('../api/agentsApi.js', () => ({ agentsApi: { listEvents } }));

import { agentEventsQuery, EVENTS_PAGE, nextEventsCursor } from './agentsQueries';
import type { AgentFeedEvent } from '../generated';

const page = (n: number, last = 1) =>
  Array.from({ length: n }, (_, i) => ({ cursor: last + n - 1 - i })) as AgentFeedEvent[];

describe('agent events feed paging', () => {
  it('continues from the oldest cursor while pages are full, stops on a short one', () => {
    expect(nextEventsCursor(page(EVENTS_PAGE, 5))).toBe(5);
    expect(nextEventsCursor(page(EVENTS_PAGE - 1))).toBeUndefined();
  });

  it('passes the page param as beforeSeq', async () => {
    await agentEventsQuery().queryFn({ pageParam: 42 });
    expect(listEvents).toHaveBeenCalledWith({ beforeSeq: 42, limit: EVENTS_PAGE });
  });
});
