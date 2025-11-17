<!--
	Project creation dialog component
	Allows users to create new projects with basic information
-->

<script lang="ts">
	import { Button } from '@/lib/components/ui/button';
	import { Input } from '@/lib/components/ui/input';
	import { Label } from '@/lib/components/ui/label';
	import { Textarea } from '@/lib/components/ui/textarea';
	import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/lib/components/ui/dialog';
	import Select from '@/lib/components/ui/select.svelte';
	import { projectService, extractContext } from '@/lib/domains/projects/services/projectService';
	import { logger } from '@/lib/domains/shared/services/logger';
	import { patternCollector, suggestionEngine } from '@/lib/domains/learning';
	import type { CreateProjectRequest } from '@/lib/domains/projects/types';
	import { Sparkles } from '@lucide/svelte';

	const log = logger.createScoped('ProjectCreateDialog');

	interface Props {
		open: boolean;
		onClose: () => void;
		onProjectCreated?: (project: any) => void;
	}

	let { open = $bindable(), onClose, onProjectCreated }: Props = $props();

	// Form state
	let name = $state('');
	let description = $state('');
	let path = $state('');
	let framework = $state('');
	let packageManager = $state('');
	let buildCommand = $state('');
	let startCommand = $state('');
	let testCommand = $state('');
	let outputDirectory = $state('');
	let devPort = $state<number | undefined>(undefined);
	let prodPort = $state<number | undefined>(undefined);

	let isLoading = $state(false);
	let error = $state('');
	let suggestions = $state<{
		framework?: string;
		packageManager?: string;
		buildCommand?: string;
		startCommand?: string;
		devPort?: number;
	} | null>(null);
	let loadingSuggestions = $state(false);
	let suggestionPatternIds = $state<{
		framework?: number;
		packageManager?: number;
		buildCommand?: number;
		startCommand?: number;
	}>({});

	// Reset form when dialog opens
	$effect(() => {
		if (open) {
			resetForm();
		}
	});

	// Load suggestions when framework or package manager changes
	$effect(() => {
		if (open && (framework || packageManager)) {
			loadSuggestions();
		}
	});

	const loadSuggestions = async () => {
		try {
			loadingSuggestions = true;
			const suggestionsData = await projectService.getProjectSetupSuggestions(
				framework || undefined,
				packageManager || undefined
			);
			suggestions = suggestionsData;
			
			// Load pattern IDs for tracking acceptance/rejection
			if (suggestions) {
				const context = extractContext(suggestions.framework, suggestions.packageManager);
				const frameworkSuggestions = await suggestionEngine.getContextualSuggestions('framework', context);
				const configSuggestions = await suggestionEngine.getContextualSuggestions('config', context);
				
				suggestionPatternIds = {
					framework: frameworkSuggestions[0]?.pattern_id,
					packageManager: configSuggestions[0]?.pattern_id,
					buildCommand: configSuggestions[0]?.pattern_id,
					startCommand: configSuggestions[0]?.pattern_id,
				};
			}
			
			log.info('Loaded project setup suggestions', suggestions);
		} catch (err) {
			log.warn('Failed to load suggestions', err);
			suggestions = null;
		} finally {
			loadingSuggestions = false;
		}
	};

	const applySuggestion = async () => {
		if (!suggestions) return;

		let appliedCount = 0;

		if (suggestions.framework && !framework) {
			framework = suggestions.framework;
			appliedCount++;
			// Track acceptance
			if (suggestionPatternIds.framework) {
				await suggestionEngine.recordSuggestionAccepted(suggestionPatternIds.framework);
			}
		}
		if (suggestions.packageManager && !packageManager) {
			packageManager = suggestions.packageManager;
			appliedCount++;
			if (suggestionPatternIds.packageManager) {
				await suggestionEngine.recordSuggestionAccepted(suggestionPatternIds.packageManager);
			}
		}
		if (suggestions.buildCommand && !buildCommand) {
			buildCommand = suggestions.buildCommand;
			appliedCount++;
			if (suggestionPatternIds.buildCommand) {
				await suggestionEngine.recordSuggestionAccepted(suggestionPatternIds.buildCommand);
			}
		}
		if (suggestions.startCommand && !startCommand) {
			startCommand = suggestions.startCommand;
			appliedCount++;
			if (suggestionPatternIds.startCommand) {
				await suggestionEngine.recordSuggestionAccepted(suggestionPatternIds.startCommand);
			}
		}
		if (suggestions.devPort && !devPort) {
			devPort = suggestions.devPort;
			appliedCount++;
		}

		if (appliedCount > 0) {
			log.info('Applied suggestions to form', { count: appliedCount });
			// Clear suggestions after applying
			suggestions = null;
		}
	};

	const resetForm = () => {
		name = '';
		description = '';
		path = '';
		framework = '';
		packageManager = '';
		buildCommand = '';
		startCommand = '';
		testCommand = '';
		outputDirectory = '';
		devPort = undefined;
		prodPort = undefined;
		error = '';
		suggestions = null;
		suggestionPatternIds = {};
	};

	const handleSubmit = async () => {
		if (!name.trim() || !path.trim()) {
			error = 'Name and path are required';
			return;
		}

		try {
			isLoading = true;
			error = '';

			const request: CreateProjectRequest = {
				name: name.trim(),
				description: description.trim() || undefined,
				path: path.trim(),
				framework: framework.trim() || undefined,
				package_manager: packageManager.trim() || undefined,
				build_command: buildCommand.trim() || undefined,
				start_command: startCommand.trim() || undefined,
				test_command: testCommand.trim() || undefined,
				output_directory: outputDirectory.trim() || undefined,
				dev_port: devPort,
				prod_port: prodPort,
			};

			const project = await projectService.createProject(request);
			
			log.info('Project created successfully', { id: project.id, name: project.name });
			
			onProjectCreated?.(project);
			onClose();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to create project';
			log.error('Failed to create project', err);
		} finally {
			isLoading = false;
		}
	};

	const handleCancel = () => {
		resetForm();
		onClose();
	};
