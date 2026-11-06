import { describe, expect, it } from 'vitest';
import type { Project } from '$lib/domains/projects/types';
import { descendantIds, nestProjects } from './nestProjects';

const proj = (id: string, path: string, parentId?: number): Project =>
  ({ id, name: id, path, status: 'active', parent_id: parentId ?? null }) as unknown as Project;

describe('nestProjects', () => {
  it('nests a child under its parent and keeps it out of roots', () => {
    const result = nestProjects([proj('a', '/code/app'), proj('b', '/code/app/packages/ui')]);
    expect(result.map((r) => r.project.id)).toEqual(['a']);
    expect(result[0].children.map((c) => c.id)).toEqual(['b']);
  });

  it('groups every descendant under the topmost root, nearest-first', () => {
    const result = nestProjects([
      proj('root', '/code/mono'),
      proj('deep', '/code/mono/apps/web/sub'),
      proj('mid', '/code/mono/apps/web'),
    ]);
    expect(result.map((r) => r.project.id)).toEqual(['root']);
    expect(result[0].children.map((c) => c.id)).toEqual(['mid', 'deep']);
  });

  it('treats unrelated and sibling projects as separate roots', () => {
    const result = nestProjects([proj('a', '/code/a'), proj('b', '/code/b'), proj('c', '/other')]);
    expect(result.map((r) => r.project.id).sort()).toEqual(['a', 'b', 'c']);
    expect(result.every((r) => r.children.length === 0)).toBe(true);
  });

  it('matches paths case- and separator-insensitively, not by bare prefix', () => {
    const result = nestProjects([
      proj('win', 'C:\\Code\\App'),
      proj('child', 'c:/code/app/api'),
      proj('sibling', 'C:\\Code\\App-2'), // shares prefix but is not inside
    ]);
    expect(result.map((r) => r.project.id).sort()).toEqual(['sibling', 'win']);
    expect(result.find((r) => r.project.id === 'win')!.children.map((c) => c.id)).toEqual([
      'child',
    ]);
  });

  it('groups by explicit parent_id even when paths are unrelated', () => {
    const result = nestProjects([proj('1', '/a'), proj('2', '/totally/elsewhere', 1)]);
    expect(result.map((r) => r.project.id)).toEqual(['1']);
    expect(result[0].children.map((c) => c.id)).toEqual(['2']);
  });

  it('lets an explicit parent override an on-disk ancestor', () => {
    // '3' sits inside '1' on disk but is manually grouped under '2'.
    const result = nestProjects([proj('1', '/a'), proj('2', '/b'), proj('3', '/a/sub', 2)]);
    expect(result.map((r) => r.project.id).sort()).toEqual(['1', '2']);
    expect(result.find((r) => r.project.id === '2')!.children.map((c) => c.id)).toEqual(['3']);
    expect(result.find((r) => r.project.id === '1')!.children).toEqual([]);
  });

  it('ignores a parent_id that points at a missing project', () => {
    const result = nestProjects([proj('1', '/a', 999)]);
    expect(result.map((r) => r.project.id)).toEqual(['1']);
  });

  it('survives a parent_id cycle without hanging', () => {
    const result = nestProjects([proj('1', '/a', 2), proj('2', '/b', 1)]);
    expect(result.map((r) => r.project.id).sort()).toEqual(['1', '2']);
  });

  it('descendantIds collects the whole explicit subtree', () => {
    const projects = [proj('1', '/a'), proj('2', '/b', 1), proj('3', '/c', 2), proj('4', '/d')];
    expect([...descendantIds(projects, '1')].sort()).toEqual(['2', '3']);
  });
});
