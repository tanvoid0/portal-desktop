<!--
	View/Edit Script Page
	Dedicated page for viewing, editing and running scripts
-->
<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { onMount, onDestroy } from 'svelte';
	import { Button } from '@/lib/components/ui/button';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/lib/components/ui/card';
	import { Input } from '@/lib/components/ui/input';
	import { Textarea } from '@/lib/components/ui/textarea';
	import { Badge } from '@/lib/components/ui/badge';
	import { Label } from '@/lib/components/ui/label';
	import Select from '@/lib/components/ui/select.svelte';
	import {
		ArrowLeft,
		Trash2,
		Plus,
		FileCode,
		Save,
		Download,
		Copy,
		Edit,
		Play,
		Square,
		RefreshCw,
		Clock,
		CheckCircle,
		XCircle,
		Loader2,
		Terminal,
		History,
	} from '@lucide/svelte';
	import type { Block, BlockParameter, CreateBlockRequest } from '@/lib/domains/projects/pipelines';
	import { blockLibraryService, blockLibraryStore } from '@/lib/domains/projects/pipelines';
	import { scriptExecutionService, type ScriptExecutionInfo, EmbeddedTerminal } from '@/lib/domains/scripts';
	import { toast } from 'svelte-sonner';
	import { setBreadcrumbs } from '@/lib/domains/shared/stores/breadcrumbStore';

	let script: Block | null = $state(null);
	let loading = $state(true);
	let isEditing = $state(false);
	let saving = $state(false);

	// Execution state
	let executing = $state(false);
	let currentExecution: ScriptExecutionInfo | null = $state(null);
	let executionHistory: ScriptExecutionInfo[] = $state([]);
	let showHistory = $state(false);
	let interactiveMode = $state(true); // Default to interactive mode
	let showInteractiveTerminal = $state(false);
	let resolvedCommand = $state('');
	let unsubscribe: (() => void) | null = null;

	// Parameter values for execution
	let parameterValues = $state<Record<string, string>>({});
	let workingDirectory = $state('');

	// Form state for editing
	let formData = $state<CreateBlockRequest>({
		name: '',
		description: '',
		category: 'utility',
		parameters: [],
		command: '',
		executionType: 'script',
		defaultConfig: {},
		tags: [],
	});

	// Parameter editor state
	let showAddParameter = $state(false);
	let newParameter = $state<BlockParameter>({
		name: '',
		type: 'string',
		description: '',
		required: false,
		defaultValue: '',
	});

	const scriptId = $derived($page.params.id);

	onMount(async () => {
		await loadScript();
		// Check if we should auto-start in edit mode from URL query
		const editParam = $page.url.searchParams.get('edit');
		if (editParam === 'true') {
			isEditing = true;
		}
		// Load execution history
		await loadExecutionHistory();
	});

	onDestroy(() => {
		// Clean up subscription
		if (unsubscribe) {
			unsubscribe();
		}
	});

	async function loadScript() {
		loading = true;
		try {
			await blockLibraryStore.loadBlocks();
			const blocks = await blockLibraryService.getBlocks();
			script = blocks.find((b) => b.id === scriptId) || null;

			if (script) {
				setBreadcrumbs([
					{ label: 'Scripts', href: '/scripts' },
					{ label: script.name, href: `/scripts/${scriptId}` },
				]);

				// Initialize form data
				formData = {
					name: script.name,
					description: script.description,
					category: script.category,
					parameters: [...script.parameters],
					command: script.command,
					executionType: script.executionType,
					defaultConfig: { ...script.defaultConfig },
					tags: [...(script.tags || [])],
				};

				// Initialize parameter values with defaults
				parameterValues = {};
				const currentScript = script;
				currentScript.parameters.forEach((param) => {
					if (param.defaultValue !== undefined && param.defaultValue !== '') {
						parameterValues[param.name] = String(param.defaultValue);
					} else if (currentScript.defaultConfig?.[param.name] !== undefined) {
						parameterValues[param.name] = String(currentScript.defaultConfig[param.name]);
					} else {
						parameterValues[param.name] = '';
					}
				});
			} else {
				toast.error('Script not found');
				goto('/scripts');
			}
		} catch (error) {
			console.error('Failed to load script', error);
			toast.error('Failed to load script');
		} finally {
			loading = false;
		}
	}

	async function loadExecutionHistory() {
		try {
			executionHistory = await scriptExecutionService.getExecutionsByBlock(scriptId, 10);
		} catch (error) {
			console.error('Failed to load execution history', error);
		}
	}

	function resolveCommandWithParams(command: string, params: Record<string, string>): string {
		let resolved = command;
		for (const [key, value] of Object.entries(params)) {
			resolved = resolved.replace(new RegExp(`\\$\\{${key}\\}`, 'g'), value);
			resolved = resolved.replace(new RegExp(`\\$${key}`, 'g'), value);
		}
		return resolved;
	}

	async function handleRun() {
		if (!script) return;

		// Validate required parameters
		const missingParams = script.parameters
			.filter((p) => p.required && (!parameterValues[p.name] || parameterValues[p.name].trim() === ''))
			.map((p) => p.name);

		if (missingParams.length > 0) {
			toast.error(`Missing required parameters: ${missingParams.join(', ')}`);
			return;
		}

		// Resolve command with parameters
		resolvedCommand = resolveCommandWithParams(script.command, parameterValues);

		if (interactiveMode) {
			// Interactive mode - use embedded terminal
			showInteractiveTerminal = true;
			executing = true;
			toast.success('Starting interactive terminal...');
			return;
		}

		// Non-interactive mode - use existing script execution service
		executing = true;
		try {
			const executionId = await scriptExecutionService.executeScript({
				blockId: scriptId,
				command: script.command,
				parameters: parameterValues,
				workingDirectory: workingDirectory || undefined,
			});

			toast.success('Script execution started');

			// Subscribe to execution updates
			unsubscribe = scriptExecutionService.subscribeToExecution(
				executionId,
				(execution) => {
					currentExecution = execution;

					// Show completion toast
					if (execution.status === 'success') {
						toast.success('Script completed successfully');
						executing = false;
						loadExecutionHistory();
					} else if (execution.status === 'failed') {
						toast.error(`Script failed: ${execution.error || 'Unknown error'}`);
						executing = false;
						loadExecutionHistory();
					} else if (execution.status === 'cancelled') {
						toast.info('Script execution cancelled');
						executing = false;
						loadExecutionHistory();
					}
				},
				500
			);

			// Get initial execution state
			currentExecution = await scriptExecutionService.getExecution(executionId);
		} catch (error) {
			console.error('Failed to execute script', error);
			toast.error('Failed to start script execution');
			executing = false;
		}
	}

	function handleInteractiveComplete(exitCode: number | null) {
		executing = false;
		if (exitCode === 0) {
			toast.success('Script completed successfully');
		} else if (exitCode !== null) {
			toast.error(`Script exited with code ${exitCode}`);
		}
		loadExecutionHistory();
	}

	function handleInteractiveStart(processId: string) {
		console.log('Interactive terminal started with process:', processId);
	}

	function closeInteractiveTerminal() {
		showInteractiveTerminal = false;
		executing = false;
	}

	async function handleCancel() {
		if (!currentExecution) return;

		try {
			await scriptExecutionService.cancelExecution(currentExecution.id);
			toast.info('Cancelling execution...');
		} catch (error) {
			console.error('Failed to cancel execution', error);
			toast.error('Failed to cancel execution');
		}
	}

	function viewExecution(execution: ScriptExecutionInfo) {
		currentExecution = execution;
		showHistory = false;
	}

	function getStatusIcon(status: string) {
		switch (status) {
			case 'pending':
				return Clock;
			case 'running':
				return Loader2;
			case 'success':
				return CheckCircle;
			case 'failed':
				return XCircle;
			case 'cancelled':
				return Square;
			default:
				return Clock;
		}
	}

	function getStatusColor(status: string) {
		switch (status) {
			case 'pending':
				return 'text-yellow-500';
			case 'running':
				return 'text-blue-500 animate-spin';
			case 'success':
				return 'text-green-500';
			case 'failed':
				return 'text-red-500';
			case 'cancelled':
				return 'text-gray-500';
			default:
				return 'text-muted-foreground';
		}
	}

	function formatDuration(start: string, end: string | null): string {
		const startDate = new Date(start);
		const endDate = end ? new Date(end) : new Date();
		const diff = endDate.getTime() - startDate.getTime();

		if (diff < 1000) return `${diff}ms`;
		if (diff < 60000) return `${Math.floor(diff / 1000)}s`;
		return `${Math.floor(diff / 60000)}m ${Math.floor((diff % 60000) / 1000)}s`;
	}

	function startEditing() {
		isEditing = true;
	}

	function cancelEditing() {
		if (script) {
			formData = {
				name: script.name,
				description: script.description,
				category: script.category,
				parameters: [...script.parameters],
				command: script.command,
				executionType: script.executionType,
				defaultConfig: { ...script.defaultConfig },
				tags: [...(script.tags || [])],
			};
		}
		isEditing = false;
		showAddParameter = false;
	}

	async function handleSave() {
		if (!formData.name || !formData.description || !formData.command) {
			toast.error('Please fill in all required fields');
			return;
		}

		saving = true;
		try {
			// Build defaultConfig from parameters with defaultValue
			const defaultConfig: Record<string, any> = {};
			formData.parameters.forEach((p) => {
				if (p.defaultValue !== undefined && p.defaultValue !== '') {
					defaultConfig[p.name] = p.defaultValue;
				}
			});
			formData.defaultConfig = defaultConfig;

			await blockLibraryService.updateBlock(scriptId, formData);
			toast.success('Script updated successfully');
			isEditing = false;
			await loadScript();
		} catch (error) {
			console.error('Failed to update script', error);
			toast.error('Failed to update script');
		} finally {
			saving = false;
		}
	}

	async function handleDelete() {
		if (!confirm('Are you sure you want to delete this script?')) return;

		try {
			await blockLibraryService.deleteBlock(scriptId);
			toast.success('Script deleted successfully');
			goto('/scripts');
		} catch (error) {
			console.error('Failed to delete script', error);
			toast.error('Failed to delete script');
		}
	}

	function handleExport() {
		if (!script) return;
		try {
			const dataStr = JSON.stringify(script, null, 2);
			const dataBlob = new Blob([dataStr], { type: 'application/json' });
			const url = URL.createObjectURL(dataBlob);
			const link = document.createElement('a');
			link.href = url;
			link.download = `${script.name.toLowerCase().replace(/\s+/g, '-')}.block.json`;
			link.click();
			URL.revokeObjectURL(url);
			toast.success('Script exported successfully');
		} catch (error) {
			console.error('Failed to export script', error);
			toast.error('Failed to export script');
		}
	}

	function handleCopyCommand() {
		if (!script) return;
		navigator.clipboard.writeText(script.command);
		toast.success('Command copied to clipboard');
	}

	function handleCopyOutput() {
		if (!currentExecution?.output) return;
		navigator.clipboard.writeText(currentExecution.output);
		toast.success('Output copied to clipboard');
	}

	function addParameter() {
		if (!newParameter.name.trim()) {
			toast.error('Parameter name is required');
			return;
		}
		if (formData.parameters.some((p) => p.name === newParameter.name)) {
			toast.error('Parameter name already exists');
			return;
		}
		formData.parameters = [...formData.parameters, { ...newParameter }];
		newParameter = {
			name: '',
			type: 'string',
			description: '',
			required: false,
			defaultValue: '',
		};
		showAddParameter = false;
	}

	function removeParameter(index: number) {
		formData.parameters = formData.parameters.filter((_, i) => i !== index);
	}

	const isValid = $derived(formData.name && formData.description && formData.command);
