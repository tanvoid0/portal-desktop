/**
 * Group agent sessions by the project whose path contains their cwd, for the
 * sessions list (`src/routes/ai/agents/sessions/+page.svelte`).
 */

import type { AgentSessionSummary } from '../generated';
import type { Project } from '$lib/domains/projects/types';
import { normalizePath } from '$lib/domains/projects/utils/nestProjects';

export interface SessionGroup {
  key: string;
  label: string;
  path: string;
  projectId?: string;
  sessions: AgentSessionSummary[];
}

function basename(path: string): string {
  const parts = path.replace(/\\/g, '/').split('/').filter(Boolean);
  return parts[parts.length - 1] ?? path;
}

/** The project whose (normalized) path exactly matches `cwd`, or — failing that —
 *  the longest project path that's an ancestor directory of it. */
function matchProject<T extends { norm: string }>(cwd: string, projects: T[]): T | undefined {
  let best: T | undefined;
  for (const p of projects) {
    const isMatch = cwd === p.norm || (cwd.length > p.norm.length && cwd.startsWith(`${p.norm}/`));
    if (isMatch && (!best || p.norm.length > best.norm.length)) best = p;
  }
  return best;
}

/**
 * Groups appear in first-seen-session order. `agents_list_sessions` returns sessions
 * most-recent-first, so the project whose session is newest naturally sorts first.
 * // ponytail: no activity re-sort — a group's position is set by its first-seen
 * // session and never demoted as the rest of its sessions age. Add a last-active
 * // sort if a long-idle group sitting above a busier one becomes annoying.
 */
export function groupSessionsByProject(
  sessions: AgentSessionSummary[],
  projects: Project[]
): SessionGroup[] {
  const projectsNorm = projects.map((p) => ({ ...p, norm: normalizePath(p.path) }));
  const groups = new Map<string, SessionGroup>();
  const order: string[] = [];

  for (const session of sessions) {
    const cwdNorm = normalizePath(session.cwd);
    const match = matchProject(cwdNorm, projectsNorm);
    const key = match ? match.id : cwdNorm;
    let group = groups.get(key);
    if (!group) {
      group = match
        ? { key, label: match.name, path: match.path, projectId: match.id, sessions: [] }
        : { key, label: basename(session.cwd), path: session.cwd, sessions: [] };
      groups.set(key, group);
      order.push(key);
    }
    group.sessions.push(session);
  }

  return order.map((key) => groups.get(key)!);
}

/** The part of a session's cwd below its group's path, or null when they're the
 *  same directory (nothing extra to show on that row). */
export function sessionSubpath(session: AgentSessionSummary, group: SessionGroup): string | null {
  const groupNorm = normalizePath(group.path);
  const cwdNorm = normalizePath(session.cwd);
  return cwdNorm === groupNorm ? null : cwdNorm.slice(groupNorm.length + 1);
}
