<script lang="ts">
	import { Input } from '$lib/components/ui/input';
	import Select from '$lib/components/ui/select.svelte';
	import { Button } from '$lib/components/ui/button';
	import Icon from '@iconify/svelte';

	interface Props {
		searchQuery?: string;
		onSearchChange?: (query: string) => void;
		typeFilter?: string;
		onTypeChange?: (type: string) => void;
		onClear?: () => void;
	}

	let {
		searchQuery = $bindable(''),
		onSearchChange,
		typeFilter = $bindable(''),
		onTypeChange,
		onClear
	}: Props = $props();

	const typeOptions = [
		{ value: '', label: 'All types' },
		{ value: 'text', label: 'Text' },
		{ value: 'code', label: 'Code' },
		{ value: 'documentation', label: 'Documentation' },
		{ value: 'examples', label: 'Examples' },
		{ value: 'other', label: 'Other' }
	];
</script>

<div class="flex items-center gap-2">
	<Input
		placeholder="Search training data..."
		bind:value={searchQuery}
		class="flex-1"
		oninput={(e) => onSearchChange?.(e.currentTarget.value)}
	/>
	<Select
		options={typeOptions}
		value={typeFilter || undefined}
		onSelect={(value) => {
			typeFilter = value;
			onTypeChange?.(value);
		}}
		placeholder="All types"
		class="w-[180px]"
	/>
	{#if (searchQuery || typeFilter) && onClear}
		<Button variant="outline" size="sm" onclick={onClear}>
			<Icon icon="lucide:x" class="h-4 w-4 mr-2" />
			Clear
		</Button>
	{/if}
</div>

