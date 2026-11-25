<script lang="ts">
  import { createMutation, useQueryClient } from '@tanstack/svelte-query';
  import { Badge, type BadgeVariant } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import * as Table from '$lib/components/ui/table';
  import { PageHeader, PageLoading, PageError, PageEmpty } from '$lib/components/shell';
  import PageContainer from '$lib/components/shell/page-container.svelte';
  import { relaunch } from '@tauri-apps/plugin-process';
  import { toast } from '$lib/utils/toast';
  import { confirmAction } from '$lib/utils/confirm';
  import { queryKeys } from '$lib/domains/shared/query/keys';
  import { pullSync, pushSync } from '../api/syncApi';
  import {
    createSyncConfigQuery,
    createSyncStatusQuery,
    createSyncVersionsQuery,
  } from '../queries/syncQueries';
  import { formatAbsolute, formatBytes, formatRelative, syncVerdict } from '../utils/syncVerdict';
  import type { SyncVerdict } from '../types';
  import ConfigDialog from './ConfigDialog.svelte';
  import PasswordDialog from './PasswordDialog.svelte';
  import ConflictDialog from './ConflictDialog.svelte';
  import UnresolvedProjects from './UnresolvedProjects.svelte';

  const client = useQueryClient();
  const config = createSyncConfigQuery();
  const status = createSyncStatusQuery();
  const versions = createSyncVersionsQuery(() => status.data?.configured === true);

  const verdictUi: Record<SyncVerdict, { label: string; variant: BadgeVariant }> = {
    'in-sync': { label: 'In sync', variant: 'default' },
    'cloud-newer': { label: 'Cloud is newer', variant: 'secondary' },
    'local-newer': { label: 'This machine is newer', variant: 'secondary' },
    diverged: { label: 'Both changed - conflict', variant: 'destructive' },
    never: { label: 'Never synced', variant: 'outline' },
    unknown: { label: 'Cloud unreachable', variant: 'destructive' },
  };

  let configOpen = $state(false);
  let pwOpen = $state(false);
  let pwMode = $state<'backup' | 'restore'>('backup');
  let pwVersion = $state<number | undefined>(undefined);
  let conflictOpen = $state(false);
  let conflict = $state<{
    local: { modifiedAt: string | null; size: number; baseVersion: number };
    cloud: { version: number; updatedAt: string; size: number; deviceId: string };
  } | null>(null);
  let password = '';
  let restartRequired = $state(false);
  let restoredCount = $state(0);

  const verdict = $derived(status.data ? syncVerdict(status.data) : 'unknown');
  const firstBackup = $derived(!status.data?.cloud && !status.data?.cloudError);

  const refresh = () => client.invalidateQueries({ queryKey: queryKeys.sync.all });

  function openPassword(mode: 'backup' | 'restore', version?: number) {
    pwMode = mode;
    pwVersion = version;
    pwOpen = true;
  }

  const push = createMutation(() => ({
    mutationFn: (v: { pw: string; force: boolean }) => pushSync(v.pw, v.force),
    onSuccess: async (r) => {
      if (r.status === 'conflict') {
        conflict = {
          local: {
            modifiedAt: r.local.localDbModifiedAt,
            size: r.local.localDbSize,
            baseVersion: r.local.baseVersion,
          },
          cloud: {
            version: r.current.version,
            updatedAt: r.current.clientUpdatedAt,
            size: r.current.size,
            deviceId: r.current.deviceId,
          },
        };
        pwOpen = false;
        conflictOpen = true;
        return;
      }
      pwOpen = false;
      conflictOpen = false;
      toast.success(`Backed up as version ${r.record.version}`);
      await refresh();
    },
  }));

  const pull = createMutation(() => ({
    mutationFn: (v: { pw: string; version?: number }) => pullSync(v.pw, v.version),
    onSuccess: async (r) => {
      pwOpen = false;
      conflictOpen = false;
      restartRequired = true;
      restoredCount = r.unresolvedCount;
      toast.success(`Restored version ${r.record.version}`);
      await refresh();
    },
  }));

  async function submitPassword(pw: string) {
    password = pw;
    if (pwMode === 'backup') {
      await push.mutateAsync({ pw, force: false });
      return;
    }
    const ok = await confirmAction(
      'Restoring replaces the data on this machine with the cloud copy. A copy of the current database is kept as a .pre-restore.bak file. Project paths from other machines are kept; this machine re-resolves them afterwards.',
      'Replace local data?',
      { confirmLabel: 'Restore', destructive: true }
    );
    if (!ok) return;
    await pull.mutateAsync({ pw, version: pwVersion });
  }

  const busy = $derived(push.isPending || pull.isPending);

  async function keepLocal() {
    try {
      await push.mutateAsync({ pw: password, force: true });
    } catch (e) {
      toast.error(String(e));
    }
  }
  async function takeCloud() {
    try {
      await pull.mutateAsync({ pw: password });
    } catch (e) {
      toast.error(String(e));
    }
  }
