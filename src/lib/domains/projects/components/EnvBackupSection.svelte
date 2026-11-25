<!--
	Settings > Vault > Project env files. Backs up every project's `.env*` files into the vault
	(each one an encrypted credential linked to its project) and writes them back on this or
	another machine. Restore never replaces a differing file without asking for that file.
	Design: docs/architecture/env-vault-backup.md
-->

<script lang="ts">
  import { Badge, type BadgeVariant } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
  } from '$lib/components/ui/table';
  import { PageError, PageLoading } from '$lib/components/shell';
  import SettingsSection from '$lib/domains/settings/components/shared/SettingsSection.svelte';
  import { vaultGate } from '$lib/domains/credentials';
  import { queryClient } from '$lib/domains/shared/query/queryClient';
  import { queryKeys } from '$lib/domains/shared/query/keys';
  import { confirmAction } from '$lib/utils/confirm';
  import { getErrorMessage } from '$lib/utils/errorMessage';
  import { toast } from '$lib/utils/toast';
  import FileLines from '~icons/fa6-solid/file-lines';
  import { importEnvFiles, restoreEnvFiles } from '../api/envBackupApi';
  import { createEnvBackupScanQuery } from '../queries/envBackupQueries';
  import type { EnvBackupFile, EnvFileState, EnvItemResult, EnvOutcome } from '../generated';

  interface Props {
    /** When set, scope the whole view to this one project (its id in `scan.files`). */
    projectId?: number;
    /** Required with `projectId` to scope the "not backed up" footer, which is a string. */
    projectName?: string;
    title?: string;
    description?: string;
  }
  let {
    projectId,
    projectName,
    title = 'Project env files',
    description = "Back up each project's .env files into the vault, encrypted, and write them back on this or another machine. They are never sent anywhere unencrypted.",
  }: Props = $props();

  const stateUi: Record<EnvFileState, { label: string; variant: BadgeVariant }> = {
    new: { label: 'Not backed up', variant: 'secondary' },
    inSync: { label: 'In sync', variant: 'outline' },
    differs: { label: 'Differs', variant: 'default' },
    onlyInVault: { label: 'Only in vault', variant: 'secondary' },
    unknown: { label: 'Vault locked', variant: 'outline' },
  };

  const outcomeUi: Record<EnvOutcome, { label: string; variant: BadgeVariant }> = {
    created: { label: 'Backed up', variant: 'default' },
    updated: { label: 'Updated', variant: 'default' },
    unchanged: { label: 'Unchanged', variant: 'outline' },
    restored: { label: 'Restored', variant: 'default' },
    overwritten: { label: 'Overwritten', variant: 'default' },
    identical: { label: 'Already identical', variant: 'outline' },
    conflict: { label: 'Kept, differs', variant: 'secondary' },
    skipped: { label: 'Skipped', variant: 'secondary' },
    failed: { label: 'Failed', variant: 'destructive' },
  };

  const scanQuery = createEnvBackupScanQuery();
  const scan = $derived(scanQuery.data);

  let filter = $state('');
  let busy = $state(false);
  /** The last run, minus the rows where nothing happened. */
  let lastRun = $state<EnvItemResult[]>([]);

  const files = $derived(
    (scan?.files ?? []).filter((f) => projectId == null || f.projectId === projectId)
  );
  /** `scan.skipped` is `projectName/rel (why)` strings; scope by name prefix for one project. */
  const skipped = $derived(
    (scan?.skipped ?? []).filter(
      (s) => projectId == null || (!!projectName && s.startsWith(`${projectName}/`))
    )
  );
  const rows = $derived.by(() => {
    const q = filter.trim().toLowerCase();
    return q
      ? files.filter((f) => `${f.projectName}/${f.relPath}`.toLowerCase().includes(q))
      : files;
  });
  /** Not in the vault yet: backing these up can never replace a vault copy. */
  const fresh = $derived(files.filter((f) => f.state === 'new'));
  /** Only in the vault, with a project folder here: restoring these can never replace a file. */
  const missing = $derived(files.filter((f) => f.state === 'onlyInVault' && f.projectPath !== ''));

  /** Which side was written last, when both exist and differ. */
  function newerSide(f: EnvBackupFile): string | null {
    if (f.state !== 'differs' || !f.diskModified || !f.vaultUpdatedAt) return null;
    return Date.parse(f.diskModified) >= Date.parse(f.vaultUpdatedAt)
      ? 'File is newer'
      : 'Vault copy is newer';
  }

  function summarize(items: EnvItemResult[]): string {
    const counts = new Map<EnvOutcome, number>();
    for (const i of items) counts.set(i.outcome, (counts.get(i.outcome) ?? 0) + 1);
    return [...counts]
      .map(([outcome, n]) => `${n} ${outcomeUi[outcome].label.toLowerCase()}`)
      .join(', ');
  }

  function finish(items: EnvItemResult[]) {
    lastRun = items.filter((i) => i.outcome !== 'unchanged' && i.outcome !== 'identical');
    const text = items.length ? summarize(items) : 'Nothing to do';
    if (items.some((i) => i.outcome === 'failed')) toast.error('Some files failed', text);
    else toast.success('Env files', text);
  }

  /** Unlock first (a clean prompt, not a command that waits on one), then run. */
  async function guarded(reason: string, work: () => Promise<void>) {
    if (busy) return;
    busy = true;
    try {
      if (!(await vaultGate.ensureUnlocked(reason))) return;
      await work();
    } catch (err) {
      toast.error(getErrorMessage(err, 'Env backup failed.'));
    } finally {
      busy = false;
      void queryClient.invalidateQueries({ queryKey: queryKeys.projects.envBackup });
      void queryClient.invalidateQueries({ queryKey: queryKeys.credentials.status });
    }
  }

  const label = (f: EnvBackupFile) => `${f.projectName}/${f.relPath}`;
  const newerNote = (f: EnvBackupFile) => {
    const side = newerSide(f);
    return side ? ` (${side.toLowerCase()})` : '';
  };

  /**
   * Runs `call` without overwriting, then asks about each conflict (a differing copy on the
   * other side) and repeats only that one with `overwrite`. Nothing is replaced unasked.
   * Conflicts are matched back to their row by project id, not by name (a name can change).
   */
  async function withConfirmedConflicts(
    targets: EnvBackupFile[],
    call: (rows: EnvBackupFile[], overwrite: boolean) => Promise<EnvItemResult[]>,
    ask: (row: EnvBackupFile) => { title: string; message: string; confirmLabel: string }
  ): Promise<EnvItemResult[]> {
    const first = await call(targets, false);
    const results = first.filter((i) => i.outcome !== 'conflict');
    for (const conflict of first.filter((i) => i.outcome === 'conflict')) {
      const row = targets.find(
        (t) => t.projectId === conflict.projectId && t.relPath === conflict.relPath
      );
      if (row) {
        const { title, message, confirmLabel } = ask(row);
        if (await confirmAction(message, title, { confirmLabel, destructive: true })) {
          results.push(...(await call([row], true)));
          continue;
        }
      }
      results.push(conflict);
    }
    return results;
  }

  /** Disk to vault. A vault copy that differs is replaced only after the user confirms it. */
  const backUp = (targets: EnvBackupFile[]) =>
    guarded('Unlock to back up env files.', async () => {
      finish(
        await withConfirmedConflicts(
          targets,
          async (rows, overwrite) =>
            (
              await importEnvFiles(
                rows.map((r) => ({ projectId: r.projectId, relPath: r.relPath, overwrite }))
              )
            ).items,
          (row) => ({
            title: 'Replace vault copy?',
            message: `The vault copy of ${label(row)} differs from the file on disk${newerNote(row)}. Replacing it loses the vault copy.`,
            confirmLabel: 'Replace vault copy',
          })
        )
      );
    });

  /** Vault to disk. A file that differs is replaced only after the user confirms it. */
  const restore = (targets: EnvBackupFile[]) =>
    guarded('Unlock to restore env files.', async () => {
      finish(
        await withConfirmedConflicts(
          targets,
          async (rows, overwrite) =>
            (
              await restoreEnvFiles(
                rows.flatMap((r) =>
                  r.credentialId ? [{ credentialId: r.credentialId, overwrite }] : []
                )
              )
            ).items,
          (row) => ({
            title: 'Replace file?',
            message: `${label(row)} on disk differs from the vault copy${newerNote(row)}. Replacing it loses what is on disk now.`,
            confirmLabel: 'Replace with vault copy',
          })
        )
      );
    });
