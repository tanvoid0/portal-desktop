<script lang="ts">
  import { goto } from '$app/navigation';
  import { PageEmpty, PageError } from '$lib/components/shell';
  import { vaultGate } from '$lib/domains/credentials/state/vault.svelte';
  import KeyRound from '~icons/fa6-solid/key';
  import { isTokenUnreadableError, isVaultLockedError } from '../utils/tokenError';

  interface Props {
    title?: string;
    message: string;
    onRetry?: () => void;
  }

  let { title, message, onRetry }: Props = $props();

  async function unlock() {
    if (await vaultGate.ensureUnlocked()) onRetry?.();
  }
</script>

{#if isVaultLockedError(message)}
  <PageEmpty
    title="Vault is locked"
    description="Unlock the vault to use your saved GitHub token."
    icon={KeyRound}
    actionLabel="Unlock vault"
    onAction={unlock}
  />
{:else if isTokenUnreadableError(message)}
  <PageEmpty
    title="Stored GitHub token can't be read"
    description="Portal can't decrypt the saved GitHub token. Reconnect GitHub to continue."
    icon={KeyRound}
    actionLabel="Reconnect GitHub"
    onAction={() => goto('/settings/github')}
  />
{:else}
  <PageError {title} {message} {onRetry} />
{/if}
