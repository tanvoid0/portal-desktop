import { afterEach, describe, expect, it, vi } from 'vitest';

const post = vi.hoisted(() => vi.fn());
vi.mock('$lib/utils/invokeClient', () => ({ invokeClient: { post } }));

import { fetchHostStats } from './hostStats';

const stats = (cpuPercent: number) => ({ cpuPercent, memoryUsedBytes: 1, memoryTotalBytes: 2 });

afterEach(() => {
  vi.useRealTimers();
  post.mockReset();
});

describe('fetchHostStats', () => {
  it('returns a measured first reading as is', async () => {
    post.mockResolvedValueOnce(stats(7));
    expect((await fetchHostStats()).cpuPercent).toBe(7);
    expect(post).toHaveBeenCalledTimes(1);
  });

  it('measures again when the first reading is the cold-start 0', async () => {
    vi.useFakeTimers();
    post.mockResolvedValueOnce(stats(0)).mockResolvedValueOnce(stats(12.5));
    const reading = fetchHostStats();
    await vi.advanceTimersByTimeAsync(1000);
    expect((await reading).cpuPercent).toBe(12.5);
    expect(post).toHaveBeenCalledTimes(2);
  });
});
