<!--
	Credentials Page
-->

<script lang="ts">
	import { onMount } from 'svelte';
	import { CredentialVault } from '$lib/domains/credentials';
	import { logger } from '$lib/domains/shared';
</script>

<svelte:head>
	<title>Credentials - Portal Desktop</title>
</svelte:head>

<div class="container mx-auto py-6">
	<CredentialVault />
</div>
