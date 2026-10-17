import { createQuery } from '@tanstack/svelte-query';
import { queryKeys } from '$lib/domains/shared/query/keys';
import { fetchAllModules, refreshProjectModules } from '../api/moduleApi';

/** All modules of all projects; group with `groupByProject`. */
export function createAllModulesQuery() {
  return createQuery(() => ({
    queryKey: queryKeys.projects.modules,
    queryFn: fetchAllModules,
  }));
}

/**
 * One project's modules. Rescans on load so a project edited on disk since the
 * last scan is never stale when you look at it.
 */
export function createProjectModulesQuery(projectId: () => number | null) {
  return createQuery(() => {
    const id = projectId();
    return {
      queryKey: [...queryKeys.projects.modules, id] as const,
      queryFn: () => refreshProjectModules(id!),
      enabled: id !== null && !Number.isNaN(id),
    };
  });
}
