<!--
	Unified project form component for both creating and updating projects
	Supports auto-detection of project properties when path is selected
-->

<script lang="ts">
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { Button } from '@/lib/components/ui/button';
	import { Input } from '@/lib/components/ui/input';
	import { Label } from '@/lib/components/ui/label';
	import { Textarea } from '@/lib/components/ui/textarea';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/lib/components/ui/card';
	import FolderPicker from '@/lib/components/ui/folder-picker.svelte';
	import { RefreshCw } from '@lucide/svelte';
	import { logger } from '@/lib/domains/shared/services/logger';
	import type { CreateProjectRequest } from '@/lib/domains/projects/types';

	interface ProjectAnalysis {
		name: string;
		project_type: string;
		framework: string | null;
		package_manager: string | null;
		build_command: string | null;
		start_command: string | null;
		test_command: string | null;
		output_directory: string | null;
		dev_port: number | null;
		prod_port: number | null;
	}

	interface Props {
		projectId?: number;
		initialData?: Partial<CreateProjectRequest>;
		onSubmit: (data: CreateProjectRequest) => Promise<void>;
		onCancel: () => void;
		isLoading?: boolean;
	}

	let {
		projectId,
		initialData = {},
		onSubmit,
		onCancel,
		isLoading = false
	}: Props = $props();

	const log = logger.createScoped('ProjectForm');

	// Form state
	let name = $state(initialData.name || '');
	let description = $state(initialData.description || '');
	let path = $state(initialData.path || '');
	let framework = $state(initialData.framework || '');
	let packageManager = $state(initialData.package_manager || '');

	let error = $state('');
	let success = $state('');
	let isAnalyzing = $state(false);

	// Set up breadcrumbs on mount
	onMount(() => {
		if (projectId) {
			// This is an update form
			log.info('Initializing project update form', { projectId });
		} else {
			// This is a create form
			log.info('Initializing project create form');
		}
	});

	async function handlePathChange(newPath: string) {
		path = newPath;
		
		if (newPath.trim()) {
			await analyzeProject(newPath);
		}
	}

	async function analyzeProject(projectPath: string, forceSync = false) {
		try {
			isAnalyzing = true;
			error = '';
			
			log.info('Analyzing project directory', { path: projectPath, forceSync });
			
			const analysis: ProjectAnalysis = await invoke('analyze_project_directory', {
				path: projectPath
			});
			
			log.info('Project analysis completed', analysis);
			
			// Auto-populate form fields
			if (forceSync || !name.trim()) {
				name = analysis.name;
			}
			
			// Always update these fields when syncing
			if (forceSync) {
				framework = analysis.framework || '';
				packageManager = analysis.package_manager || '';
			} else {
				// Only update if not already set
				if (!framework.trim()) {
					framework = analysis.framework || '';
				}
				if (!packageManager.trim()) {
					packageManager = analysis.package_manager || '';
				}
			}
			
			success = forceSync 
				? 'Project properties synced successfully!' 
				: 'Project properties auto-detected successfully!';
			
			// Clear success message after 3 seconds
			setTimeout(() => {
				success = '';
			}, 3000);
			
		} catch (err) {
			log.error('Failed to analyze project', err);
			error = 'Failed to analyze project directory. Please check the path and try again.';
		} finally {
			isAnalyzing = false;
		}
	}

	async function handleSync() {
		if (path.trim()) {
			await analyzeProject(path, true);
		} else {
			error = 'Please select a project path first.';
		}
	}

	async function handleSubmit(event: SubmitEvent) {
		event.preventDefault();
		
		if (!name.trim() || !path.trim()) {
			error = 'Please fill in all required fields.';
			return;
		}

		try {
			error = '';
			success = '';
			
			const projectData: CreateProjectRequest = {
				name: name.trim(),
				description: description.trim() || undefined,
				path: path.trim(),
				framework: framework.trim() || undefined,
				package_manager: packageManager.trim() || undefined,
				build_command: undefined, // Auto-detected but not shown in form
				start_command: undefined, // Auto-detected but not shown in form
				test_command: undefined, // Auto-detected but not shown in form
				output_directory: undefined, // Auto-detected but not shown in form
				dev_port: undefined, // Auto-detected but not shown in form
				prod_port: undefined // Auto-detected but not shown in form
			};
			
			await onSubmit(projectData);
			
		} catch (err) {
			log.error('Failed to submit project form', err);
			error = 'Failed to save project. Please try again.';
		}
	}