</script>

<PageContainer>
  <div class="space-y-4">
    <PageHeader
      title="Online sync"
      description="Back up to your own server and restore on another PC. One device at a time."
    >
      {#snippet actions()}
        <Button variant="outline" onclick={() => (configOpen = true)}>Server settings</Button>
      {/snippet}
    </PageHeader>

    {#if status.isPending || config.isPending}
      <PageLoading message="Loading sync status..." />
    {:else if status.isError}
      <PageError message={String(status.error)} onRetry={() => status.refetch()} />
    {:else if status.data && !status.data.configured}
      <PageEmpty
        title="Sync server not configured"
        description="Add your server URL and access token to start backing up."
      />
      <div class="flex justify-center">
        <Button onclick={() => (configOpen = true)}>Configure server</Button>
      </div>
    {:else if status.data}
      {@const s = status.data}
      {#if restartRequired}
        <Card class="border-primary">
          <CardContent class="flex flex-wrap items-center justify-between gap-2 py-4 text-sm">
            <div>
              <strong>Restart Portal to finish restoring.</strong> Your current data is kept as
              portal_desktop.db.pre-restore.bak.
              {#if restoredCount > 0}
                {restoredCount} project{restoredCount === 1 ? ' has' : 's have'} no folder on this machine
                yet; you can sort them out after the restart.
              {/if}
            </div>
            <Button size="sm" onclick={() => relaunch()}>Restart now</Button>
          </CardContent>
        </Card>
      {/if}

      <Card>
        <CardHeader>
          <div class="flex flex-wrap items-center justify-between gap-2">
            <CardTitle>Status</CardTitle>
            <Badge variant={verdictUi[verdict].variant}>{verdictUi[verdict].label}</Badge>
          </div>
        </CardHeader>
        <CardContent class="space-y-4">
          <div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
            <div class="rounded-md border p-3 text-sm">
              <div class="mb-1 flex items-center justify-between font-medium">
                This machine
                <Badge variant="outline">Synced v{s.baseVersion}</Badge>
              </div>
              <div>{formatAbsolute(s.localDbModifiedAt)}</div>
              <div class="text-muted-foreground">{formatRelative(s.localDbModifiedAt)}</div>
              <div>{formatBytes(s.localDbSize)}</div>
              <div class="truncate text-xs text-muted-foreground" title={s.deviceId}>
                {s.deviceId} - synced v{s.baseVersion}
                {s.lastSyncedAt ? `at ${formatAbsolute(s.lastSyncedAt)}` : ''}
              </div>
            </div>
            <div class="rounded-md border p-3 text-sm">
              <div class="mb-1 flex items-center justify-between font-medium">
                Cloud
                {#if s.cloud}<Badge>Latest v{s.cloud.version}</Badge>{/if}
              </div>
              {#if s.cloud}
                <div>{formatAbsolute(s.cloud.createdAt || s.cloud.clientUpdatedAt)}</div>
                <div class="text-muted-foreground">
                  uploaded {formatRelative(s.cloud.createdAt || s.cloud.clientUpdatedAt)}
                </div>
                <div>{formatBytes(s.cloud.size)}</div>
                <div class="truncate text-xs text-muted-foreground" title={s.cloud.deviceId}>
                  from {s.cloud.deviceId}
                </div>
              {:else if s.cloudError}
                <div class="text-destructive">{s.cloudError}</div>
              {:else}
                <div class="text-muted-foreground">No backup yet</div>
              {/if}
            </div>
          </div>
          <div class="flex flex-wrap gap-2">
            <Button disabled={busy} onclick={() => openPassword('backup')}>Back up to cloud</Button>
            <Button
              variant="outline"
              disabled={busy || !s.cloud}
              onclick={() => openPassword('restore')}
            >
              Restore from cloud
            </Button>
          </div>
        </CardContent>
      </Card>

      <UnresolvedProjects />

      <Card>
        <CardHeader><CardTitle>Version history</CardTitle></CardHeader>
        <CardContent>
          {#if versions.isPending}
            <PageLoading message="Loading versions..." />
          {:else if versions.isError}
            <PageError message={String(versions.error)} onRetry={() => versions.refetch()} />
          {:else if (versions.data ?? []).length === 0}
            <p class="text-sm text-muted-foreground">No versions yet.</p>
          {:else}
            <Table.Root>
              <Table.Header>
                <Table.Row>
                  <Table.Head>Version</Table.Head>
                  <Table.Head>Uploaded</Table.Head>
                  <Table.Head>Device</Table.Head>
                  <Table.Head>Size</Table.Head>
                  <Table.Head class="w-0"></Table.Head>
                </Table.Row>
              </Table.Header>
              <Table.Body>
                {#each [...(versions.data ?? [])].sort((a, b) => b.version - a.version) as v, i (v.version)}
                  <Table.Row class={i === 0 ? 'bg-primary/5' : ''}>
                    <Table.Cell>
                      <div class="flex flex-wrap items-center gap-1.5">
                        <span class="font-medium">v{v.version}</span>
                        {#if i === 0}<Badge>Latest</Badge>{/if}
                        {#if v.version === s.baseVersion}<Badge variant="outline"
                            >This machine</Badge
                          >{/if}
                      </div>
                    </Table.Cell>
                    <Table.Cell>
                      <div>{formatAbsolute(v.createdAt || v.clientUpdatedAt)}</div>
                      <div class="text-xs text-muted-foreground">
                        {formatRelative(v.createdAt || v.clientUpdatedAt)}
                      </div>
                    </Table.Cell>
                    <Table.Cell class="max-w-40 truncate" title={v.deviceId}
                      >{v.deviceId}</Table.Cell
                    >
                    <Table.Cell>{formatBytes(v.size)}</Table.Cell>
                    <Table.Cell>
                      <Button
                        size="sm"
                        variant="ghost"
                        disabled={busy}
                        onclick={() => openPassword('restore', v.version)}
                      >
                        Restore
                      </Button>
                    </Table.Cell>
                  </Table.Row>
                {/each}
              </Table.Body>
            </Table.Root>
          {/if}
        </CardContent>
      </Card>
    {/if}
  </div>
</PageContainer>

<ConfigDialog
  bind:open={configOpen}
  serverUrl={config.data?.serverUrl ?? ''}
  hasToken={config.data?.hasToken ?? false}
/>
<PasswordDialog
  bind:open={pwOpen}
  mode={pwMode}
  confirmTwice={pwMode === 'backup' && firstBackup}
  versionLabel={pwVersion ? ` (v${pwVersion})` : ''}
  onSubmit={submitPassword}
/>
{#if conflict}
  <ConflictDialog
    bind:open={conflictOpen}
    local={conflict.local}
    cloud={conflict.cloud}
    {busy}
    onKeepLocal={keepLocal}
    onTakeCloud={takeCloud}
  />
{/if}
