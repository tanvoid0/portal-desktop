<!--
	Project edit page - Uses unified ProjectForm component
	Allows editing existing projects with auto-detection capabilities
-->

<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import ProjectForm from '@/lib/components/projects/ProjectForm.svelte';
	import { projectService } from '@/lib/domains/projects/services/projectService';
	import { logger } from '@/lib/domains/shared/services/logger';
	import type { CreateProjectRequest } from '@/lib/domains/projects/types';
	import { breadcrumbActions } from '@/lib/domains/shared/stores/breadcrumbStore';

	const log = logger.createScoped('ProjectEditPage');

	// Get project ID from URL params
	const projectId = $page.params.id;

	// Form state
	let isLoading = $state(false);
	let project = $state<any>(null);
	let initialData = $state<Partial<CreateProjectRequest>>({});

	// Set up breadcrumbs and load project data on mount
	onMount(async () => {
		if (!projectId) {
			log.error('Project ID is required');
			goto('/projects');
			return;
		}

		try {
			// Load project data first
			project = await projectService.getProject(projectId!);
			
			if (!project) {
				log.error('Project not found', { projectId });
				goto('/projects');
				return;
			}
			
			// Set breadcrumbs for project editing
			breadcrumbActions.setProjectEditBreadcrumbs(project.name);
			
			// Set initial form data
			initialData = {
				name: project.name,
				description: project.description,
				path: project.path,
				framework: project.framework,
				package_manager: project.package_manager
			};
			
		} catch (err) {
			log.error('Failed to load project', err);
			// Redirect to projects page if project not found
			goto('/projects');
		}
	});

	async function handleSubmit(data: CreateProjectRequest) {
		if (!projectId) {
			log.error('Project ID is required for update');
			return;
		}

		try {
			isLoading = true;
			await projectService.updateProject(projectId!, data);
			
			// Redirect to projects page after successful update
			goto('/projects');
		} catch (err) {
			log.error('Failed to update project', err);
			throw err; // Let the form handle the error display
		} finally {
			isLoading = false;
		}
	}

	function handleCancel() {
		goto('/projects');
	}
</script>

<div class="container mx-auto py-8 px-4">
	<div class="mb-8">
		<h1 class="text-3xl font-bold text-foreground mb-2">Edit Project</h1>
		<p class="text-muted-foreground">
			Update your project details. You can change the path to re-analyze the project structure.
		</p>
	</div>

	{#if project && projectId}
		<ProjectForm
			projectId={parseInt(projectId)}
			initialData={initialData}
			onSubmit={handleSubmit}
			onCancel={handleCancel}
			{isLoading}
		/>
	{:else}
		<div class="flex items-center justify-center py-12">
			<div class="text-center">
				<div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary mx-auto mb-4"></div>
				<p class="text-muted-foreground">Loading project...</p>
			</div>
		</div>
	{/if}
</div>
