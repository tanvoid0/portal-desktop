import { createQuery } from "@tanstack/svelte-query";
import { queryKeys } from "$lib/domains/shared/query/keys";
import { fetchDashboardOverview } from "../services/dashboardService";

export function createDashboardOverviewQuery(options?: () => { enabled?: boolean }) {
  return createQuery(() => ({
    queryKey: queryKeys.dashboard.overview,
    queryFn: fetchDashboardOverview,
    enabled: options?.().enabled ?? true,
  }));
}
