<script lang="ts">
	import { onMount } from 'svelte';
	import { automationStore } from '../stores/automationStore';
	import type { AvailableWorkflow, WorkflowResult } from '../types';
	import { Play, Loader2, CheckCircle, XCircle } from '@lucide/svelte';

	export let project: { 
		id: string; 
		name: string; 
		path: string; 
		framework?: string; 
		package_manager?: string;
		build_command?: string;
		start_command?: string;
		test_command?: string;
		output_directory?: string;
		dev_port?: number;
		prod_port?: number;
	};
	export let onWorkflowComplete: (result: WorkflowResult) => void = () => {};

	let suggestedWorkflows: AvailableWorkflow[] = [];
	let selectedWorkflow: AvailableWorkflow | null = null;
	let isTriggering = false;
	let lastResult: WorkflowResult | null = null;

	$: if (project) {
		loadSuggestedWorkflows();
	}

	async function loadSuggestedWorkflows() {
		if (!project) return;
		
		await automationStore.getSuggestedWorkflows(
			project.framework,
			project.package_manager
		);
		
		suggestedWorkflows = $automationStore.suggestedWorkflows;
	}

	async function triggerWorkflow(workflow: AvailableWorkflow) {
		if (!project || isTriggering) return;
		
		selectedWorkflow = workflow;
		isTriggering = true;
		lastResult = null;

		try {
			const result = await automationStore.triggerWorkflow(workflow.id, {
				id: project.id,
				name: project.name,
				path: project.path,
				framework: project.framework,
				package_manager: project.package_manager,
				build_command: project.build_command,
				start_command: project.start_command,
				test_command: project.test_command,
				output_directory: project.output_directory,
				dev_port: project.dev_port,
				prod_port: project.prod_port
			});
			
			lastResult = result;
			onWorkflowComplete(result);
		} catch (error) {
			console.error('Failed to trigger workflow:', error);
		} finally {
			isTriggering = false;
		}
	}

	onMount(() => {
		automationStore.checkHealth();
	});
</script>

<div class="workflow-trigger">
	<div class="header">
		<h3 class="text-lg font-semibold text-gray-900 dark:text-white">
			Automate Project
		</h3>
		{#if !$automationStore.isN8nHealthy}
			<div class="text-sm text-red-600 dark:text-red-400">
				n8n is not running. Start it with: npm run n8n:start
			</div>
		{/if}
	</div>

	{#if $automationStore.loading}
		<div class="flex items-center justify-center p-4">
			<Loader2 class="w-5 h-5 animate-spin text-blue-600" />
			<span class="ml-2 text-sm text-gray-600 dark:text-gray-400">
				Loading workflows...
			</span>
		</div>
	{:else if suggestedWorkflows.length === 0}
		<div class="text-center p-4 text-gray-500 dark:text-gray-400">
			No automation workflows available for this project type.
		</div>
	{:else}
		<div class="space-y-2">
			{#each suggestedWorkflows as workflow (workflow.id)}
				<button
					class="w-full flex items-center justify-between p-3 rounded-lg border border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors"
					onclick={() => triggerWorkflow(workflow)}
					disabled={isTriggering}
				>
					<div class="flex-1 text-left">
						<div class="font-medium text-gray-900 dark:text-white">
							{workflow.name}
						</div>
						{#if workflow.description}
							<div class="text-sm text-gray-600 dark:text-gray-400">
								{workflow.description}
							</div>
						{/if}
					</div>
					
					<div class="flex items-center space-x-2">
						{#if isTriggering && selectedWorkflow?.id === workflow.id}
							<Loader2 class="w-4 h-4 animate-spin text-blue-600" />
						{:else if lastResult && selectedWorkflow?.id === workflow.id}
							{#if lastResult.success}
								<CheckCircle class="w-4 h-4 text-green-600" />
							{:else}
								<XCircle class="w-4 h-4 text-red-600" />
							{/if}
						{:else}
							<Play class="w-4 h-4 text-gray-400" />
						{/if}
					</div>
				</button>
			{/each}
		</div>
	{/if}

	{#if $automationStore.error}
		<div class="mt-4 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
			<div class="text-sm text-red-800 dark:text-red-200">
				{$automationStore.error}
			</div>
		</div>
	{/if}
</div>

<style>
	.workflow-trigger {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}
</style>
