import { createQuery } from "@tanstack/svelte-query";
import { queryKeys } from "$lib/domains/shared/query/keys";
import { fetchAllProjects, fetchProjectById } from "../api/projectApi";

export function createProjectsQuery() {
  return createQuery(() => ({
    queryKey: queryKeys.projects.all,
    queryFn: fetchAllProjects,
  }));
}

export function createProjectQuery(
  projectId: () => string | null | undefined,
) {
  return createQuery(() => {
    const id = projectId();
    const enabled =
      Boolean(id) && id !== undefined && !Number.isNaN(parseInt(id, 10));

    return {
      queryKey: queryKeys.projects.detail(id ?? ""),
      queryFn: () => fetchProjectById(id!),
      enabled,
    };
  });
}
