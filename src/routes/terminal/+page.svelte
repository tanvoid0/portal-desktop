<!--
	Terminal - AI-Enhanced Terminal (Default Page)
	Supports both regular commands and AI queries in unified interface
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import AITerminalContainer from '$lib/domains/terminal/components/ai/AITerminalContainer.svelte';

	let error = $state<string | null>(null);

	onMount(() => {
		console.log('AI Terminal page mounted');
		try {
			console.log('AITerminalContainer imported successfully');
		} catch (e) {
			error = e instanceof Error ? e.message : 'Unknown error';
			console.error('Error loading AI Terminal:', e);
		}
	});
</script>

<div class="h-full w-full bg-gray-900 flex flex-col overflow-hidden">
	{#if error}
		<div class="flex items-center justify-center h-full text-red-400">
			<div class="text-center">
				<p class="text-lg font-semibold mb-2">Error loading Terminal</p>
				<p class="text-sm">{error}</p>
			</div>
		</div>
	{:else}
		<AITerminalContainer />
	{/if}
</div>
