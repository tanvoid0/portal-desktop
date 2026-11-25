<script lang="ts">
  import { createMutation, useQueryClient } from '@tanstack/svelte-query';
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
  import { toast } from '$lib/utils/toast';
  import { getErrorMessage } from '$lib/utils/errorMessage';
  import { queryKeys } from '$lib/domains/shared/query/keys';
  import { setSyncConfig, syncLogin } from '../api/syncApi';

  interface Props {
    open: boolean;
    serverUrl: string;
    hasToken: boolean;
  }

  let { open = $bindable(), serverUrl, hasToken }: Props = $props();

  const client = useQueryClient();
  let url = $state('');
  let token = $state('');
  let email = $state('');
  let password = $state('');

  $effect(() => {
    if (open) {
      url = serverUrl;
      token = '';
      email = '';
      password = '';
    }
  });

  // Email + password wins: it yields a refreshable session. The pasted token is the fallback.
  const save = createMutation(() => ({
    mutationFn: () =>
      email.trim() && password
        ? syncLogin(url.trim(), email.trim(), password)
        : setSyncConfig(url.trim(), token.trim() || undefined),
    onSuccess: async () => {
      await client.invalidateQueries({ queryKey: queryKeys.sync.all });
      toast.success('Sync server saved');
      open = false;
    },
    onError: (e) => toast.error(getErrorMessage(e, 'Could not save sync server')),
  }));
</script>

<Dialog bind:open>
  <DialogContent class="max-w-[min(100vw-2rem,28rem)]">
    <DialogHeader>
      <DialogTitle>Sync server</DialogTitle>
      <DialogDescription
        >Where backups are stored. Use the same server on every PC.</DialogDescription
      >
    </DialogHeader>
    <div class="space-y-3">
      <div class="space-y-1.5">
        <Label for="sync-url">Server URL</Label>
        <Input id="sync-url" bind:value={url} placeholder="https://sync.example.com" />
      </div>
      <div class="space-y-1.5">
        <Label for="sync-email">Email</Label>
        <Input id="sync-email" type="email" bind:value={email} autocomplete="username" />
      </div>
      <div class="space-y-1.5">
        <Label for="sync-password">Password</Label>
        <Input
          id="sync-password"
          type="password"
          bind:value={password}
          autocomplete="current-password"
          placeholder={hasToken ? 'Logged in - leave blank to keep' : ''}
        />
      </div>
      <div class="space-y-1.5">
        <Label for="sync-token">Or paste an access token</Label>
        <Input
          id="sync-token"
          type="password"
          bind:value={token}
          autocomplete="off"
          placeholder={hasToken ? 'Saved - leave blank to keep' : ''}
        />
      </div>
    </div>
    <DialogFooter>
      <Button variant="outline" onclick={() => (open = false)}>Cancel</Button>
      <Button disabled={!url.trim() || save.isPending} onclick={() => save.mutate()}>Save</Button>
    </DialogFooter>
  </DialogContent>
</Dialog>
