<!--
	Utilities Page - Custom Scripts Management
	Allows users to create, manage, and run custom scripts with various parameter types
-->

<script lang="ts">
	import { onMount } from 'svelte';
	import { Button } from '$lib/components/ui/button';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Textarea } from '$lib/components/ui/textarea';
	import { Badge } from '$lib/components/ui/badge';
	import { Tabs, TabsContent, TabsList, TabsTrigger } from '$lib/components/ui/tabs';
	import {
		Dialog,
		DialogContent,
		DialogDescription,
		DialogFooter,
		DialogHeader,
		DialogTitle,
		DialogTrigger
	} from '$lib/components/ui/dialog';
	import {
		AlertDialog,
		AlertDialogAction,
		AlertDialogCancel,
		AlertDialogContent,
		AlertDialogDescription,
		AlertDialogFooter,
		AlertDialogHeader,
		AlertDialogTitle,
		AlertDialogTrigger
	} from '$lib/components/ui/alert-dialog';
	import { Plus, Play, Edit, Trash2, Terminal, Settings, FileCode, Square } from '@lucide/svelte';
	import { CustomScriptService, type CustomScript, type ScriptParameter } from '$lib/domains/custom_scripts/services/customScriptService';
	import { ScriptEditor } from '$lib/domains/custom_scripts/components/ScriptEditor';
	import { useRunningScripts } from '$lib/domains/custom_scripts/hooks/useRunningScripts';
	import RunningInstancesView from '$lib/domains/custom_scripts/components/ScriptRunner/RunningInstancesView.svelte';
	import ScriptRunner from './components/ScriptRunner.svelte';

	// State
	let scripts = $state<CustomScript[]>([]);
	let loading = $state(false);
	let error = $state<string | null>(null);
	let selectedScript = $state<CustomScript | null>(null);
	let showEditor = $state(false);
	let showRunner = $state(false);
	let showRunningInstances = $state(false);
	let searchQuery = $state('');

	// Running scripts hook
	const runningScripts = useRunningScripts();
	let runningScriptsList = $state(runningScripts.getAll());

	// Subscribe to running scripts changes
	$effect(() => {
		const unsubscribe = runningScripts.subscribe((scripts) => {
			runningScriptsList = scripts;
		});
		return unsubscribe;
	});

	// Load scripts on mount
	onMount(() => {
		loadScripts();
	});

	async function loadScripts() {
		loading = true;
		error = null;
		try {
			scripts = await CustomScriptService.getAllScripts();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load scripts';
			console.error('Failed to load scripts:', err);
		} finally {
			loading = false;
		}
	}

	function handleCreateScript() {
		selectedScript = null;
		showEditor = true;
	}

	function handleEditScript(script: CustomScript) {
		selectedScript = script;
		showEditor = true;
	}

	function handleRunScript(script: CustomScript) {
		selectedScript = script;
		showRunner = true;
	}

	function handleOpenRunningScripts(script: CustomScript) {
		selectedScript = script;
		showRunningInstances = true;
	}

	async function handleDeleteScript(script: CustomScript) {
		try {
			await CustomScriptService.deleteScript(script.id);
			await loadScripts();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to delete script';
			console.error('Failed to delete script:', err);
		}
	}

	function handleScriptSaved() {
		showEditor = false;
		selectedScript = null;
		loadScripts();
	}

	function handleScriptRun() {
		// Script runner will handle recording the run
		if (selectedScript) {
			CustomScriptService.recordScriptRun(selectedScript.id).catch(console.error);
		}
	}

	async function handleScriptStop(runningInstanceId: string) {
		const runningScript = runningScripts.getById(runningInstanceId);
		if (runningScript) {
			await runningScript.stopCallback();
		}
	}

	// Filter scripts based on search query
	const filteredScripts = $derived(() => {
		if (!searchQuery.trim()) return scripts;
		const query = searchQuery.toLowerCase();
		return scripts.filter(script =>
			script.name.toLowerCase().includes(query) ||
			script.description?.toLowerCase().includes(query) ||
			script.category?.toLowerCase().includes(query)
		);
	});
</script>

