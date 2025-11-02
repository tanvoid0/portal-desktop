<script lang="ts">
	import type { WorkflowResult } from '../types';
	import { CheckCircle, XCircle, Clock, FileText, Terminal } from 'lucide-svelte';

	export let result: WorkflowResult;

	function formatDuration(duration: number): string {
		if (duration < 60) {
			return `${duration.toFixed(1)}s`;
		} else {
			const minutes = Math.floor(duration / 60);
			const seconds = duration % 60;
			return `${minutes}m ${seconds.toFixed(1)}s`;
		}
	}
</script>

<div class="workflow-results">
	<div class="header">
		<div class="flex items-center space-x-2">
			{#if result.success}
				<CheckCircle class="w-5 h-5 text-green-600" />
			{:else}
				<XCircle class="w-5 h-5 text-red-600" />
			{/if}
			<h3 class="text-lg font-semibold text-gray-900 dark:text-white">
				Workflow Results
			</h3>
		</div>
		
		<div class="text-sm text-gray-600 dark:text-gray-400">
			Execution ID: {result.execution_id}
		</div>
	</div>

	<div class="content space-y-4">
		<!-- Summary -->
		<div class="summary">
			<div class="flex items-center space-x-4 text-sm">
				<div class="flex items-center space-x-1">
					<Clock class="w-4 h-4 text-gray-400" />
					<span class="text-gray-600 dark:text-gray-400">
						Duration: {formatDuration(result.results.duration)}
					</span>
				</div>
				
				<div class="flex items-center space-x-1">
					<Terminal class="w-4 h-4 text-gray-400" />
					<span class="text-gray-600 dark:text-gray-400">
						Commands: {result.results.commands_executed.length}
					</span>
				</div>
			</div>
		</div>

		<!-- Commands Executed -->
		{#if result.results.commands_executed.length > 0}
			<div class="commands">
				<h4 class="font-medium text-gray-900 dark:text-white mb-2">
					Commands Executed
				</h4>
				<div class="space-y-1">
					{#each result.results.commands_executed as command (command)}
						<div class="flex items-center space-x-2 p-2 bg-gray-100 dark:bg-gray-700 rounded text-sm font-mono">
							<Terminal class="w-4 h-4 text-gray-500" />
							<span class="text-gray-800 dark:text-gray-200">{command}</span>
						</div>
					{/each}
				</div>
			</div>
		{/if}

		<!-- Output -->
		{#if result.results.output}
			<div class="output">
				<h4 class="font-medium text-gray-900 dark:text-white mb-2">
					Output
				</h4>
				<div class="p-3 bg-gray-900 text-gray-100 rounded text-sm font-mono whitespace-pre-wrap">
					{result.results.output}
				</div>
			</div>
		{/if}

		<!-- Files Created -->
		{#if result.results.files_created.length > 0}
			<div class="files">
				<h4 class="font-medium text-gray-900 dark:text-white mb-2">
					Files Created
				</h4>
				<div class="space-y-1">
					{#each result.results.files_created as file (file)}
						<div class="flex items-center space-x-2 p-2 bg-green-50 dark:bg-green-900/20 rounded text-sm">
							<FileText class="w-4 h-4 text-green-600" />
							<span class="text-green-800 dark:text-green-200">{file}</span>
						</div>
					{/each}
				</div>
			</div>
		{/if}

		<!-- Errors -->
		{#if result.errors.length > 0}
			<div class="errors">
				<h4 class="font-medium text-red-900 dark:text-red-200 mb-2">
					Errors
				</h4>
				<div class="space-y-1">
					{#each result.errors as error (error)}
						<div class="p-2 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded text-sm text-red-800 dark:text-red-200">
							{error}
						</div>
					{/each}
				</div>
			</div>
		{/if}

		<!-- Suggestions -->
		{#if result.suggestions.length > 0}
			<div class="suggestions">
				<h4 class="font-medium text-blue-900 dark:text-blue-200 mb-2">
					Suggestions
				</h4>
				<div class="space-y-1">
					{#each result.suggestions as suggestion (suggestion)}
						<div class="p-2 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded text-sm text-blue-800 dark:text-blue-200">
							{suggestion}
						</div>
					{/each}
				</div>
			</div>
		{/if}
	</div>
</div>

<style>
	.workflow-results {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}
	
	.header {
		border-bottom: 1px solid rgb(229 231 235);
		padding-bottom: 0.75rem;
		@media (prefers-color-scheme: dark) {
			border-bottom-color: rgb(55 65 81);
		}
	}
	
	.content {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}
</style>
