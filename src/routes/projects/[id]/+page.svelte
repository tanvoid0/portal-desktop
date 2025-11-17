<!--
	Project Details Page
	Displays detailed information about a specific project
-->

<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { Button } from '@/lib/components/ui/button';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/lib/components/ui/card';
	import { Badge } from '@/lib/components/ui/badge';
	import { Separator } from '@/lib/components/ui/separator';
	import { Input } from '@/lib/components/ui/input';
	import { Label } from '@/lib/components/ui/label';
	import { 
		Edit, 
		FolderOpen, 
		Star, 
		Calendar, 
		Code, 
		Package, 
		Terminal,
		ExternalLink,
		ArrowLeft,
		RefreshCw,
		Check,
		X
	} from '@lucide/svelte';
	import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/lib/components/ui/tabs';
	import { ProjectTerminal } from '@/lib/domains/terminal';
	import { projectService } from '@/lib/domains/projects/services/projectService';
	import { breadcrumbActions } from '@/lib/domains/shared/stores/breadcrumbStore';
	import { logger } from '@/lib/domains/shared/services/logger';
	import type { Project } from '@/lib/domains/projects/types';
	import PipelineBuilder from '@/lib/domains/projects/pipelines/components/PipelineBuilder.svelte';
	import ExecutionMonitor from '@/lib/domains/projects/pipelines/components/ExecutionMonitor.svelte';
	import type { Pipeline, PipelineExecution } from '@/lib/domains/projects/pipelines';
	import { pipelineService, executionService } from '@/lib/domains/projects/pipelines';

	const log = logger.createScoped('ProjectDetailsPage');

	// Get project ID from URL
	const projectId = $page.params.id;
	
	// State
	let project = $state<Project | null>(null);
	let loading = $state(true);
	let error = $state('');
	let activeTab = $state('overview');
	let refreshing = $state(false);
	
	// Inline editing state
	let editingField = $state<string | null>(null);
	let editValues = $state<Record<string, string>>({});
	let savingField = $state<string | null>(null);
	
	// Pipeline state
	let pipelines = $state<Pipeline[]>([]);
	let selectedPipeline: Pipeline | null = $state(null);
	let showBuilder = $state(false);
	let currentExecution: PipelineExecution | null = $state(null);
	let pipelinesLoading = $state(false);

	// Load project details
	async function loadProject() {
		if (!projectId) {
			error = 'Project ID is required';
			loading = false;
			return;
		}

		try {
			loading = true;
			error = '';
			
			log.info('Loading project details', { projectId });
			project = await projectService.getProject(projectId);
			
			if (!project) {
				error = 'Project not found';
				return;
			}

			// Set breadcrumbs
			breadcrumbActions.setProjectDetailsBreadcrumbs(project.name);
			
			log.info('Project details loaded successfully', { projectId, name: project.name });
		} catch (err) {
			log.error('Failed to load project details', err);
			error = 'Failed to load project details. Please try again.';
		} finally {
			loading = false;
		}
	}

	// Format date for display
	function formatDate(date: Date | string | null | undefined): string {
		if (!date) return 'Never';
		const d = new Date(date);
		return d.toLocaleDateString() + ' ' + d.toLocaleTimeString();
	}

	// Format file size
	function formatFileSize(bytes: number): string {
		if (bytes === 0) return '0 B';
		const k = 1024;
		const sizes = ['B', 'KB', 'MB', 'GB'];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
	}

	// Navigate to edit page
	function handleEdit() {
		if (project) {
			goto(`/projects/edit/${project.id}`);
		}
	}

	// Navigate back to projects list
	function handleBack() {
		goto('/projects');
	}

	// Open project in file explorer
	async function handleOpenInExplorer() {
		if (!project) return;
		
		try {
			await projectService.openProjectInExplorer(project.path);
		} catch (err) {
			log.error('Failed to open project in explorer', err);
		}
	}

	// Switch to terminal tab
	function switchToTerminal() {
		activeTab = 'terminal';
		log.info('Switched to terminal tab for project', { projectId });
	}

	// Refresh project metadata
	async function handleRefreshMetadata() {
		if (!projectId || !project) return;

		try {
			refreshing = true;
			log.info('Refreshing project metadata', { projectId });

			// Refresh metadata in the backend
			await projectService.refreshProjectMetadata(projectId);

			// Reload the project to get updated data
			await loadProject();

			log.info('Project metadata refreshed successfully', { projectId });
		} catch (err) {
			log.error('Failed to refresh project metadata', err);
			error = 'Failed to refresh project metadata. Please try again.';
		} finally {
			refreshing = false;
		}
	}

	// Toggle star status
	async function handleToggleStar() {
		if (!projectId || !project) return;

		try {
			const newStarredStatus = !project.starred;
			log.info('Toggling star status', { projectId, starred: newStarredStatus });

			// Update project with new starred status
			// Note: This uses updateProject, but starred is not in UpdateProjectRequest type
			// We'll need to update the project locally and potentially extend the update request
			project = { ...project, starred: newStarredStatus };

			// Optionally update in backend if there's a star endpoint
			// For now, we'll update locally. If backend persistence is needed,
			// we may need to add a specific toggleStar method to the service
			
			log.info('Star status toggled successfully', { projectId, starred: newStarredStatus });
		} catch (err) {
			log.error('Failed to toggle star status', err);
			error = 'Failed to toggle star status. Please try again.';
		}
	}

	// Start inline editing
	function startEditing(field: string, currentValue: string | undefined) {
		editingField = field;
		editValues[field] = currentValue || '';
	}

	// Cancel editing
	function cancelEditing() {
		editingField = null;
		editValues = {};
	}

	// Save inline edit
	async function saveField(field: string) {
		if (!projectId || !project) return;

		try {
			savingField = field;
			const newValue = editValues[field]?.trim() || undefined;

			log.info('Saving field', { projectId, field, newValue });

			// Update project via service
			const updates: Record<string, any> = { [field]: newValue };
			await projectService.updateProject(projectId, updates);

			// Update local project state
			project = { ...project, [field]: newValue };

			// Clear editing state
			editingField = null;
			editValues = {};

			log.info('Field saved successfully', { projectId, field });
		} catch (err) {
			log.error('Failed to save field', err);
			error = `Failed to save ${field}. Please try again.`;
		} finally {
			savingField = null;
		}
	}

	async function loadPipelines() {
		if (!projectId) return;
		pipelinesLoading = true;
		try {
			pipelines = await pipelineService.getPipelines(projectId);
		} catch (error) {
			log.error('Failed to load pipelines', error);
		} finally {
			pipelinesLoading = false;
		}
	}
	
	function handleCreatePipeline() {
		selectedPipeline = null;
		showBuilder = true;
	}
	
	function handleEditPipeline(pipeline: Pipeline) {
		selectedPipeline = pipeline;
		showBuilder = true;
	}
	
	async function handleDeletePipeline(pipelineId: string) {
		if (confirm('Are you sure you want to delete this pipeline?')) {
			try {
				await pipelineService.deletePipeline(pipelineId);
				await loadPipelines();
			} catch (error) {
				log.error('Failed to delete pipeline', error);
			}
		}
	}
	
	async function handleExecutePipeline(pipelineId: string) {
		try {
			const execution = await executionService.executePipeline({ 
				pipelineId, 
				projectId, 
				triggeredBy: 'user' 
			});
			currentExecution = execution;
		} catch (error) {
			log.error('Failed to execute pipeline', error);
		}
	}
	
	function handleBuilderClose() {
		showBuilder = false;
		selectedPipeline = null;
		loadPipelines();
	}

	onMount(() => {
		loadProject();
		loadPipelines();
	});