<div class="container mx-auto p-6 space-y-6">
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-3xl font-bold">Utilities</h1>
			<p class="text-muted-foreground mt-1">Manage and run custom scripts</p>
		</div>
		<Button onclick={handleCreateScript}>
			<Plus class="h-4 w-4 mr-2" />
			New Script
		</Button>
	</div>

	{#if error}
		<div class="p-4 border border-red-200 bg-red-50 rounded-md">
			<p class="text-sm text-red-600">{error}</p>
		</div>
	{/if}

	{#if loading}
		<div class="text-center py-8">
			<p class="text-muted-foreground">Loading scripts...</p>
		</div>
	{:else if filteredScripts().length === 0}
		<Card>
			<CardContent class="py-12 text-center">
				<FileCode class="h-12 w-12 mx-auto text-muted-foreground mb-4" />
				<h3 class="text-lg font-semibold mb-2">No scripts found</h3>
				<p class="text-muted-foreground mb-4">
					{searchQuery ? 'No scripts match your search.' : 'Get started by creating your first custom script.'}
				</p>
				<Button onclick={handleCreateScript}>
					<Plus class="h-4 w-4 mr-2" />
					Create Script
				</Button>
			</CardContent>
		</Card>
	{:else}
		<div class="space-y-4">
			<Input
				type="text"
				placeholder="Search scripts..."
				bind:value={searchQuery}
				class="max-w-sm"
			/>

			<div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
				{#each filteredScripts() as script (script.id)}
					{@const runningInstances = runningScripts.getByScriptId(script.id)}
					<Card>
						<CardHeader>
							<div class="flex items-start justify-between">
								<div class="flex-1">
									<CardTitle class="flex items-center gap-2">
										{#if script.icon}
											<span>{script.icon}</span>
										{/if}
										{script.name}
									</CardTitle>
									{#if script.description}
										<CardDescription class="mt-1">{script.description}</CardDescription>
									{/if}
								</div>
							</div>
							<div class="flex flex-wrap gap-2 mt-2">
								{#if runningInstances.length > 0}
									<Badge variant="default" class="bg-green-500 text-white">
										Running ({runningInstances.length})
									</Badge>
								{/if}
								{#if script.category}
									<Badge variant="secondary">{script.category}</Badge>
								{/if}
								{#if script.requires_sudo}
									<Badge variant="destructive">Requires Sudo</Badge>
								{/if}
								{#if script.is_interactive}
									<Badge variant="outline">Interactive</Badge>
								{/if}
							</div>
						</CardHeader>
						<CardContent>
							<div class="space-y-2">
								<div class="text-sm text-muted-foreground">
									<p>Run count: {script.run_count}</p>
									{#if script.last_run_at}
										<p>Last run: {new Date(script.last_run_at).toLocaleString()}</p>
									{/if}
								</div>
								<div class="space-y-2">
									<div class="flex gap-2">
										{#if runningInstances.length > 0}
											<Button
												size="sm"
												onclick={() => handleOpenRunningScripts(script)}
												class="flex-1"
											>
												<Terminal class="h-4 w-4 mr-2" />
												Open ({runningInstances.length})
											</Button>
										{:else}
											<Button
												size="sm"
												onclick={() => handleRunScript(script)}
												class="flex-1"
											>
												<Play class="h-4 w-4 mr-2" />
												Run
											</Button>
										{/if}
										<Button
											size="sm"
											variant="outline"
											onclick={() => handleEditScript(script)}
											disabled={runningInstances.length > 0}
										>
											<Edit class="h-4 w-4" />
										</Button>
										<AlertDialog>
											<AlertDialogTrigger>
												<Button
													size="sm"
													variant="destructive"
													disabled={runningInstances.length > 0}
												>
													<Trash2 class="h-4 w-4" />
												</Button>
											</AlertDialogTrigger>
											<AlertDialogContent>
												<AlertDialogHeader>
													<AlertDialogTitle>Delete Script</AlertDialogTitle>
													<AlertDialogDescription>
														Are you sure you want to delete "{script.name}"? This action cannot be undone.
													</AlertDialogDescription>
												</AlertDialogHeader>
												<AlertDialogFooter>
													<AlertDialogCancel>Cancel</AlertDialogCancel>
													<AlertDialogAction onclick={() => handleDeleteScript(script)}>
														Delete
													</AlertDialogAction>
												</AlertDialogFooter>
											</AlertDialogContent>
										</AlertDialog>
									</div>
								</div>
							</div>
						</CardContent>
					</Card>
				{/each}
			</div>
		</div>
	{/if}
</div>

<!-- Script Editor Dialog -->
{#if showEditor}
	<ScriptEditor
		script={selectedScript}
		onClose={() => {
			showEditor = false;
			selectedScript = null;
		}}
		onSaved={handleScriptSaved}
	/>
{/if}

<!-- Running Instances View -->
{#if showRunningInstances && selectedScript}
	<RunningInstancesView
		scriptId={selectedScript.id}
		onClose={() => {
			showRunningInstances = false;
			selectedScript = null;
		}}
	/>
{/if}

<!-- Script Runner Dialog -->
{#if showRunner && selectedScript}
	<ScriptRunner
		script={selectedScript}
		onClose={() => {
			showRunner = false;
			selectedScript = null;
		}}
		onRun={handleScriptRun}
	/>
{/if}

