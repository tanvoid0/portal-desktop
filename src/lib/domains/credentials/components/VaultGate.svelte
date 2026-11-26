<!--
	Mounted once, globally, in the app shell. Watches vault status and
	`vaultGate.open`, and renders the setup or unlock dialog as needed.
-->

<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { isTauriEnvironment } from '$lib/utils/tauri';
  import { queryClient } from '$lib/domains/shared/query/queryClient';
  import { queryKeys } from '$lib/domains/shared/query/keys';
  import { settleVaultUnlock } from '../api/vaultApi';
  import { createVaultStatusQuery } from '../queries/vaultQueries';
  import { vaultGate } from '../state/vault.svelte';
  import VaultSetupDialog from './VaultSetupDialog.svelte';
  import VaultUnlockDialog from './VaultUnlockDialog.svelte';

  const statusQuery = createVaultStatusQuery();
  const status = $derived(statusQuery.data);

  // Rust asks for an unlock when a command needs the vault (`VaultSession::require_key`);
  // answer with the user's decision so it continues or fails.
  onMount(() => {
    if (!isTauriEnvironment()) return;
    const unlisten = listen('vault://unlock-requested', async () => {
      const ok = await vaultGate
        .ensureUnlocked('An action needs your saved credentials.')
        .catch(() => false);
      await settleVaultUnlock(ok).catch(() => {});
    });
    // The idle timeout locked the vault in the backend.
    const unlistenLocked = listen('vault://locked', () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.credentials.status });
    });
    return () => {
      void unlisten.then((fn) => fn());
      void unlistenLocked.then((fn) => fn());
    };
  });
</script>

{#if (status?.state === 'needs_setup' && vaultGate.open) || vaultGate.setupStep !== null}
  <VaultSetupDialog hasCredentials={status?.hasCredentials ?? false} />
{:else if status?.state === 'locked' && vaultGate.open}
  <VaultUnlockDialog helloAvailable={status.helloAvailable} />
{/if}
