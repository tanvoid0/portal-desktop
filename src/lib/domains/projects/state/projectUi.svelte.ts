/**
 * Project UI session state (not server data).
 */

class ProjectUiState {
  activeProjectId = $state<string | null>(null);

  setActiveProject(id: string | null): void {
    this.activeProjectId = id;
  }
}

export const projectUi = new ProjectUiState();
