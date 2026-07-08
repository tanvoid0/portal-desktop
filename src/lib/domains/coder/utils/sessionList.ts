import type { CoderThread } from "../types.js";

export type SessionSortKey =
  | "updated_desc"
  | "updated_asc"
  | "messages_desc"
  | "messages_asc"
  | "title_asc"
  | "title_desc"
  | "project_asc";

export type SessionStatusFilter = "all" | "running" | "queued" | "idle";

export interface SessionListFilters {
  search: string;
  project: string;
  status: SessionStatusFilter;
  provider: string;
  sort: SessionSortKey;
}

export interface ProjectOption {
  value: string;
  label: string;
  count: number;
}

export const DEFAULT_SESSION_FILTERS: SessionListFilters = {
  search: "",
  project: "all",
  status: "all",
  provider: "all",
  sort: "updated_desc",
};

export const SORT_OPTIONS = [
  { value: "updated_desc", label: "Recently updated" },
  { value: "updated_asc", label: "Oldest updated" },
  { value: "messages_desc", label: "Most messages" },
  { value: "messages_asc", label: "Fewest messages" },
  { value: "title_asc", label: "Title A–Z" },
  { value: "title_desc", label: "Title Z–A" },
  { value: "project_asc", label: "Project A–Z" },
] as const;

export const STATUS_OPTIONS = [
  { value: "all", label: "All statuses" },
  { value: "running", label: "Running" },
  { value: "queued", label: "Queued" },
  { value: "idle", label: "Idle" },
] as const;

export function workspaceFolderName(root: string): string {
  return (root ?? "").split(/[/\\]/).filter(Boolean).pop() ?? root ?? "";
}

export function getMessageCount(thread: CoderThread): number {
  return (
    thread.message_count ??
    thread.messages.filter((m) => m.role === "user" || m.role === "assistant")
      .length
  );
}

export function extractProjectOptions(threads: CoderThread[]): ProjectOption[] {
  const map = new Map<string, { label: string; count: number }>();
  for (const t of threads) {
    const root = t.workspace_root ?? "";
    if (!root) continue;
    const label = workspaceFolderName(root);
    const existing = map.get(root);
    if (existing) {
      existing.count += 1;
    } else {
      map.set(root, { label, count: 1 });
    }
  }
  return [...map.entries()]
    .map(([value, { label, count }]) => ({
      value,
      label: `${label} (${count})`,
      count,
    }))
    .sort((a, b) =>
      a.label.localeCompare(b.label, undefined, { sensitivity: "base" }),
    );
}

export function extractProviderOptions(
  threads: CoderThread[],
): Array<{ value: string; label: string }> {
  const counts = new Map<string, number>();
  for (const t of threads) {
    const provider = t.llm_provider ?? t.model;
    if (!provider) continue;
    counts.set(provider, (counts.get(provider) ?? 0) + 1);
  }
  return [...counts.entries()]
    .sort(([a], [b]) => a.localeCompare(b, undefined, { sensitivity: "base" }))
    .map(([value, count]) => ({ value, label: `${value} (${count})` }));
}

export function sessionStatus(
  threadId: string,
  runningThreadIds: Set<string>,
  queuedCount: number,
): SessionStatusFilter {
  if (runningThreadIds.has(threadId)) return "running";
  if (queuedCount > 0) return "queued";
  return "idle";
}

function compareSessions(
  a: CoderThread,
  b: CoderThread,
  sort: SessionSortKey,
  runningThreadIds: Set<string>,
): number {
  switch (sort) {
    case "updated_asc":
      return (a.updated_at || "").localeCompare(b.updated_at || "");
    case "messages_desc":
      return getMessageCount(b) - getMessageCount(a);
    case "messages_asc":
      return getMessageCount(a) - getMessageCount(b);
    case "title_asc":
      return (a.title ?? "").localeCompare(b.title ?? "", undefined, {
        sensitivity: "base",
      });
    case "title_desc":
      return (b.title ?? "").localeCompare(a.title ?? "", undefined, {
        sensitivity: "base",
      });
    case "project_asc": {
      const projectCompare = workspaceFolderName(a.workspace_root).localeCompare(
        workspaceFolderName(b.workspace_root),
        undefined,
        { sensitivity: "base" },
      );
      if (projectCompare !== 0) return projectCompare;
      return (b.updated_at || "").localeCompare(a.updated_at || "");
    }
    case "updated_desc":
    default: {
      const aRunning = runningThreadIds.has(a.id);
      const bRunning = runningThreadIds.has(b.id);
      if (aRunning !== bRunning) return aRunning ? -1 : 1;
      return (b.updated_at || "").localeCompare(a.updated_at || "");
    }
  }
}

export function filterAndSortSessions(
  threads: CoderThread[],
  filters: SessionListFilters,
  runningThreadIds: Set<string>,
  queuedCountFor?: (id: string) => number,
): CoderThread[] {
  let result = [...threads];

  const q = filters.search.trim().toLowerCase();
  if (q) {
    result = result.filter((t) => {
      const title = (t.title ?? "").toLowerCase();
      const root = (t.workspace_root ?? "").toLowerCase();
      const provider = (t.llm_provider ?? t.model ?? "").toLowerCase();
      return title.includes(q) || root.includes(q) || provider.includes(q);
    });
  }

  if (filters.project !== "all") {
    result = result.filter((t) => t.workspace_root === filters.project);
  }

  if (filters.provider !== "all") {
    result = result.filter(
      (t) => (t.llm_provider ?? t.model ?? "") === filters.provider,
    );
  }

  if (filters.status !== "all") {
    result = result.filter((t) => {
      const queued = queuedCountFor?.(t.id) ?? 0;
      return (
        sessionStatus(t.id, runningThreadIds, queued) === filters.status
      );
    });
  }

  result.sort((a, b) =>
    compareSessions(a, b, filters.sort, runningThreadIds),
  );
  return result;
}

export function hasActiveFilters(filters: SessionListFilters): boolean {
  return (
    filters.search.trim() !== "" ||
    filters.project !== "all" ||
    filters.status !== "all" ||
    filters.provider !== "all" ||
    filters.sort !== "updated_desc"
  );
}