</script>

<SettingsSection {title} {description} icon={FileLines}>
  {#if scanQuery.isPending}
    <PageLoading message="Scanning projects..." />
  {:else if !scan}
    <PageError
      message={getErrorMessage(scanQuery.error, 'Failed to scan project env files.')}
      onRetry={() => scanQuery.refetch()}
    />
  {:else}
    {#if scan.vaultLocked && files.some((f) => f.state === 'unknown')}
      <div
        class="flex flex-wrap items-center justify-between gap-3 rounded-md border bg-muted/40 p-3 text-sm"
      >
        <span>The vault is locked, so files on both sides cannot be compared yet.</span>
        <Button
          size="sm"
          variant="outline"
          onclick={async () => {
            if (await vaultGate.ensureUnlocked('Unlock to compare env files.'))
              void scanQuery.refetch();
          }}
        >
          Unlock
        </Button>
      </div>
    {/if}

    <div class="flex flex-wrap items-center gap-2">
      <Input
        class="max-w-xs"
        placeholder="Filter by project or file"
        aria-label="Filter env files"
        bind:value={filter}
      />
      <div class="ml-auto flex flex-wrap gap-2">
        <Button variant="outline" size="sm" disabled={busy} onclick={() => scanQuery.refetch()}>
          Rescan
        </Button>
        <Button
          variant="outline"
          size="sm"
          disabled={busy || missing.length === 0}
          onclick={() => restore(missing)}
        >
          Restore missing ({missing.length})
        </Button>
        <Button size="sm" disabled={busy || fresh.length === 0} onclick={() => backUp(fresh)}>
          Back up new files ({fresh.length})
        </Button>
      </div>
    </div>

    {#if files.length === 0}
      <p class="text-sm text-muted-foreground">
        {projectId == null
          ? 'No .env files found in your projects, and none in the vault.'
          : 'No .env files in this project, and none in the vault.'}
      </p>
    {:else if rows.length === 0}
      <p class="text-sm text-muted-foreground">Nothing matches that filter.</p>
    {:else}
      <div class="overflow-x-auto rounded-md border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Project</TableHead>
              <TableHead>File</TableHead>
              <TableHead>State</TableHead>
              <TableHead class="text-right">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {#each rows as row, i (`${row.projectId}:${row.relPath}`)}
              {@const ui = stateUi[row.state]}
              {@const hint = newerSide(row)}
              <TableRow>
                <TableCell class="font-medium">
                  {i === 0 || rows[i - 1].projectId !== row.projectId ? row.projectName : ''}
                </TableCell>
                <TableCell class="font-mono text-xs">{row.relPath}</TableCell>
                <TableCell>
                  <Badge variant={ui.variant}>{ui.label}</Badge>
                  {#if hint}<span class="ml-2 text-xs text-muted-foreground">{hint}</span>{/if}
                </TableCell>
                <TableCell class="text-right">
                  <div class="flex justify-end gap-2">
                    {#if row.diskBytes !== null && row.state !== 'inSync'}
                      <Button
                        size="sm"
                        variant="outline"
                        disabled={busy}
                        onclick={() => backUp([row])}
                      >
                        Back up
                      </Button>
                    {/if}
                    {#if row.credentialId && row.projectPath && row.state !== 'inSync'}
                      <Button
                        size="sm"
                        variant="outline"
                        disabled={busy}
                        onclick={() => restore([row])}
                      >
                        Restore
                      </Button>
                    {/if}
                  </div>
                </TableCell>
              </TableRow>
            {/each}
          </TableBody>
        </Table>
      </div>
    {/if}

    {#if lastRun.length > 0}
      <div class="space-y-2 rounded-md border p-3 text-sm" role="status">
        <div class="flex items-center justify-between">
          <span class="font-medium">Last run</span>
          <Button size="sm" variant="ghost" onclick={() => (lastRun = [])}>Dismiss</Button>
        </div>
        <ul class="space-y-1">
          {#each lastRun as item, i (i)}
            <li class="flex flex-wrap items-center gap-2">
              <Badge variant={outcomeUi[item.outcome].variant}>
                {outcomeUi[item.outcome].label}
              </Badge>
              <span class="font-mono text-xs">{item.projectName}/{item.relPath}</span>
              {#if item.detail}<span class="text-muted-foreground">{item.detail}</span>{/if}
            </li>
          {/each}
        </ul>
      </div>
    {/if}

    {#if skipped.length > 0}
      <p class="text-sm text-muted-foreground">
        Not backed up (not UTF-8 text, or over 1 MiB): {skipped.join(', ')}.
      </p>
    {/if}

    {#if projectId == null && scan.unmatched.length > 0}
      <p class="text-sm text-muted-foreground">
        {scan.unmatched.length} vault {scan.unmatched.length === 1
          ? 'entry belongs'
          : 'entries belong'}
        to projects that are not in Portal on this machine, so they cannot be restored here.
      </p>
    {/if}
  {/if}
</SettingsSection>
