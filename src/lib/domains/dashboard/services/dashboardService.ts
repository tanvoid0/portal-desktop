import { invokeClient } from "$lib/utils/invokeClient";
import { cache } from "$lib/domains/shared/services/cache";
import type { Project } from "$lib/domains/projects/types";

export const DASHBOARD_OVERVIEW_CACHE_KEY = "dashboard_overview";

export interface DashboardTaskStats {
  total: number;
  pending: number;
  in_progress: number;
  completed: number;
  cancelled: number;
  completion_percentage: number;
}

export interface DashboardProjectStats {
  total_projects: number;
  active_projects: number;
  archived_projects: number;
  total_size: number;
  most_used_framework: string;
  recent_projects: Project[];
}

export interface DashboardOverview {
  project_stats: DashboardProjectStats;
  task_stats: DashboardTaskStats;
  running_services_count: number;
}

const DEFAULT_TTL_MS = 5 * 60 * 1000;

const EMPTY_OVERVIEW: DashboardOverview = {
  project_stats: {
    total_projects: 0,
    active_projects: 0,
    archived_projects: 0,
    total_size: 0,
    most_used_framework: "Unknown",
    recent_projects: [],
  },
  task_stats: {
    total: 0,
    pending: 0,
    in_progress: 0,
    completed: 0,
    cancelled: 0,
    completion_percentage: 0,
  },
  running_services_count: 0,
};

/** Pure fetch for TanStack Query — no manual cache layer. */
export async function fetchDashboardOverview(): Promise<DashboardOverview> {
  try {
    const overview = await invokeClient.post<DashboardOverview>(
      "get_dashboard_overview",
    );
    return overview ?? EMPTY_OVERVIEW;
  } catch {
    return EMPTY_OVERVIEW;
  }
}

export async function getDashboardOverview(opts?: {
  force?: boolean;
}): Promise<{ overview: DashboardOverview; fromBackend: boolean }> {
  const force = opts?.force ?? false;

  if (!force) {
    const cached = cache.get<DashboardOverview>(DASHBOARD_OVERVIEW_CACHE_KEY);
    if (cached !== null) {
      return { overview: cached, fromBackend: false };
    }
  }

  try {
    const overview = await fetchDashboardOverview();
    return { overview, fromBackend: true };
  } catch {
    return { overview: EMPTY_OVERVIEW, fromBackend: false };
  }
}

export function invalidateDashboardOverview(): void {
  cache.delete(DASHBOARD_OVERVIEW_CACHE_KEY);
}

export function primeDashboardOverview(
  overview: DashboardOverview,
  ttlMs: number = DEFAULT_TTL_MS,
): void {
  cache.set(DASHBOARD_OVERVIEW_CACHE_KEY, overview, ttlMs);
}
