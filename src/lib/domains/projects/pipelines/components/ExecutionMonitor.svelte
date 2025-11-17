<!--
	Execution Monitor - Real-time execution monitoring
-->
<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { Button } from '@/lib/components/ui/button';
	import { Card, CardContent, CardHeader, CardTitle } from '@/lib/components/ui/card';
	import type { PipelineExecution, StepExecution } from '../types';
	import { executionService, executionStore } from '../index';

	interface Props {
		executionId: string;
		onClose?: () => void;
	}

	let { executionId, onClose }: Props = $props();

	let execution = $state<PipelineExecution | null>(null);
	let unsubscribe: (() => void) | null = $state(null);

	onMount(async () => {
		// Load initial execution
		execution = await executionService.getExecution(executionId);
		if (execution) {
			executionStore.updateExecution(execution);
		}

		// Subscribe to updates
		unsubscribe = executionService.subscribeToExecution(executionId, (updated) => {
			execution = updated;
			executionStore.updateExecution(updated);
		});

		// Poll for updates if not already completed
		const interval = setInterval(async () => {
			if (execution && ['pending', 'running'].includes(execution.status)) {
				const updated = await executionService.getExecution(executionId);
				if (updated) {
					execution = updated;
					executionStore.updateExecution(updated);
				}
			} else {
				clearInterval(interval);
			}
		}, 1000);

		onDestroy(() => {
			unsubscribe?.();
			clearInterval(interval);
		});
	});

	function getStatusColor(status: string): string {
		switch (status) {
			case 'success':
				return 'text-green-600';
			case 'failed':
				return 'text-red-600';
			case 'running':
				return 'text-blue-600';
			case 'pending':
				return 'text-gray-600';
			case 'cancelled':
				return 'text-yellow-600';
			default:
				return 'text-gray-600';
		}
	}

	async function handleCancel() {
		if (execution && ['pending', 'running'].includes(execution.status)) {
			await executionService.cancelExecution(executionId);
		}
	}
</script>

<Card class="w-full max-w-4xl">
	<CardHeader>
		<div class="flex items-center justify-between">
			<CardTitle>Pipeline Execution</CardTitle>
			<div class="flex gap-2">
				{#if execution && ['pending', 'running'].includes(execution.status)}
					<Button variant="destructive" size="sm" onclick={handleCancel}>
						Cancel
					</Button>
				{/if}
				{#if onClose}
					<Button variant="outline" size="sm" onclick={onClose}>Close</Button>
				{/if}
			</div>
		</div>
	</CardHeader>
	<CardContent>
		{#if !execution}
			<p class="text-center text-muted-foreground py-8">Loading execution...</p>
		{:else}
			<!-- Execution Status -->
			<div class="mb-4">
				<div class="flex items-center gap-2">
					<span class="font-medium">Status:</span>
					<span class={getStatusColor(execution.status)}>{execution.status}</span>
				</div>
				<div class="text-sm text-muted-foreground mt-1">
					Started: {new Date(execution.startedAt).toLocaleString()}
					{#if execution.finishedAt}
						| Finished: {new Date(execution.finishedAt).toLocaleString()}
					{/if}
				</div>
			</div>

			<!-- Step Executions -->
			<div class="space-y-2">
				<h3 class="font-medium">Steps</h3>
				{#each execution.stepExecutions as stepExecution (stepExecution.id)}
					<div class="p-3 border rounded-md">
						<div class="flex items-center justify-between mb-2">
							<div>
								<p class="font-medium">{stepExecution.stepName}</p>
								<p class="text-sm text-muted-foreground">
									Status: <span class={getStatusColor(stepExecution.status)}>
										{stepExecution.status}
									</span>
								</p>
							</div>
							{#if stepExecution.duration}
								<p class="text-sm text-muted-foreground">
									{Math.round(stepExecution.duration / 1000)}s
								</p>
							{/if}
						</div>
						{#if stepExecution.output}
							<details class="mt-2">
								<summary class="cursor-pointer text-sm text-muted-foreground">
									View Output
								</summary>
								<pre
									class="mt-2 p-2 bg-muted rounded text-xs overflow-auto max-h-40"
								>{stepExecution.output}</pre>
							</details>
						{/if}
						{#if stepExecution.error}
							<div class="mt-2 p-2 bg-red-50 dark:bg-red-900/20 rounded text-sm text-red-800 dark:text-red-200">
								{stepExecution.error}
							</div>
						{/if}
					</div>
				{/each}
			</div>

			{#if execution.error}
				<div class="mt-4 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded">
					<p class="text-sm font-medium text-red-800 dark:text-red-200">Error:</p>
					<p class="text-sm text-red-700 dark:text-red-300">{execution.error}</p>
				</div>
			{/if}
		{/if}
	</CardContent>
</Card>

