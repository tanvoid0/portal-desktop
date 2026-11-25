<script lang="ts">
  import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
  } from '$lib/components/ui/dialog';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { getErrorMessage } from '$lib/utils/errorMessage';

  interface Props {
    open: boolean;
    mode: 'backup' | 'restore';
    /** First backup: ask twice, since losing the password makes the backup unreadable. */
    confirmTwice?: boolean;
    versionLabel?: string;
    /** Throw to show the message inline and keep the dialog open. */
    onSubmit: (password: string) => Promise<void>;
  }

  let { open = $bindable(), mode, confirmTwice = false, versionLabel, onSubmit }: Props = $props();

  let password = $state('');
  let again = $state('');
  let busy = $state(false);
  let error = $state('');

  $effect(() => {
    if (!open) {
      password = '';
      again = '';
      error = '';
    }
  });

  const mismatch = $derived(confirmTwice && again !== '' && again !== password);
  const canSubmit = $derived(password !== '' && (!confirmTwice || again === password) && !busy);

  async function submit() {
    if (!canSubmit) return;
    busy = true;
    error = '';
    try {
      await onSubmit(password);
    } catch (e) {
      error = getErrorMessage(e, 'Sync failed');
    } finally {
      busy = false;
    }
  }
</script>

<Dialog bind:open>
  <DialogContent class="max-w-[min(100vw-2rem,28rem)]">
    <DialogHeader>
      <DialogTitle>
        {mode === 'backup' ? 'Back up to cloud' : `Restore from cloud${versionLabel ?? ''}`}
      </DialogTitle>
      <DialogDescription>
        {#if mode === 'backup'}
          Your data, including the credentials vault, is encrypted with this password before it
          leaves this machine.
          {#if confirmTwice}
            <strong>If you lose it, the backup cannot be read.</strong>
          {/if}
        {:else}
          Enter the password this backup was created with.
        {/if}
      </DialogDescription>
    </DialogHeader>
    <form
      class="space-y-3"
      onsubmit={(e) => {
        e.preventDefault();
        void submit();
      }}
    >
      <div class="space-y-1.5">
        <Label for="sync-password">Snapshot password</Label>
        <Input id="sync-password" type="password" bind:value={password} autocomplete="off" />
      </div>
      {#if confirmTwice}
        <div class="space-y-1.5">
          <Label for="sync-password-2">Confirm password</Label>
          <Input id="sync-password-2" type="password" bind:value={again} autocomplete="off" />
          {#if mismatch}<p class="text-xs text-destructive">Passwords do not match.</p>{/if}
        </div>
      {/if}
      {#if error}<p class="text-sm text-destructive">{error}</p>{/if}
      <DialogFooter>
        <Button type="button" variant="outline" onclick={() => (open = false)}>Cancel</Button>
        <Button type="submit" disabled={!canSubmit}>
          {busy ? 'Working...' : mode === 'backup' ? 'Back up' : 'Restore'}
        </Button>
      </DialogFooter>
    </form>
  </DialogContent>
</Dialog>
