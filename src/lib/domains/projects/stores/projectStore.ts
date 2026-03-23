/**
 * Projects store using Svelte 5 runes
 */

import { writable, derived } from 'svelte/store';
import type { Project, CreateProjectRequest, UpdateProjectRequest } from '@/lib/domains/projects/types';
import { ProjectStatus } from '@/lib/domains/projects/types';
import { generateId } from '@/lib/domains/shared/utils';

interface ProjectState {
	projects: Project[];
	activeProjectId: string | null;
	loading: boolean;
	error: string | null;
}

const initialState: ProjectState = {
	projects: [],
	activeProjectId: null,
	loading: false,
	error: null
};

function createProjectStore() {
	const { subscribe, set, update } = writable<ProjectState>(initialState);

	return {
		subscribe,
		
		// Actions
		setLoading: (loading: boolean) => {
			update(state => ({ ...state, loading }));
		},

		setError: (error: string | null) => {
			update(state => ({ ...state, error }));
		},

		setProjects: (projects: Project[] | ((prev: Project[]) => Project[])) => {
			update(state => ({ 
				...state, 
				projects: typeof projects === 'function' ? projects(state.projects) : projects 
			}));
		},

		addProject: (projectData: CreateProjectRequest) => {
			const project: Project = {
				id: generateId(),
				name: projectData.name,
				description: projectData.description,
				path: projectData.path,
				status: ProjectStatus.ACTIVE,
				framework_ids: projectData.framework_ids,
				package_manager_ids: projectData.package_manager_ids,
				language_ids: projectData.language_ids,
				build_command: projectData.build_command,
				start_command: projectData.start_command,
				test_command: projectData.test_command,
				output_directory: projectData.output_directory,
				dev_port: projectData.dev_port,
				prod_port: projectData.prod_port,
				starred: false,
				open_count: 0,
				last_opened: undefined,
				size: 0,
				file_count: 0,
				git_repository: undefined,
				git_branch: undefined,
				git_commit: undefined,
				has_uncommitted_changes: false,
				last_commit: undefined,
				created_at: new Date(),
				updated_at: new Date(),
				metadata: {
					openCount: 0,
					size: 0,
					fileCount: 0
				},
				createdAt: new Date(),
				updatedAt: new Date()
			};

			update(state => ({
				...state,
				projects: [...state.projects, project]
			}));

			return project;
		},

		updateProject: (id: string, updates: UpdateProjectRequest) => {
			update(state => ({
				...state,
				projects: state.projects.map(project =>
					project.id === id
						? { ...project, ...updates, updatedAt: new Date() }
						: project
				)
			}));
		},

		deleteProject: (id: string) => {
			update(state => ({
				...state,
				projects: state.projects.filter(project => project.id !== id),
				activeProjectId: state.activeProjectId === id ? null : state.activeProjectId
			}));
		},

		setActiveProject: (id: string | null) => {
			update(state => ({ ...state, activeProjectId: id }));
		},

		incrementOpenCount: (id: string) => {
			update(state => ({
				...state,
				projects: state.projects.map(project =>
					project.id === id
						? {
								...project,
								metadata: {
									...project.metadata,
									lastOpened: new Date(),
									openCount: project.metadata.openCount + 1
								}
							}
						: project
				)
			}));
		},

		updateProjectMetadata: (id: string, metadata: Partial<Project['metadata']>) => {
			update(state => ({
				...state,
				projects: state.projects.map(project =>
					project.id === id
						? {
								...project,
								metadata: { ...project.metadata, ...metadata }
							}
						: project
				)
			}));
		},

		// Getters
		getProject: (id: string) => {
			let project: Project | undefined;
			subscribe(state => {
				project = state.projects.find(p => p.id === id);
			})();
			return project;
		},

		reset: () => set(initialState)
	};
}

export const projectStore = createProjectStore();

// Derived stores
export const activeProject = derived(
	projectStore,
	$store => $store.projects.find(p => p.id === $store.activeProjectId) || null
);

export const activeProjects = derived(
	projectStore,
	$store => $store.projects.filter(p => p.status === 'active')
);

export const archivedProjects = derived(
	projectStore,
	$store => $store.projects.filter(p => p.status === 'archived')
);


export const recentProjects = derived(
	projectStore,
	$store => [...$store.projects]
		.filter(p => p.status === 'active')
		.sort((a, b) => {
			const aTime = a.last_opened?.getTime() || 0;
			const bTime = b.last_opened?.getTime() || 0;
			return bTime - aTime;
		})
		.slice(0, 5)
);

export const projectStats = derived(
	projectStore,
	$store => {
		const active = $store.projects.filter(p => p.status === 'active');
		const archived = $store.projects.filter(p => p.status === 'archived');
		
		// Framework counting is now based on framework_ids array
		// For now, we'll use a placeholder since we'd need to resolve IDs to names
		const mostUsedFramework = 'Unknown';

		return {
			totalProjects: $store.projects.length,
			activeProjects: active.length,
			archivedProjects: archived.length,
			totalSize: $store.projects.reduce((sum, p) => sum + p.metadata.size, 0),
			mostUsedFramework,
			recentProjects: recentProjects
		};
	}
);
