/**
 * Group nested project roots into list sections: a "Starred" section first, then
 * one section per area (`discovery_path` minus its last segment, else the parent
 * folder of `path`). Order within a section follows the input order.
 */

import type { NestedProject } from './nestProjects';

export interface ProjectGroup {
  key: string;
  title: string;
  items: NestedProject[];
}

export const STARRED_KEY = '__starred';

export function areaOf(p: { discovery_path?: string; path: string }): string {
  const segments = (p.discovery_path || p.path).replace(/\\/g, '/').split('/').filter(Boolean);
  // discovery_path ends in the project itself; a bare path ends in it too.
  return p.discovery_path ? segments.slice(0, -1).join('/') || '/' : (segments.at(-2) ?? '/');
}

export function groupByArea(nodes: NestedProject[]): ProjectGroup[] {
  const areas = new Map<string, NestedProject[]>();
  for (const node of nodes) {
    const area = areaOf(node.project);
    const list = areas.get(area);
    if (list) list.push(node);
    else areas.set(area, [node]);
  }
  const groups: ProjectGroup[] = [...areas]
    .sort(([a], [b]) => a.localeCompare(b))
    .map(([area, items]) => ({ key: area, title: area, items }));
  const starred = nodes.filter((n) => n.project.starred);
  if (starred.length) groups.unshift({ key: STARRED_KEY, title: 'Starred', items: starred });
  return groups;
}
