import type { QueryClient } from "@tanstack/svelte-query";
import { queryKeys } from "./keys";

export function invalidateDashboardOverview(queryClient: QueryClient): void {
  void queryClient.invalidateQueries({ queryKey: queryKeys.dashboard.overview });
}
