<!--
	Per-project .env viewer: live off disk every load, one tab per .env file
	(plus "All" overview), copy/move/edit values between files, and optional
	linking to shared vault credentials so drift shows up as a badge.
-->
<script lang="ts">
  import { createProjectEnvQuery } from '../queries/projectEnvQueries';
  import {
    linkProjectEnv,
    unlinkProjectEnv,
    resyncProjectEnvLink,
    addProjectEnvFile,
    removeProjectEnvFile,
    setProjectEnvVar,
  } from '../api/projectEnvApi';
  import type { EnvLinkView, EnvVarView } from '../generated';
  import ProjectEnvRow from './ProjectEnvRow.svelte';
  import ProjectEnvOverview from './ProjectEnvOverview.svelte';
  import ProjectEnvFileDialog from './ProjectEnvFileDialog.svelte';
  import ProjectEnvValueDialog from './ProjectEnvValueDialog.svelte';
  import LinkCredentialDialog from './LinkCredentialDialog.svelte';
  import { queryClient } from '$lib/domains/shared/query/queryClient';
  import { queryKeys } from '$lib/domains/shared/query/keys';
  import { credentialService, vaultGate, CredentialForm } from '$lib/domains/credentials';
  import type { Credential } from '$lib/domains/credentials/types';
  import { confirmAction } from '$lib/utils/confirm';
  import { toast } from '$lib/utils/toast';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Table, TableBody, TableHead, TableHeader, TableRow } from '$lib/components/ui/table';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import { PageEmpty, PageError, PageLoading } from '$lib/components/shell';
  import RefreshCw from '~icons/fa6-solid/rotate';
  import UnlinkIcon from '~icons/fa6-solid/link-slash';
  import FileKey from '~icons/fa6-solid/key';
  import PlusIcon from '~icons/fa6-solid/plus';

  interface Props {
    projectId: number;
    projectPath?: string;
    enabled?: boolean;
  }

  const { projectId, projectPath, enabled = true }: Props = $props();

  const envQuery = createProjectEnvQuery(
    () => projectId,
    () => enabled
  );
  const projectEnv = $derived(envQuery.data);

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

  /** 'all' or a file name. */
  let scope = $state('all');
  const files = $derived(projectEnv?.files ?? []);
  const activeFile = $derived(files.find((f) => f.name === scope) ?? null);
  // A removed file falls back to the overview instead of an empty pane.
  $effect(() => {
    if (scope !== 'all' && !activeFile && files.length) scope = 'all';
  });

  let addFileOpen = $state(false);
  let editTarget = $state<{ fileName: string; key: string; value: string } | null>(null);

  async function run(action: () => Promise<void>, ok: string) {
    try {
      await action();
      await invalidate();
      toast.success(ok);
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed');
    }
  }

  async function handleAddFile(path: string) {
    try {
      const rel = await addProjectEnvFile(projectId, path);
      await invalidate();
      addFileOpen = false;
      scope = rel;
      toast.success(`Linked ${rel}`);
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to link file');
    }
  }

  const handleRemoveFile = (name: string) =>
    run(() => removeProjectEnvFile(projectId, name), `Stopped showing ${name}`);

  const handleCopy = (v: EnvVarView, from: string, to: string, move: boolean) =>
    run(
      async () => {
        await setProjectEnvVar(projectId, to, v.key, v.value);
        if (move) await setProjectEnvVar(projectId, from, v.key, null);
      },
      `${move ? 'Moved' : 'Copied'} ${v.key} to ${to}`
    );

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
      Read from disk each time. Portal only writes a .env line when you edit, copy or move a value.
    </p>
    <div class="flex items-center gap-1">
      <Button variant="outline" size="sm" class="gap-2" onclick={() => (addFileOpen = true)}>
        <PlusIcon class="h-3.5 w-3.5" />
        Link file
      </Button>
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
    {#if files.length === 0}
      <PageEmpty
        title="No .env files found"
        description="None in the project root. Use “Link file” for one in a sub-folder."
        icon={FileKey}
      />
    {:else}
      <div class="flex flex-wrap items-center gap-1.5">
        <Button
          size="sm"
          variant={scope === 'all' ? 'default' : 'outline'}
          onclick={() => (scope = 'all')}
        >
          All
        </Button>
        {#each files as f (f.name)}
          <Button
            size="sm"
            variant={scope === f.name ? 'default' : 'outline'}
            class="font-mono"
            onclick={() => (scope = f.name)}
          >
            {f.name}
            <Badge variant="secondary" class="ml-1.5 px-1.5">{f.vars.length}</Badge>
          </Button>
        {/each}
      </div>

      <Card>
        <CardContent>
          {#if activeFile}
            <div class="mb-3 flex items-center justify-between gap-2">
              <span class="truncate font-mono text-sm">{activeFile.name}</span>
              {#if activeFile.linked}
                <Button variant="ghost" size="sm" onclick={() => handleRemoveFile(activeFile.name)}>
                  <UnlinkIcon class="mr-2 h-3.5 w-3.5" />
                  Stop showing
                </Button>
              {/if}
            </div>
            {#if activeFile.vars.length === 0}
              <p class="py-6 text-center text-sm text-muted-foreground">
                No variables in this file.
              </p>
            {:else}
              {@const otherFiles = files
                .filter((f) => f.name !== activeFile.name)
                .map((f) => f.name)}
              <Table class="table-fixed">
                <TableHeader>
                  <TableRow>
                    <TableHead class="w-[30%]">Key</TableHead>
                    <TableHead>Value</TableHead>
                    <TableHead class="w-[24%]">Link</TableHead>
                    <TableHead class="w-12"></TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {#each activeFile.vars as envVar, i (envVar.key + ':' + i)}
                    {@const id = `${activeFile.name}:${envVar.key}`}
                    <ProjectEnvRow
                      {envVar}
                      fileName={activeFile.name}
                      {otherFiles}
                      onCopyTo={(t) => handleCopy(envVar, activeFile.name, t, false)}
                      onMoveTo={(t) => handleCopy(envVar, activeFile.name, t, true)}
                      onEdit={() =>
                        (editTarget = {
                          fileName: activeFile.name,
                          key: envVar.key,
                          value: envVar.value,
                        })}
                      isRevealed={revealed.has(id)}
                      onToggleReveal={() => toggleReveal(id)}
                      onUnlockToCompare={handleUnlockToCompare}
                      onCopyShared={handleCopyShared}
                      onUpdateShared={handleUpdateSharedFromFile}
                      onResync={handleResync}
                      onUnlink={handleUnlink}
                      onLink={(t) => (linkDialogTarget = t)}
                      onCreate={(t) => (createFor = t)}
                    />
                  {/each}
                </TableBody>
              </Table>
            {/if}
          {:else}
            <ProjectEnvOverview {files} onSelect={(n) => (scope = n)} />
          {/if}
        </CardContent>
      </Card>
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

{#if addFileOpen}
  <ProjectEnvFileDialog
    {projectPath}
    onSubmit={handleAddFile}
    onClose={() => (addFileOpen = false)}
  />
{/if}

{#if editTarget}
  <ProjectEnvValueDialog
    envKey={editTarget.key}
    fileName={editTarget.fileName}
    value={editTarget.value}
    onSubmit={async (v) => {
      const t = editTarget!;
      await run(() => setProjectEnvVar(projectId, t.fileName, t.key, v), 'Saved');
      editTarget = null;
    }}
    onClose={() => (editTarget = null)}
  />
{/if}

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
