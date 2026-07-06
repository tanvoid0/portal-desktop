import { invokeClient } from "$lib/utils/invokeClient";
import type { Project } from "$lib/domains/projects/types";
import {
  normalizeProject,
  normalizeProjects,
} from "$lib/domains/projects/utils/normalizeProject";

export async function fetchAllProjects(): Promise<Project[]> {
  const projects = await invokeClient.post<unknown>("get_all_projects");
  return normalizeProjects(projects);
}

export async function fetchProjectById(id: string): Promise<Project | null> {
  if (!id) {
    throw new Error("Project ID is required");
  }

  const projectId = parseInt(id, 10);
  if (isNaN(projectId)) {
    throw new Error(`Invalid project ID: ${id}`);
  }

  const project = await invokeClient.post<unknown>("get_project", {
    id: projectId,
  });

  return project
    ? normalizeProject(project as Record<string, unknown>)
    : null;
}
