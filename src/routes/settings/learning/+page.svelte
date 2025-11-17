<!--
	Learning Settings Page
-->

<script lang="ts">
	import LearningSettings from '$lib/domains/settings/components/LearningSettings.svelte';
</script>

<svelte:head>
	<title>Learning Settings - Portal Desktop</title>
</svelte:head>

<div class="space-y-6">
	<div>
		<h2 class="text-2xl font-bold tracking-tight">Learning & AI Settings</h2>
		<p class="text-muted-foreground">
			Configure the ML learning system, resource usage, and memory management
		</p>
	</div>

	<LearningSettings />
</div>

