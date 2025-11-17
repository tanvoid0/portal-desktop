<script lang="ts">
	import { Input } from '$lib/components/ui/input';
	import Select from '$lib/components/ui/select.svelte';
	import { Button } from '$lib/components/ui/button';
	import Icon from '@iconify/svelte';
	import type { ProviderType, LogFilters } from '../../types/index.js';

	interface Props {
		filters?: LogFilters;
		onFiltersChange?: (filters: LogFilters) => void;
		onClear?: () => void;
	}

	let {
		filters = $bindable<LogFilters>({}),
		onFiltersChange,
		onClear
	}: Props = $props();

	const providerOptions = [
		{ value: '', label: 'All providers' },
		{ value: 'Ollama', label: 'Ollama' },
		{ value: 'Gemini', label: 'Gemini' }
	];
	const logTypeOptions = [
		{ value: '', label: 'All types' },
		{ value: 'request', label: 'Request' },
		{ value: 'response', label: 'Response' },
		{ value: 'error', label: 'Error' }
	];
</script>

<div class="flex flex-wrap items-center gap-2">
	<Input
		placeholder="Search logs..."
		value={filters.search_query || ''}
		class="flex-1 min-w-[200px]"
		oninput={(e) => {
			filters = { ...filters, search_query: e.currentTarget.value || undefined };
			onFiltersChange?.(filters);
		}}
	/>
	<Select
		options={providerOptions}
		value={filters.provider || undefined}
		onSelect={(value) => {
			filters = { ...filters, provider: value ? (value as ProviderType) : undefined };
			onFiltersChange?.(filters);
		}}
		placeholder="All providers"
		class="w-[180px]"
	/>
	<Select
		options={logTypeOptions}
		value={filters.log_type || undefined}
		onSelect={(value) => {
			filters = { ...filters, log_type: value ? (value as 'request' | 'response' | 'error') : undefined };
			onFiltersChange?.(filters);
		}}
		placeholder="All types"
		class="w-[180px]"
	/>
	{#if (filters.search_query || filters.provider || filters.log_type) && onClear}
		<Button variant="outline" size="sm" onclick={onClear}>
			<Icon icon="lucide:x" class="h-4 w-4 mr-2" />
			Clear
		</Button>
	{/if}
</div>

