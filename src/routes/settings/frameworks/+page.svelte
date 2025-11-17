<!--
	Frameworks Settings Page
-->

<script lang="ts">
	import FrameworksSettings from '$lib/domains/settings/components/FrameworksSettings.svelte';
</script>

<svelte:head>
	<title>Frameworks - Portal Desktop</title>
</svelte:head>

<div class="space-y-6">
	<div>
		<h2 class="text-2xl font-bold tracking-tight">Framework Management</h2>
		<p class="text-muted-foreground">
			Manage your frameworks and view intelligent recommendations based on your usage
		</p>
	</div>

	<FrameworksSettings />
</div>

