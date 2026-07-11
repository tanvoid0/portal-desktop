/**
 * Coder sidebar UI state — active workspace, expansion, scope.
 * Separate from session runtime state in coderSession.
 */

import { projectUi } from "$lib/domains/projects/state/projectUi.svelte";

const STORAGE_KEY = "portal-coder-last-workspace";
const PROJECT_ID_STORAGE_KEY = "portal-coder-last-project-id";

class CoderUiState {
  /** Currently selected workspace path (portal project path). */
  activeProjectPath = $state<string | null>(null);
  /** Portal project id for the active workspace. */
  activeProjectId = $state<string | null>(null);
  /** Which workspace sections are expanded in the sidebar. */
  expandedProjects = $state<Record<string, boolean>>({});
  /** When true, show sessions grouped under all workspaces. */
  showAllWorkspaces = $state(false);

  initFromStorage(): void {
    if (typeof window === "undefined") return;
    try {
      const cached = localStorage.getItem(STORAGE_KEY);
      const cachedId = localStorage.getItem(PROJECT_ID_STORAGE_KEY);
      if (cached) {
        this.activeProjectPath = cached;
        this.expandedProjects = { [cached]: true };
      }
      if (cachedId) {
        this.activeProjectId = cachedId;
        projectUi.setActiveProject(cachedId);
      }
    } catch {
      // ignore
    }
  }

  persistProject(path: string, projectId?: string | null): void {
    if (typeof window === "undefined" || !path) return;
    try {
      localStorage.setItem(STORAGE_KEY, path);
      if (projectId) {
        localStorage.setItem(PROJECT_ID_STORAGE_KEY, projectId);
      }
    } catch {
      // ignore
    }
  }

  setActiveProject(path: string | null, projectId?: string | null): void {
    this.activeProjectPath = path;
    if (projectId != null) {
      this.activeProjectId = projectId;
      projectUi.setActiveProject(projectId);
    }
    if (path) {
      this.expandedProjects = { ...this.expandedProjects, [path]: true };
      this.persistProject(path, projectId ?? this.activeProjectId);
    }
  }

  resolveProjectFromList(
    projects: Array<{ id: string; path: string }>,
  ): void {
    if (!this.activeProjectPath) return;
    const match = projects.find((p) => p.path === this.activeProjectPath);
    if (match) {
      this.activeProjectId = match.id;
      projectUi.setActiveProject(match.id);
      this.persistProject(match.path, match.id);
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
      projectUi.setActiveProject(id);
    }
  }

  activeProjectIdAsNumber(): number | null {
    if (!this.activeProjectId) return null;
    const n = Number.parseInt(this.activeProjectId, 10);
    return Number.isNaN(n) ? null : n;
  }
}

export const coderUi = new CoderUiState();
