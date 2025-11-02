<!--
	Deployments Page - Local Docker-based deployments
-->

<script lang="ts">
	import { onMount } from 'svelte';
	import { logger } from '$lib/domains/shared';
	import { DeploymentDashboard } from '$lib/domains/deployments';
</script>

<svelte:head>
	<title>Deployments - Portal Desktop</title>
</svelte:head>

<div class="container mx-auto py-6">
	<DeploymentDashboard />
</div>
