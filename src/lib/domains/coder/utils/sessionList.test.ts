import { describe, expect, it } from 'vitest';
import { groupSessionsByProject } from './sessionList.js';
import type { CoderThread } from '../types.js';
import type { Project } from '$lib/domains/projects/types';

function thread(id: string, root: string, updated: string): CoderThread {
  return {
    id,
    workspace_root: root,
    updated_at: updated,
    title: id,
    messages: [],
  } as unknown as CoderThread;
}

function project(id: string, name: string, path: string): Project {
  return { id, name, path } as unknown as Project;
}

// Three workspaces whose alphabetical order is the reverse of their activity order.
const PROJECTS = [
  project('1', 'alpha', '/w/alpha'),
  project('2', 'beta', '/w/beta'),
  project('3', 'zeta', '/w/zeta'),
];

const THREADS = [
  thread('a1', '/w/alpha', '2026-11-01T00:00:00Z'),
  thread('b1', '/w/beta', '2026-11-03T00:00:00Z'),
  thread('z1', '/w/zeta', '2026-11-05T00:00:00Z'),
];

const labels = (ws: { label: string }[]) => ws.map((w) => w.label);

describe('groupSessionsByProject', () => {
  it('puts the most recently used workspace first, not the alphabetical one', () => {
    expect(labels(groupSessionsByProject(THREADS, PROJECTS, 'updated_desc'))).toEqual([
      'zeta',
      'beta',
      'alpha',
    ]);
  });

  it('reverses for oldest-first', () => {
    expect(labels(groupSessionsByProject(THREADS, PROJECTS, 'updated_asc'))).toEqual([
      'alpha',
      'beta',
      'zeta',
    ]);
  });

  it('still orders by name for the name-based sorts', () => {
    expect(labels(groupSessionsByProject(THREADS, PROJECTS, 'title_asc'))).toEqual([
      'alpha',
      'beta',
      'zeta',
    ]);
    expect(labels(groupSessionsByProject(THREADS, PROJECTS, 'title_desc'))).toEqual([
      'zeta',
      'beta',
      'alpha',
    ]);
  });

  it('sinks workspaces with no sessions below active ones', () => {
    const withEmpty = [...PROJECTS, project('4', 'aaa-empty', '/w/empty')];
    const order = labels(groupSessionsByProject(THREADS, withEmpty, 'updated_desc'));
    expect(order[order.length - 1]).toBe('aaa-empty');
  });

  it('ranks a running session above a more recently updated idle one', () => {
    const running = new Set(['a1']);
    expect(labels(groupSessionsByProject(THREADS, PROJECTS, 'updated_desc', running))[0]).toBe(
      'alpha'
    );
  });

  it('defaults to recent-first when no sort is given', () => {
    expect(labels(groupSessionsByProject(THREADS, PROJECTS))[0]).toBe('zeta');
  });
});
