<script lang="ts">
	import { onMount } from 'svelte';
	import AILogsViewer from '$lib/domains/ai/components/logs/AILogsViewer.svelte';
	import { toastActions } from '$lib/domains/shared/stores/toastStore';
	import { aiLogService } from '$lib/domains/ai';
	import type { AILog, LogFilters } from '$lib/domains/ai/types/index.js';

	let logs = $state<AILog[]>([]);
	let filters = $state<LogFilters>({});
	let isLoading = $state(false);

	onMount(async () => {
		await loadLogs();
	});

	async function loadLogs() {
		isLoading = true;
		try {
			if (filters.search_query) {
				logs = await aiLogService.searchLogs(filters.search_query, filters);
			} else {
				logs = await aiLogService.getLogs(filters);
			}
		} catch (error) {
			console.error('Failed to load logs:', error);
			toastActions.error('Failed to load logs', error);
		} finally {
			isLoading = false;
		}
	}

	async function handleFiltersChange(newFilters: LogFilters) {
		filters = newFilters;
		await loadLogs();
	}

	async function handleExport() {
		try {
			const filePath = await aiLogService.exportLogs(filters);
			toastActions.success('Logs exported', `Exported to ${filePath}`);
		} catch (error) {
			console.error('Failed to export logs:', error);
			toastActions.error('Failed to export logs', error);
		}
	}

	$effect(() => {
		loadLogs();
	});
</script>

<div class="h-full w-full p-6">
	<div class="space-y-4">
		<div>
			<h1 class="text-2xl font-bold">AI Logs</h1>
			<p class="text-muted-foreground">View all AI interactions, requests, responses, and errors</p>
		</div>
		{#if isLoading}
			<p class="text-muted-foreground">Loading...</p>
		{:else}
			<AILogsViewer
				bind:logs
				bind:filters
				onFiltersChange={handleFiltersChange}
				onExport={handleExport}
			/>
		{/if}
	</div>
</div>

