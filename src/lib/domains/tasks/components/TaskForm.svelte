<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Label } from '$lib/components/ui/label';
	import { Textarea } from '$lib/components/ui/textarea';
	import { Badge } from '$lib/components/ui/badge';
	import { Separator } from '$lib/components/ui/separator';
	import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '$lib/components/ui/collapsible';
	import Select from '$lib/components/ui/select.svelte';
	import { toastActions } from '$lib/domains/shared/stores/toastStore';
	import { taskActions, tasks } from '../stores/taskStore';
	import type { Task, CreateTaskRequest, UpdateTaskRequest, TaskStatus, TaskPriority } from '../types';
	import { TASK_STATUS_OPTIONS, TASK_PRIORITY_OPTIONS, TASK_TYPE_OPTIONS, TaskStatusEnum, TaskPriorityEnum, RecurringPatternEnum } from '../types';
	import { ResourceType } from '$lib/domains/shared/types/resourceType';
	import { documentActions, documents } from '$lib/domains/documents';
	import DocumentLinkSelector from '$lib/domains/documents/components/DocumentLinkSelector.svelte';
	import type { Document } from '$lib/domains/documents/types';
	import Icon from '@iconify/svelte';
	import { goto } from '$app/navigation';

	interface Props {
		task?: Task;
		onSave?: (task: Task) => void;
		onCancel?: () => void;
		parentId?: string;
	}

	let { task, onSave, onCancel, parentId }: Props = $props();

	// Form state
	let title = $state(task?.title || '');
	let description = $state(task?.description || '');
	let status = $state<TaskStatus>(task?.status || 'pending');
	let priority = $state<TaskPriority>(task?.priority || 'medium');
	let type = $state(task?.type || '');
	let dueDate = $state(task?.dueDate ? task.dueDate.toISOString().split('T')[0] : '');
	
	// New advanced fields
	let estimatedTime = $state(task?.estimatedTime || 0);
	let actualTime = $state(task?.actualTime || 0);
	let tags = $state<string[]>(task?.tags || []);
	let assignee = $state(task?.assignee || '');
	let recurringPattern = $state(task?.recurring?.pattern || '');
	let recurringInterval = $state(task?.recurring?.interval || 1);
	let recurringEndDate = $state(task?.recurring?.endDate ? task.recurring.endDate.toISOString().split('T')[0] : '');
	let blockedBy = $state<string[]>(task?.blockedBy || []);
	let blocks = $state<string[]>(task?.blocks || []);
	
	// Document linking
	let linkedDocumentId = $state<number | null>(
		task?.resourceType === ResourceType.DOCUMENT && task?.resourceId
			? parseInt(task.resourceId)
			: null
	);

	// Load documents when component mounts
	$effect(() => {
		documentActions.loadDocuments();
	});

	// Using enums for recurring patterns
	const RECURRING_PATTERNS = Object.values(RecurringPatternEnum);
	
	// UI state
	let newTag = $state('');
	let showAdvanced = $state(false);
	let isSubmitting = $state(false);
	let errors = $state<Record<string, string>>({});
	

	// Helper functions
	function addTag() {
		if (newTag.trim() && !tags.includes(newTag.trim())) {
			tags = [...tags, newTag.trim()];
			newTag = '';
		}
	}

	function removeTag(tagToRemove: string) {
		tags = tags.filter(tag => tag !== tagToRemove);
	}

	function handleTagKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter') {
			event.preventDefault();
			addTag();
		}
	}

	// Validation
	function validateForm(): boolean {
		errors = {};
		
		if (!title.trim()) {
			errors.title = 'Title is required';
		}
		
		if (title.length > 100) {
			errors.title = 'Title must be less than 100 characters';
		}
		
		if (description && description.length > 500) {
			errors.description = 'Description must be less than 500 characters';
		}

		if (estimatedTime < 0) {
			errors.estimatedTime = 'Estimated time cannot be negative';
		}

		if (actualTime < 0) {
			errors.actualTime = 'Actual time cannot be negative';
		}

		return Object.keys(errors).length === 0;
	}

	// Form submission
	async function handleSubmit() {
		if (!validateForm()) {
			toastActions.error('Please fix the errors below');
			return;
		}

		try {
			isSubmitting = true;

			const taskData: CreateTaskRequest | UpdateTaskRequest = {
				title: title.trim(),
				description: description.trim() || undefined,
				status,
				priority,
				type: type.trim() || undefined,
				...(dueDate ? { dueDate: new Date(dueDate) } : {}),
				...(parentId ? { parentId } : {}),
				// Document linking
				resourceId: linkedDocumentId ? linkedDocumentId.toString() : undefined,
				resourceType: linkedDocumentId ? ResourceType.DOCUMENT : undefined,
				// New advanced fields
				estimatedTime: estimatedTime || undefined,
				actualTime: actualTime || undefined,
				tags: tags.length > 0 ? tags : undefined,
				assignee: assignee.trim() || undefined,
				recurring: recurringPattern ? {
					pattern: recurringPattern as any,
					interval: recurringInterval,
					endDate: recurringEndDate ? new Date(recurringEndDate) : undefined
				} : undefined,
				blockedBy: blockedBy.length > 0 ? blockedBy : undefined,
				blocks: blocks.length > 0 ? blocks : undefined
			};

			let savedTask: Task;
			if (task) {
				savedTask = await taskActions.updateTask(task.id, taskData as UpdateTaskRequest);
				toastActions.success('Task updated successfully');
			} else {
				savedTask = await taskActions.createTask(taskData as CreateTaskRequest);
				toastActions.success('Task created successfully');
			}

			onSave?.(savedTask);
		} catch (error) {
			toastActions.error(
				'Failed to save task',
				error instanceof Error ? error.message : 'An unexpected error occurred'
			);
		} finally {
			isSubmitting = false;
		}
	}

	// Keyboard shortcuts
	function handleKeydown(event: KeyboardEvent) {
		if (event.ctrlKey || event.metaKey) {
			if (event.key === 'Enter') {
				event.preventDefault();
				handleSubmit();
			} else if (event.key === 'Escape') {
				event.preventDefault();
				onCancel?.();
			}
		}
	}

