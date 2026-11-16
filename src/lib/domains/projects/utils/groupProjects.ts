/**
 * Group nested project roots into list sections: a "Starred" section first, then
 * one section per area (`discovery_path` minus its last segment, else the last two
 * folders of `path` — same shape, so a manually-added project groups with a
 * discovery-found sibling in the same folder). Order within a section follows the
 * input order.
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
  // discovery_path ends in the project itself; a bare path ends in it too — drop it
  // to get the ancestor folders.
  const ancestors = segments.slice(0, -1);
  // discovery_path is already root-relative (`<root>/<category>/<name>`); a bare
  // path isn't, so cap it to the last two folders — discovery's default category
  // depth — to land a manually-added project in the same area as a discovery-found
  // sibling, without spilling the whole absolute path (drive letter, username) into
  // the UI.
  // ponytail: approximates the default project_depth (2) rather than reading the
  // real DiscoveryConfig; wire the configured roots through if a non-default depth
  // needs an exact match.
  return (p.discovery_path ? ancestors : ancestors.slice(-2)).join('/') || '/';
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