</script>

<div class="space-y-6">
	<!-- Success Message -->
	{#if success}
		<Card class="mb-6 border-green-200 bg-green-50 dark:border-green-800 dark:bg-green-950">
			<CardContent class="pt-6">
				<div class="flex items-center gap-2 text-green-800 dark:text-green-200">
					<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
					</svg>
					<span class="font-medium">Success</span>
				</div>
				<p class="text-sm text-green-600 dark:text-green-300 mt-1">{success}</p>
			</CardContent>
		</Card>
	{/if}

	<!-- Error Message -->
	{#if error}
		<Card class="mb-6 border-red-200 bg-red-50 dark:border-red-800 dark:bg-red-950">
			<CardContent class="pt-6">
				<div class="flex items-center gap-2 text-red-800 dark:text-red-200">
					<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
					</svg>
					<span class="font-medium">Error</span>
				</div>
				<p class="text-sm text-red-600 dark:text-red-300 mt-1">{error}</p>
			</CardContent>
		</Card>
	{/if}

	<form onsubmit={handleSubmit} class="space-y-8">
		<!-- Basic Information -->
		<Card>
			<CardHeader>
				<div class="flex items-center justify-between">
					<div>
						<CardTitle>Basic Information</CardTitle>
						<CardDescription>
							Essential details about your project
						</CardDescription>
					</div>
					<Button
						type="button"
						variant="outline"
						size="sm"
						onclick={handleSync}
						disabled={isLoading || isAnalyzing || !path.trim()}
						class="flex items-center gap-2"
						title="Sync and re-detect project properties"
					>
						<RefreshCw class="h-4 w-4 {isAnalyzing ? 'animate-spin' : ''}" />
						<span class="hidden sm:inline">Sync</span>
					</Button>
				</div>
			</CardHeader>
			<CardContent class="space-y-6">
				<!-- Project Path - First and most important -->
				<FolderPicker
					bind:value={path}
					label="Project Path"
					description="Select the directory where your project will be located"
					placeholder="/path/to/your/project"
					disabled={isLoading || isAnalyzing}
					required
					onChange={handlePathChange}
				/>

				<div class="grid grid-cols-1 md:grid-cols-2 gap-6">
					<div class="space-y-2">
						<Label for="name">Project Name *</Label>
						<Input
							id="name"
							bind:value={name}
							placeholder="My Awesome Project"
							disabled={isLoading || isAnalyzing}
							required
						/>
					</div>
				</div>

				<div class="space-y-2">
					<Label for="description">Description</Label>
					<Textarea
						id="description"
						bind:value={description}
						placeholder="Brief description of your project..."
						disabled={isLoading || isAnalyzing}
						rows={3}
					/>
				</div>
			</CardContent>
		</Card>

		<!-- Framework & Package Manager -->
		<Card>
			<CardHeader>
				<CardTitle>Framework & Package Manager</CardTitle>
				<CardDescription>
					Development tools and frameworks used in your project
				</CardDescription>
			</CardHeader>
			<CardContent class="space-y-6">
				<div class="grid grid-cols-1 md:grid-cols-2 gap-6">
					<div class="space-y-2">
						<Label for="framework">Framework</Label>
						<Input
							id="framework"
							bind:value={framework}
							placeholder="e.g., React, Vue.js, Angular"
							disabled={isLoading || isAnalyzing}
						/>
					</div>
					<div class="space-y-2">
						<Label for="package-manager">Package Manager</Label>
						<Input
							id="package-manager"
							bind:value={packageManager}
							placeholder="e.g., npm, yarn, pnpm"
							disabled={isLoading || isAnalyzing}
						/>
					</div>
				</div>
			</CardContent>
		</Card>

		<!-- Form Actions -->
		<div class="flex justify-end space-x-4">
			<Button
				type="button"
				variant="outline"
				onclick={onCancel}
				disabled={isLoading || isAnalyzing}
			>
				Cancel
			</Button>
			<Button
				type="submit"
				disabled={isLoading || isAnalyzing}
			>
				{#if isAnalyzing}
					Analyzing...
				{:else if isLoading}
					{#if projectId}
						Updating...
					{:else}
						Creating...
					{/if}
				{:else}
					{#if projectId}
						Update Project
					{:else}
						Create Project
					{/if}
				{/if}
			</Button>
		</div>
	</form>
</div>
