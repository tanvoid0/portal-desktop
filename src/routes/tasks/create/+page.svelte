<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import TaskForm from '@/lib/domains/tasks/components/TaskForm.svelte';
	import type { Task } from '@/lib/domains/tasks/types';

	// Get parentId from URL params
	const parentId = $page.url.searchParams.get('parentId') || undefined;


	function handleSave(task: Task) {
		// Navigate to task details or back to task list
		goto(`/tasks/${task.id}`);
	}

	function handleCancel() {
		goto('/tasks');
	}
</script>

<div class="container mx-auto p-6">
	<TaskForm 
		{parentId}
		onSave={handleSave}
		onCancel={handleCancel}
	/>
</div>