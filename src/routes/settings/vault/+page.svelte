<script lang="ts">
  import VaultSettings from '$lib/domains/credentials/components/VaultSettings.svelte';
  import EnvBackupSection from '$lib/domains/projects/components/EnvBackupSection.svelte';
</script>

<svelte:head>
  <title>Vault - Portal Desktop</title>
</svelte:head>

<div class="space-y-6">
  <VaultSettings />
  <EnvBackupSection />
</div>
