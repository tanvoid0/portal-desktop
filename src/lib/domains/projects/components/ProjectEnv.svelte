<!--
	Per-project .env viewer: live off disk every load, with optional linking to
	shared vault credentials so drift between the file and the shared value
	shows up as a badge instead of a surprise.
-->
<script lang="ts">
  import { createProjectEnvQuery } from '../queries/projectEnvQueries';
  import { linkProjectEnv, unlinkProjectEnv, resyncProjectEnvLink } from '../api/projectEnvApi';
  import type { EnvLinkView, EnvVarView } from '../generated';
  import LinkCredentialDialog from './LinkCredentialDialog.svelte';
  import { queryClient } from '$lib/domains/shared/query/queryClient';
  import { queryKeys } from '$lib/domains/shared/query/keys';
  import { credentialService, vaultGate, CredentialForm } from '$lib/domains/credentials';
  import type { Credential } from '$lib/domains/credentials/types';
  import { confirmAction } from '$lib/utils/confirm';
  import { toast } from '$lib/utils/toast';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
  } from '$lib/components/ui/table';
  import { Badge, type BadgeVariant } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuSeparator,
    DropdownMenuTrigger,
  } from '$lib/components/ui/dropdown-menu';
  import { PageEmpty, PageError, PageLoading } from '$lib/components/shell';
  import Eye from '~icons/fa6-solid/eye';
  import EyeOff from '~icons/fa6-solid/eye-slash';
  import RefreshCw from '~icons/fa6-solid/rotate';
  import Ellipsis from '~icons/fa6-solid/ellipsis';
  import Copy from '~icons/fa6-solid/copy';
  import LinkIcon from '~icons/fa6-solid/link';
  import UnlinkIcon from '~icons/fa6-solid/link-slash';
  import Plus from '~icons/fa6-solid/plus';
  import Check from '~icons/fa6-solid/check';
  import PenToSquare from '~icons/fa6-solid/pen-to-square';
  import FileKey from '~icons/fa6-solid/key';

  interface Props {
    projectId: number;
    enabled?: boolean;
  }

  const { projectId, enabled = true }: Props = $props();

  const envQuery = createProjectEnvQuery(
    () => projectId,
    () => enabled
  );
  const projectEnv = $derived(envQuery.data);

  const WARNING_BADGE = 'border-amber-500/40 bg-amber-500/10 text-amber-700 dark:text-amber-400';

  function driftBadge(state: EnvLinkView['state']): {
    label: string;
    variant: BadgeVariant;
    class?: string;
  } {
    switch (state) {
      case 'inSync':
        return { label: 'Linked', variant: 'outline' };
      case 'fileChanged':
        return { label: 'Changed on disk', variant: 'outline', class: WARNING_BADGE };
      case 'credentialChanged':
        return { label: 'Shared value updated', variant: 'outline', class: WARNING_BADGE };
      case 'differs':
        return { label: 'Differs from shared', variant: 'destructive' };
      case 'notInFile':
        return { label: 'Not in file', variant: 'outline', class: 'text-muted-foreground' };
      case 'unknown':
        return { label: 'Unlock to compare', variant: 'outline', class: 'text-muted-foreground' };
    }
  }

  let revealed = $state<Set<string>>(new Set());
  function toggleReveal(id: string) {
    const next = new Set(revealed);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    revealed = next;
  }

  function invalidate() {
    return queryClient.invalidateQueries({ queryKey: queryKeys.projects.env(projectId) });
  }

  async function handleUnlockToCompare() {
    if (await vaultGate.ensureUnlocked()) void invalidate();
  }

  async function handleCopyShared(link: EnvLinkView) {
    try {
      const value = await credentialService.decryptCredential(link.credentialId);
      await navigator.clipboard.writeText(value);
      toast.success('Value copied to clipboard');
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to copy value');
    }
  }

  async function handleUpdateSharedFromFile(envVar: EnvVarView) {
    if (!envVar.link) return;
    try {
      await credentialService.updateCredential(envVar.link.credentialId, { value: envVar.value });
      await resyncProjectEnvLink(envVar.link.id);
      await invalidate();
      toast.success('Shared credential updated');
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to update shared credential');
    }
  }

  async function handleResync(link: EnvLinkView) {
    try {
      await resyncProjectEnvLink(link.id);
      await invalidate();
      toast.success('Acknowledged');
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to re-sync');
    }
  }

  async function handleUnlink(link: EnvLinkView, envKey: string) {
    const ok = await confirmAction(
      `Unlink "${envKey}" from "${link.credentialName}"? Portal will stop tracking drift for it.`,
      'Unlink credential'
    );
    if (!ok) return;
    try {
      await unlinkProjectEnv(link.id);
      await invalidate();
      toast.success('Unlinked');
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to unlink');
    }
  }

  let linkDialogTarget = $state<{ envKey: string; fileName: string } | null>(null);
  let createFor = $state<{ envKey: string; fileName: string; value: string } | null>(null);

  async function handleCredentialCreated(credential: Credential) {
    const target = createFor;
    createFor = null;
    if (!target) return;
    try {
      await linkProjectEnv(projectId, target.envKey, target.fileName, credential.id);
      await invalidate();
      toast.success(`Linked "${target.envKey}" to ${credential.name}`);
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to link credential');
    }
  }
