<!-- Logs Display Component - Shows formatted log entries -->
<script lang="ts">
	import LogEntry from './LogEntry.svelte';
	import { Button } from '@/lib/components/ui/button';
	import type { K8sLog } from '../../types/k8s';
	import { filterLogsBySeverity } from '../../utils/logFilterUtils';
	
	interface Props {
		logs: K8sLog[];
		searchQuery?: string;
		severityFilter?: string;
		viewMode?: 'detailed' | 'compact' | 'raw';
		onFilterBySeverity?: (severity: string) => void;
	}
	
	let {
		logs = [],
		searchQuery = '',
		severityFilter = '',
		viewMode = 'detailed',
		onFilterBySeverity
	}: Props = $props();
	
	// Filter logs
	const filteredLogs = $derived.by(() => {
		if (!logs || logs.length === 0) {
			return [];
		}
		
		let filtered = [...logs];
		
		// Apply severity filter
		if (severityFilter) {
			filtered = filterLogsBySeverity(filtered, severityFilter);
		}
		
		// Apply search filter
		if (searchQuery) {
			const searchLower = searchQuery.toLowerCase();
			filtered = filtered.filter((log: K8sLog) => {
				const messageMatch = (log.message || '').toLowerCase().includes(searchLower);
				const levelMatch = (log.level || '').toLowerCase().includes(searchLower);
				const podMatch = (log.pod || '').toLowerCase().includes(searchLower);
				return messageMatch || levelMatch || podMatch;
			});
		}
		
		return filtered;
	});
	
	let logsContainerRef: HTMLElement | null = null;
	
	function scrollToBottom() {
		if (logsContainerRef) {
			logsContainerRef.scrollTo({ top: logsContainerRef.scrollHeight, behavior: 'smooth' });
		}
	}
	
	function scrollToTop() {
		if (logsContainerRef) {
			logsContainerRef.scrollTo({ top: 0, behavior: 'smooth' });
		}
	}
</script>

<div class="space-y-2">
	<!-- Controls -->
	<div class="flex items-center justify-between">
		<div class="text-sm text-muted-foreground">
			Showing {filteredLogs.length} of {logs.length} logs
		</div>
		<div class="flex items-center gap-2">
			<Button
				variant="outline"
				size="sm"
				onclick={scrollToTop}
				class="text-xs"
				title="Scroll to top"
			>
				↑ Top
			</Button>
			<Button
				variant="outline"
				size="sm"
				onclick={scrollToBottom}
				class="text-xs"
				title="Scroll to bottom"
			>
				↓ Bottom
			</Button>
		</div>
	</div>
	
	<!-- Logs Container -->
	<div
		bind:this={logsContainerRef}
		class="max-h-[600px] overflow-y-auto border rounded-lg p-4 bg-muted/30"
	>
		{#if filteredLogs.length === 0}
			<div class="text-center py-8 text-muted-foreground">
				<p>No logs found</p>
				{#if searchQuery || severityFilter}
					<p class="text-xs mt-2">Try adjusting your filters</p>
				{/if}
			</div>
		{:else}
			<div class="space-y-1">
				{#each filteredLogs as log, index (log.timestamp + log.pod + log.message + log.container + String(index))}
					<LogEntry
						{log}
						{viewMode}
						onFilterBySeverity={onFilterBySeverity}
					/>
				{/each}
			</div>
		{/if}
	</div>
</div>

