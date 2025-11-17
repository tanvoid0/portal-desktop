<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import SubtaskPreviewCard from './SubtaskPreviewCard.svelte';
	import type { GeneratedSubtask } from '../services/aiTaskService';
	import Icon from '@iconify/svelte';

	interface Props {
		subtasks: GeneratedSubtask[];
		onUpdate?: (index: number, subtask: GeneratedSubtask) => void;
		onAdd?: (index: number, subtask: GeneratedSubtask) => void | Promise<void>;
		onRemove?: (index: number) => void;
		showAddButtons?: boolean;
	}

	let {
		subtasks = $bindable<GeneratedSubtask[]>([]),
		onUpdate,
		onAdd,
		onRemove,
		showAddButtons = true
	}: Props = $props();

	function handleUpdate(index: number, subtask: GeneratedSubtask) {
		subtasks[index] = subtask;
		subtasks = [...subtasks]; // Trigger reactivity
		onUpdate?.(index, subtask);
	}

	async function handleAdd(index: number, subtask: GeneratedSubtask) {
		if (onAdd) {
			await onAdd(index, subtask);
		}
	}

	function handleRemove(index: number) {
		subtasks = subtasks.filter((_, i) => i !== index);
		onRemove?.(index);
	}

	function addEmptySubtask() {
		subtasks = [
			...subtasks,
			{
				title: '',
				description: '',
				estimated_time: null,
				dependencies: [],
				order: subtasks.length + 1,
			},
		];
	}
</script>

<Card>
	<CardHeader>
		<CardTitle class="flex items-center justify-between">
			<div class="flex items-center gap-2">
				<Icon icon="lucide:list-checks" class="h-5 w-5" />
				Subtasks ({subtasks.length})
			</div>
			<Button variant="outline" size="sm" onclick={addEmptySubtask}>
				<Icon icon="lucide:plus" class="h-4 w-4 mr-1" />
				Add Subtask
			</Button>
		</CardTitle>
	</CardHeader>
	<CardContent class="space-y-4">
		{#each subtasks as subtask, index}
			<SubtaskPreviewCard
				{subtask}
				{index}
				onUpdate={(s) => handleUpdate(index, s)}
				onAdd={showAddButtons && onAdd ? () => handleAdd(index, subtask) : undefined}
				onRemove={onRemove ? () => handleRemove(index) : undefined}
				showAddButton={showAddButtons}
			/>
		{:else}
			<p class="text-sm text-muted-foreground text-center py-4">No subtasks</p>
		{/each}
	</CardContent>
</Card>

