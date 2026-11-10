import { invokeClient } from '$lib/utils/invokeClient';
import type { Project, ProjectStage } from '$lib/domains/projects/types';
import { normalizeProject, normalizeProjects } from '$lib/domains/projects/utils/normalizeProject';

export async function fetchAllProjects(): Promise<Project[]> {
  const projects = await invokeClient.post<unknown>('get_all_projects');
  return normalizeProjects(projects);
}

export async function fetchProjectById(id: string): Promise<Project | null> {
  if (!id) {
    throw new Error('Project ID is required');
  }

  const projectId = parseInt(id, 10);
  if (isNaN(projectId)) {
    throw new Error(`Invalid project ID: ${id}`);
  }

  const project = await invokeClient.post<unknown>('get_project', {
    id: projectId,
  });

  return project ? normalizeProject(project as Record<string, unknown>) : null;
}

/** Group a project under another, or ungroup it when parentId is null. */
export async function setProjectParent(
  id: number,
  parentId: number | null
): Promise<Project | null> {
  const project = await invokeClient.post<unknown>('set_project_parent', {
    id,
    parentId,
  });
  return project ? normalizeProject(project as Record<string, unknown>) : null;
}

/** Opens the project in the user's default editor; resolves to the editor name used. */
export async function openProjectInIde(id: number): Promise<string> {
  return invokeClient.post<string>('open_project_in_ide', { id });
}

/** Flips the project's star; resolves to the updated project. */
export async function toggleProjectStar(id: number): Promise<Project | null> {
  const project = await invokeClient.post<unknown>('toggle_project_star', { id });
  return project ? normalizeProject(project as Record<string, unknown>) : null;
}

/** Sets or clears a project's SDLC stage. */
export async function setProjectStage(
  id: number,
  stage: ProjectStage | null
): Promise<Project | null> {
  const project = await invokeClient.post<unknown>('set_project_stage', { id, stage });
  return project ? normalizeProject(project as Record<string, unknown>) : null;
}

/** Flags a project archived or active. */
export async function setProjectArchived(id: number, archived: boolean): Promise<Project | null> {
  const project = await invokeClient.post<unknown>('set_project_archived', { id, archived });
  return project ? normalizeProject(project as Record<string, unknown>) : null;
}
