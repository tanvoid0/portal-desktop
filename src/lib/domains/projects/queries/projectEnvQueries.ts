import { createQuery } from '@tanstack/svelte-query';
import { queryKeys } from '$lib/domains/shared/query/keys';
import { fetchProjectEnv } from '../api/projectEnvApi';

/** Always live off disk — never cached, never stale-while-revalidate.
 *  `enabled` lets the caller gate this off a tab's active state — `TabsContent`
 *  renders every tab's content even when hidden, so without it this (and its
 *  focus-refetch) would run regardless of which project tab is actually open. */
export function createProjectEnvQuery(
  projectId: () => number | null,
  enabled: () => boolean = () => true
) {
  return createQuery(() => {
    const id = projectId();
    return {
      queryKey: queryKeys.projects.env(id ?? -1),
      queryFn: () => fetchProjectEnv(id!),
      enabled: id !== null && enabled(),
      staleTime: 0,
      refetchOnWindowFocus: true,
    };
  });
}
