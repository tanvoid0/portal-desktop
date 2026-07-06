import type { QueryClient } from "@tanstack/svelte-query";
import { queryKeys } from "./keys";

export function invalidateProjectsList(queryClient: QueryClient): void {
  void queryClient.invalidateQueries({ queryKey: queryKeys.projects.all });
}

export function invalidateProjectDetail(
  queryClient: QueryClient,
  id: string | number,
): void {
  void queryClient.invalidateQueries({
    queryKey: queryKeys.projects.detail(id),
  });
}
