/**
 * Projects domain exports
 */

// Types
export type {
  Project,
  ProjectSettings,
  ProjectMetadata,
  GitInfo,
  DependencyInfo,
  CreateProjectRequest,
  UpdateProjectRequest,
  ProjectTemplate,
  ProjectStats,
} from "./types";

export { ProjectStatus } from "./types";

// Session state (UI only)
export { projectStore } from "./stores/projectStore";
export { projectUi } from "./state/projectUi.svelte";

// Services
export { projectService } from "./services/projectService";

// Queries
export { createProjectsQuery, createProjectQuery } from "./queries/projectQueries";
export { fetchAllProjects, fetchProjectById } from "./api/projectApi";
