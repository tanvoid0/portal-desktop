<!--
	Project creation page - Uses unified ProjectForm component
	Provides a better UX with auto-detection of project properties
-->

<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import ProjectForm from '@/lib/components/projects/ProjectForm.svelte';
	import { projectService } from '@/lib/domains/projects/services/projectService';
	import { logger } from '@/lib/domains/shared/services/logger';
	import type { CreateProjectRequest } from '@/lib/domains/projects/types';
	import { breadcrumbActions } from '@/lib/domains/shared/stores/breadcrumbStore';

	const log = logger.createScoped('ProjectCreatePage');

	// Set up breadcrumbs on mount
	onMount(() => {
		breadcrumbActions.setCreateProjectBreadcrumbs();
	});

	// Form state
	let isLoading = $state(false);

	async function handleSubmit(data: CreateProjectRequest) {
		try {
			isLoading = true;
			await projectService.createProject(data);
			
			// Redirect to projects page after successful creation
			goto('/projects');
		} catch (err) {
			log.error('Failed to create project', err);
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
		<h1 class="text-3xl font-bold text-foreground mb-2">Create New Project</h1>
		<p class="text-muted-foreground">
			Create a new project and let us auto-detect its properties from the directory structure.
		</p>
	</div>

	<ProjectForm
		onSubmit={handleSubmit}
		onCancel={handleCancel}
		{isLoading}
	/>
</div>