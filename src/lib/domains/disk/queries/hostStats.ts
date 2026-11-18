import { createQuery } from '@tanstack/svelte-query';
import { invokeClient } from '$lib/utils/invokeClient';
import { queryKeys } from '$lib/domains/shared/query/keys';
import type { DiskUsage, HostStats } from '../generated';

/** CPU % is a delta between two refreshes, so the backend's first reading after it starts is a
 *  meaningless 0. Take a second one before reporting it — the page shows "Measuring…" meanwhile. */
export async function fetchHostStats(): Promise<HostStats> {
  const first = await invokeClient.post<HostStats>('host_stats');
  if (first.cpuPercent > 0) return first;
  await new Promise((resolve) => setTimeout(resolve, 500));
  return invokeClient.post<HostStats>('host_stats');
}

/** Polled every 2s; TanStack pauses it while the window is hidden. `staleTime: 0` so a revisit
 *  refetches at once instead of showing minutes-old numbers. */
export function createHostStatsQuery() {
  return createQuery(() => ({
    queryKey: queryKeys.disk.hostStats,
    queryFn: fetchHostStats,
    staleTime: 0,
    refetchInterval: 2000,
  }));
}

/** Volume capacity moves slowly — 30s is plenty. */
export function createDiskUsageQuery() {
  return createQuery(() => ({
    queryKey: queryKeys.disk.usage,
    queryFn: () => invokeClient.post<DiskUsage[]>('disk_usage'),
    staleTime: 0,
    refetchInterval: 30_000,
  }));
}
