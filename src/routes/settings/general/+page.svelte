<!--
	General Settings Page
-->

<script lang="ts">
	import { settings } from '$lib/domains/settings/stores/settingsStore';
	import GeneralSettings from '$lib/domains/settings/components/GeneralSettings.svelte';
	import { get } from 'svelte/store';

	const settingsData = $derived($settings);

	function handleGeneralUpdate(updates: Partial<import('$lib/domains/settings/types').AppSettings>) {
		const current = get(settings);
		if (!current) return;
		const newSettings = {
			...current,
			app: { ...current.app, ...updates }
		};
		// Update store directly (will be saved when user clicks Save Changes)
		settings.set(newSettings);
	}
</script>

<svelte:head>
	<title>General Settings - Portal Desktop</title>
</svelte:head>

<div class="space-y-6">
	<div>
		<h2 class="text-2xl font-bold tracking-tight">General Settings</h2>
		<p class="text-muted-foreground">
			Configure application preferences and general behavior
		</p>
	</div>

	{#if settingsData}
		<GeneralSettings 
			settings={settingsData.app} 
			onUpdate={handleGeneralUpdate} 
		/>
	{:else}
		<div class="text-center py-12 text-muted-foreground">
			<p>Loading settings...</p>
		</div>
	{/if}
</div>

