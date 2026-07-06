import { createQuery } from "@tanstack/svelte-query";
import { queryKeys } from "$lib/domains/shared/query/keys";
import { fetchAllTasks, fetchTaskById } from "../api/taskApi";

export function createTasksQuery() {
  return createQuery(() => ({
    queryKey: queryKeys.tasks.all,
    queryFn: fetchAllTasks,
  }));
}

export function createTaskQuery(taskId: () => string | null | undefined) {
  return createQuery(() => {
    const id = taskId();

    return {
      queryKey: queryKeys.tasks.detail(id ?? ""),
      queryFn: () => fetchTaskById(id!),
      enabled: Boolean(id),
    };
  });
}
