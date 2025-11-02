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
	import { projectService } from '@/lib/domains/projects/services/projectService';
	import { logger } from '@/lib/domains/shared/services/logger';
	import type { CreateProjectRequest } from '@/lib/domains/projects/types';

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

	// Reset form when dialog opens
	$effect(() => {
		if (open) {
			resetForm();
		}
	});

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
					<Input
						id="framework"
						bind:value={framework}
						placeholder="React, Vue, Angular, etc."
						disabled={isLoading}
					/>
				</div>
				<div class="space-y-2">
					<Label for="packageManager">Package Manager</Label>
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
					/>
				</div>
			</div>

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
