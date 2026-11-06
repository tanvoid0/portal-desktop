/**
 * AI tidy: the model proposes a triage decision per project, this file decides
 * whether to believe it. The model only ever sees facts; every safety rule is
 * enforced here in code, because a prompt is a request, not a guarantee.
 */
import type { TriageDecision } from '$lib/domains/projects/api/discoveryApi';

/** Registry row as `get_all_projects` returns it (snake_case). Only what tidy reads. */
export interface TidyRow {
  id: number;
  name: string;
  framework?: string | null;
  has_remote?: boolean | null;
  unpushed_commits?: number | null;
  stash_count?: number | null;
  has_uncommitted_changes?: boolean | null;
  activity_status?: string | null;
  last_activity?: string | null;
  size?: number | null;
  duplicate_group?: string | null;
  triage_decision?: string | null;
}

export interface TidySuggestion {
  id: number;
  action: TriageDecision;
  reason: string;
  /** The model asked for `delete` but the facts forbid it; downgraded to `review`. */
  downgraded: boolean;
}

const ACTIONS: readonly TriageDecision[] = ['keep', 'archive', 'delete', 'review'];

/** Work that exists nowhere else: deleting it loses something. */
export function holdsUniqueWork(r: TidyRow): boolean {
  return (
    (r.unpushed_commits ?? 0) > 0 ||
    (r.stash_count ?? 0) > 0 ||
    !!r.has_uncommitted_changes ||
    !r.has_remote
  );
}

export const TIDY_SYSTEM = `You triage developer projects on a disk. Use only the facts given.
Actions: keep (active or valuable), archive (stale but worth preserving), delete (safe to remove: pushed, clean, redundant), review (unclear).
Never choose delete when a project has unpushed commits, stashes, uncommitted changes or no git remote.
You get a JSON array of projects. Reply ONLY with a JSON array: [{"id": <id>, "action": "...", "reason": "<one short sentence>"}], one entry per project.`;

/** Whole days since `iso`, or null when missing/unparseable. A model has no clock. */
export function daysSince(iso: string | null | undefined, now: number): number | null {
  const t = iso ? Date.parse(iso) : NaN;
  return Number.isNaN(t) ? null : Math.max(0, Math.floor((now - t) / 86_400_000));
}

export function buildFacts(rows: TidyRow[], now = Date.now()): string {
  return JSON.stringify(
    rows.map((r) => ({
      id: r.id,
      name: r.name,
      framework: r.framework ?? null,
      has_remote: !!r.has_remote,
      unpushed_commits: r.unpushed_commits ?? 0,
      stashes: r.stash_count ?? 0,
      uncommitted_changes: !!r.has_uncommitted_changes,
      activity: r.activity_status ?? null,
      days_idle: daysSince(r.last_activity, now),
      size_mb: Math.round((r.size ?? 0) / 1e6),
      duplicate_group: r.duplicate_group ?? null,
    }))
  );
}

/**
 * Parses a reply for `rows`. Drops entries whose id was not asked about, whose
 * action is unknown, or that repeat an id. Downgrades unsafe deletes to review.
 * Reasoning models may wrap the array in prose or a code fence, so find it.
 */
export function parseSuggestions(text: string, rows: TidyRow[]): TidySuggestion[] {
  const start = text.indexOf('[');
  const end = text.lastIndexOf(']');
  if (start < 0 || end < start) return [];
  let raw: unknown;
  try {
    raw = JSON.parse(text.slice(start, end + 1));
  } catch {
    return [];
  }
  if (!Array.isArray(raw)) return [];

  const byId = new Map(rows.map((r) => [r.id, r]));
  const seen = new Set<number>();
  const out: TidySuggestion[] = [];
  for (const item of raw) {
    const e = item as { id?: unknown; action?: unknown; reason?: unknown };
    const row = typeof e.id === 'number' ? byId.get(e.id) : undefined;
    if (!row || seen.has(row.id)) continue;
    if (!ACTIONS.includes(e.action as TriageDecision)) continue;
    seen.add(row.id);
    let action = e.action as TriageDecision;
    const downgraded = action === 'delete' && holdsUniqueWork(row);
    if (downgraded) action = 'review';
    out.push({
      id: row.id,
      action,
      reason: typeof e.reason === 'string' ? e.reason.trim().slice(0, 240) : '',
      downgraded,
    });
  }
  return out;
}
