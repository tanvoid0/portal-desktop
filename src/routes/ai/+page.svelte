<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';

	onMount(() => {
		// Redirect to chat page by default
		goto('/ai/chat');
	});
</script>

<div class="flex items-center justify-center h-full">
	<p class="text-muted-foreground">Redirecting to chat...</p>
</div>

