/**
 * Coder sidebar UI state — active workspace, expansion, scope.
 * Separate from session runtime state in coderSession.
 */

import { projectUi } from '$lib/domains/projects/state/projectUi.svelte';

export interface CoderUiOptions {
  storageKey?: string;
  projectIdKey?: string;
  /** Mirror the active project into the app-wide projectUi (Portal only). */
  linkProjectUi?: boolean;
  /** Start with every workspace expanded. */
  showAll?: boolean;
}

export class CoderUiState {
  private storageKey: string;
  private projectIdKey: string;
  private linkProjectUi: boolean;

  constructor(opts: CoderUiOptions = {}) {
    this.storageKey = opts.storageKey ?? 'portal-coder-last-workspace';
    this.projectIdKey = opts.projectIdKey ?? 'portal-coder-last-project-id';
    this.linkProjectUi = opts.linkProjectUi ?? true;
    this.showAllWorkspaces = opts.showAll ?? false;
  }

  /** Currently selected workspace path (portal project path). */
  activeProjectPath = $state<string | null>(null);
  /** Portal project id for the active workspace. */
  activeProjectId = $state<string | null>(null);
  /** Which workspace sections are expanded in the sidebar. */
  expandedProjects = $state<Record<string, boolean>>({});
  /** When true, show sessions grouped under all workspaces. */
  showAllWorkspaces = $state(false);

  initFromStorage(): void {
    if (typeof window === 'undefined') return;
    try {
      const cached = localStorage.getItem(this.storageKey);
      const cachedId = localStorage.getItem(this.projectIdKey);
      if (cached) {
        this.activeProjectPath = cached;
        this.expandedProjects = { [cached]: true };
      }
      if (cachedId) {
        this.activeProjectId = cachedId;
        if (this.linkProjectUi) projectUi.setActiveProject(cachedId);
      }
    } catch {
      // ignore
    }
  }

  persistProject(path: string, projectId?: string | null): void {
    if (typeof window === 'undefined' || !path) return;
    try {
      localStorage.setItem(this.storageKey, path);
      if (projectId) {
        localStorage.setItem(this.projectIdKey, projectId);
      }
    } catch {
      // ignore
    }
  }

  setActiveProject(path: string | null, projectId?: string | null): void {
    this.activeProjectPath = path;
    if (projectId != null) {
      this.activeProjectId = projectId;
      if (this.linkProjectUi) projectUi.setActiveProject(projectId);
    }
    if (path) {
      this.expandedProjects = { ...this.expandedProjects, [path]: true };
      this.persistProject(path, projectId ?? this.activeProjectId);
    }
  }

  /** A folder opened on purpose this run. Unlike a stale saved path, it need not be a project. */
  private openedPath: string | null = null;

  openWorkspace(path: string, projectId?: string | null): void {
    // setActiveProject keeps the old id, which belongs to a different folder.
    if (!projectId) {
      this.activeProjectId = null;
      try {
        localStorage.removeItem(this.projectIdKey);
      } catch {
        // ignore
      }
    }
    this.openedPath = path;
    this.setActiveProject(path, projectId);
  }

  resolveProjectFromList(projects: Array<{ id: string; path: string }>): void {
    if (!this.activeProjectPath) return;
    const match = projects.find((p) => p.path === this.activeProjectPath);
    if (match) {
      this.activeProjectId = match.id;
      if (this.linkProjectUi) projectUi.setActiveProject(match.id);
      this.persistProject(match.path, match.id);
    } else if (projects.length > 0 && this.activeProjectPath !== this.openedPath) {
      // A saved path that is no project here (a /media/... Linux path on Windows) must not seed
      // the workspace: drop it so the caller falls back to projects[0]. An empty list means the
      // projects have not loaded yet, which must not wipe a valid saved choice.
      // ponytail: also drops a real folder that was never a project; validate_project_path
      // (exists + is_dir) would keep those, at the cost of an async check here.
      this.activeProjectPath = null;
      try {
        localStorage.removeItem(this.storageKey);
      } catch {
        // ignore
      }
    }
  }

  toggleExpanded(path: string): void {
    this.expandedProjects = {
      ...this.expandedProjects,
      [path]: !this.isExpanded(path),
    };
  }

  isExpanded(path: string): boolean {
    if (this.showAllWorkspaces) {
      return this.expandedProjects[path] ?? true;
    }
    if (path === this.activeProjectPath) return true;
    return this.expandedProjects[path] ?? false;
  }

  syncFromThread(workspaceRoot: string, projectId?: number | null): void {
    if (!workspaceRoot && projectId == null) return;
    const id = projectId != null ? String(projectId) : this.activeProjectId;
    if (workspaceRoot) {
      this.setActiveProject(workspaceRoot, id);
    } else if (id) {
      this.activeProjectId = id;
      if (this.linkProjectUi) projectUi.setActiveProject(id);
    }
  }

  activeProjectIdAsNumber(): number | null {
    if (!this.activeProjectId) return null;
    const n = Number.parseInt(this.activeProjectId, 10);
    return Number.isNaN(n) ? null : n;
  }
}

export const coderUi = new CoderUiState();
