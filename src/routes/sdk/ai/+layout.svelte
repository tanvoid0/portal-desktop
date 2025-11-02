<!--
	AI Section Layout - minimal wrapper for AI pages
	All sidebar logic is handled by the root layout
-->

<script lang="ts">
	import type { Snippet } from 'svelte';

	// Get children snippet from props for Svelte 5
	let { children }: { children: Snippet<[]> } = $props();
</script>

<!-- AI Content Area - just render children directly -->
{@render children()}
