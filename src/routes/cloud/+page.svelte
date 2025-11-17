<!-- Redirect /cloud to /cloud/workloads -->
<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	
	onMount(() => {
		goto('/cloud/workloads', { replaceState: true });
	});
</script>

<div class="flex items-center justify-center min-h-[400px]">
	<p class="text-muted-foreground">Redirecting...</p>
</div>