</script>

<div class="space-y-6">
  <div class="flex flex-wrap items-start justify-between gap-3">
    <p class="text-sm text-muted-foreground">
      Read from disk each time. Portal never edits your .env files.
    </p>
    <Button
      variant="ghost"
      size="sm"
      class="gap-2"
      onclick={() => envQuery.refetch()}
      disabled={envQuery.isFetching}
    >
      <RefreshCw class="h-4 w-4 {envQuery.isFetching ? 'animate-spin' : ''}" />
      Refresh
    </Button>
  </div>

  {#if envQuery.isPending}
    <PageLoading message="Reading .env files…" />
  {:else if envQuery.isError}
    <PageError
      title="Failed to read .env files"
      message={envQuery.error instanceof Error ? envQuery.error.message : 'Unknown error'}
      onRetry={() => envQuery.refetch()}
    />
  {:else if projectEnv}
    {#if projectEnv.files.length === 0}
      <PageEmpty
        title="No .env files found"
        description="No .env files found in this project's folder."
        icon={FileKey}
      />
    {:else}
      {#each projectEnv.files as file (file.name)}
        <Card>
          <CardHeader>
            <CardTitle class="font-mono text-sm">{file.name}</CardTitle>
          </CardHeader>
          <CardContent>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Key</TableHead>
                  <TableHead>Value</TableHead>
                  <TableHead>Link</TableHead>
                  <TableHead class="w-10"></TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {#each file.vars as envVar, i (envVar.key + ':' + i)}
                  {@const id = `${file.name}:${envVar.key}`}
                  {@const isRevealed = revealed.has(id)}
                  <TableRow>
                    <TableCell class="font-mono">{envVar.key}</TableCell>
                    <TableCell>
                      <div class="flex items-center gap-2">
                        {#if isRevealed}
                          <span class="font-mono tabular-nums">{envVar.value}</span>
                        {:else}
                          <span class="font-mono text-muted-foreground">••••••••</span>
                        {/if}
                        <Button
                          variant="ghost"
                          size="icon-sm"
                          aria-label={isRevealed
                            ? `Hide value of ${envVar.key}`
                            : `Reveal value of ${envVar.key}`}
                          onclick={() => toggleReveal(id)}
                        >
                          {#if isRevealed}
                            <EyeOff class="h-3.5 w-3.5" />
                          {:else}
                            <Eye class="h-3.5 w-3.5" />
                          {/if}
                        </Button>
                      </div>
                    </TableCell>
                    <TableCell>
                      {#if envVar.link}
                        {@const badge = driftBadge(envVar.link.state)}
                        <div class="flex flex-wrap items-center gap-1.5">
                          <Badge variant="secondary">→ {envVar.link.credentialName}</Badge>
                          {#if envVar.link.state === 'unknown'}
                            <Button
                              variant="ghost"
                              size="sm"
                              class="h-auto gap-1 p-0"
                              onclick={handleUnlockToCompare}
                            >
                              <Badge variant={badge.variant} class={badge.class}>
                                {badge.label}
                              </Badge>
                            </Button>
                          {:else}
                            <Badge variant={badge.variant} class={badge.class}>{badge.label}</Badge>
                          {/if}
                        </div>
                      {/if}
                    </TableCell>
                    <TableCell>
                      <DropdownMenu>
                        <DropdownMenuTrigger>
                          {#snippet child({ props })}
                            <Button
                              {...props}
                              variant="ghost"
                              size="icon-lg"
                              aria-label={`Actions for ${envVar.key}`}
                            >
                              <Ellipsis class="h-4 w-4" />
                            </Button>
                          {/snippet}
                        </DropdownMenuTrigger>
                        <DropdownMenuContent align="end">
                          {#if envVar.link}
                            <DropdownMenuItem onclick={() => handleCopyShared(envVar.link!)}>
                              <Copy class="mr-2 h-4 w-4" />
                              Copy shared value
                            </DropdownMenuItem>
                            <DropdownMenuItem onclick={() => handleUpdateSharedFromFile(envVar)}>
                              <PenToSquare class="mr-2 h-4 w-4" />
                              Update shared credential from this file value
                            </DropdownMenuItem>
                            <DropdownMenuItem onclick={() => handleResync(envVar.link!)}>
                              <Check class="mr-2 h-4 w-4" />
                              Acknowledge (re-sync)
                            </DropdownMenuItem>
                            <DropdownMenuSeparator />
                            <DropdownMenuItem
                              variant="destructive"
                              onclick={() => handleUnlink(envVar.link!, envVar.key)}
                            >
                              <UnlinkIcon class="mr-2 h-4 w-4" />
                              Unlink
                            </DropdownMenuItem>
                          {:else}
                            <DropdownMenuItem
                              onclick={() =>
                                (linkDialogTarget = { envKey: envVar.key, fileName: file.name })}
                            >
                              <LinkIcon class="mr-2 h-4 w-4" />
                              Link to shared credential…
                            </DropdownMenuItem>
                            <DropdownMenuItem
                              onclick={() =>
                                (createFor = {
                                  envKey: envVar.key,
                                  fileName: file.name,
                                  value: envVar.value,
                                })}
                            >
                              <Plus class="mr-2 h-4 w-4" />
                              Create shared credential from this value…
                            </DropdownMenuItem>
                          {/if}
                        </DropdownMenuContent>
                      </DropdownMenu>
                    </TableCell>
                  </TableRow>
                {/each}
              </TableBody>
            </Table>
          </CardContent>
        </Card>
      {/each}
    {/if}

    {#if projectEnv.orphanLinks.length > 0}
      <Card>
        <CardHeader>
          <CardTitle class="text-sm">Orphan links</CardTitle>
          <p class="text-xs text-muted-foreground">No longer in any .env file.</p>
        </CardHeader>
        <CardContent class="space-y-2">
          {#each projectEnv.orphanLinks as orphan (orphan.link.id)}
            <div class="flex items-center justify-between gap-2 rounded-md border px-3 py-2">
              <span class="truncate text-sm">
                <span class="font-mono">{orphan.envKey}</span>
                <span class="text-muted-foreground">→ {orphan.link.credentialName}</span>
              </span>
              <Button
                variant="outline"
                size="sm"
                onclick={() => handleUnlink(orphan.link, orphan.envKey)}
              >
                <UnlinkIcon class="mr-2 h-3.5 w-3.5" />
                Unlink
              </Button>
            </div>
          {/each}
        </CardContent>
      </Card>
    {/if}
  {/if}
</div>

{#if linkDialogTarget}
  <LinkCredentialDialog
    {projectId}
    envKey={linkDialogTarget.envKey}
    fileName={linkDialogTarget.fileName}
    onClose={() => (linkDialogTarget = null)}
  />
{/if}

{#if createFor}
  <CredentialForm
    initialName={createFor.envKey}
    initialValue={createFor.value}
    onSave={handleCredentialCreated}
    onClose={() => (createFor = null)}
  />
{/if}
