<!--
	IDE Settings Page
-->

<script lang="ts">
	import IdeSettings from '$lib/domains/settings/components/IdeSettings.svelte';
</script>

<svelte:head>
	<title>IDE Settings - Portal Desktop</title>
</svelte:head>

<div class="space-y-6">
	<div>
		<h2 class="text-2xl font-bold tracking-tight">IDE Configuration</h2>
		<p class="text-muted-foreground">
			Configure integrated development environments
		</p>
	</div>

	<IdeSettings />
</div>

