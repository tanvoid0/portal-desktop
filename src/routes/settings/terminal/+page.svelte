<!--
	Terminal Settings Page
-->

<script lang="ts">
	import { settings } from '$lib/domains/settings/stores/settingsStore';
	import TerminalSettings from '$lib/domains/settings/components/TerminalSettings.svelte';
	import { get } from 'svelte/store';

	const settingsData = $derived($settings);

	function handleTerminalUpdate(updates: Partial<import('$lib/domains/settings/types').TerminalSettings>) {
		const current = get(settings);
		if (!current) return;
		const newSettings = {
			...current,
			terminal: { ...current.terminal, ...updates }
		};
		// Update store directly (will be saved when user clicks Save Changes)
		settings.set(newSettings);
	}
</script>

<svelte:head>
	<title>Terminal Settings - Portal Desktop</title>
</svelte:head>

<div class="space-y-6">
	<div>
		<h2 class="text-2xl font-bold tracking-tight">Terminal Settings</h2>
		<p class="text-muted-foreground">
			Configure terminal preferences and behavior
		</p>
	</div>

	{#if settingsData}
		<TerminalSettings 
			settings={settingsData.terminal} 
			onUpdate={handleTerminalUpdate} 
		/>
	{:else}
		<div class="text-center py-12 text-muted-foreground">
			<p>Loading settings...</p>
		</div>
	{/if}
</div>

