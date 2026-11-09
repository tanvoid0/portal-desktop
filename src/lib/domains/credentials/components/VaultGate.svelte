<!--
	Mounted once, globally, in the app shell. Watches vault status and
	`vaultGate.open`, and renders the setup or unlock dialog as needed.
-->

<script lang="ts">
  import { createVaultStatusQuery } from '../queries/vaultQueries';
  import { vaultGate } from '../state/vault.svelte';
  import VaultSetupDialog from './VaultSetupDialog.svelte';
  import VaultUnlockDialog from './VaultUnlockDialog.svelte';

  const statusQuery = createVaultStatusQuery();
  const status = $derived(statusQuery.data);
</script>

{#if (status?.state === 'needs_setup' && vaultGate.open) || vaultGate.setupStep !== null}
  <VaultSetupDialog hasCredentials={status?.hasCredentials ?? false} />
{:else if status?.state === 'locked' && vaultGate.open}
  <VaultUnlockDialog helloAvailable={status.helloAvailable} />
{/if}
