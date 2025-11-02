<!--
	Theme Settings Page
-->

<script lang="ts">
	import { settings } from '$lib/domains/settings/stores/settingsStore';
	import ThemeCustomizer from '$lib/domains/settings/components/ThemeCustomizer.svelte';
	import { get } from 'svelte/store';

	const settingsData = $derived($settings);

	function handleThemeUpdate(updates: Partial<import('$lib/domains/settings/types').ThemeSettings>) {
		const current = get(settings);
		if (!current) return;
		const newSettings = {
			...current,
			theme: { ...current.theme, ...updates }
		};
		// Update store directly (will be saved when user clicks Save Changes)
		settings.set(newSettings);
	}
</script>

<svelte:head>
	<title>Theme Settings - Portal Desktop</title>
</svelte:head>

<div class="space-y-6">
	<div>
		<h2 class="text-2xl font-bold tracking-tight">Theme Settings</h2>
		<p class="text-muted-foreground">
			Customize appearance and color schemes
		</p>
	</div>

	{#if settingsData}
		<ThemeCustomizer 
			settings={settingsData.theme} 
			onUpdate={handleThemeUpdate} 
		/>
	{:else}
		<div class="text-center py-12 text-muted-foreground">
			<p>Loading settings...</p>
		</div>
	{/if}
</div>

