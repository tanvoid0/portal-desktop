<!--
	Autonomy Settings Page
-->

<script lang="ts">
	import AutonomySettings from '$lib/domains/settings/components/AutonomySettings.svelte';
</script>

<svelte:head>
	<title>Autonomy Settings - Portal Desktop</title>
</svelte:head>

<div class="space-y-6">
	<div>
		<h2 class="text-2xl font-bold tracking-tight">Autonomous Action Settings</h2>
		<p class="text-muted-foreground">
			Configure how the system autonomously executes actions based on learned patterns
		</p>
	</div>

	<AutonomySettings />
</div>

