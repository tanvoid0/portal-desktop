import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { coderUi } from './coderUi.svelte';

const KEY = 'portal-coder-last-workspace';
const STALE = '/media/tan/Workspace/production/portal/portal_desktop';
const PROJECTS = [
  { id: '1', path: 'D:\\code\\portal_desktop' },
  { id: '2', path: 'D:\\code\\other' },
];

const store = new Map<string, string>();

beforeEach(() => {
  store.clear();
  coderUi.activeProjectPath = null;
  coderUi.activeProjectId = null;
  // The node env has no window/localStorage; coderUi only touches them behind a window guard.
  vi.stubGlobal('window', {});
  vi.stubGlobal('localStorage', {
    getItem: (k: string) => store.get(k) ?? null,
    setItem: (k: string, v: string) => void store.set(k, v),
    removeItem: (k: string) => void store.delete(k),
  });
});

afterEach(() => vi.unstubAllGlobals());

describe('coderUi.resolveProjectFromList (QA-134)', () => {
  it('keeps a saved path that is a known project and adopts its id', () => {
    store.set(KEY, PROJECTS[1].path);
    coderUi.initFromStorage();
    coderUi.resolveProjectFromList(PROJECTS);
    expect(coderUi.activeProjectPath).toBe(PROJECTS[1].path);
    expect(coderUi.activeProjectId).toBe('2');
  });

  it('drops a saved path that matches no project, so the caller falls back to projects[0]', () => {
    store.set(KEY, STALE);
    coderUi.initFromStorage();
    expect(coderUi.activeProjectPath).toBe(STALE); // the unvalidated seed behind the bug
    coderUi.resolveProjectFromList(PROJECTS);
    expect(coderUi.activeProjectPath).toBeNull();
    expect(store.has(KEY)).toBe(false);
  });

  it('keeps the saved path while the project list has not loaded', () => {
    store.set(KEY, STALE);
    coderUi.initFromStorage();
    coderUi.resolveProjectFromList([]);
    expect(coderUi.activeProjectPath).toBe(STALE);
    expect(store.get(KEY)).toBe(STALE);
  });
});
