/**
 * Project service for managing projects
 */

import { invoke } from '@tauri-apps/api/core';
import type { 
	Project, 
	CreateProjectRequest, 
	UpdateProjectRequest, 
	ProjectTemplate,
	ProjectStats,
	ProjectMetadata
} from '@/lib/domains/projects/types';
import { projectStore } from '@/lib/domains/projects/stores/projectStore';
import { logger } from '@/lib/domains/shared/services/logger';
import { cache } from '@/lib/domains/shared/services/cache';

const log = logger.createScoped('ProjectService');

class ProjectService {
	private initialized = false;

	/**
	 * Initialize the project service
	 */
	async initialize(): Promise<void> {
		if (this.initialized) return;

		try {
			log.info('Initializing project service');
			await this.loadProjects();
			this.initialized = true;
			log.info('Project service initialized successfully');
		} catch (error) {
			log.error('Failed to initialize project service', error);
			throw error;
		}
	}

	/**
	 * Load all projects from the backend
	 */
	async loadProjects(): Promise<Project[]> {
		try {
			projectStore.setLoading(true);
			
			// Check cache first
			const cached = cache.get<Project[]>('projects');
			if (cached) {
				projectStore.setProjects(cached);
				projectStore.setLoading(false);
				return cached;
			}

			// Load from backend
			const projects = await invoke<Project[]>('get_all_projects');
			
			// Update store and cache
			projectStore.setProjects(projects);
			cache.set('projects', projects, 5 * 60 * 1000); // 5 minutes
			
			projectStore.setLoading(false);
			log.info(`Loaded ${projects.length} projects`);
			return projects;
		} catch (error) {
			projectStore.setError('Failed to load projects');
			projectStore.setLoading(false);
			log.error('Failed to load projects', error);
			throw error;
		}
	}

	/**
	 * Get a specific project by ID
	 */
	async getProject(id: string): Promise<Project | null> {
		try {
			log.info('Loading project', { id });
			
			const project = await invoke<Project | null>('get_project', { id: parseInt(id) });
			
			log.info('Project loaded successfully', { id, found: !!project });
			return project;
		} catch (error) {
			log.error('Failed to load project', error);
			throw error;
		}
	}

	/**
	 * Create a new project
	 */
	async createProject(request: CreateProjectRequest): Promise<Project> {
		try {
			log.info('Creating new project', { name: request.name });
			
			const invokeParams = {
				name: request.name,
				description: request.description,
				path: request.path,
				framework: request.framework,
				package_manager: request.package_manager,
				build_command: request.build_command,
				start_command: request.start_command,
				test_command: request.test_command,
				output_directory: request.output_directory,
				dev_port: request.dev_port,
				prod_port: request.prod_port
			};
			
			const project = await invoke<Project>('create_project', invokeParams);
			
			// Update store
			projectStore.setProjects(prev => [...prev, project]);
			
			// Update cache
			cache.delete('projects');
			
			log.info('Project created successfully', { id: project.id });
			return project;
		} catch (error) {
			log.error('Failed to create project', error);
			throw error;
		}
	}

	/**
	 * Update an existing project
	 */
	async updateProject(id: string, updates: UpdateProjectRequest): Promise<Project> {
		try {
			log.info('Updating project', { id, updates });
			
			const project = await invoke<Project>('update_project', {
				id: parseInt(id),
				name: updates.name,
				description: updates.description,
				path: updates.path,
				status: updates.status,
				framework: updates.framework,
				package_manager: updates.package_manager,
				build_command: updates.build_command,
				start_command: updates.start_command,
				test_command: updates.test_command,
				output_directory: updates.output_directory,
				dev_port: updates.dev_port,
				prod_port: updates.prod_port
			});
			
			// Update store
			projectStore.updateProject(id, updates);
			
			// Update cache
			cache.delete('projects');
			
			log.info('Project updated successfully', { id });
			return project;
		} catch (error) {
			log.error('Failed to update project', error);
			throw error;
		}
	}

	/**
	 * Delete a project
	 */
	async deleteProject(id: string): Promise<void> {
		try {
			log.info('Deleting project', { id });
			
			await invoke('delete_project', { id: parseInt(id) });
			
			// Update store
			projectStore.deleteProject(id);
			
			// Update cache
			cache.delete('projects');
			
			log.info('Project deleted successfully', { id });
		} catch (error) {
			log.error('Failed to delete project', error);
			throw error;
		}
	}

