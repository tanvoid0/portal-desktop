<script lang="ts">
	import { page } from '$app/stores';
	import TaskGenerationPage from '@/lib/domains/tasks/components/TaskGenerationPage.svelte';

	const taskId = $derived($page.params.id);
</script>

<TaskGenerationPage {taskId} />

