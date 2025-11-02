<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { Button } from '@/lib/components/ui/button';
	import TaskForm from '@/lib/domains/tasks/components/TaskForm.svelte';
	import { taskActions, tasks } from '@/lib/domains/tasks/stores/taskStore';
	import { toastActions } from '@/lib/domains/shared/stores/toastStore';
	import LoadingSpinner from '@/lib/components/ui/loading-spinner.svelte';
	import type { Task } from '@/lib/domains/tasks/types';

	let task = $state<Task | null>(null);
	let isLoading = $state(true);
	let error = $state<string | null>(null);

	const taskId = $page.params.id;

	onMount(async () => {
		try {
			// Load the specific task
			await taskActions.loadTasks();
			const allTasks = $tasks;
			task = allTasks.find((t: Task) => t.id === taskId) || null;
			
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
	});

	function handleSave(updatedTask: Task) {
		// Navigate to task details or back to task list
		goto(`/tasks/${updatedTask.id}`);
	}

	function handleCancel() {
		if (task) {
			goto(`/tasks/${task.id}`);
		} else {
			goto('/tasks');
		}
	}
</script>

<div class="container mx-auto p-6">
	{#if isLoading}
		<div class="flex items-center justify-center py-12">
			<LoadingSpinner size="lg" text="Loading task..." />
		</div>
	{:else if error}
		<div class="flex items-center justify-center py-12">
			<div class="text-center space-y-4">
				<h3 class="text-lg font-semibold text-destructive">Failed to load task</h3>
				<p class="text-muted-foreground">{error}</p>
				<Button onclick={() => goto('/tasks')} variant="outline">
					Back to Tasks
				</Button>
			</div>
		</div>
	{:else if task}
		<TaskForm 
			{task}
			onSave={handleSave}
			onCancel={handleCancel}
		/>
	{/if}
</div>