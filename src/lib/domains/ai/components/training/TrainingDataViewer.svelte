<script lang="ts">
	import TrainingDataTable from './TrainingDataTable.svelte';
	import TrainingDataFilters from './TrainingDataFilters.svelte';
	import * as Dialog from '$lib/components/ui/dialog';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { ScrollArea } from '$lib/components/ui/scroll-area/index.js';
	import { Button } from '$lib/components/ui/button';
	import type { TrainingData } from '../../types/index.js';

	interface Props {
		data: TrainingData[];
		onView?: (data: TrainingData) => void;
		onEdit?: (data: TrainingData) => void;
		onDelete?: (data: TrainingData) => void;
	}

	let {
		data = $bindable<TrainingData[]>([]),
		onView,
		onEdit,
		onDelete
	}: Props = $props();

	let searchQuery = $state('');
	let typeFilter = $state('');
	let selectedData = $state<TrainingData | null>(null);
	let showViewDialog = $state(false);

	let filteredData = $derived(
		data.filter((item) => {
			const matchesSearch = item.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
				item.content.toLowerCase().includes(searchQuery.toLowerCase());
			const matchesType = !typeFilter || item.type === typeFilter;
			return matchesSearch && matchesType;
		})
	);

	function handleView(data: TrainingData) {
		selectedData = data;
		showViewDialog = true;
		if (onView) onView(data);
	}

	function handleClearFilters() {
		searchQuery = '';
		typeFilter = '';
	}
</script>

<div class="space-y-4">
	<TrainingDataFilters
		bind:searchQuery
		bind:typeFilter
		onClear={handleClearFilters}
	/>
	<TrainingDataTable
		data={filteredData}
		onView={handleView}
		onEdit={onEdit}
		onDelete={onDelete}
	/>
</div>

<Dialog.Root bind:open={showViewDialog}>
	<Dialog.Content class="max-w-3xl max-h-[90vh]">
		<Dialog.Header>
			<Dialog.Title>{selectedData?.name || 'Training Data'}</Dialog.Title>
		</Dialog.Header>
		{#if selectedData}
			<ScrollArea class="max-h-[70vh]">
				<div class="space-y-4">
					<div>
						<p class="text-sm font-medium mb-1">Type</p>
						<p class="text-sm text-muted-foreground">{selectedData.type}</p>
					</div>
					<div>
						<p class="text-sm font-medium mb-1">Content</p>
						<Card>
							<CardContent class="p-4">
								<pre class="text-sm whitespace-pre-wrap">{selectedData.content}</pre>
							</CardContent>
						</Card>
					</div>
					{#if Object.keys(selectedData.metadata).length > 0}
						<div>
							<p class="text-sm font-medium mb-1">Metadata</p>
							<Card>
								<CardContent class="p-4">
									<pre class="text-sm">{JSON.stringify(selectedData.metadata, null, 2)}</pre>
								</CardContent>
							</Card>
						</div>
					{/if}
				</div>
			</ScrollArea>
		{/if}
		<Dialog.Footer>
			<Button variant="outline" onclick={() => (showViewDialog = false)}>
				Close
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>

