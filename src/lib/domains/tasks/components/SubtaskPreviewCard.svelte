<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Textarea } from '$lib/components/ui/textarea';
	import { Label } from '$lib/components/ui/label';
	import { Badge } from '$lib/components/ui/badge';
	import type { GeneratedSubtask } from '../services/aiTaskService';
	import Icon from '@iconify/svelte';

	interface Props {
		subtask: GeneratedSubtask;
		index: number;
		onUpdate?: (subtask: GeneratedSubtask) => void;
		onAdd?: (subtask: GeneratedSubtask) => void | Promise<void>;
		onRemove?: () => void;
		showAddButton?: boolean;
	}

	let {
		subtask: initialSubtask,
		index,
		onUpdate,
		onAdd,
		onRemove,
		showAddButton = true
	}: Props = $props();

	let subtask = $state<GeneratedSubtask>({ ...initialSubtask });
	let isAdding = $state(false);

	// Sync with external updates
	$effect(() => {
		subtask = { ...initialSubtask };
	});

	function handleFieldChange() {
		onUpdate?.(subtask);
	}

	async function handleAdd() {
		if (onAdd) {
			isAdding = true;
			try {
				await onAdd(subtask);
			} finally {
				isAdding = false;
			}
		}
	}
</script>

<div class="border rounded-lg p-4 space-y-3">
	<div class="flex items-center justify-between">
		<Badge variant="outline">#{index + 1}</Badge>
		<div class="flex items-center gap-2">
			{#if showAddButton && onAdd}
				<Button
					variant="outline"
					size="sm"
					onclick={handleAdd}
					disabled={isAdding || !subtask.title.trim()}
					
				>
					<Icon icon="lucide:plus" class="h-4 w-4 mr-1" />
					Add
				</Button>
			{/if}
			{#if onRemove}
				<Button
					variant="ghost"
					size="sm"
					onclick={onRemove}
					class="text-destructive"
				>
					<Icon icon="lucide:trash-2" class="h-4 w-4" />
				</Button>
			{/if}
		</div>
	</div>

	<div class="space-y-2">
		<Label>Title</Label>
		<Input
			bind:value={subtask.title}
			placeholder="Subtask title..."
			oninput={handleFieldChange}
		/>
	</div>

	<div class="space-y-2">
		<Label>Description</Label>
		<Textarea
			bind:value={subtask.description}
			placeholder="Subtask description..."
			rows={2}
			oninput={handleFieldChange}
		/>
	</div>

	<div class="grid grid-cols-2 gap-4">
		<div class="space-y-2">
			<Label>Estimated Time (minutes)</Label>
			<Input
				type="number"
				bind:value={subtask.estimated_time}
				placeholder="30"
				oninput={handleFieldChange}
			/>
		</div>
		<div class="space-y-2">
			<Label>Order</Label>
			<Input
				type="number"
				bind:value={subtask.order}
				placeholder="1"
				oninput={handleFieldChange}
			/>
		</div>
	</div>
</div>

