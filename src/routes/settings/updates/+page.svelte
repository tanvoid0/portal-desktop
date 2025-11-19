<!--
	Updates Settings Page
-->

<script lang="ts">
	import UpdateChecker from '$lib/domains/updates/components/UpdateChecker.svelte';
</script>

<svelte:head>
	<title>Updates - Portal Desktop</title>
</svelte:head>

<div class="space-y-6">
	<div>
		<h2 class="text-2xl font-bold tracking-tight">Updates</h2>
		<p class="text-muted-foreground">
			Check for application updates and manage update settings
		</p>
	</div>

	<UpdateChecker />
</div>

