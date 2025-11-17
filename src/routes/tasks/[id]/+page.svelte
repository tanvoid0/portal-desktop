<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { Button } from '@/lib/components/ui/button';
	import { Card, CardContent, CardHeader, CardTitle } from '@/lib/components/ui/card';
	import { Badge } from '@/lib/components/ui/badge';
	import { Separator } from '@/lib/components/ui/separator';
	import { Textarea } from '@/lib/components/ui/textarea';
	import { Input } from '@/lib/components/ui/input';
	import { Label } from '@/lib/components/ui/label';
	import Select from '@/lib/components/ui/select.svelte';
	import Icon from '@iconify/svelte';
	import MarkdownView from '@/lib/components/ui/markdown-view.svelte';
	import { taskActions, tasks } from '@/lib/domains/tasks/stores/taskStore';
	import { toastActions } from '@/lib/domains/shared/stores/toastStore';
	import { documentActions, documents } from '@/lib/domains/documents';
	import { ResourceType } from '@/lib/domains/shared/types/resourceType';
	import LoadingSpinner from '@/lib/components/ui/loading-spinner.svelte';
	import type { Task, TaskStatus, TaskPriority, UpdateTaskRequest } from '@/lib/domains/tasks/types';
	import { TASK_STATUS_OPTIONS, TASK_PRIORITY_OPTIONS, TASK_TYPE_OPTIONS } from '@/lib/domains/tasks/types';

	// Reactive task that updates when store changes
	let task = $derived($tasks.find((t: Task) => t.id === taskId) || null);
	let isLoading = $state(true);
	let error = $state<string | null>(null);
	let isEditing = $state(false);
	let isSaving = $state(false);

	// Edit form state
	let editTitle = $state('');
	let editDescription = $state('');
	let editStatus = $state<TaskStatus>('pending');
	let editPriority = $state<TaskPriority>('medium');
	let editType = $state('');
	let editDueDate = $state('');

	// Subtask management
	let showAddSubtask = $state(false);
	let newSubtaskTitle = $state('');
	let newSubtaskDescription = $state('');
	let newSubtaskPriority = $state<TaskPriority>('medium');
	let newSubtaskType = $state('');
	let isAddingSubtask = $state(false);
	let editingSubtaskId = $state<string | null>(null);
	let editSubtaskTitle = $state('');
	let editSubtaskDescription = $state('');
	let editSubtaskPriority = $state<TaskPriority>('medium');
	let editSubtaskType = $state('');
	let editSubtaskStatus = $state<TaskStatus>('pending');

	const taskId = $page.params.id;

	// Load tasks when component mounts or taskId changes
	async function loadTaskData() {
		try {
			isLoading = true;
			error = null;
			await taskActions.loadTasks();
			
			// Check if task exists after loading
			if (!task) {
				error = 'Task not found';
				toastActions.error('Task not found', 'The requested task could not be found');
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load task';
			toastActions.error('Failed to load task', error);
		} finally {
			isLoading = false;
		}
	}

	onMount(() => {
		loadTaskData();
		documentActions.loadDocuments();
	});

	// Watch for taskId changes and reload data
	$effect(() => {
		if (taskId) {
			loadTaskData();
		}
	});

	// Reactive effect to update edit form when task changes
	$effect(() => {
		if (task) {
			editTitle = task.title;
			editDescription = task.description || '';
			editStatus = task.status;
			editPriority = task.priority;
			editType = task.type || '';
			editDueDate = task.dueDate ? task.dueDate.toISOString().split('T')[0] : '';
		}
	});

	async function handleSave() {
		if (!task) return;

		try {
			isSaving = true;
			const updateData: UpdateTaskRequest = {
				title: editTitle.trim(),
				description: editDescription.trim() || undefined,
				status: editStatus,
				priority: editPriority,
				type: editType.trim() || undefined,
				dueDate: editDueDate ? new Date(editDueDate) : undefined
			};

			await taskActions.updateTask(task.id, updateData);
			toastActions.success('Task updated successfully');
			isEditing = false;
		} catch (err) {
			toastActions.error(
				'Failed to update task',
				err instanceof Error ? err.message : 'An unexpected error occurred'
			);
		} finally {
			isSaving = false;
		}
	}

	function handleEdit() {
		isEditing = true;
	}

	function handleCancel() {
		if (task) {
			editTitle = task.title;
			editDescription = task.description || '';
			editStatus = task.status;
			editPriority = task.priority;
			editType = task.type || '';
			editDueDate = task.dueDate ? task.dueDate.toISOString().split('T')[0] : '';
		}
		isEditing = false;
	}

	// Subtask management functions
	async function handleAddSubtask() {
		if (!task || !newSubtaskTitle.trim()) return;

		isAddingSubtask = true;
		try {
			await taskActions.createTask({
				title: newSubtaskTitle,
				description: newSubtaskDescription,
				status: 'pending',
				priority: newSubtaskPriority,
				type: newSubtaskType,
				parentId: task.id
			});

			// Reset form
			newSubtaskTitle = '';
			newSubtaskDescription = '';
			newSubtaskPriority = 'medium';
			newSubtaskType = '';
			showAddSubtask = false;

			toastActions.success('Subtask created', 'Subtask has been created successfully');
		} catch (err) {
			toastActions.error('Failed to create subtask', err instanceof Error ? err.message : 'Unknown error');
		} finally {
			isAddingSubtask = false;
		}
	}

	function startEditSubtask(subtask: Task) {
		editingSubtaskId = subtask.id;
		editSubtaskTitle = subtask.title;
		editSubtaskDescription = subtask.description || '';
		editSubtaskPriority = subtask.priority;
		editSubtaskType = subtask.type || '';
		editSubtaskStatus = subtask.status;
	}

	async function handleSaveSubtask() {
		if (!editingSubtaskId) return;

		try {
			await taskActions.updateTask(editingSubtaskId, {
				title: editSubtaskTitle,
				description: editSubtaskDescription,
				priority: editSubtaskPriority,
				type: editSubtaskType,
				status: editSubtaskStatus
			});

			editingSubtaskId = null;
			toastActions.success('Subtask updated', 'Subtask has been updated successfully');
		} catch (err) {
			toastActions.error('Failed to update subtask', err instanceof Error ? err.message : 'Unknown error');
		}
	}

	function cancelEditSubtask() {
		editingSubtaskId = null;
		editSubtaskTitle = '';
		editSubtaskDescription = '';
		editSubtaskPriority = 'medium';
		editSubtaskType = '';
		editSubtaskStatus = 'pending';
	}

	async function handleDeleteSubtask(subtaskId: string) {
		if (confirm('Are you sure you want to delete this subtask?')) {
			try {
				await taskActions.deleteTask(subtaskId);
				toastActions.success('Subtask deleted', 'Subtask has been deleted successfully');
			} catch (err) {
				toastActions.error('Failed to delete subtask', err instanceof Error ? err.message : 'Unknown error');
			}
		}
	}

	async function handleDelete() {
		if (!task) return;

		if (confirm(`Are you sure you want to delete "${task.title}"? This action cannot be undone.`)) {
			try {
				await taskActions.deleteTask(task.id);
				toastActions.success('Task deleted successfully');
				goto('/tasks');
			} catch (err) {
				toastActions.error(
					'Failed to delete task',
					err instanceof Error ? err.message : 'An unexpected error occurred'
				);
			}
		}
	}

	function getTaskStatusColor(status: string) {
		switch (status) {
			case 'completed': return 'text-green-500';
			case 'in-progress': return 'text-blue-500';
			case 'cancelled': return 'text-red-500';
			default: return 'text-gray-400';
		}
	}

	function getStatusBadgeColor(status: string) {
		switch (status) {
			case 'completed': return 'bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-300';
			case 'in-progress': return 'bg-blue-100 text-blue-800 dark:bg-blue-900/20 dark:text-blue-300';
			case 'cancelled': return 'bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-300';
			default: return 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-300';
		}
	}

	function getPriorityColor(priority: string) {
		switch (priority) {
			case 'high': return 'text-red-500';
			case 'medium': return 'text-yellow-500';
			case 'low': return 'text-green-500';
			default: return 'text-gray-400';
		}
	}

	function getTaskIcon(task: Task) {
		switch (task.status) {
			case 'completed': return 'mdi:check-circle';
			case 'in-progress': return 'mdi:progress-clock';
			case 'cancelled': return 'mdi:cancel';
			default: return 'mdi:circle-outline';
		}
	}

	// Keyboard shortcuts
	function handleKeydown(event: KeyboardEvent) {
		if (event.ctrlKey || event.metaKey) {
			if (event.key === 'e') {
				event.preventDefault();
				if (!isEditing) handleEdit();
			} else if (event.key === 's' && isEditing) {
				event.preventDefault();
				handleSave();
			} else if (event.key === 'Escape') {
				event.preventDefault();
				if (isEditing) handleCancel();
			}
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="min-h-screen bg-background">
	{#if isLoading}
		<div class="flex items-center justify-center py-12">
			<LoadingSpinner size="lg" text="Loading task..." />
		</div>
	{:else if error}
		<div class="flex items-center justify-center py-12">
			<div class="text-center space-y-4">
				<Icon icon="mdi:alert-circle" class="w-12 h-12 text-destructive mx-auto" />
				<div>
					<h3 class="text-lg font-semibold text-destructive">Failed to load task</h3>
					<p class="text-muted-foreground">{error}</p>
				</div>
				<Button onclick={() => goto('/tasks')} variant="outline">
					Back to Tasks
				</Button>
			</div>
		</div>
	{:else if task}
		<div class="container mx-auto p-6">
			<!-- Header -->
			<div class="flex items-center justify-between mb-6">
				<div class="flex items-center gap-4">
					<Button
						variant="ghost"
						onclick={() => goto('/tasks')}
						class="flex items-center gap-2"
					>
						<Icon icon="mdi:arrow-left" class="w-4 h-4" />
						Back to Tasks
					</Button>
					<Separator orientation="vertical" class="h-6" />
					<div class="flex items-center gap-2">
						<Icon icon={getTaskIcon(task)} class="w-5 h-5 {getTaskStatusColor(task.status)}" />
            <Badge class={getStatusBadgeColor(task.status)}>
							{task.status === 'pending' ? 'To Do' : 
							 task.status === 'in-progress' ? 'In Progress' : 
							 task.status === 'completed' ? 'Completed' : 'Cancelled'}
						</Badge>
					</div>
				</div>

				<div class="flex items-center gap-2">
					{#if isEditing}
						<Button
							variant="outline"
							onclick={handleCancel}
							disabled={isSaving}
						>
							Cancel
						</Button>
						<Button
							onclick={handleSave}
							disabled={isSaving}
							loading={isSaving}
						>
							Save Changes
						</Button>
					{:else}
						<Button
							variant="outline"
							onclick={() => goto(`/tasks/${task.id}/generate`)}
							class="flex items-center gap-2"
						>
							<Icon icon="lucide:sparkles" class="w-4 h-4" />
							Generate Tasks with AI
						</Button>
						<Button
							variant="outline"
							onclick={handleEdit}
							class="flex items-center gap-2"
						>
							<Icon icon="mdi:pencil" class="w-4 h-4" />
							Edit
						</Button>
						<Button
							variant="destructive"
							onclick={handleDelete}
							class="flex items-center gap-2"
						>
							<Icon icon="mdi:delete" class="w-4 h-4" />
							Delete
						</Button>
					{/if}
				</div>
			</div>

			<div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
				<!-- Main Content -->
				<div class="lg:col-span-2 space-y-6">
					<!-- Task Details -->
					<Card>
						<CardHeader>
							<CardTitle class="flex items-center gap-2">
								<Icon icon="mdi:clipboard-text" class="w-5 h-5" />
								Task Details
							</CardTitle>
						</CardHeader>
						<CardContent class="space-y-4">
							{#if isEditing}
								<div class="space-y-4">
									<div>
										<Label for="edit-title">Title</Label>
										<Input
											id="edit-title"
											bind:value={editTitle}
											placeholder="Enter task title..."
										/>
									</div>
									<div>
										<Label for="edit-description">Description</Label>
										<Textarea
											id="edit-description"
											bind:value={editDescription}
											placeholder="Enter task description..."
											rows={4}
										/>
									</div>
									<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
										<div>
											<Label for="edit-status">Status</Label>
											<Select 
												options={TASK_STATUS_OPTIONS}
												defaultValue={editStatus}
												placeholder="Select status..."
												onSelect={(value) => editStatus = value as TaskStatus}
											/>
										</div>
										<div>
											<Label for="edit-priority">Priority</Label>
											<Select 
												options={TASK_PRIORITY_OPTIONS}
												defaultValue={editPriority}
												placeholder="Select priority..."
												onSelect={(value) => editPriority = value as TaskPriority}
											/>
										</div>
									</div>
									<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
										<div>
											<Label for="edit-type">Type</Label>
											<Input
												id="edit-type"
												bind:value={editType}
												placeholder="e.g., Story, Bug, Feature..."
											/>
										</div>
										<div>
											<Label for="edit-due-date">Due Date</Label>
											<Input
												id="edit-due-date"
												type="date"
												bind:value={editDueDate}
											/>
										</div>
									</div>
								</div>
							{:else}
								<div class="space-y-4">
									<div>
										<h2 class="text-2xl font-bold text-foreground">{task.title}</h2>
										{#if task.description}
											<div class="mt-3">
												<MarkdownView content={task.description} truncateAt={500} />
											</div>
										{:else}
											<p class="text-muted-foreground mt-2 italic">No description provided</p>
										{/if}
									</div>
									
									<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
										<div>
											<Label class="text-sm font-medium text-muted-foreground">Status</Label>
											<div class="flex items-center gap-2 mt-1">
												<Icon icon={getTaskIcon(task)} class="w-4 h-4 {getTaskStatusColor(task.status)}" />
            <Badge class={getStatusBadgeColor(task.status)}>
													{task.status === 'pending' ? 'To Do' : 
													 task.status === 'in-progress' ? 'In Progress' : 
													 task.status === 'completed' ? 'Completed' : 'Cancelled'}
												</Badge>
											</div>
										</div>
										
										<div>
											<Label class="text-sm font-medium text-muted-foreground">Priority</Label>
											<div class="flex items-center gap-2 mt-1">
												<Icon icon="mdi:flag" class="w-4 h-4 {getPriorityColor(task.priority)}" />
												<Badge variant="outline" class="uppercase">
													{task.priority}
												</Badge>
											</div>
										</div>
									</div>

									{#if task.type}
										<div>
											<Label class="text-sm font-medium text-muted-foreground">Type</Label>
											<div class="mt-1">
												<Badge variant="outline">{task.type}</Badge>
											</div>
										</div>
									{/if}

									{#if task.dueDate}
										<div>
											<Label class="text-sm font-medium text-muted-foreground">Due Date</Label>
											<div class="flex items-center gap-2 mt-1">
												<Icon icon="mdi:calendar" class="w-4 h-4 text-muted-foreground" />
												<span class="text-sm">{new Date(task.dueDate).toLocaleDateString()}</span>
											</div>
										</div>
									{/if}
								</div>
							{/if}
						</CardContent>
					</Card>

					<!-- Subtasks -->
					{#if !task.parentId}
						<Card>
							<CardHeader>
								<CardTitle class="flex items-center justify-between">
									<div class="flex items-center gap-2">
										<Icon icon="mdi:subdirectory-arrow-right" class="w-5 h-5" />
										Subtasks
									</div>
									<Button
										variant="outline"
										size="sm"
										onclick={() => showAddSubtask = !showAddSubtask}
									>
										<Icon icon="mdi:plus" class="w-4 h-4 mr-1" />
										{showAddSubtask ? 'Cancel' : 'Add Subtask'}
									</Button>
								</CardTitle>
							</CardHeader>
							<CardContent>
								<!-- Add Subtask Form -->
								{#if showAddSubtask}
									<div class="mb-4 p-4 border border-border rounded-lg bg-muted/20">
										<h4 class="font-medium mb-3">Add New Subtask</h4>
										<div class="space-y-3">
											<div>
												<Label for="new-subtask-title">Title</Label>
												<Input
													id="new-subtask-title"
													bind:value={newSubtaskTitle}
													placeholder="Enter subtask title..."
												/>
											</div>
											<div>
												<Label for="new-subtask-description">Description</Label>
												<Textarea
													id="new-subtask-description"
													bind:value={newSubtaskDescription}
													placeholder="Enter subtask description..."
													rows={2}
												/>
											</div>
											<div class="grid grid-cols-2 gap-3">
												<div>
													<Label for="new-subtask-priority">Priority</Label>
													<Select 
														options={TASK_PRIORITY_OPTIONS}
														defaultValue={newSubtaskPriority}
														placeholder="Select priority..."
														onSelect={(value) => newSubtaskPriority = value as TaskPriority}
													/>
												</div>
												<div>
													<Label for="new-subtask-type">Type</Label>
													<Select 
														options={TASK_TYPE_OPTIONS}
														defaultValue={newSubtaskType}
														placeholder="Select type..."
														onSelect={(value) => newSubtaskType = value}
													/>
												</div>
											</div>
											<div class="flex gap-2">
												<Button
													onclick={handleAddSubtask}
													disabled={!newSubtaskTitle.trim() || isAddingSubtask}
													class="flex-1"
												>
													{#if isAddingSubtask}
														<LoadingSpinner size="sm" class="mr-2" />
													{/if}
													Add Subtask
												</Button>
												<Button
													variant="outline"
													onclick={() => {
														showAddSubtask = false;
														newSubtaskTitle = '';
														newSubtaskDescription = '';
														newSubtaskPriority = 'medium';
														newSubtaskType = '';
													}}
												>
													Cancel
												</Button>
											</div>
										</div>
									</div>
								{/if}

								<!-- Subtasks List -->
								{#if task && $tasks.filter((t: Task) => t.parentId === task!.id).length > 0}
									<div class="space-y-2">
										{#each task ? $tasks.filter((t: Task) => t.parentId === task!.id) : [] as subtask}
											{#if editingSubtaskId === subtask.id}
												<!-- Edit Subtask Form -->
												<div class="p-4 border border-primary rounded-lg bg-primary/5">
													<h4 class="font-medium mb-3">Edit Subtask</h4>
													<div class="space-y-3">
														<div>
															<Label for="edit-subtask-title">Title</Label>
															<Input
																id="edit-subtask-title"
																bind:value={editSubtaskTitle}
																placeholder="Enter subtask title..."
															/>
														</div>
														<div>
															<Label for="edit-subtask-description">Description</Label>
															<Textarea
																id="edit-subtask-description"
																bind:value={editSubtaskDescription}
																placeholder="Enter subtask description..."
																rows={2}
															/>
														</div>
														<div class="grid grid-cols-3 gap-3">
															<div>
																<Label for="edit-subtask-status">Status</Label>
																<Select 
																	options={TASK_STATUS_OPTIONS}
																	defaultValue={editSubtaskStatus}
																	placeholder="Select status..."
																	onSelect={(value) => editSubtaskStatus = value as TaskStatus}
																/>
															</div>
															<div>
																<Label for="edit-subtask-priority">Priority</Label>
																<Select 
																	options={TASK_PRIORITY_OPTIONS}
																	defaultValue={editSubtaskPriority}
																	placeholder="Select priority..."
																	onSelect={(value) => editSubtaskPriority = value as TaskPriority}
																/>
															</div>
															<div>
																<Label for="edit-subtask-type">Type</Label>
																<Select 
																	options={TASK_TYPE_OPTIONS}
																	defaultValue={editSubtaskType}
																	placeholder="Select type..."
																	onSelect={(value) => editSubtaskType = value}
																/>
															</div>
														</div>
														<div class="flex gap-2">
															<Button onclick={handleSaveSubtask} class="flex-1">
																Save Changes
															</Button>
															<Button variant="outline" onclick={cancelEditSubtask}>
																Cancel
															</Button>
														</div>
													</div>
												</div>
											{:else}
												<!-- Subtask Display -->
												<div class="p-3 border border-border rounded-lg hover:bg-muted/50 transition-colors">
													<div class="flex items-center justify-between">
														<div class="flex items-center gap-2 flex-1">
															<Icon icon={getTaskIcon(subtask)} class="w-4 h-4 {getTaskStatusColor(subtask.status)}" />
															<button
																onclick={() => goto(`/tasks/${subtask.id}`)}
																class="font-medium hover:text-primary transition-colors text-left"
																title="View subtask details"
															>
																{subtask.title}
															</button>
															{#if subtask.type}
																<Badge variant="outline" class="text-xs">{subtask.type}</Badge>
															{/if}
														</div>
														<div class="flex items-center gap-2">
                              <Badge class={getStatusBadgeColor(subtask.status)}>
																{subtask.status === 'pending' ? 'To Do' : 
																 subtask.status === 'in-progress' ? 'In Progress' : 
																 subtask.status === 'completed' ? 'Completed' : 'Cancelled'}
															</Badge>
															<div class="flex gap-1">
																<Button
																	variant="ghost"
																	size="sm"
																	onclick={() => goto(`/tasks/${subtask.id}`)}
																	title="View subtask details"
																>
																	<Icon icon="mdi:eye" class="w-4 h-4" />
																</Button>
																<Button
																	variant="ghost"
																	size="sm"
																	onclick={() => startEditSubtask(subtask)}
																	title="Edit subtask"
																>
																	<Icon icon="mdi:pencil" class="w-4 h-4" />
																</Button>
																<Button
																	variant="ghost"
																	size="sm"
																	onclick={() => handleDeleteSubtask(subtask.id)}
																	title="Delete subtask"
																	class="text-destructive hover:text-destructive"
																>
																	<Icon icon="mdi:delete" class="w-4 h-4" />
																</Button>
															</div>
														</div>
													</div>
													{#if subtask.description}
														<p class="text-sm text-muted-foreground mt-2 ml-6">{subtask.description}</p>
													{/if}
												</div>
											{/if}
										{/each}
									</div>
								{:else}
									<div class="text-center py-8 text-muted-foreground">
										<Icon icon="mdi:subdirectory-arrow-right" class="w-8 h-8 mx-auto mb-2 opacity-50" />
										<p>No subtasks yet</p>
										<Button
											variant="outline"
											size="sm"
											onclick={() => showAddSubtask = true}
											class="mt-2"
										>
											<Icon icon="mdi:plus" class="w-4 h-4 mr-1" />
											Add First Subtask
										</Button>
									</div>
								{/if}
							</CardContent>
						</Card>
					{/if}
				</div>

				<!-- Sidebar -->
				<div class="space-y-6">
					<!-- Task Info -->
					<Card>
						<CardHeader>
							<CardTitle class="text-lg">Task Information</CardTitle>
						</CardHeader>
						<CardContent class="space-y-4">
							<div>
								<Label class="text-sm font-medium text-muted-foreground">Created</Label>
								<div class="flex items-center gap-2 mt-1">
									<Icon icon="mdi:calendar-plus" class="w-4 h-4 text-muted-foreground" />
									<span class="text-sm">{new Date(task.createdAt).toLocaleDateString()}</span>
								</div>
							</div>

							<div>
								<Label class="text-sm font-medium text-muted-foreground">Last Updated</Label>
								<div class="flex items-center gap-2 mt-1">
									<Icon icon="mdi:calendar-edit" class="w-4 h-4 text-muted-foreground" />
									<span class="text-sm">{new Date(task.updatedAt).toLocaleDateString()}</span>
								</div>
							</div>

							{#if task.completedAt}
								<div>
									<Label class="text-sm font-medium text-muted-foreground">Completed</Label>
									<div class="flex items-center gap-2 mt-1">
										<Icon icon="mdi:check-circle" class="w-4 h-4 text-green-500" />
										<span class="text-sm">{new Date(task.completedAt).toLocaleDateString()}</span>
									</div>
								</div>
							{/if}

							{#if task.parentId}
								<div>
									<Label class="text-sm font-medium text-muted-foreground">Parent Task</Label>
									<div class="mt-1">
										<Button
											variant="outline"
											size="sm"
											onclick={() => task && goto(`/tasks/${task.parentId}`)}
											class="w-full justify-start"
										>
											<Icon icon="mdi:subdirectory-arrow-left" class="w-4 h-4 mr-2" />
											View Parent Task
										</Button>
									</div>
								</div>
							{/if}
						</CardContent>
					</Card>

					<!-- Linked Document -->
					{#if task && task.resourceType === ResourceType.DOCUMENT && task.resourceId}
						{@const linkedDoc = $documents.find((d) => d.id === parseInt(task.resourceId!))}
						{#if linkedDoc}
							<Card>
								<CardHeader>
									<CardTitle class="text-lg flex items-center gap-2">
										<Icon icon="lucide:file-text" class="h-5 w-5" />
										Linked Document
									</CardTitle>
								</CardHeader>
								<CardContent class="space-y-3">
									<div>
										<h4 class="font-medium">{linkedDoc.title}</h4>
										<p class="text-sm text-muted-foreground line-clamp-2 mt-1">
											{linkedDoc.content.substring(0, 100)}
											{linkedDoc.content.length > 100 ? '...' : ''}
										</p>
									</div>
									<div class="flex gap-2">
										<Button
											variant="outline"
											size="sm"
											onclick={() => goto(`/documents/${linkedDoc.id}`)}
											class="flex-1"
										>
											<Icon icon="lucide:external-link" class="h-4 w-4 mr-2" />
											Open Document
										</Button>
										<Button
											variant="outline"
											size="sm"
											onclick={() => goto(`/documents/${linkedDoc.id}/edit`)}
										>
											<Icon icon="lucide:edit" class="h-4 w-4" />
										</Button>
									</div>
								</CardContent>
							</Card>
						{/if}
					{:else if task}
						<Card>
							<CardHeader>
								<CardTitle class="text-lg flex items-center gap-2">
									<Icon icon="lucide:file-text" class="h-5 w-5" />
									Document
								</CardTitle>
							</CardHeader>
							<CardContent>
								<Button
									variant="outline"
									onclick={() => goto(`/documents/create?taskId=${task.id}&taskTitle=${encodeURIComponent(task.title)}`)}
									class="w-full"
								>
									<Icon icon="lucide:plus" class="h-4 w-4 mr-2" />
									Create Document for Task
								</Button>
							</CardContent>
						</Card>
					{/if}

					<!-- Quick Actions -->
					<Card>
						<CardHeader>
							<CardTitle class="text-lg">Quick Actions</CardTitle>
						</CardHeader>
						<CardContent class="space-y-2">
							<Button
								variant="outline"
								onclick={() => task && taskActions.toggleTaskStatus(task.id)}
								class="w-full justify-start"
							>
								<Icon icon="mdi:progress-clock" class="w-4 h-4 mr-2" />
								Toggle Status
							</Button>
							
							{#if task && !task.parentId}
								<Button
									variant="outline"
									onclick={() => showAddSubtask = !showAddSubtask}
									class="w-full justify-start"
								>
									<Icon icon="mdi:plus" class="w-4 h-4 mr-2" />
									{showAddSubtask ? 'Cancel Add Subtask' : 'Add Subtask'}
								</Button>
							{/if}

							<Button
								variant="outline"
								onclick={() => task && goto(`/tasks/${task.id}/edit`)}
								class="w-full justify-start"
							>
								<Icon icon="mdi:pencil" class="w-4 h-4 mr-2" />
								Edit Task
							</Button>
						</CardContent>
					</Card>
				</div>
			</div>

			<!-- Keyboard shortcuts hint -->
			<div class="mt-6 text-xs text-muted-foreground text-center">
				<kbd class="px-1.5 py-0.5 bg-muted rounded text-xs">Ctrl+E</kbd> to edit,
				<kbd class="px-1.5 py-0.5 bg-muted rounded text-xs ml-1">Ctrl+S</kbd> to save,
				<kbd class="px-1.5 py-0.5 bg-muted rounded text-xs ml-1">Esc</kbd> to cancel
			</div>
		</div>
	{/if}
</div>