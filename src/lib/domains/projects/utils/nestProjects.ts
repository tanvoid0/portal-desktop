/**
 * Group projects by nesting. A project's parent is its explicit `parent_id`
 * (manual grouping) when set, otherwise the nearest project whose path contains
 * it (on-disk nesting). Every project groups under the root it chains up to, so
 * the root list shows only top-level projects with their descendants hanging off.
 */

import type { Project } from '$lib/domains/projects/types';

export interface NestedProject {
  project: Project;
  /** Every descendant project, flattened (outermost-first by path depth). */
  children: Project[];
}

/** Lowercase, forward slashes, no trailing slash — Windows paths are case-insensitive. */
function normalizePath(path: string): string {
  return path.replace(/\\/g, '/').replace(/\/+$/, '').toLowerCase();
}

/** True when `child` lives strictly inside the `parent` directory. */
function isUnder(child: string, parent: string): boolean {
  return child.length > parent.length && child.startsWith(parent + '/');
}

/**
 * Split projects into roots with their nested descendants. Input order is
 * preserved among roots and within each child list.
 * ponytail: O(n²) ancestor scan, fine for the hundreds of projects discovered.
 */
export function nestProjects(projects: Project[]): NestedProject[] {
  const byId = new Map(projects.map((p) => [p.id, p]));
  const norm = new Map(projects.map((p) => [p.id, normalizePath(p.path)]));

  // Nearest (deepest) project whose path contains this one.
  const nearestPathAncestor = (p: Project): string | undefined => {
    const path = norm.get(p.id)!;
    let best: Project | undefined;
    for (const q of projects) {
      if (q === p) continue;
      const qPath = norm.get(q.id)!;
      if (isUnder(path, qPath) && (!best || qPath.length > norm.get(best.id)!.length)) {
        best = q;
      }
    }
    return best?.id;
  };

  // Direct parent: explicit grouping wins, else on-disk nesting. An explicit
  // parent that isn't in the set (deleted/filtered out) falls back to path.
  const directParent = new Map<Project['id'], Project['id']>();
  for (const p of projects) {
    const explicit = p.parent_id != null ? byId.get(String(p.parent_id)) : undefined;
    if (explicit && explicit.id !== p.id) {
      directParent.set(p.id, explicit.id);
    } else {
      const ancestor = nearestPathAncestor(p);
      if (ancestor) directParent.set(p.id, ancestor);
    }
  }

  // Walk the parent chain to its root. Cycles (shouldn't happen — the UI and
  // backend forbid them) break safely by treating the repeated node as a root.
  const rootOf = (p: Project): Project['id'] => {
    const seen = new Set<Project['id']>();
    let cur = p;
    for (;;) {
      if (seen.has(cur.id)) return cur.id;
      seen.add(cur.id);
      const parentId = directParent.get(cur.id);
      const parent = parentId != null ? byId.get(parentId) : undefined;
      if (!parent) return cur.id;
      cur = parent;
    }
  };

  const nodeByRoot = new Map<Project['id'], NestedProject>();
  const roots: NestedProject[] = [];
  for (const p of projects) {
    if (rootOf(p) === p.id) {
      const node: NestedProject = { project: p, children: [] };
      roots.push(node);
      nodeByRoot.set(p.id, node);
    }
  }
  for (const p of projects) {
    const root = rootOf(p);
    if (root !== p.id) nodeByRoot.get(root)?.children.push(p);
  }
  for (const node of roots) {
    node.children.sort((a, b) => norm.get(a.id)!.length - norm.get(b.id)!.length);
  }

  return roots;
}

/**
 * IDs of every project that chains up to `rootId` (its descendants). Used to keep
 * a project from being grouped under one of its own descendants — which would
 * form a cycle.
 */
export function descendantIds(projects: Project[], rootId: Project['id']): Set<Project['id']> {
  const out = new Set<Project['id']>();
  let added = true;
  while (added) {
    added = false;
    for (const p of projects) {
      if (out.has(p.id)) continue;
      const parent = p.parent_id != null ? String(p.parent_id) : undefined;
      if (parent === rootId || (parent != null && out.has(parent))) {
        out.add(p.id);
        added = true;
      }
    }
  }
  return out;
}