</script>

<svelte:head>
	<title>{script?.name || 'Script'} - Portal Desktop</title>
</svelte:head>

<div class="container mx-auto p-6 max-w-4xl">
	{#if loading}
		<div class="flex items-center justify-center py-12">
			<p class="text-muted-foreground">Loading script...</p>
		</div>
	{:else if script}
		<!-- Header -->
		<div class="flex items-center gap-4 mb-6">
			<Button variant="ghost" size="icon" onclick={() => goto('/scripts')}>
				<ArrowLeft class="h-5 w-5" />
			</Button>
			<div class="flex-1">
				<h1 class="text-2xl font-bold flex items-center gap-2">
					<FileCode class="h-6 w-6" />
					{isEditing ? 'Edit Script' : script.name}
				</h1>
				<p class="text-muted-foreground">{isEditing ? 'Modify script configuration' : script.description}</p>
			</div>
			<div class="flex gap-2">
				{#if isEditing}
					<Button variant="outline" onclick={cancelEditing}>Cancel</Button>
					<Button onclick={handleSave} disabled={!isValid || saving}>
						<Save class="h-4 w-4 mr-2" />
						{saving ? 'Saving...' : 'Save'}
					</Button>
				{:else}
					<Button variant="outline" onclick={() => (showHistory = !showHistory)}>
						<History class="h-4 w-4 mr-2" />
						History
					</Button>
					<Button variant="outline" onclick={handleExport}>
						<Download class="h-4 w-4 mr-2" />
						Export
					</Button>
					<Button variant="outline" onclick={startEditing}>
						<Edit class="h-4 w-4 mr-2" />
						Edit
					</Button>
					<Button variant="destructive" onclick={handleDelete}>
						<Trash2 class="h-4 w-4 mr-2" />
						Delete
					</Button>
				{/if}
			</div>
		</div>

		<div class="space-y-6">
			{#if isEditing}
				<!-- Edit Mode -->
				<!-- Basic Information -->
				<Card>
					<CardHeader>
						<CardTitle>Basic Information</CardTitle>
					</CardHeader>
					<CardContent class="space-y-4">
						<div class="grid grid-cols-2 gap-4">
							<div>
								<Label for="name">Name *</Label>
								<Input id="name" bind:value={formData.name} class="mt-1" />
							</div>
							<div>
								<Label for="category">Category *</Label>
								<Select
									options={[
										{ value: 'utility', label: 'Utility' },
										{ value: 'build', label: 'Build' },
										{ value: 'test', label: 'Test' },
										{ value: 'deploy', label: 'Deploy' },
										{ value: 'custom', label: 'Custom' },
									]}
									bind:value={formData.category}
									class="mt-1"
								/>
							</div>
						</div>
						<div>
							<Label for="description">Description *</Label>
							<Textarea id="description" bind:value={formData.description} rows={3} class="mt-1" />
						</div>
						<div>
							<Label for="tags">Tags</Label>
							<Input
								id="tags"
								value={(formData.tags || []).join(', ')}
								oninput={(e) => {
									formData.tags = (e.target as HTMLInputElement).value
										.split(',')
										.map((t) => t.trim())
										.filter((t) => t.length > 0);
								}}
								placeholder="comma-separated"
								class="mt-1"
							/>
						</div>
					</CardContent>
				</Card>

				<!-- Command -->
				<Card>
					<CardHeader>
						<CardTitle>Command Configuration</CardTitle>
					</CardHeader>
					<CardContent class="space-y-4">
						<div>
							<Label for="command">Command *</Label>
							<Textarea
								id="command"
								bind:value={formData.command}
								rows={3}
								class="font-mono text-sm mt-1"
							/>
						</div>
						<div>
							<Label for="executionType">Execution Type</Label>
							<Select
								options={[
									{ value: 'script', label: 'Script' },
									{ value: 'command', label: 'Command' },
									{ value: 'docker', label: 'Docker' },
								]}
								bind:value={formData.executionType}
								class="mt-1"
							/>
						</div>
					</CardContent>
				</Card>

				<!-- Parameters -->
				<Card>
					<CardHeader>
						<div class="flex items-center justify-between">
							<CardTitle>Parameters</CardTitle>
							<Button variant="outline" size="sm" onclick={() => (showAddParameter = !showAddParameter)}>
								<Plus class="h-4 w-4 mr-2" />
								Add
							</Button>
						</div>
					</CardHeader>
					<CardContent class="space-y-4">
						{#if showAddParameter}
							<div class="rounded-lg border p-4 space-y-4 bg-muted/30">
								<div class="grid grid-cols-2 gap-4">
									<div>
										<Label class="text-sm">Name *</Label>
										<Input bind:value={newParameter.name} class="mt-1" />
									</div>
									<div>
										<Label class="text-sm">Type</Label>
										<Select
											options={[
												{ value: 'string', label: 'String' },
												{ value: 'number', label: 'Number' },
												{ value: 'boolean', label: 'Boolean' },
												{ value: 'select', label: 'Select' },
												{ value: 'file', label: 'File' },
												{ value: 'directory', label: 'Directory' },
											]}
											bind:value={newParameter.type}
											class="mt-1"
										/>
									</div>
								</div>
								<div>
									<Label class="text-sm">Description</Label>
									<Input bind:value={newParameter.description} class="mt-1" />
								</div>
								<div class="grid grid-cols-2 gap-4">
									<div>
										<Label class="text-sm">Default Value</Label>
										<Input bind:value={newParameter.defaultValue} class="mt-1" />
									</div>
									<div class="flex items-end pb-2">
										<label class="flex items-center gap-2">
											<input type="checkbox" bind:checked={newParameter.required} class="rounded" />
											<span class="text-sm">Required</span>
										</label>
									</div>
								</div>
								<div class="flex gap-2">
									<Button size="sm" onclick={addParameter}>Add</Button>
									<Button size="sm" variant="outline" onclick={() => (showAddParameter = false)}>Cancel</Button>
								</div>
							</div>
						{/if}

						{#if formData.parameters.length > 0}
							<div class="rounded-lg border divide-y">
								{#each formData.parameters as param, i}
									<div class="flex items-center justify-between p-3">
										<div class="flex items-center gap-2">
											<code class="font-mono text-sm bg-muted px-2 py-0.5 rounded">{param.name}</code>
											<Badge variant="outline">{param.type}</Badge>
											{#if param.required}
												<Badge variant="destructive" class="text-xs">required</Badge>
											{/if}
										</div>
										<Button variant="ghost" size="icon" onclick={() => removeParameter(i)}>
											<Trash2 class="h-4 w-4" />
										</Button>
									</div>
								{/each}
							</div>
						{:else}
							<p class="text-center py-4 text-muted-foreground">No parameters</p>
						{/if}
					</CardContent>
				</Card>
			{:else}
				<!-- View Mode -->

				<!-- Execution History Sidebar -->
				{#if showHistory}
					<Card>
						<CardHeader>
							<div class="flex items-center justify-between">
								<CardTitle>Execution History</CardTitle>
								<Button variant="ghost" size="sm" onclick={() => loadExecutionHistory()}>
									<RefreshCw class="h-4 w-4" />
								</Button>
							</div>
						</CardHeader>
						<CardContent>
							{#if executionHistory.length > 0}
								<div class="space-y-2">
									{#each executionHistory as execution}
										{@const StatusIcon = getStatusIcon(execution.status)}
										<button
											class="w-full text-left p-3 rounded-lg border hover:bg-muted/50 transition-colors"
											class:bg-muted={currentExecution?.id === execution.id}
											onclick={() => viewExecution(execution)}
										>
											<div class="flex items-center justify-between">
												<div class="flex items-center gap-2">
													<StatusIcon class="h-4 w-4 {getStatusColor(execution.status)}" />
													<span class="font-medium text-sm">{execution.status}</span>
												</div>
												<span class="text-xs text-muted-foreground">
													{new Date(execution.startedAt).toLocaleString()}
												</span>
											</div>
											{#if execution.finishedAt}
												<p class="text-xs text-muted-foreground mt-1">
													Duration: {formatDuration(execution.startedAt, execution.finishedAt)}
												</p>
											{/if}
										</button>
									{/each}
								</div>
							{:else}
								<p class="text-center py-4 text-muted-foreground">No executions yet</p>
							{/if}
						</CardContent>
					</Card>
				{/if}

				<!-- Run Script Card -->
				<Card class="border-primary/50">
					<CardHeader>
						<div class="flex items-center justify-between">
							<div>
								<CardTitle class="flex items-center gap-2">
									<Play class="h-5 w-5" />
									Run Script
								</CardTitle>
								<CardDescription>Configure parameters and execute</CardDescription>
							</div>
							{#if executing}
								<Button variant="destructive" onclick={handleCancel}>
									<Square class="h-4 w-4 mr-2" />
									Stop
								</Button>
							{:else}
								<Button onclick={handleRun}>
									<Play class="h-4 w-4 mr-2" />
									Run
								</Button>
							{/if}
						</div>
					</CardHeader>
					<CardContent class="space-y-4">
						<!-- Working Directory -->
						<div>
							<Label for="workingDir">Working Directory (optional)</Label>
							<Input
								id="workingDir"
								bind:value={workingDirectory}
								placeholder="Leave empty to use current directory"
								class="mt-1"
							/>
						</div>

						<!-- Interactive Mode Toggle -->
						<div class="flex items-center justify-between p-3 rounded-lg bg-muted/50">
							<div>
								<Label class="text-sm font-medium">Interactive Mode</Label>
								<p class="text-xs text-muted-foreground mt-0.5">
									Enable for scripts that require user input (passwords, confirmations, etc.)
								</p>
							</div>
							<label class="relative inline-flex items-center cursor-pointer">
								<input
									type="checkbox"
									bind:checked={interactiveMode}
									class="sr-only peer"
								/>
								<div class="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-primary/20 dark:peer-focus:ring-primary/40 rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all dark:border-gray-600 peer-checked:bg-primary"></div>
							</label>
						</div>

						<!-- Parameters -->
						{#if script.parameters.length > 0}
							<div class="space-y-3">
								<Label class="text-sm font-medium">Parameters</Label>
								{#each script.parameters as param}
									<div class="grid grid-cols-3 gap-4 items-start">
										<div class="flex items-center gap-2">
											<code class="font-mono text-sm">{param.name}</code>
											{#if param.required}
												<span class="text-red-500">*</span>
											{/if}
										</div>
										<div class="col-span-2">
											{#if param.type === 'select' && param.options}
												<Select
													options={param.options.map((o) => ({ value: o, label: o }))}
													bind:value={parameterValues[param.name]}
													placeholder={param.description || `Select ${param.name}`}
												/>
											{:else if param.type === 'boolean'}
												<label class="flex items-center gap-2">
													<input
														type="checkbox"
														checked={parameterValues[param.name] === 'true'}
														onchange={(e) => {
															parameterValues[param.name] = (e.target as HTMLInputElement).checked ? 'true' : 'false';
														}}
														class="rounded"
													/>
													<span class="text-sm text-muted-foreground">{param.description}</span>
												</label>
											{:else}
												<Input
													bind:value={parameterValues[param.name]}
													placeholder={param.description || param.name}
												/>
											{/if}
										</div>
									</div>
								{/each}
							</div>
						{/if}

						<!-- Interactive Terminal -->
						{#if showInteractiveTerminal && resolvedCommand}
							<div class="space-y-3 pt-4 border-t">
								<div class="flex items-center justify-between">
									<div class="flex items-center gap-2">
										<Terminal class="h-4 w-4" />
										<span class="font-medium">Interactive Terminal</span>
									</div>
									<Button variant="outline" size="sm" onclick={closeInteractiveTerminal}>
										Close Terminal
									</Button>
								</div>
								<EmbeddedTerminal
									command={resolvedCommand}
									workingDirectory={workingDirectory || undefined}
									onComplete={handleInteractiveComplete}
									onStart={handleInteractiveStart}
								/>
							</div>
						{/if}

						<!-- Execution Output (non-interactive mode) -->
						{#if currentExecution && !showInteractiveTerminal}
							{@const StatusIcon = getStatusIcon(currentExecution.status)}
							<div class="space-y-3 pt-4 border-t">
								<div class="flex items-center justify-between">
									<div class="flex items-center gap-2">
										<Terminal class="h-4 w-4" />
										<span class="font-medium">Output</span>
										{#if currentExecution.output}
											<Button variant="ghost" size="sm" onclick={handleCopyOutput} class="h-6 px-2">
												<Copy class="h-3 w-3 mr-1" />
												Copy
											</Button>
										{/if}
									</div>
									<div class="flex items-center gap-2">
										<StatusIcon class="h-4 w-4 {getStatusColor(currentExecution.status)}" />
										<Badge variant={currentExecution.status === 'success' ? 'default' : currentExecution.status === 'failed' ? 'destructive' : 'secondary'}>
											{currentExecution.status}
										</Badge>
										{#if currentExecution.finishedAt}
											<span class="text-xs text-muted-foreground">
												{formatDuration(currentExecution.startedAt, currentExecution.finishedAt)}
											</span>
										{/if}
									</div>
								</div>
								<div class="bg-black text-green-400 rounded-lg p-4 font-mono text-sm max-h-80 overflow-auto">
									{#if currentExecution.output}
										<pre class="whitespace-pre-wrap">{currentExecution.output}</pre>
									{:else if currentExecution.status === 'running'}
										<span class="animate-pulse">Waiting for output...</span>
									{:else}
										<span class="text-gray-500">No output</span>
									{/if}
								</div>
								{#if currentExecution.error}
									<div class="bg-red-500/10 border border-red-500/30 rounded-lg p-4">
										<p class="text-sm text-red-500 font-medium">Error</p>
										<p class="text-sm text-red-400 mt-1">{currentExecution.error}</p>
									</div>
								{/if}
								{#if currentExecution.exitCode !== null}
									<p class="text-sm text-muted-foreground">
										Exit code: <code class="bg-muted px-1 rounded">{currentExecution.exitCode}</code>
									</p>
								{/if}
							</div>
						{/if}
					</CardContent>
				</Card>

				<!-- Command -->
				<Card>
					<CardHeader>
						<CardTitle>Command</CardTitle>
					</CardHeader>
					<CardContent>
						<div class="bg-muted rounded-lg p-4 relative group">
							<pre class="text-sm font-mono whitespace-pre-wrap">{script.command}</pre>
							<Button
								variant="ghost"
								size="sm"
								class="absolute top-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity"
								onclick={handleCopyCommand}
							>
								<Copy class="h-4 w-4 mr-1" />
								Copy
							</Button>
						</div>
					</CardContent>
				</Card>

				<!-- Details -->
				<Card>
					<CardHeader>
						<CardTitle>Details</CardTitle>
					</CardHeader>
					<CardContent>
						<div class="grid grid-cols-3 gap-6">
							<div>
								<Label class="text-xs text-muted-foreground">Category</Label>
								<p class="font-medium mt-1">{script.category}</p>
							</div>
							<div>
								<Label class="text-xs text-muted-foreground">Execution Type</Label>
								<p class="font-medium mt-1">{script.executionType}</p>
							</div>
							<div>
								<Label class="text-xs text-muted-foreground">Version</Label>
								<p class="font-medium mt-1">{script.version}</p>
							</div>
						</div>
						{#if script.tags && script.tags.length > 0}
							<div class="mt-4">
								<Label class="text-xs text-muted-foreground">Tags</Label>
								<div class="flex flex-wrap gap-1 mt-2">
									{#each script.tags as tag}
										<Badge variant="secondary">{tag}</Badge>
									{/each}
								</div>
							</div>
						{/if}
					</CardContent>
				</Card>

				<!-- Parameters Reference -->
				{#if script.parameters.length > 0}
					<Card>
						<CardHeader>
							<CardTitle>Parameters Reference ({script.parameters.length})</CardTitle>
						</CardHeader>
						<CardContent>
							<div class="rounded-lg border divide-y">
								{#each script.parameters as param}
									<div class="p-4">
										<div class="flex items-center gap-2">
											<code class="font-mono font-medium">{param.name}</code>
											<Badge variant="outline">{param.type}</Badge>
											{#if param.required}
												<Badge variant="destructive" class="text-xs">required</Badge>
											{/if}
										</div>
										{#if param.description}
											<p class="text-sm text-muted-foreground mt-1">{param.description}</p>
										{/if}
										{#if param.defaultValue !== undefined && param.defaultValue !== ''}
											<p class="text-xs text-muted-foreground mt-1">
												Default: <code class="bg-muted px-1 rounded">{param.defaultValue}</code>
											</p>
										{/if}
										{#if param.options && param.options.length > 0}
											<p class="text-xs text-muted-foreground mt-1">
												Options: {param.options.join(', ')}
											</p>
										{/if}
									</div>
								{/each}
							</div>
						</CardContent>
					</Card>
				{/if}

				<!-- JSON -->
				<Card>
					<CardHeader>
						<CardTitle>JSON Definition</CardTitle>
						<CardDescription>Full script configuration as JSON</CardDescription>
					</CardHeader>
					<CardContent>
						<pre class="bg-muted rounded-lg p-4 text-xs font-mono overflow-x-auto max-h-80">{JSON.stringify(script, null, 2)}</pre>
					</CardContent>
				</Card>
			{/if}
		</div>
	{/if}
</div>
