<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Textarea } from '$lib/components/ui/textarea';
	import { Label } from '$lib/components/ui/label';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import Select from '$lib/components/ui/select.svelte';
	import { TaskPriorityEnum } from '../types';
	import type { GeneratedTask } from '../services/aiTaskService';
	import Icon from '@iconify/svelte';

	interface Props {
		task: GeneratedTask;
		suggestedLabels?: string[];
		onUpdate?: (task: GeneratedTask) => void;
		onAdd?: (task: GeneratedTask) => void | Promise<void>;
		onAddAll?: () => void | Promise<void>;
		showAddButton?: boolean;
		hasSubtasks?: boolean;
		originalStatus?: string;
		isUpdateMode?: boolean;
	}

	let {
		task: initialTask,
		suggestedLabels = $bindable<string[]>([]),
		onUpdate,
		onAdd,
		onAddAll,
		showAddButton = true,
		hasSubtasks = false,
		originalStatus,
		isUpdateMode = false
	}: Props = $props();

	let task = $state<GeneratedTask>({ ...initialTask });
	let isAdding = $state(false);
	let isAddingAll = $state(false);

	// Sync with external updates
	$effect(() => {
		task = { ...initialTask };
	});

	function handleFieldChange() {
		onUpdate?.(task);
	}

	async function handleAdd() {
		if (onAdd) {
			isAdding = true;
			try {
				await onAdd(task);
			} finally {
				isAdding = false;
			}
		}
	}

	async function handleAddAll() {
		if (onAddAll) {
			isAddingAll = true;
			try {
				await onAddAll();
			} finally {
				isAddingAll = false;
			}
		}
	}

	function removeTag(tag: string) {
		suggestedLabels = suggestedLabels.filter(t => t !== tag);
	}

	function addTag(tag: string) {
		if (tag.trim() && !suggestedLabels.includes(tag.trim())) {
			suggestedLabels = [...suggestedLabels, tag.trim()];
		}
	}
</script>

<Card>
	<CardHeader>
		<div class="flex items-center justify-between">
			<div class="flex items-center gap-2">
				<CardTitle class="flex items-center gap-2">
					<Icon icon="lucide:target" class="h-5 w-5" />
					Main Task
				</CardTitle>
				{#if isUpdateMode && originalStatus}
					<Badge variant="outline" class="text-xs">
						Status: {originalStatus}
					</Badge>
				{/if}
			</div>
			<div class="flex items-center gap-2">
				{#if onAddAll && hasSubtasks}
					<Button
						variant="default"
						size="sm"
						onclick={handleAddAll}
						disabled={isAddingAll || !task.title.trim()}
						loading={isAddingAll}
					>
						<Icon icon={isUpdateMode ? "lucide:save-all" : "lucide:plus-circle"} class="h-4 w-4 mr-1" />
						{isUpdateMode ? 'Update All' : 'Add All'}
					</Button>
				{/if}
				{#if showAddButton && onAdd}
					<Button
						size="sm"
						onclick={handleAdd}
						disabled={isAdding || isAddingAll || !task.title.trim()}
						loading={isAdding}
					>
						<Icon icon={isUpdateMode ? "lucide:save" : "lucide:plus"} class="h-4 w-4 mr-1" />
						{isUpdateMode ? 'Update Task' : 'Add Task'}
					</Button>
				{/if}
			</div>
		</div>
	</CardHeader>
	<CardContent class="space-y-4">
		<div class="space-y-2">
			<Label for="task-title">Title</Label>
			<Input
				id="task-title"
				bind:value={task.title}
				placeholder="Task title..."
				oninput={handleFieldChange}
			/>
		</div>

		<div class="space-y-2">
			<Label for="task-description">Description</Label>
			<Textarea
				id="task-description"
				bind:value={task.description}
				placeholder="Task description..."
				rows={4}
				oninput={handleFieldChange}
			/>
		</div>

		<div class="grid grid-cols-1 md:grid-cols-3 gap-4">
			<div class="space-y-2">
				<Label>Priority</Label>
				<Select
					options={TaskPriorityEnum}
					defaultValue={task.priority}
					onSelect={(value) => {
						task.priority = value as string;
						handleFieldChange();
					}}
				/>
			</div>

			<div class="space-y-2">
				<Label for="task-type">Type</Label>
				<Input
					id="task-type"
					bind:value={task.type_}
					placeholder="Story, Bug, Feature..."
					oninput={handleFieldChange}
				/>
			</div>

			<div class="space-y-2">
				<Label for="task-estimate">Estimated Time (minutes)</Label>
				<Input
					id="task-estimate"
					type="number"
					bind:value={task.estimated_time}
					placeholder="120"
					oninput={handleFieldChange}
				/>
			</div>
		</div>

		<div class="space-y-2">
			<Label>Tags</Label>
			<div class="flex flex-wrap gap-2">
				{#each suggestedLabels as tag, index}
					<Badge variant="secondary" class="cursor-pointer">
						{tag}
						<Icon
							icon="lucide:x"
							class="h-3 w-3 ml-1"
							onclick={() => removeTag(tag)}
						/>
					</Badge>
				{/each}
				<Input
					placeholder="Add tag..."
					class="w-auto min-w-[120px]"
					onkeydown={(e) => {
						if (e.key === 'Enter' && e.currentTarget.value.trim()) {
							addTag(e.currentTarget.value.trim());
							e.currentTarget.value = '';
						}
					}}
				/>
			</div>
		</div>
	</CardContent>
</Card>

