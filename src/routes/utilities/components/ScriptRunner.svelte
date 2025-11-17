<!--
	Script Runner Component
	Runs custom scripts with parameter inputs and terminal integration
-->

<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Switch } from '$lib/components/ui/switch';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import { X, Play, Square } from '@lucide/svelte';
	import { CustomScriptService, type CustomScript, type ScriptParameter } from '$lib/domains/custom_scripts/services/customScriptService';
	import { TerminalService } from '$lib/domains/terminal/services/terminalService';
	import type { TerminalProcess, TerminalOutput } from '$lib/domains/terminal/types';
	import { useRunningScripts } from '$lib/domains/custom_scripts/hooks/useRunningScripts';
	import FilePicker from '$lib/components/ui/file-picker.svelte';

	interface Props {
		script: CustomScript;
		runningInstanceId?: string; // ID of this running instance (if viewing existing)
		onClose: () => void;
		onRun?: () => void;
		onStopped?: () => void;
	}

	let { script, runningInstanceId: providedInstanceId, onClose, onRun, onStopped }: Props = $props();

	const runningScripts = useRunningScripts();
	let runningInstanceId = $state<string | null>(providedInstanceId || null);

	// Parse parameters
	const parameters = $derived(CustomScriptService.parseParameters(script.parameters_json));

	// Parameter values
	let parameterValues = $state<Record<string, string>>({});
	let booleanValues = $state<Record<string, boolean>>({});
	let terminalProcess = $state<TerminalProcess | null>(null);
	let terminalOutput = $state<string>('');
	let isRunning = $state(false);
	let error = $state<string | null>(null);
	let outputContainer = $state<HTMLDivElement | null>(null);

	// If viewing an existing running instance, load its state and subscribe to output
	$effect(() => {
		if (providedInstanceId) {
			const instance = runningScripts.getById(providedInstanceId);
			if (instance) {
				terminalOutput = instance.output;
				isRunning = true;
				runningInstanceId = providedInstanceId;
				
				// Try to get the process
				TerminalService.getProcess(instance.processId).then((process) => {
					if (process) {
						terminalProcess = process;
					}
				}).catch(console.error);

				// Subscribe to output updates if not already subscribed
				if (!instance.outputUnsubscribe) {
					TerminalService.subscribeToOutput(
						instance.processId,
						(output: TerminalOutput) => {
							terminalOutput += output.content;
							runningScripts.appendOutput(instance.id, output.content);
						}
					).then((unsub) => {
						outputUnsubscribe = unsub;
						const updated = runningScripts.getById(instance.id);
						if (updated) {
							updated.outputUnsubscribe = unsub;
						}
					}).catch(console.error);
				}
			}
		}
	});

	// Initialize default values
	$effect(() => {
		const defaults: Record<string, string> = {};
		const boolDefaults: Record<string, boolean> = {};
		for (const param of parameters) {
			if (param.parameter_type === 'boolean') {
				boolDefaults[param.name] = param.default_value === 'true';
			} else {
				// Initialize all non-boolean parameters with default value or empty string
				defaults[param.name] = param.default_value || '';
			}
		}
		parameterValues = defaults;
		booleanValues = boolDefaults;
	});


	// Terminal output handler
	let outputUnsubscribe: (() => void) | null = null;

	// Sync output from running scripts manager when viewing existing instance
	$effect(() => {
		if (runningInstanceId && providedInstanceId) {
			const instance = runningScripts.getById(runningInstanceId);
			if (instance) {
				terminalOutput = instance.output;
			}
		}
	});

	async function handleRun() {
		// Validate required parameters
		for (const param of parameters) {
			if (param.required) {
				if (param.parameter_type === 'boolean') {
					// Boolean params are always valid
					continue;
				}
				if (!parameterValues[param.name]?.trim()) {
					error = `Parameter "${param.label}" is required`;
					return;
				}
			}
		}

		error = null;
		isRunning = true;
		terminalOutput = '';

		try {
			// Build the command with parameter values (include boolean values)
			const allValues = { ...parameterValues };
			for (const param of parameters) {
				if (param.parameter_type === 'boolean') {
					allValues[param.name] = booleanValues[param.name] ? 'true' : 'false';
				}
			}
			const command = CustomScriptService.buildCommand(
				script.command,
				parameters,
				allValues,
				script.requires_sudo
			);

			// Record script run
			await CustomScriptService.recordScriptRun(script.id);

			// Create terminal process
			const tabId = `script-${script.id}-${Date.now()}`;
			terminalProcess = await TerminalService.createProcess(tabId, {
				shell: 'bash',
				working_directory: '/home/tan',
			});

			// Register this running instance first (before subscribing to output)
			runningInstanceId = runningScripts.add(
				script,
				terminalProcess.id,
				tabId,
				handleStop
			);

			// Subscribe to output
			outputUnsubscribe = await TerminalService.subscribeToOutput(
				terminalProcess.id,
				(output: TerminalOutput) => {
					terminalOutput += output.content;
					// Update output in running scripts manager
					if (runningInstanceId) {
						runningScripts.appendOutput(runningInstanceId, output.content);
					}
				}
			);

			// Update the instance with output unsubscribe
			const instance = runningScripts.getById(runningInstanceId);
			if (instance) {
				instance.outputUnsubscribe = outputUnsubscribe;
			}

			// Start tracking the command
			TerminalService.startCommandTracking(command, tabId);

			// Execute the command (sudo/admin is already included in the command if requires_sudo is true)
			await TerminalService.sendInput(terminalProcess.id, `${command}\n`);

			onRun?.();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to run script';
			console.error('Failed to run script:', err);
			isRunning = false;
			if (runningInstanceId) {
				runningScripts.remove(runningInstanceId);
				runningInstanceId = null;
			}
		}
	}

	async function handleStop() {
		if (terminalProcess) {
			try {
				await TerminalService.killProcess(terminalProcess.id);
			} catch (err) {
				console.error('Failed to kill process:', err);
			}
		}
		if (outputUnsubscribe) {
			outputUnsubscribe();
			outputUnsubscribe = null;
		}
		isRunning = false;
		terminalProcess = null;
		
		// Remove from running scripts manager
		if (runningInstanceId) {
			runningScripts.remove(runningInstanceId);
			runningInstanceId = null;
		}
		
		onStopped?.();
	}

	// Handle process exit
	$effect(() => {
		if (terminalProcess) {
			// Listen for exit events
			const checkExit = setInterval(async () => {
				if (terminalProcess) {
					const process = await TerminalService.getProcess(terminalProcess.id);
					if (process && process.status === 'completed') {
						clearInterval(checkExit);
						isRunning = false;
						if (outputUnsubscribe) {
							outputUnsubscribe();
							outputUnsubscribe = null;
						}
						
						// Remove from running scripts manager
						if (runningInstanceId) {
							runningScripts.remove(runningInstanceId);
							runningInstanceId = null;
						}
						
						onStopped?.();
					}
				}
			}, 1000);

			return () => clearInterval(checkExit);
		}
	});

	onDestroy(() => {
		// Don't remove running instances from the manager when closing the dialog
		// The instances should persist even when the dialog is closed
		// They will only be removed when the script actually stops (via handleStop or process completion)
		
		// Only clean up local subscription if we created one
		// But don't remove from manager - the script is still running!
		if (outputUnsubscribe && providedInstanceId) {
			// This is viewing an existing instance - just clean up local subscription
			// Don't remove from manager or kill process
			outputUnsubscribe();
		}
		// If it's a new run (no providedInstanceId), the outputUnsubscribe is already
		// stored in the instance, so we don't need to clean it up here
		// The instance will persist in the manager even after this component is destroyed
	});

	function sendInput(input: string) {
		if (terminalProcess && isRunning) {
			TerminalService.sendInput(terminalProcess.id, input).catch(console.error);
		}
	}

	// Auto-focus input when script starts running
	$effect(() => {
		if (isRunning) {
			// Small delay to ensure the input is rendered
			setTimeout(() => {
				const input = document.getElementById('interactive-input') as HTMLInputElement;
				if (input) {
					input.focus();
				}
			}, 100);
		}
	});

	// Auto-scroll output to bottom when new content arrives
	$effect(() => {
		if (outputContainer && terminalOutput) {
			// Use requestAnimationFrame to ensure DOM is updated
			requestAnimationFrame(() => {
				if (outputContainer) {
					outputContainer.scrollTop = outputContainer.scrollHeight;
				}
			});
		}
	});
