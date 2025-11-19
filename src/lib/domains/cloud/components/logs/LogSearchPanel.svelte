<!-- Log Search Panel Component -->
<script lang="ts">
	import { Card, CardContent, CardHeader, CardTitle } from '@/lib/components/ui/card';
	import { Input } from '@/lib/components/ui/input';
	import { Button } from '@/lib/components/ui/button';
	import { Label } from '@/lib/components/ui/label';
	import { Badge } from '@/lib/components/ui/badge';
	import Select from '@/lib/components/ui/select.svelte';
	import { Search, X, Filter } from '@lucide/svelte';
	
	interface Props {
		searchQuery?: string;
		selectedContainer?: string;
		selectedSeverity?: string;
		tailLines?: number;
		containers?: string[];
		severities?: string[];
		onSearchChange?: (query: string) => void;
		onContainerChange?: (container: string) => void;
		onSeverityChange?: (severity: string) => void;
		onTailLinesChange?: (lines: number) => void;
		onClearFilters?: () => void;
		className?: string;
	}
	
	let {
		searchQuery = $bindable(''),
		selectedContainer = $bindable(''),
		selectedSeverity = $bindable(''),
		tailLines = $bindable(1000),
		containers = [],
		severities = ['DEBUG', 'INFO', 'WARN', 'ERROR', 'FATAL'],
		onSearchChange,
		onContainerChange,
		onSeverityChange,
		onTailLinesChange,
		onClearFilters,
		className = ''
	}: Props = $props();
	
	const hasActiveFilters = $derived(
		!!searchQuery || !!selectedContainer || !!selectedSeverity || tailLines !== 1000
	);
	
	function handleSearchInput(e: Event) {
		const value = (e.target as HTMLInputElement).value;
		searchQuery = value;
		if (onSearchChange) onSearchChange(value);
	}
	
	function handleContainerChange(value: string) {
		selectedContainer = value;
		if (onContainerChange) onContainerChange(value);
	}
	
	function handleSeverityChange(value: string) {
		selectedSeverity = value;
		if (onSeverityChange) onSeverityChange(value);
	}
	
	function handleTailLinesChange(value: string | { value: string; label: string }) {
		const valueStr = typeof value === 'string' ? value : value.value;
		const lines = parseInt(valueStr) || 1000;
		tailLines = lines;
		if (onTailLinesChange) onTailLinesChange(lines);
	}
	
	function handleClear() {
		searchQuery = '';
		selectedContainer = '';
		selectedSeverity = '';
		tailLines = 1000;
		if (onClearFilters) onClearFilters();
	}
	
	const tailOptions = [
		{ value: '100', label: 'Last 100 lines' },
		{ value: '500', label: 'Last 500 lines' },
		{ value: '1000', label: 'Last 1000 lines' },
		{ value: '5000', label: 'Last 5000 lines' },
		{ value: '10000', label: 'Last 10000 lines' }
	];
</script>

<Card class={className}>
	<CardHeader>
		<div class="flex items-center justify-between">
			<CardTitle class="flex items-center gap-2">
				<Filter class="h-4 w-4" />
				Log Search & Filters
			</CardTitle>
			{#if hasActiveFilters}
				<Button variant="ghost" size="sm" onclick={handleClear}>
					<X class="h-4 w-4 mr-1" />
					Clear
				</Button>
			{/if}
		</div>
	</CardHeader>
	<CardContent class="space-y-4">
		<!-- Search Input -->
		<div class="space-y-2">
			<Label for="log-search">Search Logs</Label>
			<div class="relative">
				<Search class="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
				<Input
					id="log-search"
					type="text"
					placeholder="Search in log messages (supports regex)..."
					value={searchQuery}
					oninput={handleSearchInput}
					class="pl-10"
				/>
			</div>
		</div>
		
		<!-- Filters Grid -->
		<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
			<!-- Container Filter -->
			{#if containers.length > 0}
				<div class="space-y-2">
					<Label for="container-filter">Container</Label>
					<Select
						options={[
							{ value: '', label: 'All Containers' },
							...containers.map(c => ({ value: c, label: c }))
						]}
						bind:value={selectedContainer}
						onSelect={handleContainerChange}
						placeholder="All Containers"
					/>
				</div>
			{/if}
			
			<!-- Severity Filter -->
			<div class="space-y-2">
				<Label for="severity-filter">Severity</Label>
				<Select
					options={[
						{ value: '', label: 'All Severities' },
						...severities.map(s => ({ value: s, label: s }))
					]}
					bind:value={selectedSeverity}
					onSelect={handleSeverityChange}
					placeholder="All Severities"
				/>
			</div>
			
			<!-- Tail Lines Filter -->
			<div class="space-y-2">
				<Label for="tail-lines">Lines to Show</Label>
				<Select
					options={tailOptions}
					value={String(tailLines)}
					onSelect={handleTailLinesChange}
					placeholder="Select lines"
				/>
			</div>
		</div>
		
		<!-- Active Filters Badges -->
		{#if hasActiveFilters}
			<div class="flex flex-wrap gap-2 pt-2 border-t">
				<span class="text-sm font-medium self-center">Active Filters:</span>
				{#if searchQuery}
					<Badge variant="secondary" class="gap-1">
						Search: "{searchQuery}"
						<Button
							variant="ghost"
							size="sm"
							onclick={() => { searchQuery = ''; }}
							class="ml-1 h-4 w-4 p-0 hover:bg-destructive/20 rounded-full"
						>
							<X class="h-3 w-3" />
						</Button>
					</Badge>
				{/if}
				{#if selectedContainer}
					<Badge variant="secondary" class="gap-1">
						Container: {selectedContainer}
						<Button
							variant="ghost"
							size="sm"
							onclick={() => { selectedContainer = ''; }}
							class="ml-1 h-4 w-4 p-0 hover:bg-destructive/20 rounded-full"
						>
							<X class="h-3 w-3" />
						</Button>
					</Badge>
				{/if}
				{#if selectedSeverity}
					<Badge variant="secondary" class="gap-1">
						Severity: {selectedSeverity}
						<Button
							variant="ghost"
							size="sm"
							onclick={() => { selectedSeverity = ''; }}
							class="ml-1 h-4 w-4 p-0 hover:bg-destructive/20 rounded-full"
						>
							<X class="h-3 w-3" />
						</Button>
					</Badge>
				{/if}
				{#if tailLines !== 1000}
					<Badge variant="secondary" class="gap-1">
						Lines: {tailLines}
						<Button
							variant="ghost"
							size="sm"
							onclick={() => { tailLines = 1000; }}
							class="ml-1 h-4 w-4 p-0 hover:bg-destructive/20 rounded-full"
						>
							<X class="h-3 w-3" />
						</Button>
					</Badge>
				{/if}
			</div>
		{/if}
	</CardContent>
</Card>

