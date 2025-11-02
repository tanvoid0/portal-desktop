<script lang="ts">
	import { onDestroy } from 'svelte';
	import { automationStore } from '../stores/automationStore';
	import type { WorkflowExecution } from '../types';
	import { 
		Play, 
		Pause, 
		CheckCircle, 
		XCircle, 
		Clock, 
		Loader2 
	} from 'lucide-svelte';

	export let executionId: string;
	export let onStatusChange: (execution: WorkflowExecution) => void = () => {};

	let execution: WorkflowExecution | null = null;
	let pollingInterval: ReturnType<typeof setInterval> | null = null;

	$: if (executionId) {
		startPolling();
	}

	async function startPolling() {
		if (!executionId) return;
		
		// Initial check
		await checkStatus();
		
		// Poll every 2 seconds
		pollingInterval = setInterval(checkStatus, 2000) as ReturnType<typeof setInterval>;
	}

	async function checkStatus() {
		if (!executionId) return;
		
		try {
			const exec = await automationStore.checkWorkflowStatus(executionId);
			execution = exec;
			onStatusChange(exec);
			
			// Stop polling if execution is finished
			if (exec.status === 'success' || exec.status === 'error' || exec.status === 'canceled') {
				stopPolling();
			}
		} catch (error) {
			console.error('Failed to check workflow status:', error);
		}
	}

	function stopPolling() {
		if (pollingInterval) {
			clearInterval(pollingInterval);
			pollingInterval = null;
		}
	}

	onDestroy(() => {
		stopPolling();
	});

	function getStatusIcon(status: string) {
		switch (status) {
			case 'running':
				return Loader2;
			case 'success':
				return CheckCircle;
			case 'error':
				return XCircle;
			case 'waiting':
				return Clock;
			case 'canceled':
				return Pause;
			default:
				return Play;
		}
	}

	function getStatusColor(status: string) {
		switch (status) {
			case 'running':
				return 'text-blue-600';
			case 'success':
				return 'text-green-600';
			case 'error':
				return 'text-red-600';
			case 'waiting':
				return 'text-yellow-600';
			case 'canceled':
				return 'text-gray-600';
			default:
				return 'text-gray-400';
		}
	}

	function getStatusText(status: string) {
		switch (status) {
			case 'running':
				return 'Running';
			case 'success':
				return 'Completed';
			case 'error':
				return 'Failed';
			case 'waiting':
				return 'Waiting';
			case 'canceled':
				return 'Canceled';
			default:
				return 'Unknown';
		}
	}
</script>

{#if execution}
	<div class="workflow-status">
		<div class="flex items-center space-x-3">
			{#if execution.status === 'running'}
				<Loader2 class="w-5 h-5 animate-spin text-blue-600" />
			{:else}
				<svelte:component 
					this={getStatusIcon(execution.status)} 
					class="w-5 h-5 {getStatusColor(execution.status)}" 
				/>
			{/if}
			
			<div class="flex-1">
				<div class="font-medium text-gray-900 dark:text-white">
					{getStatusText(execution.status)}
				</div>
				<div class="text-sm text-gray-600 dark:text-gray-400">
					Execution ID: {execution.id}
				</div>
			</div>
		</div>

		{#if execution.started_at}
			<div class="mt-2 text-xs text-gray-500 dark:text-gray-400">
				Started: {new Date(execution.started_at).toLocaleString()}
			</div>
		{/if}

		{#if execution.finished_at}
			<div class="mt-1 text-xs text-gray-500 dark:text-gray-400">
				Finished: {new Date(execution.finished_at).toLocaleString()}
			</div>
		{/if}
	</div>
{/if}

<style>
	.workflow-status {
		padding: 0.75rem;
		background-color: rgb(249 250 251);
		border-radius: 0.5rem;
		border: 1px solid rgb(229 231 235);
		@media (prefers-color-scheme: dark) {
			background-color: rgb(31 41 55);
			border-color: rgb(55 65 81);
		}
	}
</style>
