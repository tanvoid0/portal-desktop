<!--
	Package Managers Settings Page
-->

<script lang="ts">
	import PackageManagersSettings from '$lib/domains/settings/components/PackageManagersSettings.svelte';
</script>

<svelte:head>
	<title>Package Managers - Portal Desktop</title>
</svelte:head>

<div class="space-y-6">
	<div>
		<h2 class="text-2xl font-bold tracking-tight">Package Manager Management</h2>
		<p class="text-muted-foreground">
			Manage your package managers and view intelligent recommendations based on your usage
		</p>
	</div>

	<PackageManagersSettings />
</div>

