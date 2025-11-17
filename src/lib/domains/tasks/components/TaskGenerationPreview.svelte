<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import { Separator } from '$lib/components/ui/separator';
	import { Input } from '$lib/components/ui/input';
	import { Textarea } from '$lib/components/ui/textarea';
	import { Label } from '$lib/components/ui/label';
	import Select from '$lib/components/ui/select.svelte';
	import { TASK_PRIORITY_OPTIONS, TaskPriorityEnum } from '../types';
	import type { GeneratedTaskStructure, GeneratedTask, GeneratedSubtask } from '../services/aiTaskService';
	import Icon from '@iconify/svelte';

	interface Props {
		generated: GeneratedTaskStructure;
		originalText: string;
		onApprove: (data: GeneratedTaskStructure) => void;
		onCancel: () => void;
		onRegenerate?: (previousResult: GeneratedTaskStructure) => void;
	}

	let { generated, originalText, onApprove, onCancel, onRegenerate }: Props = $props();

	// Editable state
	let mainTask = $state<GeneratedTask>({ ...generated.main_task });
	let subtasks = $state<GeneratedSubtask[]>([...generated.subtasks]);
	let suggestedProject = $state(generated.suggested_project);
	let suggestedLabels = $state<string[]>([...generated.suggested_labels]);

	// UI state
	let showOriginalText = $state(false);

	function handleApprove() {
		const approved: GeneratedTaskStructure = {
			main_task: mainTask,
			subtasks: subtasks,
			suggested_project: suggestedProject,
			suggested_labels: suggestedLabels,
			confidence: generated.confidence,
			model_used: generated.model_used,
		};
		onApprove(approved);
	}

	function removeSubtask(index: number) {
		subtasks = subtasks.filter((_, i) => i !== index);
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

<div class="space-y-6 max-w-4xl mx-auto">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div>
			<h2 class="text-2xl font-bold">Generated Tasks Preview</h2>
			<p class="text-sm text-muted-foreground">
				Review and edit the generated tasks before creating them
			</p>
		</div>
		<Badge variant="outline" class="text-xs">
			Confidence: {Math.round(generated.confidence * 100)}% | Model: {generated.model_used}
		</Badge>
	</div>

	<!-- Main Task -->
	<Card>
		<CardHeader>
			<CardTitle class="flex items-center gap-2">
				<Icon icon="lucide:target" class="h-5 w-5" />
				Main Task
			</CardTitle>
		</CardHeader>
		<CardContent class="space-y-4">
			<div class="space-y-2">
				<Label for="main-title">Title</Label>
				<Input id="main-title" bind:value={mainTask.title} placeholder="Task title..." />
			</div>

			<div class="space-y-2">
				<Label for="main-description">Description</Label>
				<Textarea
					id="main-description"
					bind:value={mainTask.description}
					placeholder="Task description..."
					rows={4}
				/>
			</div>

			<div class="grid grid-cols-1 md:grid-cols-3 gap-4">
				<div class="space-y-2">
					<Label>Priority</Label>
					<Select
						options={TaskPriorityEnum}
						defaultValue={mainTask.priority}
						onSelect={(value) => (mainTask.priority = value as string)}
					/>
				</div>

				<div class="space-y-2">
					<Label for="main-type">Type</Label>
					<Input id="main-type" bind:value={mainTask.type_} placeholder="Story, Bug, Feature..." />
				</div>

				<div class="space-y-2">
					<Label for="main-estimate">Estimated Time (minutes)</Label>
					<Input
						id="main-estimate"
						type="number"
						bind:value={mainTask.estimated_time}
						placeholder="120"
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
								onclick={() => {
									suggestedLabels = suggestedLabels.filter((_, i) => i !== index);
								}}
							/>
						</Badge>
					{/each}
					<Input
						placeholder="Add tag..."
						class="w-auto min-w-[120px]"
						onkeydown={(e) => {
							if (e.key === 'Enter' && e.currentTarget.value.trim()) {
								const newTag = e.currentTarget.value.trim();
								if (!suggestedLabels.includes(newTag)) {
									suggestedLabels = [...suggestedLabels, newTag];
								}
								e.currentTarget.value = '';
							}
						}}
					/>
				</div>
			</div>
		</CardContent>
	</Card>

	<!-- Subtasks -->
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
				<div class="border rounded-lg p-4 space-y-3">
					<div class="flex items-center justify-between">
						<Badge variant="outline">#{index + 1}</Badge>
						<Button
							variant="ghost"
							size="sm"
							onclick={() => removeSubtask(index)}
							class="text-destructive"
						>
							<Icon icon="lucide:trash-2" class="h-4 w-4" />
						</Button>
					</div>

					<div class="space-y-2">
						<Label>Title</Label>
						<Input bind:value={subtask.title} placeholder="Subtask title..." />
					</div>

					<div class="space-y-2">
						<Label>Description</Label>
						<Textarea bind:value={subtask.description} placeholder="Subtask description..." rows={2} />
					</div>

					<div class="grid grid-cols-2 gap-4">
						<div class="space-y-2">
							<Label>Estimated Time (minutes)</Label>
							<Input
								type="number"
								bind:value={subtask.estimated_time}
								placeholder="30"
							/>
						</div>
						<div class="space-y-2">
							<Label>Order</Label>
							<Input type="number" bind:value={subtask.order} placeholder="1" />
						</div>
					</div>
				</div>
			{:else}
				<p class="text-sm text-muted-foreground text-center py-4">No subtasks</p>
			{/each}
		</CardContent>
	</Card>

	<!-- Suggestions -->
	{#if suggestedProject}
		<Card>
			<CardHeader>
				<CardTitle class="flex items-center gap-2">
					<Icon icon="lucide:folder" class="h-5 w-5" />
					Suggested Project
				</CardTitle>
				<CardDescription>
					Confidence: {Math.round(suggestedProject.confidence * 100)}%
				</CardDescription>
			</CardHeader>
			<CardContent>
				<p class="font-medium">{suggestedProject.name}</p>
				<p class="text-sm text-muted-foreground mt-1">{suggestedProject.reason}</p>
			</CardContent>
		</Card>
	{/if}

	<!-- Original Text Reference -->
	<Card>
		<CardHeader>
			<CardTitle class="flex items-center justify-between">
				<div class="flex items-center gap-2">
					<Icon icon="lucide:file-text" class="h-5 w-5" />
					Original Text Reference
				</div>
				<Button variant="ghost" size="sm" onclick={() => (showOriginalText = !showOriginalText)}>
					<Icon
						icon={showOriginalText ? 'lucide:chevron-up' : 'lucide:chevron-down'}
						class="h-4 w-4"
					/>
				</Button>
			</CardTitle>
		</CardHeader>
		{#if showOriginalText}
			<CardContent>
				<div class="rounded-lg border bg-muted/50 p-4">
					<pre class="text-sm whitespace-pre-wrap font-mono">{originalText}</pre>
				</div>
			</CardContent>
		{/if}
	</Card>

	<!-- Actions -->
	<div class="flex justify-end gap-3 pt-4 border-t">
		{#if onRegenerate}
			<Button
				variant="outline"
				onclick={() => {
					onRegenerate(generated);
				}}
			>
				<Icon icon="lucide:refresh-cw" class="h-4 w-4 mr-2" />
				Regenerate
			</Button>
		{/if}
		<Button variant="outline" onclick={onCancel}>
			Cancel
		</Button>
		<Button onclick={handleApprove}>
			<Icon icon="lucide:check" class="h-4 w-4 mr-2" />
			Approve & Create Tasks
		</Button>
	</div>
</div>

