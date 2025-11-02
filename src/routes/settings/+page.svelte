<!--
	Settings Index Page - Redirects to general settings
-->

<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';

	onMount(() => {
		goto('/settings/general', { replaceState: true });
	});
</script>

<div class="flex items-center justify-center py-12">
	<p class="text-muted-foreground">Redirecting to settings...</p>
</div>
