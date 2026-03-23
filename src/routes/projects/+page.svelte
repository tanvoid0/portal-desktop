<!--
	Projects page - Project management interface
	Showcases the projects domain with full CRUD operations
-->

<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { Button } from '@/lib/components/ui/button';
	import { Input } from '@/lib/components/ui/input';
	import { Badge } from '@/lib/components/ui/badge';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle, CardInfo } from '@/lib/components/ui/card';
	import { FolderOpen, CheckCircle, Database, Code } from '@lucide/svelte';
	import ProjectCard from '@/lib/domains/projects/components/ProjectCard.svelte';
	import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/lib/components/ui/tabs';
	import Select from '@/lib/components/ui/select.svelte';
	import { projectStore, projectService } from '@/lib/domains/projects';
	import { logger } from '@/lib/domains/shared/services/logger';
	import { formatBytes, formatRelativeTime } from '@/lib/domains/shared/utils';
	import { breadcrumbActions } from '@/lib/domains/shared/stores/breadcrumbStore';
	import { WorkflowTrigger, WorkflowResults } from '@/lib/domains/automation';
	import type { WorkflowResult } from '@/lib/domains/automation/types';

	const log = logger.createScoped('ProjectsPage');

	let searchQuery = $state('');
	let isLoading = $state(false);
	let selectedProject = $state<any>(null);
	let showAutomation = $state(false);
	let workflowResult = $state<WorkflowResult | null>(null);
	
	// Filter state
	let filterFramework = $state<string | null>(null);
	let filterStatus = $state<string | null>(null);
	let filterPackageManager = $state<string | null>(null);
	
	// Sort state
	type SortOption = 'name' | 'last_opened' | 'size' | 'created_at';
	let sortBy = $state<SortOption>('name');
	let sortDirection = $state<'asc' | 'desc'>('asc');

	// Initialize projects on mount
	onMount(async () => {
		try {
			isLoading = true;
			await projectService.loadProjects();
			
			// Set breadcrumbs for projects page
			breadcrumbActions.setProjectBreadcrumbs();
		} catch (error) {
			log.error('Failed to load projects', error);
		} finally {
			isLoading = false;
		}
	});

	// Filter projects based on search and type
	const filteredProjects = $derived(() => {
		let filtered = $projectStore.projects;

		// Filter by search query
		if (searchQuery.trim()) {
			const query = searchQuery.toLowerCase();
			filtered = filtered.filter(project => 
				project.name.toLowerCase().includes(query) ||
				project.description?.toLowerCase().includes(query) ||
				project.path.toLowerCase().includes(query)
			);
		}

		// Filter by framework
		if (filterFramework) {
			filtered = filtered.filter(project => project.metadata?.framework === filterFramework);
		}

		// Filter by status
		if (filterStatus) {
			filtered = filtered.filter(project => project.status === filterStatus);
		}

		// Filter by package manager
		if (filterPackageManager) {
			// Package manager filtering would need to resolve package_manager_ids
			// For now, skip this filter
		}

		// Sort projects
		filtered = [...filtered].sort((a, b) => {
			let comparison = 0;
			
			switch (sortBy) {
				case 'name':
					comparison = a.name.localeCompare(b.name);
					break;
				case 'last_opened':
					const aTime = a.last_opened ? new Date(a.last_opened).getTime() : 0;
					const bTime = b.last_opened ? new Date(b.last_opened).getTime() : 0;
					comparison = aTime - bTime;
					break;
				case 'size':
					comparison = a.size - b.size;
					break;
				case 'created_at':
					const aCreated = a.created_at ? new Date(a.created_at).getTime() : 0;
					const bCreated = b.created_at ? new Date(b.created_at).getTime() : 0;
					comparison = aCreated - bCreated;
					break;
			}
			
			return sortDirection === 'asc' ? comparison : -comparison;
		});

		return filtered;
	});

	// Get unique values for filter options
	const uniqueFrameworks = $derived(() => {
		const frameworks = $projectStore.projects
			.map(p => p.metadata?.framework)
			.filter((f): f is string => !!f);
		return [...new Set(frameworks)].sort();
	});

	const uniquePackageManagers = $derived(() => {
		const managers: string[] = []; // Package managers would need to be resolved from package_manager_ids
		return [...new Set(managers)].sort();
	});

	// Get framework icon class
	const getFrameworkIconClass = (framework: string | null | undefined): string => {
		if (!framework) return 'devicon-folder-plain';
		
		const icons: Record<string, string> = {
			'React': 'devicon-react-original',
			'Vue': 'devicon-vuejs-plain',
			'Angular': 'devicon-angularjs-plain',
			'Svelte': 'devicon-svelte-plain',
			'Next.js': 'devicon-nextjs-plain',
			'Nuxt': 'devicon-nuxtjs-plain',
			'Node.js': 'devicon-nodejs-plain',
			'Express': 'devicon-express-original',
			'FastAPI': 'devicon-fastapi-plain',
			'Django': 'devicon-django-plain',
			'Flask': 'devicon-flask-plain',
			'Laravel': 'devicon-laravel-plain',
			'Spring': 'devicon-spring-plain',
			'ASP.NET': 'devicon-dotnetcore-plain',
			'Rails': 'devicon-rails-plain',
			'Flutter': 'devicon-flutter-plain',
			'React Native': 'devicon-react-original',
			'Ionic': 'devicon-ionic-original',
			'Electron': 'devicon-electron-original',
			'Tauri': 'devicon-rust-plain',
			'Python': 'devicon-python-plain',
			'Java': 'devicon-java-plain',
			'Go': 'devicon-go-plain',
			'Rust': 'devicon-rust-plain',
			'PHP': 'devicon-php-plain',
			'Ruby': 'devicon-ruby-plain',
			'Swift': 'devicon-swift-plain',
			'TypeScript': 'devicon-typescript-plain',
			'JavaScript': 'devicon-javascript-plain'
		};
		return icons[framework] || 'devicon-folder-plain';
	};

	// Get framework color
	const getFrameworkColor = (framework: string | null | undefined): string => {
		if (!framework) return 'bg-neutral-100 text-neutral-800 dark:bg-neutral-800 dark:text-neutral-200';
		
		const colors: Record<string, string> = {
			'React': 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200',
			'Vue': 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200',
			'Angular': 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200',
			'Svelte': 'bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-200',
			'Next.js': 'bg-gray-100 text-gray-800 dark:bg-gray-900 dark:text-gray-200',
			'Node.js': 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200',
			'Express': 'bg-gray-100 text-gray-800 dark:bg-gray-900 dark:text-gray-200',
			'FastAPI': 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200',
			'Django': 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200',
			'Flask': 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200',
			'Laravel': 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200',
			'Spring': 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200',
			'ASP.NET': 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200',
			'Rails': 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200',
			'Flutter': 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200',
			'React Native': 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200',
			'Electron': 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200',
			'Tauri': 'bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-200'
		};
		return colors[framework] || 'bg-neutral-100 text-neutral-800 dark:bg-neutral-800 dark:text-neutral-200';
	};

	// Handle project click - navigate to details page
	const handleProjectClick = (project: any) => {
		goto(`/projects/${project.id}`);
	};

	// Handle automation
	function handleAutomationClick(project: any, event: Event) {
		event.stopPropagation();
		selectedProject = project;
		showAutomation = true;
		workflowResult = null;
	}

	function handleWorkflowComplete(result: WorkflowResult) {
		workflowResult = result;
		log.info('Workflow completed', { result });
	}

	function closeAutomation() {
		showAutomation = false;
		selectedProject = null;
		workflowResult = null;
	}

	// Handle project delete
	const handleProjectDelete = async (project: any) => {
		if (confirm(`Are you sure you want to delete "${project.name}"?`)) {
			try {
				await projectService.deleteProject(project.id);
				log.info('Project deleted', { id: project.id, name: project.name });
			} catch (error) {
				log.error('Failed to delete project', error);
			}
		}
	};
