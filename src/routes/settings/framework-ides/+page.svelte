<!--
	Framework IDE Settings Page - Redirects to IDE settings
-->

<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';

	onMount(() => {
		goto('/settings/ides', { replaceState: true });
	});
</script>

<div class="flex items-center justify-center py-12">
	<p class="text-muted-foreground">Redirecting to IDE settings...</p>
</div>