</script>

<svelte:head>
	<title>{project ? `${project.name} - Project Details` : 'Project Details'} - Portal Desktop</title>
</svelte:head>

<div class="container mx-auto p-6">
	<!-- Header -->
	<div class="flex items-center justify-between mb-6">
		<div class="flex items-center gap-4">
			<Button variant="ghost" size="sm" onclick={handleBack} class="flex items-center gap-2">
				<ArrowLeft class="h-4 w-4" />
				Back to Projects
			</Button>
		</div>
		
		{#if project}
			<div class="flex items-center gap-2">
				<Button 
					variant={project.starred ? "default" : "outline"}
					onclick={handleToggleStar}
					class="flex items-center gap-2"
					title={project.starred ? "Unstar project" : "Star project"}
				>
					<Star class="h-4 w-4 {project.starred ? 'fill-current' : ''}" />
					{project.starred ? 'Starred' : 'Star'}
				</Button>
				<Button 
					variant="outline" 
					onclick={handleRefreshMetadata} 
					disabled={refreshing}
					class="flex items-center gap-2"
					title="Refresh project metadata (size, file count, git info)"
				>
					<RefreshCw class="h-4 w-4 {refreshing ? 'animate-spin' : ''}" />
					Refresh
				</Button>
				<Button variant="outline" onclick={handleOpenInExplorer} class="flex items-center gap-2">
					<ExternalLink class="h-4 w-4" />
					Open in Explorer
				</Button>
				<Button variant="outline" onclick={switchToTerminal} class="flex items-center gap-2">
					<Terminal class="h-4 w-4" />
					Open Terminal
				</Button>
				<Button onclick={handleEdit} class="flex items-center gap-2">
					<Edit class="h-4 w-4" />
					Edit Project
				</Button>
			</div>
		{/if}
	</div>

	{#if loading}
		<div class="flex items-center justify-center py-12">
			<div class="text-center">
				<div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary mx-auto mb-4"></div>
				<p class="text-muted-foreground">Loading project details...</p>
			</div>
		</div>
	{:else if error}
		<Card>
			<CardContent class="pt-6">
				<div class="text-center">
					<p class="text-red-600 dark:text-red-400 mb-4">{error}</p>
					<Button onclick={loadProject}>Try Again</Button>
				</div>
			</CardContent>
		</Card>
	{:else if project}
		<div class="space-y-6">
			<!-- Project Header -->
			<Card>
				<CardHeader>
					<div class="flex items-start justify-between">
						<div>
							<CardTitle class="text-2xl flex items-center gap-2">
								{project.name}
								{#if project.starred}
									<Star class="h-5 w-5 text-yellow-500 fill-current" />
								{/if}
							</CardTitle>
							{#if project.description}
								<CardDescription class="mt-2">{project.description}</CardDescription>
							{/if}
						</div>
						<Badge variant={project.status === 'active' ? 'default' : 'secondary'}>
							{project.status}
						</Badge>
					</div>
				</CardHeader>
			</Card>

			<!-- Project Tabs -->
			<Tabs bind:value={activeTab} class="w-full">
				<TabsList class="grid w-full" style="grid-template-columns: repeat({project.metadata?.dependencies ? 4 : 3}, minmax(0, 1fr));">
					<TabsTrigger value="overview" class="flex items-center gap-2">
						<FolderOpen class="h-4 w-4" />
						Overview
					</TabsTrigger>
					{#if project.metadata?.dependencies}
						<TabsTrigger value="dependencies" class="flex items-center gap-2">
							<Package class="h-4 w-4" />
							Dependencies
						</TabsTrigger>
					{/if}
					<TabsTrigger value="pipelines" class="flex items-center gap-2">
						<Code class="h-4 w-4" />
						Pipelines
					</TabsTrigger>
					<TabsTrigger value="terminal" class="flex items-center gap-2">
						<Terminal class="h-4 w-4" />
						Terminal
					</TabsTrigger>
				</TabsList>
				
				<TabsContent value="overview" class="mt-6">
					<!-- Project Information -->
					<div class="grid grid-cols-1 md:grid-cols-2 gap-6">
				<!-- Basic Information -->
				<Card>
					<CardHeader>
						<CardTitle class="flex items-center gap-2">
							<FolderOpen class="h-5 w-5" />
							Basic Information
						</CardTitle>
					</CardHeader>
					<CardContent class="space-y-4">
						<div>
							<p class="text-sm font-medium text-muted-foreground">Project Path</p>
							<p class="text-sm font-mono bg-muted p-2 rounded mt-1 break-all">{project.path}</p>
						</div>
						
						{#if project.framework}
							<div>
								<p class="text-sm font-medium text-muted-foreground">Framework</p>
								<p class="text-sm mt-1">{project.framework}</p>
							</div>
						{/if}
						
						{#if project.package_manager}
							<div>
								<p class="text-sm font-medium text-muted-foreground">Package Manager</p>
								<p class="text-sm mt-1">{project.package_manager}</p>
							</div>
						{/if}
					</CardContent>
				</Card>

				<!-- Statistics -->
				<Card>
					<CardHeader>
						<CardTitle class="flex items-center gap-2">
							<Calendar class="h-5 w-5" />
							Statistics
						</CardTitle>
					</CardHeader>
					<CardContent class="space-y-4">
						<div>
							<p class="text-sm font-medium text-muted-foreground">Open Count</p>
							<p class="text-sm mt-1">{project.open_count} times</p>
						</div>
						
						<div>
							<p class="text-sm font-medium text-muted-foreground">Project Size</p>
							<p class="text-sm mt-1">{formatFileSize(project.size)}</p>
						</div>
						
						<div>
							<p class="text-sm font-medium text-muted-foreground">File Count</p>
							<p class="text-sm mt-1">{project.file_count} files</p>
						</div>
						
						{#if project.last_opened}
							<div>
								<p class="text-sm font-medium text-muted-foreground">Last Opened</p>
								<p class="text-sm mt-1">{formatDate(project.last_opened)}</p>
							</div>
						{/if}
					</CardContent>
				</Card>
			</div>

			<!-- Commands & Configuration -->
			<Card>
					<CardHeader>
						<CardTitle class="flex items-center gap-2">
							<Terminal class="h-5 w-5" />
							Commands & Configuration
						</CardTitle>
					</CardHeader>
					<CardContent class="space-y-4">
						{#if project.build_command || editingField === 'build_command'}
							<div>
								<div class="flex items-center justify-between mb-1">
									<Label class="text-sm font-medium text-muted-foreground">Build Command</Label>
									{#if editingField !== 'build_command'}
										<Button
											variant="ghost"
											size="sm"
											onclick={() => project && startEditing('build_command', project.build_command)}
											class="h-6 px-2"
										>
											<Edit class="h-3 w-3" />
										</Button>
									{/if}
								</div>
								{#if editingField === 'build_command'}
									<div class="flex items-center gap-2">
										<Input
											bind:value={editValues.build_command}
											class="font-mono text-sm"
											placeholder="npm run build"
										/>
										<Button
											variant="ghost"
											size="sm"
											onclick={() => saveField('build_command')}
											disabled={savingField === 'build_command'}
											class="h-8"
										>
											<Check class="h-4 w-4 text-green-600" />
										</Button>
										<Button
											variant="ghost"
											size="sm"
											onclick={cancelEditing}
											disabled={savingField === 'build_command'}
											class="h-8"
										>
											<X class="h-4 w-4 text-red-600" />
										</Button>
									</div>
								{:else}
									<p class="text-sm font-mono bg-muted p-2 rounded">{project.build_command}</p>
								{/if}
							</div>
						{/if}
						
						{#if project.start_command || editingField === 'start_command'}
							<div>
								<div class="flex items-center justify-between mb-1">
									<Label class="text-sm font-medium text-muted-foreground">Start Command</Label>
									{#if editingField !== 'start_command'}
										<Button
											variant="ghost"
											size="sm"
											onclick={() => project && startEditing('start_command', project.start_command)}
											class="h-6 px-2"
										>
											<Edit class="h-3 w-3" />
										</Button>
									{/if}
								</div>
								{#if editingField === 'start_command'}
									<div class="flex items-center gap-2">
										<Input
											bind:value={editValues.start_command}
											class="font-mono text-sm"
											placeholder="npm start"
										/>
										<Button
											variant="ghost"
											size="sm"
											onclick={() => saveField('start_command')}
											disabled={savingField === 'start_command'}
											class="h-8"
										>
											<Check class="h-4 w-4 text-green-600" />
										</Button>
										<Button
											variant="ghost"
											size="sm"
											onclick={cancelEditing}
											disabled={savingField === 'start_command'}
											class="h-8"
										>
											<X class="h-4 w-4 text-red-600" />
										</Button>
									</div>
								{:else}
									<p class="text-sm font-mono bg-muted p-2 rounded">{project.start_command}</p>
								{/if}
							</div>
						{/if}
						
						{#if project.test_command || editingField === 'test_command'}
							<div>
								<div class="flex items-center justify-between mb-1">
									<Label class="text-sm font-medium text-muted-foreground">Test Command</Label>
									{#if editingField !== 'test_command'}
										<Button
											variant="ghost"
											size="sm"
											onclick={() => project && startEditing('test_command', project.test_command)}
											class="h-6 px-2"
										>
											<Edit class="h-3 w-3" />
										</Button>
									{/if}
								</div>
								{#if editingField === 'test_command'}
									<div class="flex items-center gap-2">
										<Input
											bind:value={editValues.test_command}
											class="font-mono text-sm"
											placeholder="npm test"
										/>
										<Button
											variant="ghost"
											size="sm"
											onclick={() => saveField('test_command')}
											disabled={savingField === 'test_command'}
											class="h-8"
										>
											<Check class="h-4 w-4 text-green-600" />
										</Button>
										<Button
											variant="ghost"
											size="sm"
											onclick={cancelEditing}
											disabled={savingField === 'test_command'}
											class="h-8"
										>
											<X class="h-4 w-4 text-red-600" />
										</Button>
									</div>
								{:else}
									<p class="text-sm font-mono bg-muted p-2 rounded">{project.test_command}</p>
								{/if}
							</div>
						{/if}
						
						{#if project.output_directory || editingField === 'output_directory'}
							<div>
								<div class="flex items-center justify-between mb-1">
									<Label class="text-sm font-medium text-muted-foreground">Output Directory</Label>
									{#if editingField !== 'output_directory'}
										<Button
											variant="ghost"
											size="sm"
											onclick={() => project && startEditing('output_directory', project.output_directory)}
											class="h-6 px-2"
										>
											<Edit class="h-3 w-3" />
										</Button>
									{/if}
								</div>
								{#if editingField === 'output_directory'}
									<div class="flex items-center gap-2">
										<Input
											bind:value={editValues.output_directory}
											class="font-mono text-sm"
											placeholder="dist"
										/>
										<Button
											variant="ghost"
											size="sm"
											onclick={() => saveField('output_directory')}
											disabled={savingField === 'output_directory'}
											class="h-8"
										>
											<Check class="h-4 w-4 text-green-600" />
										</Button>
										<Button
											variant="ghost"
											size="sm"
											onclick={cancelEditing}
											disabled={savingField === 'output_directory'}
											class="h-8"
										>
											<X class="h-4 w-4 text-red-600" />
										</Button>
									</div>
								{:else}
									<p class="text-sm font-mono bg-muted p-2 rounded">{project.output_directory}</p>
								{/if}
							</div>
						{/if}
						
						<!-- Add New Command Section -->
						{#if !project.build_command && !project.start_command && !project.test_command && !project.output_directory && editingField === null}
							<div class="pt-2 border-t">
								<Button
									variant="outline"
									size="sm"
									onclick={() => startEditing('build_command', '')}
									class="w-full"
								>
									+ Add Build Command
								</Button>
							</div>
						{:else if editingField === null}
							<div class="pt-2 border-t">
								<div class="flex gap-2 flex-wrap">
									{#if !project.build_command && editingField !== 'build_command'}
										<Button
											variant="outline"
											size="sm"
											onclick={() => startEditing('build_command', '')}
										>
											+ Build
										</Button>
									{/if}
									{#if !project.start_command && editingField !== 'start_command'}
										<Button
											variant="outline"
											size="sm"
											onclick={() => startEditing('start_command', '')}
										>
											+ Start
										</Button>
									{/if}
									{#if !project.test_command && editingField !== 'test_command'}
										<Button
											variant="outline"
											size="sm"
											onclick={() => startEditing('test_command', '')}
										>
											+ Test
										</Button>
									{/if}
									{#if !project.output_directory && editingField !== 'output_directory'}
										<Button
											variant="outline"
											size="sm"
											onclick={() => startEditing('output_directory', '')}
										>
											+ Output Dir
										</Button>
									{/if}
								</div>
							</div>
						{/if}
						
						{#if project.dev_port || project.prod_port}
							<div>
								<p class="text-sm font-medium text-muted-foreground">Ports</p>
								<div class="flex gap-2 mt-1">
									{#if project.dev_port}
										<Badge variant="outline">Dev: {project.dev_port}</Badge>
									{/if}
									{#if project.prod_port}
										<Badge variant="outline">Prod: {project.prod_port}</Badge>
									{/if}
								</div>
							</div>
						{/if}
					</CardContent>
				</Card>

			<!-- Git Information -->
			{#if project.git_repository || project.git_branch}
				<Card>
					<CardHeader>
						<CardTitle class="flex items-center gap-2">
							<Code class="h-5 w-5" />
							Git Information
						</CardTitle>
					</CardHeader>
					<CardContent class="space-y-4">
						{#if project.git_repository}
							<div>
								<p class="text-sm font-medium text-muted-foreground">Repository</p>
								<p class="text-sm font-mono bg-muted p-2 rounded mt-1 break-all">{project.git_repository}</p>
							</div>
						{/if}
						
						{#if project.git_branch}
							<div>
								<p class="text-sm font-medium text-muted-foreground">Branch</p>
								<p class="text-sm mt-1">{project.git_branch}</p>
							</div>
						{/if}
						
						{#if project.git_commit}
							<div>
								<p class="text-sm font-medium text-muted-foreground">Last Commit</p>
								<p class="text-sm font-mono bg-muted p-2 rounded mt-1">{project.git_commit}</p>
							</div>
						{/if}
						
						{#if project.has_uncommitted_changes !== undefined}
							<div>
								<p class="text-sm font-medium text-muted-foreground">Uncommitted Changes</p>
								<Badge variant={project.has_uncommitted_changes ? 'destructive' : 'default'}>
									{project.has_uncommitted_changes ? 'Yes' : 'No'}
								</Badge>
							</div>
						{/if}
					</CardContent>
				</Card>
			{/if}
				</TabsContent>
				
				{#if project.metadata?.dependencies}
					<TabsContent value="dependencies" class="mt-6">
						<div class="space-y-6">
							<!-- Package Manager Info -->
							<Card>
								<CardHeader>
									<CardTitle class="flex items-center gap-2">
										<Package class="h-5 w-5" />
										Package Manager
									</CardTitle>
								</CardHeader>
								<CardContent>
									<Badge variant="outline" class="text-base px-3 py-1">
										{project.metadata.dependencies.packageManager}
									</Badge>
								</CardContent>
							</Card>

							<!-- Dependencies -->
							{#if Object.keys(project.metadata.dependencies.dependencies || {}).length > 0}
								<Card>
									<CardHeader>
										<CardTitle>Dependencies</CardTitle>
										<CardDescription>
											{Object.keys(project.metadata.dependencies.dependencies).length} production dependencies
										</CardDescription>
									</CardHeader>
									<CardContent>
										<div class="space-y-2 max-h-[400px] overflow-y-auto">
											{#each Object.entries(project.metadata.dependencies.dependencies) as [name, version]}
												<div class="flex items-center justify-between p-2 rounded border {project.metadata.dependencies.outdated?.includes(name) ? 'bg-yellow-50 dark:bg-yellow-900/20 border-yellow-200 dark:border-yellow-800' : ''}">
													<div class="flex items-center gap-2">
														<Package class="h-4 w-4 text-muted-foreground" />
														<span class="font-mono text-sm font-medium">{name}</span>
													</div>
													<div class="flex items-center gap-2">
														<span class="text-sm text-muted-foreground">{version}</span>
														{#if project.metadata.dependencies.outdated?.includes(name)}
															<Badge variant="outline" class="bg-yellow-100 dark:bg-yellow-900 text-yellow-800 dark:text-yellow-200 border-yellow-300 dark:border-yellow-700">
																Outdated
															</Badge>
														{/if}
														{#if project.metadata.dependencies.vulnerabilities?.includes(name)}
															<Badge variant="destructive">
																Vulnerable
															</Badge>
														{/if}
													</div>
												</div>
											{/each}
										</div>
									</CardContent>
								</Card>
							{/if}

							<!-- Dev Dependencies -->
							{#if Object.keys(project.metadata.dependencies.devDependencies || {}).length > 0}
								<Card>
									<CardHeader>
										<CardTitle>Dev Dependencies</CardTitle>
										<CardDescription>
											{Object.keys(project.metadata.dependencies.devDependencies).length} development dependencies
										</CardDescription>
									</CardHeader>
									<CardContent>
										<div class="space-y-2 max-h-[400px] overflow-y-auto">
											{#each Object.entries(project.metadata.dependencies.devDependencies) as [name, version]}
												<div class="flex items-center justify-between p-2 rounded border {project.metadata.dependencies.outdated?.includes(name) ? 'bg-yellow-50 dark:bg-yellow-900/20 border-yellow-200 dark:border-yellow-800' : ''}">
													<div class="flex items-center gap-2">
														<Package class="h-4 w-4 text-muted-foreground" />
														<span class="font-mono text-sm font-medium">{name}</span>
													</div>
													<div class="flex items-center gap-2">
														<span class="text-sm text-muted-foreground">{version}</span>
														{#if project.metadata.dependencies.outdated?.includes(name)}
															<Badge variant="outline" class="bg-yellow-100 dark:bg-yellow-900 text-yellow-800 dark:text-yellow-200 border-yellow-300 dark:border-yellow-700">
																Outdated
															</Badge>
														{/if}
														{#if project.metadata.dependencies.vulnerabilities?.includes(name)}
															<Badge variant="destructive">
																Vulnerable
															</Badge>
														{/if}
													</div>
												</div>
											{/each}
										</div>
									</CardContent>
								</Card>
							{/if}

							<!-- Summary -->
							{#if (project.metadata.dependencies.outdated?.length || 0) > 0 || (project.metadata.dependencies.vulnerabilities?.length || 0) > 0}
								<Card class="border-orange-200 dark:border-orange-800">
									<CardHeader>
										<CardTitle>Summary</CardTitle>
									</CardHeader>
									<CardContent class="space-y-2">
										{#if (project.metadata.dependencies.outdated?.length || 0) > 0}
											<div class="flex items-center gap-2">
												<Badge variant="outline" class="bg-yellow-100 dark:bg-yellow-900 text-yellow-800 dark:text-yellow-200">
													{project.metadata.dependencies.outdated.length} outdated packages
												</Badge>
											</div>
										{/if}
										{#if (project.metadata.dependencies.vulnerabilities?.length || 0) > 0}
											<div class="flex items-center gap-2">
												<Badge variant="destructive">
													{project.metadata.dependencies.vulnerabilities.length} vulnerable packages
												</Badge>
											</div>
										{/if}
									</CardContent>
								</Card>
							{/if}
						</div>
					</TabsContent>
				{/if}
				
				<TabsContent value="pipelines" class="mt-6">
					<div class="space-y-6">
						<div class="flex items-center justify-between">
							<h2 class="text-xl font-semibold">Pipelines</h2>
							<div class="flex gap-2">
								<Button variant="outline" onclick={() => goto(`/projects/${projectId}/pipelines/new`)}>
									Create from Template
								</Button>
								<Button onclick={handleCreatePipeline}>Create Pipeline</Button>
							</div>
						</div>
						
						{#if showBuilder}
							<PipelineBuilder
								pipeline={selectedPipeline || undefined}
								projectId={projectId}
								onSave={handleBuilderClose}
								onCancel={handleBuilderClose}
							/>
						{:else if currentExecution}
							<ExecutionMonitor executionId={currentExecution.id} onClose={() => (currentExecution = null)} />
						{:else}
							{#if pipelinesLoading}
								<p class="text-center text-muted-foreground py-8">Loading pipelines...</p>
							{:else if pipelines.length === 0}
								<Card>
									<CardContent class="py-12 text-center">
										<p class="text-muted-foreground mb-4">No pipelines yet</p>
										<Button onclick={handleCreatePipeline}>Create Your First Pipeline</Button>
									</CardContent>
								</Card>
							{:else}
								<div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
									{#each pipelines as pipeline (pipeline.id)}
										<Card>
											<CardHeader>
												<CardTitle>{pipeline.name}</CardTitle>
												{#if pipeline.description}
													<CardDescription>{pipeline.description}</CardDescription>
												{/if}
											</CardHeader>
											<CardContent class="space-y-2">
												<p class="text-sm text-muted-foreground">
													{pipeline.steps.length} step{pipeline.steps.length !== 1 ? 's' : ''}
												</p>
												<div class="flex gap-2">
													<Button
														size="sm"
														onclick={() => handleExecutePipeline(pipeline.id)}
														disabled={!pipeline.enabled}
													>
														Run
													</Button>
													<Button
														size="sm"
														variant="outline"
														onclick={() => handleEditPipeline(pipeline)}
													>
														Edit
													</Button>
													<Button
														size="sm"
														variant="destructive"
														onclick={() => handleDeletePipeline(pipeline.id)}
													>
														Delete
													</Button>
												</div>
											</CardContent>
										</Card>
									{/each}
								</div>
							{/if}
						{/if}
					</div>
				</TabsContent>
				
				<TabsContent value="terminal" class="mt-6">
					{#if project}
						<div class="h-[600px] border border-gray-700 rounded-lg overflow-hidden">
							<ProjectTerminal 
								projectId={project.id}
								projectName={project.name}
								projectPath={project.path}
								settings={{
									theme: 'dark',
									fontSize: 14,
									fontFamily: 'Monaco, Consolas, "Courier New", monospace',
									cursorStyle: 'block',
									scrollbackLines: 1000,
									bellSound: false,
									autoClose: true,
									confirmClose: true,
									defaultShell: navigator.userAgent.includes('Windows') ? 'cmd.exe' : 'bash',
									workingDirectory: project.path
								}}
							/>
						</div>
					{:else}
						<Card>
							<CardContent class="flex flex-col items-center justify-center py-12">
								<Terminal class="h-12 w-12 text-muted-foreground mb-4" />
								<h3 class="text-lg font-semibold mb-2">Project Terminal</h3>
								<p class="text-muted-foreground text-center mb-4">
									Loading project details...
								</p>
							</CardContent>
						</Card>
					{/if}
				</TabsContent>
			</Tabs>
		</div>
	{/if}
</div>