</script>

<Dialog bind:open>
	<DialogContent class="sm:max-w-[600px]">
		<DialogHeader>
			<DialogTitle>Create New Project</DialogTitle>
			<DialogDescription>
				Create a new project with basic configuration. You can modify these settings later.
			</DialogDescription>
		</DialogHeader>

		<div class="grid gap-4 py-4">
			<!-- Basic Information -->
			<div class="grid grid-cols-2 gap-4">
				<div class="space-y-2">
					<Label for="name">Project Name *</Label>
					<Input
						id="name"
						bind:value={name}
						placeholder="My Awesome Project"
						disabled={isLoading}
					/>
				</div>
			</div>

			<div class="space-y-2">
				<Label for="description">Description</Label>
				<Textarea
					id="description"
					bind:value={description}
					placeholder="Brief description of your project..."
					disabled={isLoading}
				/>
			</div>

			<div class="space-y-2">
				<Label for="path">Project Path *</Label>
				<Input
					id="path"
					bind:value={path}
					placeholder="/path/to/your/project"
					disabled={isLoading}
				/>
			</div>

			<!-- Framework & Package Manager -->
			<div class="grid grid-cols-2 gap-4">
				<div class="space-y-2">
					<Label for="framework">Framework</Label>
					<div class="flex gap-2">
						<Input
							id="framework"
							bind:value={framework}
							placeholder="React, Vue, Angular, etc."
							disabled={isLoading}
							class="flex-1"
						/>
						{#if suggestions?.framework && !framework}
							<Button
								type="button"
								variant="ghost"
								size="sm"
								onclick={() => {
									if (suggestions?.framework) {
										framework = suggestions.framework;
									}
								}}
								title="Apply suggested framework: {suggestions.framework}"
								class="flex items-center gap-1"
							>
								<Sparkles class="w-4 h-4 text-primary" />
							</Button>
						{/if}
					</div>
				</div>
				<div class="space-y-2">
					<Label for="packageManager">Package Manager</Label>
					<div class="flex gap-2">
						<Select
							options={[
								{ value: '', label: 'None' },
								{ value: 'npm', label: 'npm' },
								{ value: 'yarn', label: 'yarn' },
								{ value: 'pnpm', label: 'pnpm' }
							]}
							defaultValue={packageManager}
							onSelect={(value) => packageManager = value}
							disabled={isLoading}
							class="flex-1"
						/>
						{#if suggestions?.packageManager && !packageManager}
							<Button
								type="button"
								variant="ghost"
								size="sm"
								onclick={async () => {
									if (suggestions?.packageManager) {
										packageManager = suggestions.packageManager;
										if (suggestionPatternIds.packageManager) {
											await suggestionEngine.recordSuggestionAccepted(suggestionPatternIds.packageManager);
										}
									}
								}}
								title="Apply suggested package manager: {suggestions.packageManager}"
								class="flex items-center gap-1"
							>
								<Sparkles class="w-4 h-4 text-primary" />
							</Button>
						{/if}
					</div>
				</div>
			</div>

			<!-- Intelligent Suggestions Badge -->
			{#if suggestions && (suggestions.buildCommand || suggestions.startCommand || suggestions.devPort)}
				<div class="flex items-center gap-2 p-3 bg-primary/5 border border-primary/20 rounded-md">
					<Sparkles class="w-4 h-4 text-primary flex-shrink-0" />
					<div class="flex-1 text-sm">
						<span class="font-medium text-primary">Smart Suggestions:</span>
						<span class="text-muted-foreground ml-2">
							{#if suggestions.buildCommand && !buildCommand}
								Build: {suggestions.buildCommand}
							{/if}
							{#if suggestions.startCommand && !startCommand}
								{#if suggestions.buildCommand && !buildCommand}, {/if}
								Start: {suggestions.startCommand}
							{/if}
							{#if suggestions.devPort && !devPort}
								{#if (suggestions.buildCommand && !buildCommand) || (suggestions.startCommand && !startCommand)}, {/if}
								Port: {suggestions.devPort}
							{/if}
						</span>
					</div>
					<Button
						type="button"
						variant="outline"
						size="sm"
						onclick={applySuggestion}
						disabled={isLoading}
					>
						Apply
					</Button>
				</div>
			{/if}

			<!-- Commands -->
			<div class="grid grid-cols-3 gap-4">
				<div class="space-y-2">
					<Label for="buildCommand">Build Command</Label>
					<Input
						id="buildCommand"
						bind:value={buildCommand}
						placeholder="npm run build"
						disabled={isLoading}
					/>
				</div>
				<div class="space-y-2">
					<Label for="startCommand">Start Command</Label>
					<Input
						id="startCommand"
						bind:value={startCommand}
						placeholder="npm start"
						disabled={isLoading}
					/>
				</div>
				<div class="space-y-2">
					<Label for="testCommand">Test Command</Label>
					<Input
						id="testCommand"
						bind:value={testCommand}
						placeholder="npm test"
						disabled={isLoading}
					/>
				</div>
			</div>

			<!-- Output Directory & Ports -->
			<div class="grid grid-cols-3 gap-4">
				<div class="space-y-2">
					<Label for="outputDirectory">Output Directory</Label>
					<Input
						id="outputDirectory"
						bind:value={outputDirectory}
						placeholder="dist, build, etc."
						disabled={isLoading}
					/>
				</div>
				<div class="space-y-2">
					<Label for="devPort">Dev Port</Label>
					<Input
						id="devPort"
						type="number"
						bind:value={devPort}
						placeholder="3000"
						disabled={isLoading}
					/>
				</div>
				<div class="space-y-2">
					<Label for="prodPort">Prod Port</Label>
					<Input
						id="prodPort"
						type="number"
						bind:value={prodPort}
						placeholder="8080"
						disabled={isLoading}
					/>
				</div>
			</div>

			{#if error}
				<div class="text-sm text-destructive bg-destructive/10 p-3 rounded-md">
					{error}
				</div>
			{/if}
		</div>

		<DialogFooter>
			<Button variant="outline" onclick={handleCancel} disabled={isLoading}>
				Cancel
			</Button>
			<Button onclick={handleSubmit} disabled={isLoading}>
				{#if isLoading}
					<div class="animate-spin rounded-full h-4 w-4 border-b-2 border-white mr-2"></div>
				{/if}
				Create Project
			</Button>
		</DialogFooter>
	</DialogContent>
</Dialog>