</script>

	<Card class="max-w-2xl mx-auto">
		<CardHeader>
			<div class="flex items-center justify-between">
				<div>
					<CardTitle>{task ? 'Edit Task' : 'Create New Task'}</CardTitle>
					<CardDescription>
						{task ? 'Update the task details below' : 'Fill in the details to create a new task'}
					</CardDescription>
				</div>
				{#if !task}
					<Button
						variant="outline"
						size="sm"
						onclick={() => goto('/tasks/generate')}
					>
						<Icon icon="lucide:sparkles" class="h-4 w-4 mr-2" />
						Generate Tasks with AI
					</Button>
				{/if}
			</div>
		</CardHeader>
	<CardContent class="space-y-6" onkeydown={handleKeydown}>
		<!-- Title -->
		<div class="space-y-2">
			<Label for="title">Title *</Label>
			<Input
				id="title"
				bind:value={title}
				placeholder="Enter task title..."
				class={errors.title ? 'border-destructive' : ''}
			/>
			{#if errors.title}
				<p class="text-sm text-destructive">{errors.title}</p>
			{/if}
		</div>

		<!-- Description -->
		<div class="space-y-2">
			<Label for="description">Description</Label>
			<Textarea
				id="description"
				bind:value={description}
				placeholder="Enter task description..."
				rows={3}
				class={errors.description ? 'border-destructive' : ''}
			/>
			{#if errors.description}
				<p class="text-sm text-destructive">{errors.description}</p>
			{/if}
		</div>

		<!-- Status and Priority -->
		<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
			<div class="space-y-2">
				<Label for="status">Status</Label>
				<Select 
					options={TaskStatusEnum}
					defaultValue={status}
					placeholder="Select status..."
					onSelect={(value) => status = value as TaskStatus}
				/>
			</div>

			<div class="space-y-2">
				<Label for="priority">Priority</Label>
				<Select 
					options={TaskPriorityEnum}
					defaultValue={priority}
					placeholder="Select priority..."
					onSelect={(value) => priority = value as TaskPriority}
				/>
			</div>
		</div>

		<!-- Type and Due Date -->
		<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
			<div class="space-y-2">
				<Label for="type">Type</Label>
				<Select 
					options={TASK_TYPE_OPTIONS}
					defaultValue={type}
					placeholder="Select type..."
					onSelect={(value) => type = value}
				/>
			</div>

			<div class="space-y-2">
				<Label for="dueDate">Due Date</Label>
				<Input
					id="dueDate"
					type="date"
					bind:value={dueDate}
				/>
			</div>
		</div>

		<!-- Time Estimates -->
		<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
			<div class="space-y-2">
				<Label for="estimatedTime">Estimated Time (minutes)</Label>
				<Input
					id="estimatedTime"
					type="number"
					min="0"
					bind:value={estimatedTime}
					placeholder="e.g., 120 for 2 hours"
					class={errors.estimatedTime ? 'border-destructive' : ''}
				/>
				{#if errors.estimatedTime}
					<p class="text-sm text-destructive">{errors.estimatedTime}</p>
				{/if}
			</div>

			<div class="space-y-2">
				<Label for="actualTime">Actual Time (minutes)</Label>
				<Input
					id="actualTime"
					type="number"
					min="0"
					bind:value={actualTime}
					placeholder="e.g., 90 for 1.5 hours"
					class={errors.actualTime ? 'border-destructive' : ''}
				/>
				{#if errors.actualTime}
					<p class="text-sm text-destructive">{errors.actualTime}</p>
				{/if}
			</div>
		</div>

		<!-- Tags -->
		<div class="space-y-2">
			<Label>Tags</Label>
			<div class="flex gap-2">
				<Input
					placeholder="Add a tag..."
					bind:value={newTag}
					onkeydown={handleTagKeydown}
				/>
				<Button type="button" variant="outline" onclick={addTag}>
					<Icon icon="mdi:plus" class="w-4 h-4" />
				</Button>
			</div>
			{#if tags.length > 0}
				<div class="flex flex-wrap gap-2">
					{#each tags as tag}
						<Badge variant="secondary" class="flex items-center gap-1">
							{tag}
							<Button
								type="button"
								variant="ghost"
								size="sm"
								onclick={() => removeTag(tag)}
								class="h-4 w-4 p-0 hover:bg-destructive hover:text-destructive-foreground"
							>
								<Icon icon="mdi:close" class="w-3 h-3" />
							</Button>
						</Badge>
					{/each}
				</div>
			{/if}
		</div>

		<!-- Assignee -->
		<div class="space-y-2">
			<Label for="assignee">Assignee</Label>
			<Input
				id="assignee"
				bind:value={assignee}
				placeholder="Enter assignee name or email..."
			/>
		</div>

		<!-- Document Link -->
		<div class="space-y-2">
			<DocumentLinkSelector
				selectedDocumentId={linkedDocumentId}
				onSelect={(doc) => {
					linkedDocumentId = doc?.id || null;
				}}
				onCreateNew={() => {
					// Create new document and link it
					goto(`/documents/create?taskId=${task?.id || ''}&taskTitle=${encodeURIComponent(title || '')}`);
				}}
			/>
		</div>

		<!-- Advanced Settings -->
		<Collapsible bind:open={showAdvanced}>
			<CollapsibleTrigger>
				<Button variant="outline" class="w-full">
					<Icon icon="mdi:settings" class="w-4 h-4 mr-2" />
					Advanced Settings
					<Icon icon="mdi:chevron-down" class="w-4 h-4 ml-auto" />
				</Button>
			</CollapsibleTrigger>
			<CollapsibleContent class="space-y-4 mt-4">
				<Separator />
				
				<!-- Recurring Task Settings -->
				<div class="space-y-4">
					<h4 class="text-sm font-medium">Recurring Task</h4>
					<div class="grid grid-cols-1 md:grid-cols-3 gap-4">
						<div class="space-y-2">
							<Label for="recurringPattern">Pattern</Label>
							<Select 
								options={['', ...RECURRING_PATTERNS]}
								defaultValue={recurringPattern}
								placeholder="Select pattern..."
								onSelect={(value) => recurringPattern = value}
							/>
						</div>
						<div class="space-y-2">
							<Label for="recurringInterval">Interval</Label>
							<Input
								id="recurringInterval"
								type="number"
								min="1"
								bind:value={recurringInterval}
								placeholder="1"
							/>
						</div>
						<div class="space-y-2">
							<Label for="recurringEndDate">End Date</Label>
							<Input
								id="recurringEndDate"
								type="date"
								bind:value={recurringEndDate}
							/>
						</div>
					</div>
				</div>

				<!-- Dependencies -->
				<div class="space-y-4">
					<h4 class="text-sm font-medium">Dependencies</h4>
					<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
						<div class="space-y-2">
							<Label>Blocked By (Task IDs)</Label>
							<Textarea
								placeholder="Enter task IDs separated by commas..."
								rows={3}
								value={blockedBy.join(', ')}
								oninput={(e) => {
									const target = e.target as HTMLTextAreaElement;
									blockedBy = target.value.split(',').map((id: string) => id.trim()).filter((id: string) => id);
								}}
							/>
						</div>
						<div class="space-y-2">
							<Label>Blocks (Task IDs)</Label>
							<Textarea
								placeholder="Enter task IDs separated by commas..."
								rows={3}
								value={blocks.join(', ')}
								oninput={(e) => {
									const target = e.target as HTMLTextAreaElement;
									blocks = target.value.split(',').map((id: string) => id.trim()).filter((id: string) => id);
								}}
							/>
						</div>
					</div>
				</div>
			</CollapsibleContent>
		</Collapsible>

		<!-- Actions -->
		<div class="flex justify-end gap-3 pt-4 border-t">
			<Button
				variant="outline"
				onclick={onCancel}
				disabled={isSubmitting}
			>
				Cancel
			</Button>
			<Button
				onclick={handleSubmit}
				disabled={isSubmitting}
				
			>
				{task ? 'Update Task' : 'Create Task'}
			</Button>
		</div>

		<!-- Keyboard shortcuts hint -->
		<div class="text-xs text-muted-foreground">
			<kbd class="px-1.5 py-0.5 bg-muted rounded text-xs">Ctrl+Enter</kbd> to save,
			<kbd class="px-1.5 py-0.5 bg-muted rounded text-xs ml-1">Ctrl+Escape</kbd> to cancel
		</div>
	</CardContent>
</Card>