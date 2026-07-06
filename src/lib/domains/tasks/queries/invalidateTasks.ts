import type { QueryClient } from "@tanstack/svelte-query";
import { queryKeys } from "$lib/domains/shared/query/keys";

export function invalidateTasksList(queryClient: QueryClient): void {
  void queryClient.invalidateQueries({ queryKey: queryKeys.tasks.all });
}

export function invalidateTaskDetail(
  queryClient: QueryClient,
  id: string,
): void {
  void queryClient.invalidateQueries({ queryKey: queryKeys.tasks.detail(id) });
}

export function invalidateTaskCaches(
  queryClient: QueryClient,
  taskId?: string,
): void {
  invalidateTasksList(queryClient);
  if (taskId) {
    invalidateTaskDetail(queryClient, taskId);
  }
}