</script>

<div class="container mx-auto p-6 space-y-6">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-3xl font-bold">Projects</h1>
			<p class="text-muted-foreground">Manage your development projects</p>
		</div>
		<Button onclick={() => goto('/projects/create')}>
			<svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
				<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
			</svg>
			New Project
		</Button>
	</div>

	<!-- Stats Cards -->
	<div class="grid grid-cols-1 md:grid-cols-4 gap-4">
		<CardInfo
			title="Total Projects"
			value={$projectStore.projects.length}
			icon={FolderOpen}
			gradient={true}
		/>

		<CardInfo
			title="Active Projects"
			value={$projectStore.projects.filter(p => p.status === 'active').length}
			icon={CheckCircle}
			gradient={true}
		/>

		<CardInfo
			title="Total Size"
			value={formatBytes($projectStore.projects.reduce((sum, p) => sum + p.size, 0))}
			icon={Database}
			gradient={true}
		/>

		<CardInfo
			title="Most Used Type"
			value={Object.entries(
				$projectStore.projects.reduce((acc, p) => {
					const framework = p.metadata?.framework || 'Unknown';
					acc[framework] = (acc[framework] || 0) + 1;
					return acc;
				}, {} as Record<string, number>)
			).sort(([,a], [,b]) => (b as number) - (a as number))[0]?.[0] || 'none'}
			icon={Code}
			gradient={true}
		/>
	</div>

	<!-- Filters -->
	<div class="flex items-center gap-4 flex-wrap">
		<Input
			placeholder="Search projects..."
			bind:value={searchQuery}
			class="max-w-sm"
		/>
		
		<Select
			defaultValue={filterFramework || ''}
			placeholder="All Frameworks"
			options={uniqueFrameworks().map(f => ({ value: f, label: f }))}
			onSelect={(value) => filterFramework = value || null}
			class="min-w-[150px]"
		/>
		
		<Select
			defaultValue={filterStatus || ''}
			placeholder="All Statuses"
			options={[
				{ value: 'active', label: 'Active' },
				{ value: 'archived', label: 'Archived' },
				{ value: 'deleted', label: 'Deleted' }
			]}
			onSelect={(value) => filterStatus = value || null}
			class="min-w-[130px]"
		/>
		
		<Select
			defaultValue={filterPackageManager || ''}
			placeholder="All Package Managers"
			options={uniquePackageManagers().map(m => ({ value: m, label: m }))}
			onSelect={(value) => filterPackageManager = value || null}
			class="min-w-[180px]"
		/>
		
		<Select
			defaultValue={sortBy}
			placeholder="Sort by"
			options={[
				{ value: 'name', label: 'Name' },
				{ value: 'last_opened', label: 'Last Opened' },
				{ value: 'size', label: 'Size' },
				{ value: 'created_at', label: 'Created Date' }
			]}
			onSelect={(value) => sortBy = value as SortOption}
			class="min-w-[140px]"
		/>
		
		<Button
			variant="outline"
			size="sm"
			onclick={() => sortDirection = sortDirection === 'asc' ? 'desc' : 'asc'}
			class="flex items-center gap-1"
			title={`Sort ${sortDirection === 'asc' ? 'Ascending' : 'Descending'}`}
		>
			<svg 
				class="w-4 h-4 {sortDirection === 'desc' ? 'rotate-180' : ''}" 
				fill="none" 
				stroke="currentColor" 
				viewBox="0 0 24 24"
			>
				<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 15l7-7 7 7" />
			</svg>
			{sortDirection === 'asc' ? 'Asc' : 'Desc'}
		</Button>
		
		{#if filterFramework || filterStatus || filterPackageManager}
			<Button
				variant="ghost"
				size="sm"
				onclick={() => {
					filterFramework = null;
					filterStatus = null;
					filterPackageManager = null;
				}}
				class="text-muted-foreground"
			>
				Clear Filters
			</Button>
		{/if}
	</div>

	<!-- Projects Grid -->
	{#if isLoading}
		<div class="flex items-center justify-center py-12">
			<div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
		</div>
	{:else if $projectStore.error}
		<Card>
			<CardContent class="flex flex-col items-center justify-center py-12">
				<svg class="w-12 h-12 text-destructive mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
				</svg>
				<h3 class="text-lg font-semibold mb-2 text-destructive">Failed to Load Projects</h3>
				<p class="text-muted-foreground text-center mb-4 max-w-md">
					{$projectStore.error}
				</p>
				<Button onclick={() => projectService.loadProjects()}>
					<svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
					</svg>
					Retry
				</Button>
			</CardContent>
		</Card>
	{:else if filteredProjects().length === 0}
		<Card>
			<CardContent class="flex flex-col items-center justify-center py-12">
				<svg class="w-12 h-12 text-muted-foreground mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
				</svg>
				<h3 class="text-lg font-semibold mb-2">No projects found</h3>
				<p class="text-muted-foreground text-center mb-4">
					{searchQuery 
						? 'Try adjusting your search criteria'
						: 'Create your first project to get started'
					}
				</p>
				<Button onclick={() => goto('/projects/create')}>
					<svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
					</svg>
					Create Project
				</Button>
			</CardContent>
		</Card>
	{:else}
		<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
			{#each filteredProjects() as project (project.id)}
				<ProjectCard
					{project}
					onClick={(p) => handleProjectClick(p)}
					onEdit={(p) => goto(`/projects/${p.id}`)}
					onDelete={handleProjectDelete}
					showActions={true}
				/>
			{/each}
		</div>
	{/if}

	<!-- Automation Modal -->
	{#if showAutomation && selectedProject}
		<div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
			<div class="bg-white dark:bg-gray-900 rounded-lg shadow-xl max-w-2xl w-full mx-4 max-h-[90vh] overflow-y-auto">
				<div class="p-6">
					<div class="flex items-center justify-between mb-4">
						<h2 class="text-xl font-semibold text-gray-900 dark:text-white">
							Automate: {selectedProject.name}
						</h2>
						<Button variant="ghost" size="sm" onclick={closeAutomation}>
							<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
							</svg>
						</Button>
					</div>
					
					{#if workflowResult}
						<WorkflowResults result={workflowResult} />
					{:else}
						<WorkflowTrigger 
							project={selectedProject} 
							onWorkflowComplete={handleWorkflowComplete}
						/>
					{/if}
				</div>
			</div>
		</div>
	{/if}
</div>

