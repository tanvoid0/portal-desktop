/**
 * Project service for managing projects
 */

import { invokeClient } from "$lib/utils/invokeClient";
import type {
  Project,
  CreateProjectRequest,
  UpdateProjectRequest,
  ProjectStats,
} from "$lib/domains/projects/types";
import { fetchAllProjects, fetchProjectById } from "$lib/domains/projects/api/projectApi";
import { projectUi } from "$lib/domains/projects/state/projectUi.svelte";
import { projectStore } from "$lib/domains/projects/stores/projectStore";
import { logger } from "$lib/domains/shared/services/logger";
import { cache } from "$lib/domains/shared/services/cache";
import {
  queryClient,
  invalidateDashboardOverview,
  invalidateProjectsList,
  invalidateProjectDetail,
} from "$lib/domains/shared/query";
import { patternCollector, suggestionEngine } from "$lib/domains/learning";
import {
  normalizeProject,
  toProjectInvokePayload,
} from "$lib/domains/projects/utils/normalizeProject";

const log = logger.createScoped("ProjectService");

function invalidateProjectCaches(id: string): void {
  invalidateProjectsList(queryClient);
  invalidateProjectDetail(queryClient, id);
  invalidateDashboardOverview(queryClient);
}

// InvokeClient is used for all backend calls

/**
 * Extract context string from project information
 * Enhanced with hierarchical context support
 * Exported for use in UI components
 */
export function extractContext(
  framework: string | null | undefined,
  packageManager: string | null | undefined,
  projectPath?: string | null,
): string {
  const parts: string[] = [];

  if (framework) {
    parts.push(`fw_${framework.toLowerCase().replace(/\s+/g, "_")}`);
  }

  if (packageManager) {
    parts.push(`pm_${packageManager.toLowerCase()}`);
  }

  if (projectPath) {
    // Extract project identifier from path (last directory name)
    try {
      const pathParts = projectPath.split(/[/\\]/).filter((p) => p);
      if (pathParts.length > 0) {
        const projectName = pathParts[pathParts.length - 1];
        parts.push(`proj_${projectName.toLowerCase().replace(/\s+/g, "_")}`);
      }
    } catch {
      // Ignore path parsing errors
    }
  }

  return parts.length > 0 ? parts.join("_") : "global";
}

/**
 * Get context hierarchy (most specific to least specific)
 * For better pattern matching across similar contexts
 */
function getContextHierarchy(context: string): string[] {
  const parts = context.split("_");
  const hierarchy: string[] = [];

  // Build increasingly general contexts
  for (let i = 1; i <= parts.length; i++) {
    const partial = parts.slice(0, i).join("_");
    hierarchy.push(partial);
  }

  // Always include global as fallback
  if (!hierarchy.includes("global")) {
    hierarchy.push("global");
  }

  return hierarchy;
}

class ProjectService {
  private initialized = false;

  /**
   * Initialize the project service
   */
  async initialize(): Promise<void> {
    if (this.initialized) return;
    log.info("Project service initialized");
    this.initialized = true;
  }

  /**
   * Load all projects from the backend (invoke-only; caching handled by TanStack Query).
   */
  async loadProjects(): Promise<Project[]> {
    const projects = await fetchAllProjects();
    log.info(`Loaded ${projects.length} projects`);
    return projects;
  }

  /**
   * Get a specific project by ID
   */
  async getProject(id: string): Promise<Project | null> {
    try {
      log.info("Loading project", { id });
      const project = await fetchProjectById(id);
      log.info("Project loaded successfully", { id, found: !!project });
      return project;
    } catch (error) {
      log.error("Failed to load project", { error });
      throw error;
    }
  }

  /**
   * Create a new project
   */
  async createProject(request: CreateProjectRequest): Promise<Project> {
    try {
      log.info("Creating new project", { name: request.name });

      const invokeParams = toProjectInvokePayload({
        name: request.name,
        description: request.description,
        path: request.path,
        framework_ids: request.framework_ids ?? [],
        package_manager_ids: request.package_manager_ids ?? [],
        language_ids: request.language_ids ?? [],
        build_command: request.build_command,
        start_command: request.start_command,
        test_command: request.test_command,
        output_directory: request.output_directory,
        dev_port: request.dev_port,
        prod_port: request.prod_port,
      });

      const project = normalizeProject(
        (await invokeClient.post<unknown>("create_project", invokeParams)) as Record<
          string,
          unknown
        >,
      );

      invalidateProjectCaches(String(project.id));

      // Learn from project creation pattern
      // TODO: Update learning pattern collection to work with framework_ids and package_manager_ids arrays
      // For now, we skip this since we need to fetch framework/package manager names from IDs

      log.info("Project created successfully", { id: project.id });
      return project;
    } catch (error) {
      log.error("Failed to create project", { error });
      throw error;
    }
  }

  /**
   * Update an existing project
   */
  async updateProject(
    id: string,
    updates: UpdateProjectRequest,
  ): Promise<Project> {
    try {
      log.info("Updating project", { id, updates });

      const project = normalizeProject(
        (await invokeClient.post<unknown>(
          "update_project",
          toProjectInvokePayload({
            id: parseInt(id, 10),
            ...updates,
          }),
        )) as Record<string, unknown>,
      );

      invalidateProjectCaches(id);

      log.info("Project updated successfully", { id });
      return project;
    } catch (error) {
      log.error("Failed to update project", { error });
      throw error;
    }
  }

