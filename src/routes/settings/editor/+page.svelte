<!--
	Editor Settings Page
-->

<script lang="ts">
	import { settings } from '$lib/domains/settings/stores/settingsStore';
	import EditorSettings from '$lib/domains/settings/components/EditorSettings.svelte';
	import { get } from 'svelte/store';

	const settingsData = $derived($settings);

	function handleEditorUpdate(updates: Partial<import('$lib/domains/settings/types').EditorSettings>) {
		const current = get(settings);
		if (!current) return;
		const newSettings = {
			...current,
			editor: { ...current.editor, ...updates }
		};
		// Update store directly (will be saved when user clicks Save Changes)
		settings.set(newSettings);
	}
</script>

<svelte:head>
	<title>Editor Settings - Portal Desktop</title>
</svelte:head>

<div class="space-y-6">
	<div>
		<h2 class="text-2xl font-bold tracking-tight">Editor Settings</h2>
		<p class="text-muted-foreground">
			Configure code editor preferences and behavior
		</p>
	</div>

	{#if settingsData}
		<EditorSettings 
			settings={settingsData.editor} 
			onUpdate={handleEditorUpdate} 
		/>
	{:else}
		<div class="text-center py-12 text-muted-foreground">
			<p>Loading settings...</p>
		</div>
	{/if}
</div>

