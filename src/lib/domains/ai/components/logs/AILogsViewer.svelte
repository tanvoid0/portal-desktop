<script lang="ts">
	import { ScrollArea } from '$lib/components/ui/scroll-area/index.js';
	import LogEntry from './LogEntry.svelte';
	import LogFilters from './LogFilters.svelte';
	import { Button } from '$lib/components/ui/button';
	import Icon from '@iconify/svelte';
	import type { AILog, LogFilters } from '../../types/index.js';

	interface Props {
		logs: AILog[];
		filters?: LogFilters;
		onFiltersChange?: (filters: LogFilters) => void;
		onExport?: () => void;
	}

	let {
		logs = $bindable<AILog[]>([]),
		filters = $bindable<LogFilters>({}),
		onFiltersChange,
		onExport
	}: Props = $props();

	function handleClearFilters() {
		filters = {};
		onFiltersChange?.(filters);
	}
</script>

<div class="space-y-4">
	<div class="flex items-center justify-between">
		<LogFilters bind:filters onFiltersChange={onFiltersChange} onClear={handleClearFilters} />
		{#if onExport}
			<Button variant="outline" onclick={onExport}>
				<Icon icon="lucide:download" class="h-4 w-4 mr-2" />
				Export
			</Button>
		{/if}
	</div>
	<ScrollArea class="h-[calc(100vh-300px)]">
		<div class="space-y-2">
			{#if logs.length === 0}
				<div class="text-center text-muted-foreground py-8">
					<Icon icon="lucide:file-text" class="h-12 w-12 mx-auto mb-2 opacity-50" />
					<p class="text-sm">No logs found</p>
				</div>
			{:else}
				{#each logs as log}
					<LogEntry {log} />
				{/each}
			{/if}
		</div>
	</ScrollArea>
</div>

