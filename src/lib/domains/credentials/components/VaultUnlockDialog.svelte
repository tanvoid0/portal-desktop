<!--
	Vault unlock — password or Windows Hello, with a "forgot password" recovery
	flow via `vault_reset_password`. Dismissing settles the gate with `false`.
-->

<script lang="ts">
  import * as Dialog from '$lib/components/ui/dialog';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import {
    unlockVaultWithPassword,
    unlockVaultWithHello,
    resetVaultPassword,
  } from '../api/vaultApi';
  import { vaultGate } from '../state/vault.svelte';
  import { queryClient } from '$lib/domains/shared/query/queryClient';
  import { queryKeys } from '$lib/domains/shared/query/keys';
  import { toast } from '$lib/utils/toast';
  import { getErrorMessage } from '$lib/utils/errorMessage';

  interface Props {
    helloAvailable: boolean;
  }
  let { helloAvailable }: Props = $props();

  let mode = $state<'unlock' | 'recovery'>('unlock');
  let password = $state('');
  let submitting = $state(false);
  let helloPending = $state(false);
  let formError = $state<string | null>(null);

  let recoveryKey = $state('');
  let newPassword = $state('');
  let confirmPassword = $state('');

  function settleUnlocked() {
    void queryClient.invalidateQueries({ queryKey: queryKeys.credentials.status });
    vaultGate.settle(true);
  }

  async function handleUnlock() {
    formError = null;
    submitting = true;
    try {
      await unlockVaultWithPassword(password);
      settleUnlocked();
    } catch (err) {
      formError = getErrorMessage(err, 'Failed to unlock.');
    } finally {
      submitting = false;
    }
  }

  async function handleHello() {
    helloPending = true;
    try {
      await unlockVaultWithHello();
      settleUnlocked();
    } catch (err) {
      toast.error(getErrorMessage(err, 'Windows Hello was cancelled.'));
    } finally {
      helloPending = false;
    }
  }

  /** Groups into `XXXX-XXXX-...` as the user types; the backend parser accepts any spacing anyway. */
  function formatRecoveryKeyInput(value: string): string {
    const clean = value
      .replace(/[^a-zA-Z0-9]/g, '')
      .toUpperCase()
      .slice(0, 32);
    return clean.match(/.{1,4}/g)?.join('-') ?? clean;
  }

  function handleRecoveryKeyInput(e: Event) {
    recoveryKey = formatRecoveryKeyInput((e.target as HTMLInputElement).value);
  }

  async function handleReset() {
    formError = null;
    if (newPassword.length < 8) {
      formError = 'Password must be at least 8 characters.';
      return;
    }
    if (newPassword !== confirmPassword) {
      formError = 'Passwords do not match.';
      return;
    }
    submitting = true;
    try {
      await resetVaultPassword(recoveryKey, newPassword);
      settleUnlocked();
    } catch (err) {
      formError = getErrorMessage(err, 'Failed to reset the password.');
    } finally {
      submitting = false;
    }
  }
</script>

<Dialog.Root
  open={true}
  onOpenChange={(isOpen) => {
    if (!isOpen) vaultGate.settle(false);
  }}
>
  <Dialog.Content class="sm:max-w-md">
    {#if mode === 'unlock'}
      <Dialog.Header>
        <Dialog.Title>Vault locked</Dialog.Title>
        <Dialog.Description>Enter your master password to unlock it.</Dialog.Description>
      </Dialog.Header>

      <form
        class="space-y-4"
        onsubmit={(e) => {
          e.preventDefault();
          handleUnlock();
        }}
      >
        <div class="space-y-2">
          <Label for="vault-unlock-password">Master password</Label>
          <Input
            id="vault-unlock-password"
            type="password"
            bind:value={password}
            autocomplete="current-password"
          />
        </div>
        {#if formError}
          <p class="text-sm text-destructive">{formError}</p>
        {/if}
        <Button type="submit" disabled={submitting || !password} class="w-full">
          {submitting ? 'Unlocking…' : 'Unlock'}
        </Button>
      </form>

      {#if helloAvailable}
        <Button variant="outline" disabled={helloPending} onclick={handleHello} class="w-full">
          {helloPending ? 'Waiting for Windows Hello…' : 'Unlock with Windows Hello'}
        </Button>
      {/if}

      <Button
        variant="link"
        size="sm"
        onclick={() => (mode = 'recovery')}
        class="h-auto self-start px-0"
      >
        Forgot password?
      </Button>
    {:else}
      <Dialog.Header>
        <Dialog.Title>Reset your password</Dialog.Title>
        <Dialog.Description>
          Enter your recovery key and choose a new master password. Existing credentials stay
          intact.
        </Dialog.Description>
      </Dialog.Header>

      <form
        class="space-y-4"
        onsubmit={(e) => {
          e.preventDefault();
          handleReset();
        }}
      >
        <div class="space-y-2">
          <Label for="vault-recovery-key">Recovery key</Label>
          <Input
            id="vault-recovery-key"
            value={recoveryKey}
            oninput={handleRecoveryKeyInput}
            placeholder="XXXX-XXXX-XXXX-XXXX-XXXX-XXXX-XXXX-XXXX"
            class="font-mono tracking-wide"
            autocomplete="off"
          />
        </div>
        <div class="space-y-2">
          <Label for="vault-recovery-new">New master password</Label>
          <Input
            id="vault-recovery-new"
            type="password"
            bind:value={newPassword}
            autocomplete="new-password"
          />
        </div>
        <div class="space-y-2">
          <Label for="vault-recovery-confirm">Confirm new password</Label>
          <Input
            id="vault-recovery-confirm"
            type="password"
            bind:value={confirmPassword}
            autocomplete="new-password"
          />
        </div>
        {#if formError}
          <p class="text-sm text-destructive">{formError}</p>
        {/if}
        <Button type="submit" disabled={submitting} class="w-full">
          {submitting ? 'Resetting…' : 'Reset password & unlock'}
        </Button>
      </form>

      <Button
        variant="link"
        size="sm"
        onclick={() => (mode = 'unlock')}
        class="h-auto self-start px-0"
      >
        Back to password unlock
      </Button>
    {/if}
  </Dialog.Content>
</Dialog.Root>
