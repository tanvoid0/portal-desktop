import { describe, expect, it } from 'vitest';
import type { Project } from '$lib/domains/projects/types';
import { areaOf, groupByArea } from './groupProjects';

const node = (id: string, path: string, discovery_path?: string, starred = false) => ({
  project: { id, path, discovery_path, starred } as unknown as Project,
  children: [],
});

describe('groupByArea', () => {
  it('derives the area from discovery_path, falling back to the last two folders of path', () => {
    expect(areaOf({ path: 'C:\\x\\y', discovery_path: 'production/flutter/aipairs' })).toBe(
      'production/flutter'
    );
    expect(areaOf({ path: 'C:\\code\\tools\\app' })).toBe('code/tools');
  });

  it('keys a manually-added project the same as a discovery-found sibling in the same folder', () => {
    const discovered = areaOf({ path: 'C:\\x', discovery_path: 'production/devstrail/sibling' });
    const manual = areaOf({ path: 'C:\\Users\\me\\production\\devstrail\\my-manual-project' });
    expect(discovered).toBe('production/devstrail');
    expect(manual).toBe(discovered);
  });

  it('puts starred first, then areas alphabetically, keeping starred in their area', () => {
    const groups = groupByArea([
      node('1', '/a', 'z/app1'),
      node('2', '/b', 'a/app2', true),
      node('3', '/c', 'a/app3'),
    ]);
    expect(groups.map((g) => g.title)).toEqual(['Starred', 'a', 'z']);
    expect(groups[2].items.map((i) => i.project.id)).toEqual(['1']);
    expect(groups[1].items.map((i) => i.project.id)).toEqual(['2', '3']);
  });
});
