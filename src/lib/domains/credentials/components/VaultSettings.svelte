<!--
	Vault settings - lock state, lock/unlock, and master password rotation. Changing the
	password re-wraps the vault's master key (`vault_change_password`), so saved credentials are
	not re-encrypted and the recovery key stays valid.
-->

<script lang="ts">
  import { Badge, type BadgeVariant } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { PageError, PageLoading } from '$lib/components/shell';
  import SettingsSection from '$lib/domains/settings/components/shared/SettingsSection.svelte';
  import { queryClient } from '$lib/domains/shared/query/queryClient';
  import { queryKeys } from '$lib/domains/shared/query/keys';
  import { toast } from '$lib/utils/toast';
  import { getErrorMessage } from '$lib/utils/errorMessage';
  import Lock from '~icons/fa6-solid/lock';
  import Key from '~icons/fa6-solid/key';
  import { changeVaultPassword, lockVault } from '../api/vaultApi';
  import { createVaultStatusQuery } from '../queries/vaultQueries';
  import { vaultGate } from '../state/vault.svelte';
  import type { VaultStatus } from '../types';

  /** Same minimum the setup and reset forms use; the backend enforces it too. */
  const MIN_PASSWORD_LENGTH = 8;

  const stateUi: Record<VaultStatus['state'], { label: string; variant: BadgeVariant }> = {
    needs_setup: { label: 'Not set up', variant: 'outline' },
    locked: { label: 'Locked', variant: 'secondary' },
    unlocked: { label: 'Unlocked', variant: 'default' },
  };

  const statusQuery = createVaultStatusQuery();
  const status = $derived(statusQuery.data);

  // Plain handler instead of `createMutation`: a mutation would keep the passwords in the
  // query client's mutation cache after the call. Here they live only in these fields, and
  // are cleared on success.
  let oldPassword = $state('');
  let newPassword = $state('');
  let confirmPassword = $state('');
  let submitting = $state(false);
  let formError = $state<string | null>(null);

  /** Flags only what the user has already typed into, so the form does not open in an error. */
  const problem = $derived.by(() => {
    if (newPassword && newPassword.length < MIN_PASSWORD_LENGTH)
      return `New password must be at least ${MIN_PASSWORD_LENGTH} characters.`;
    if (newPassword && newPassword === oldPassword)
      return 'New password must be different from the current one.';
    if (confirmPassword && confirmPassword !== newPassword) return 'Passwords do not match.';
    return null;
  });
  const canSubmit = $derived(
    !!oldPassword && !!newPassword && !!confirmPassword && !problem && !submitting
  );

  const refreshStatus = () =>
    queryClient.invalidateQueries({ queryKey: queryKeys.credentials.status });

  async function handleLock() {
    try {
      await lockVault();
      toast.success('Vault locked');
    } catch (err) {
      toast.error(getErrorMessage(err, 'Failed to lock the vault.'));
    }
    void refreshStatus();
  }

  async function handleSubmit() {
    if (!canSubmit) return;
    formError = null;
    submitting = true;
    try {
      await changeVaultPassword(oldPassword, newPassword);
      oldPassword = '';
      newPassword = '';
      confirmPassword = '';
      toast.success('Master password changed', 'Your recovery key still works.');
      void refreshStatus();
    } catch (err) {
      formError = getErrorMessage(err, 'Failed to change the password.');
    } finally {
      submitting = false;
    }
  }
</script>

{#if statusQuery.isPending}
  <PageLoading message="Loading vault..." />
{:else if !status}
  <PageError
    message={getErrorMessage(statusQuery.error, 'Failed to load the vault status.')}
    onRetry={() => statusQuery.refetch()}
  />
{:else}
  <div class="space-y-6">
    <SettingsSection
      title="Vault"
      description="Your saved credentials are encrypted with a key that your master password protects."
      icon={Lock}
    >
      <div class="flex flex-wrap items-center justify-between gap-3">
        <div class="flex flex-wrap items-center gap-2 text-sm">
          <Badge variant={stateUi[status.state].variant}>{stateUi[status.state].label}</Badge>
          <span class="text-muted-foreground">
            {status.hasCredentials ? 'Credentials are stored.' : 'No credentials stored yet.'}
          </span>
        </div>
        {#if status.state === 'unlocked'}
          <Button variant="outline" size="sm" onclick={handleLock}>Lock now</Button>
        {:else if status.state === 'locked'}
          <Button size="sm" onclick={() => void vaultGate.ensureUnlocked('Unlock the vault.')}>
            Unlock
          </Button>
        {/if}
      </div>
    </SettingsSection>

    <SettingsSection
      title="Change master password"
      description="Saved credentials stay intact and your recovery key keeps working."
      icon={Key}
    >
      {#if status.state === 'needs_setup'}
        <p class="text-sm text-muted-foreground">
          The vault isn't set up yet. You'll be asked to choose a master password the first time
          something needs to save a credential; you can change it here afterwards.
        </p>
      {:else}
        <form
          class="space-y-4"
          onsubmit={(e) => {
            e.preventDefault();
            void handleSubmit();
          }}
        >
          <div class="space-y-2">
            <Label for="vault-change-old">Current password</Label>
            <Input
              id="vault-change-old"
              type="password"
              bind:value={oldPassword}
              autocomplete="current-password"
            />
          </div>
          <div class="space-y-2">
            <Label for="vault-change-new">New password</Label>
            <Input
              id="vault-change-new"
              type="password"
              bind:value={newPassword}
              autocomplete="new-password"
              placeholder="At least {MIN_PASSWORD_LENGTH} characters"
            />
          </div>
          <div class="space-y-2">
            <Label for="vault-change-confirm">Confirm new password</Label>
            <Input
              id="vault-change-confirm"
              type="password"
              bind:value={confirmPassword}
              autocomplete="new-password"
            />
          </div>
          {#if problem || formError}
            <p class="text-sm text-destructive" role="alert">{problem ?? formError}</p>
          {/if}
          <Button type="submit" disabled={!canSubmit}>
            {submitting ? 'Changing…' : 'Change password'}
          </Button>
        </form>
      {/if}
    </SettingsSection>
  </div>
{/if}
