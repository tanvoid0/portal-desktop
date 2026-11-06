import { createQuery } from '@tanstack/svelte-query';
import { queryKeys } from '$lib/domains/shared/query/keys';
import { fetchAllProjects, fetchProjectById } from '../api/projectApi';
import { findUnavailableProjects } from '../api/discoveryApi';

export function createProjectsQuery() {
  return createQuery(() => ({
    queryKey: queryKeys.projects.all,
    queryFn: fetchAllProjects,
  }));
}

export function createProjectQuery(projectId: () => string | null | undefined) {
  return createQuery(() => {
    const id = projectId();
    // `!!id` narrows away null/undefined; Boolean(id) does not.
    const enabled = !!id && !Number.isNaN(parseInt(id, 10));

    return {
      queryKey: queryKeys.projects.detail(id ?? ''),
      queryFn: () => fetchProjectById(id!),
      enabled,
    };
  });
}

/** Projects whose folder no longer exists. Walks the audit roots, so it rides the default stale time. */
export function createUnavailableProjectsQuery() {
  return createQuery(() => ({
    queryKey: queryKeys.projects.unavailable,
    queryFn: findUnavailableProjects,
  }));
}
