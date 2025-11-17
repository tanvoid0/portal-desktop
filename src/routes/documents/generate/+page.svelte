<script lang="ts">
	import { page } from '$app/stores';
	import DocumentGenerationPage from '$lib/domains/documents/components/DocumentGenerationPage.svelte';

	const taskId = $derived($page.url.searchParams.get('taskId'));
	const taskTitle = $derived($page.url.searchParams.get('taskTitle'));
	const taskDescription = $derived($page.url.searchParams.get('taskDescription'));
</script>

<svelte:head>
	<title>Generate Document with AI - Portal Desktop</title>
</svelte:head>

<DocumentGenerationPage
	taskId={taskId ? parseInt(taskId) : undefined}
	taskTitle={taskTitle || undefined}
	taskDescription={taskDescription || undefined}
/>

