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
	ProjectStats
} from './types';

export { ProjectStatus } from './types';

// Stores
export {
	projectStore,
	activeProject,
	activeProjects,
	archivedProjects,
	recentProjects,
	projectStats
} from './stores/projectStore';

// Services
export { projectService } from './services/projectService';