</script>

<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
	<div class="bg-background rounded-lg shadow-lg w-full max-w-6xl max-h-[90vh] overflow-hidden flex flex-col">
		<div class="p-6 border-b flex items-center justify-between">
			<div>
				<h2 class="text-2xl font-semibold flex items-center gap-2">
					{#if script.icon}
						<span>{script.icon}</span>
					{/if}
					{script.name}
				</h2>
				{#if script.description}
					<p class="text-sm text-muted-foreground mt-1">{script.description}</p>
				{/if}
			</div>
			<Button variant="ghost" size="sm" onclick={onClose}>
				<X class="h-4 w-4" />
			</Button>
		</div>

		<div class="flex-1 overflow-y-auto p-6 space-y-6">
			{#if error}
				<div class="p-4 border border-red-200 bg-red-50 rounded-md">
					<p class="text-sm text-red-600">{error}</p>
				</div>
			{/if}

			<!-- Parameters Section -->
			{#if parameters.length > 0}
				<Card>
					<CardHeader>
						<CardTitle>Parameters</CardTitle>
						<CardDescription>Configure script parameters</CardDescription>
					</CardHeader>
					<CardContent class="space-y-4">
						{#each parameters as param (param.name)}
							<div class="space-y-2">
								<Label>
									{param.label}
									{#if param.required}
										<span class="text-red-500">*</span>
									{/if}
								</Label>
								{#if param.description}
									<p class="text-sm text-muted-foreground">{param.description}</p>
								{/if}

								{#if param.parameter_type === 'file'}
									<FilePicker
										value={parameterValues[param.name] || ''}
										label=""
										description=""
										filters={param.file_filters ? [{ name: 'Files', extensions: param.file_filters }] : []}
										selectFolder={false}
										required={param.required}
										onChange={(path) => {
											parameterValues = { ...parameterValues, [param.name]: path };
										}}
									/>
								{:else if param.parameter_type === 'folder'}
									<FilePicker
										value={parameterValues[param.name] || ''}
										label=""
										description=""
										selectFolder={true}
										required={param.required}
										onChange={(path) => {
											parameterValues = { ...parameterValues, [param.name]: path };
										}}
									/>
								{:else if param.parameter_type === 'boolean'}
									{@const boolKey = param.name}
									<div class="flex items-center gap-2">
										<Switch
											checked={booleanValues[boolKey] ?? false}
											onCheckedChange={(checked) => {
												booleanValues = { ...booleanValues, [boolKey]: checked };
											}}
										/>
										<span class="text-sm text-muted-foreground">
											{booleanValues[boolKey] ? 'Enabled' : 'Disabled'}
										</span>
									</div>
								{:else if param.parameter_type === 'password'}
									<Input
										type="password"
										value={parameterValues[param.name] || ''}
										oninput={(e) => {
											parameterValues = { ...parameterValues, [param.name]: (e.target as HTMLInputElement).value };
										}}
										placeholder={param.default_value || `Enter ${param.label.toLowerCase()}`}
										required={param.required}
									/>
								{:else if param.parameter_type === 'number'}
									<Input
										type="number"
										value={parameterValues[param.name] || ''}
										oninput={(e) => {
											parameterValues = { ...parameterValues, [param.name]: (e.target as HTMLInputElement).value };
										}}
										placeholder={param.default_value || `Enter ${param.label.toLowerCase()}`}
										required={param.required}
									/>
								{:else}
									<Input
										type="text"
										value={parameterValues[param.name] || ''}
										oninput={(e) => {
											parameterValues = { ...parameterValues, [param.name]: (e.target as HTMLInputElement).value };
										}}
										placeholder={param.default_value || `Enter ${param.label.toLowerCase()}`}
										required={param.required}
									/>
								{/if}
							</div>
						{/each}
					</CardContent>
				</Card>
			{/if}

			<!-- Command Preview -->
			<Card>
				<CardHeader>
					<CardTitle>Command Preview</CardTitle>
				</CardHeader>
				<CardContent class="space-y-2">
					<pre class="bg-muted p-4 rounded-md text-sm font-mono overflow-x-auto">
{(() => {
							const allValues = { ...parameterValues };
							for (const param of parameters) {
								if (param.parameter_type === 'boolean') {
									allValues[param.name] = booleanValues[param.name] ? 'true' : 'false';
								}
							}
							return CustomScriptService.buildCommand(script.command, parameters, allValues, script.requires_sudo);
						})()}</pre
					>
					{#if script.requires_sudo}
						{@const isWindows = navigator.userAgent.includes('Windows')}
						{#if isWindows}
							<div class="p-3 border border-amber-200 bg-amber-50 dark:bg-amber-950 dark:border-amber-800 rounded-md">
								<p class="text-sm text-amber-800 dark:text-amber-200">
									<strong>Windows Admin Required:</strong> This command requires administrator privileges. 
									Make sure the terminal is running as administrator, or the command may fail.
								</p>
							</div>
						{:else}
							<div class="p-3 border border-blue-200 bg-blue-50 dark:bg-blue-950 dark:border-blue-800 rounded-md">
								<p class="text-sm text-blue-800 dark:text-blue-200">
									<strong>Note:</strong> This command will prompt for your password to run with sudo privileges.
								</p>
							</div>
						{/if}
					{/if}
				</CardContent>
			</Card>

			<!-- Terminal Output -->
			{#if isRunning || terminalOutput}
				<Card>
					<CardHeader>
						<div class="flex items-center justify-between">
							<CardTitle>Output</CardTitle>
							{#if isRunning}
								<Badge variant="outline">Interactive - Type below to send input</Badge>
							{/if}
						</div>
					</CardHeader>
					<CardContent class="space-y-4">
						<div 
							bind:this={outputContainer}
							class="bg-black text-green-400 p-4 rounded-md font-mono text-sm h-64 overflow-y-auto"
						>
							{terminalOutput || 'Waiting for output...'}
						</div>

						{#if isRunning}
							<div class="flex gap-2">
								<Input
									id="interactive-input"
									type="text"
									placeholder="Type input and press Enter (e.g., password for sudo)..."
									class="flex-1"
									autofocus
									onkeydown={(e) => {
										if (e.key === 'Enter') {
											const input = (e.target as HTMLInputElement).value + '\n';
											sendInput(input);
											(e.target as HTMLInputElement).value = '';
										}
									}}
								/>
							</div>
						{/if}
					</CardContent>
				</Card>
			{/if}
		</div>

		<div class="p-6 border-t flex justify-end gap-2">
			{#if isRunning}
				<Button variant="destructive" onclick={handleStop}>
					<Square class="h-4 w-4 mr-2" />
					Stop
				</Button>
			{:else}
				<Button variant="outline" onclick={onClose}>
					Close
				</Button>
				<Button onclick={handleRun}>
					<Play class="h-4 w-4 mr-2" />
					Run Script
				</Button>
			{/if}
		</div>
	</div>
</div>

