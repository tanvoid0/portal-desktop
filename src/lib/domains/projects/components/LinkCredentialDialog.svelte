<!--
	Picks an existing shared credential to link to a project's .env key.
	Search starts prefilled with the env key — the likeliest name match.
-->
<script lang="ts">
  import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogHeader,
    DialogTitle,
  } from '$lib/components/ui/dialog';
  import { Input } from '$lib/components/ui/input';
  import { Button } from '$lib/components/ui/button';
  import { credentialService } from '$lib/domains/credentials';
  import type { Credential } from '$lib/domains/credentials/types';
  import { linkProjectEnv } from '../api/projectEnvApi';
  import type { EnvLinkView } from '../generated';
  import { queryClient } from '$lib/domains/shared/query/queryClient';
  import { queryKeys } from '$lib/domains/shared/query/keys';
  import { toast } from '$lib/utils/toast';
  import Search from '~icons/fa6-solid/magnifying-glass';
  import Loader2 from '~icons/fa6-solid/spinner';

  interface Props {
    projectId: number;
    envKey: string;
    fileName: string;
    onLinked?: (link: EnvLinkView) => void;
    onClose?: () => void;
  }

  const { projectId, envKey, fileName, onLinked, onClose }: Props = $props();

  let search = $state(envKey);
  let credentials = $state<Credential[]>([]);
  let loading = $state(true);
  let linkingId = $state<string | null>(null);

  async function load(query: string) {
    loading = true;
    try {
      credentials = query.trim()
        ? await credentialService.searchCredentials(query.trim())
        : await credentialService.getCredentials();
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to load credentials');
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    void load(search);
  });

  async function handlePick(credential: Credential) {
    linkingId = credential.id;
    try {
      const link = await linkProjectEnv(projectId, envKey, fileName, credential.id);
      await queryClient.invalidateQueries({ queryKey: queryKeys.projects.env(projectId) });
      toast.success(`Linked "${envKey}" to ${credential.name}`);
      onLinked?.(link);
      onClose?.();
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to link credential');
    } finally {
      linkingId = null;
    }
  }
</script>

<Dialog open={true} onOpenChange={(open) => !open && onClose?.()}>
  <DialogContent class="sm:max-w-md">
    <DialogHeader>
      <DialogTitle>Link "{envKey}" to a shared credential</DialogTitle>
      <DialogDescription>
        Pick the credential this .env key should track for drift.
      </DialogDescription>
    </DialogHeader>

    <div class="space-y-3">
      <div class="relative">
        <Search class="absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
        <Input placeholder="Search credentials…" bind:value={search} class="pl-10" />
      </div>

      {#if loading}
        <p class="py-6 text-center text-sm text-muted-foreground">Loading…</p>
      {:else if credentials.length === 0}
        <p class="py-6 text-center text-sm text-muted-foreground">No matching credentials.</p>
      {:else}
        <div class="max-h-72 space-y-1 overflow-y-auto">
          {#each credentials as credential (credential.id)}
            <Button
              variant="outline"
              class="h-auto w-full justify-between"
              disabled={linkingId !== null}
              onclick={() => handlePick(credential)}
            >
              <span class="truncate text-sm font-medium">{credential.name}</span>
              {#if linkingId === credential.id}
                <Loader2 class="h-4 w-4 animate-spin" />
              {/if}
            </Button>
          {/each}
        </div>
      {/if}
    </div>
  </DialogContent>
</Dialog>