	/**
	 * Open a project (increment open count, set as active)
	 */
	async openProject(id: string): Promise<void> {
		try {
			log.info('Opening project', { id });
			
			// Update metadata
			projectStore.incrementOpenCount(id);
			projectStore.setActiveProject(id);
			
			// Update last opened time in backend
			await invoke('open_project', { id: parseInt(id) });
			
			log.info('Project opened successfully', { id });
		} catch (error) {
			log.error('Failed to open project', error);
			throw error;
		}
	}

	/**
	 * Get project templates
	 */
	async getTemplates(): Promise<ProjectTemplate[]> {
		try {
			// Check cache first
			const cached = cache.get<ProjectTemplate[]>('project_templates');
			if (cached) {
				return cached;
			}

			const templates = await invoke<ProjectTemplate[]>('get_project_templates');
			
			// Cache for 1 hour
			cache.set('project_templates', templates, 60 * 60 * 1000);
			
			return templates;
		} catch (error) {
			log.error('Failed to get project templates', error);
			throw error;
		}
	}

	/**
	 * Create project from template
	 */
	async createFromTemplate(templateId: string, projectData: Omit<CreateProjectRequest, 'type'>): Promise<Project> {
		try {
			log.info('Creating project from template', { templateId, name: projectData.name });
			
			const project = await invoke<Project>('create_project_from_template', {
				templateId,
				projectData
			});
			
			// Update store
			projectStore.setProjects(prev => [...prev, project]);
			
			// Update cache
			cache.delete('projects');
			
			log.info('Project created from template successfully', { id: project.id });
			return project;
		} catch (error) {
			log.error('Failed to create project from template', error);
			throw error;
		}
	}

	/**
	 * Get project statistics
	 */
	async getStats(): Promise<ProjectStats> {
		try {
			// Check cache first
			const cached = cache.get<ProjectStats>('project_stats');
			if (cached) {
				return cached;
			}

			const stats = await invoke<ProjectStats>('get_project_stats');
			
			// Cache for 5 minutes
			cache.set('project_stats', stats, 5 * 60 * 1000);
			
			return stats;
		} catch (error) {
			log.error('Failed to get project stats', error);
			throw error;
		}
	}

	/**
	 * Refresh project metadata (size, file count, git info, etc.)
	 */
	async refreshProjectMetadata(id: string): Promise<void> {
		try {
			log.info('Refreshing project metadata', { id });
			
			const metadata = await invoke<Partial<ProjectMetadata>>('get_project_metadata', { id });
			
			// Update store
			projectStore.updateProjectMetadata(id, metadata);
			
			log.info('Project metadata refreshed successfully', { id });
		} catch (error) {
			log.error('Failed to refresh project metadata', error);
			throw error;
		}
	}

	/**
	 * Search projects
	 */
	async searchProjects(query: string): Promise<Project[]> {
		try {
			const projects = await invoke<Project[]>('search_projects', { query });
			return projects;
		} catch (error) {
			log.error('Failed to search projects', error);
			throw error;
		}
	}

	/**
	 * Open project in file explorer
	 */
	async openProjectInExplorer(path: string): Promise<void> {
		try {
			log.info('Opening project in explorer', { path });
			
			await invoke('open_project_in_explorer', { path });
			
			log.info('Project opened in explorer successfully', { path });
		} catch (error) {
			log.error('Failed to open project in explorer', error);
			throw error;
		}
	}

	/**
	 * Export project
	 */
	async exportProject(id: string, format: 'zip' | 'tar' = 'zip'): Promise<string> {
		try {
			log.info('Exporting project', { id, format });
			
			const exportPath = await invoke<string>('export_project', { id, format });
			
			log.info('Project exported successfully', { id, exportPath });
			return exportPath;
		} catch (error) {
			log.error('Failed to export project', error);
			throw error;
		}
	}

	/**
	 * Import project
	 */
	async importProject(filePath: string): Promise<Project> {
		try {
			log.info('Importing project', { filePath });
			
			const project = await invoke<Project>('import_project', { filePath });
			
			// Update store
			projectStore.setProjects(prev => [...prev, project]);
			
			// Update cache
			cache.delete('projects');
			
			log.info('Project imported successfully', { id: project.id });
			return project;
		} catch (error) {
			log.error('Failed to import project', error);
			throw error;
		}
	}
}

// Export singleton instance
export const projectService = new ProjectService();
