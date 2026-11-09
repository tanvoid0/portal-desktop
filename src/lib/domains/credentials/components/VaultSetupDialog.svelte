<!--
	First-time vault setup. Forced, two steps: choose a master password, then
	save the recovery key — shown exactly once. Cannot be dismissed.
-->

<script lang="ts">
  import * as Dialog from '$lib/components/ui/dialog';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { Checkbox } from '$lib/components/ui/checkbox';
  import { setupVault } from '../api/vaultApi';
  import { vaultGate } from '../state/vault.svelte';
  import { queryClient } from '$lib/domains/shared/query/queryClient';
  import { queryKeys } from '$lib/domains/shared/query/keys';
  import { toast } from '$lib/utils/toast';
  import { getErrorMessage } from '$lib/utils/errorMessage';
  import Copy from '~icons/fa6-solid/copy';

  interface Props {
    hasCredentials: boolean;
  }
  let { hasCredentials }: Props = $props();

  // Independent of `vaultGate.setupStep` on purpose: that one only keeps `VaultGate`
  // from unmounting this dialog early (see its doc comment). Deriving this from it
  // would flash back to the password step the instant `handleDone` clears it, before
  // the status refetch resolves and actually unmounts.
  let step = $state<'password' | 'recovery'>('password');
  let password = $state('');
  let confirmPassword = $state('');
  let submitting = $state(false);
  let formError = $state<string | null>(null);
  let recoveryKey = $state('');
  let savedConfirmed = $state(false);

  async function handleSubmit() {
    formError = null;
    if (password.length < 8) {
      formError = 'Password must be at least 8 characters.';
      return;
    }
    if (password !== confirmPassword) {
      formError = 'Passwords do not match.';
      return;
    }
    submitting = true;
    try {
      const result = await setupVault(password);
      recoveryKey = result.recoveryKey;
      step = 'recovery';
      vaultGate.setupStep = 'recovery';
    } catch (err) {
      formError = getErrorMessage(err, 'Failed to set up the vault.');
    } finally {
      submitting = false;
    }
  }

  async function copyRecoveryKey() {
    try {
      await navigator.clipboard.writeText(recoveryKey);
      toast.success('Recovery key copied to clipboard');
    } catch {
      toast.error('Failed to copy to clipboard');
    }
  }

  function handleDone() {
    vaultGate.setupStep = null;
    void queryClient.invalidateQueries({ queryKey: queryKeys.credentials.status });
    vaultGate.settle(true);
  }
</script>

<Dialog.Root open={true}>
  <Dialog.Content
    showCloseButton={false}
    interactOutsideBehavior="ignore"
    escapeKeydownBehavior="ignore"
    class="sm:max-w-md"
  >
    {#if step === 'password'}
      <Dialog.Header>
        <Dialog.Title>Set up your vault</Dialog.Title>
        <Dialog.Description>
          {#if hasCredentials}
            Set a master password to control access to the credentials already stored here.
          {:else}
            Set a master password to control access to credentials you store in Portal Desktop.
          {/if}
        </Dialog.Description>
      </Dialog.Header>

      <form
        class="space-y-4"
        onsubmit={(e) => {
          e.preventDefault();
          handleSubmit();
        }}
      >
        <div class="space-y-2">
          <Label for="vault-setup-password">Master password</Label>
          <Input
            id="vault-setup-password"
            type="password"
            bind:value={password}
            autocomplete="new-password"
            placeholder="At least 8 characters"
          />
        </div>
        <div class="space-y-2">
          <Label for="vault-setup-confirm">Confirm password</Label>
          <Input
            id="vault-setup-confirm"
            type="password"
            bind:value={confirmPassword}
            autocomplete="new-password"
          />
        </div>
        {#if formError}
          <p class="text-sm text-destructive">{formError}</p>
        {/if}
        <Dialog.Footer>
          <Button type="submit" disabled={submitting} class="w-full">
            {submitting ? 'Setting up…' : 'Set up vault'}
          </Button>
        </Dialog.Footer>
      </form>
    {:else}
      <Dialog.Header>
        <Dialog.Title>Save your recovery key</Dialog.Title>
        <Dialog.Description>
          This is the only way to reset your password if you forget it, and it will never be shown
          again.
        </Dialog.Description>
      </Dialog.Header>

      <div class="flex items-center justify-between gap-3 rounded-md border bg-muted/40 p-3">
        <code class="font-mono text-sm tracking-wider break-all">{recoveryKey}</code>
        <Button
          variant="ghost"
          size="icon"
          onclick={copyRecoveryKey}
          aria-label="Copy recovery key"
        >
          <Copy class="h-4 w-4" />
        </Button>
      </div>

      <div class="flex items-center gap-2">
        <Checkbox id="vault-setup-saved" bind:checked={savedConfirmed} />
        <Label for="vault-setup-saved" class="text-sm font-normal">
          I've saved my recovery key somewhere safe
        </Label>
      </div>

      <Dialog.Footer>
        <Button disabled={!savedConfirmed} onclick={handleDone} class="w-full">Done</Button>
      </Dialog.Footer>
    {/if}
  </Dialog.Content>
</Dialog.Root>