  /**
   * Delete a project
   */
  async deleteProject(id: string): Promise<void> {
    try {
      if (!id) {
        throw new Error("Project ID is required");
      }

      const projectId = parseInt(id, 10);
      if (isNaN(projectId)) {
        throw new Error(`Invalid project ID: ${id}`);
      }

      log.info("Deleting project", { id: projectId });

      await invokeClient.post("delete_project", { id: projectId });

      invalidateProjectCaches(id);

      log.info("Project deleted successfully", { id });
    } catch (error) {
      log.error("Failed to delete project", { error });
      throw error;
    }
  }

  /**
   * Open a project (increment open count, set as active)
   */
  async openProject(id: string): Promise<void> {
    try {
      if (!id) {
        throw new Error("Project ID is required");
      }

      const projectId = parseInt(id, 10);
      if (isNaN(projectId)) {
        throw new Error(`Invalid project ID: ${id}`);
      }

      log.info("Opening project", { id: projectId });

      projectUi.setActiveProject(id);
      projectStore.setActiveProject(id);

      await invokeClient.post("open_project", { id: projectId });

      log.info("Project opened successfully", { id });
    } catch (error) {
      log.error("Failed to open project", { error });
      throw error;
    }
  }

  // Note: getTemplates() and createFromTemplate() removed - no backend support
  // These features can be implemented when backend commands are added

  /**
   * Get project statistics
   */
  async getStats(): Promise<ProjectStats> {
    try {
      // Check cache first
      const cached = cache.get<ProjectStats>("project_stats");
      if (cached !== null) {
        return cached;
      }

      const stats = await invokeClient.post<ProjectStats>("get_project_stats");

      // Cache for 5 minutes
      cache.set("project_stats", stats, 5 * 60 * 1000);

      return stats;
    } catch (error) {
      log.error("Failed to get project stats", { error });
      throw error;
    }
  }

  /**
   * Refresh project metadata (size, file count, git info, etc.)
   */
  async refreshProjectMetadata(id: string): Promise<void> {
    try {
      log.info("Refreshing project metadata", { id });

      const result = normalizeProject(
        (await invokeClient.post<unknown>("refresh_project_metadata", {
          id: parseInt(id, 10),
        })) as Record<string, unknown>,
      );

      if (!result) {
        throw new Error("Project not found");
      }

      invalidateProjectCaches(id);

      log.info("Project metadata refreshed successfully", { id });
    } catch (error) {
      log.error("Failed to refresh project metadata", { error });
      throw error;
    }
  }

  // Note: searchProjects() removed - no backend support
  // Use getProjects() with filters instead, or implement backend command

  /**
   * Open project in file explorer
   */
  async openProjectInExplorer(path: string): Promise<void> {
    try {
      log.info("Opening project in explorer", { path });

      await invokeClient.post("open_project_in_explorer", { path });

      log.info("Project opened in explorer successfully", { path });
    } catch (error) {
      log.error("Failed to open project in explorer", { error });
      throw error;
    }
  }

  // Note: exportProject() and importProject() removed - no backend support
  // These features can be implemented when backend commands are added

  /**
   * Get intelligent suggestions for new project setup
   * Based on learned patterns from previous projects
   */
  async getProjectSetupSuggestions(
    framework?: string,
    packageManager?: string,
  ): Promise<{
    framework?: string;
    packageManager?: string;
    buildCommand?: string;
    startCommand?: string;
    devPort?: number;
  }> {
    try {
      const context = extractContext(framework, packageManager);

      // Get framework suggestions
      const frameworkSuggestions =
        await suggestionEngine.getContextualSuggestions("framework", context);

      // Get config suggestions
      const configSuggestions = await suggestionEngine.getContextualSuggestions(
        "config",
        context,
      );

      const suggestions: {
        framework?: string;
        packageManager?: string;
        buildCommand?: string;
        startCommand?: string;
        devPort?: number;
      } = {};

      // Extract most common framework if suggested
      if (frameworkSuggestions.length > 0) {
        const topSuggestion = frameworkSuggestions[0];
        if (topSuggestion.pattern_data.framework) {
          suggestions.framework = topSuggestion.pattern_data
            .framework as string;
        }
      }

      // Extract config suggestions
      if (configSuggestions.length > 0) {
        const topConfig = configSuggestions[0].pattern_data as Record<
          string,
          unknown
        >;
        if (topConfig.package_manager) {
          suggestions.packageManager = topConfig.package_manager as string;
        }
        if (topConfig.build_command) {
          suggestions.buildCommand = topConfig.build_command as string;
        }
        if (topConfig.start_command) {
          suggestions.startCommand = topConfig.start_command as string;
        }
        if (topConfig.dev_port) {
          suggestions.devPort = topConfig.dev_port as number;
        }
      }

      log.info("Project setup suggestions retrieved", { suggestions, context });
      return suggestions;
    } catch (error) {
      log.error("Failed to get project setup suggestions", { error });
      return {};
    }
  }
}

// Export singleton instance
export const projectService = new ProjectService();
