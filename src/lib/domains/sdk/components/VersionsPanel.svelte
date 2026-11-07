<!--
	Versions across every SDK, grouped by the SDK they belong to.

	Used by the SDK overview's Versions tab and by /sdk/versions, which previously
	each carried their own copy of this flow.
-->

<script lang="ts">
  import { createQuery, useQueryClient } from '@tanstack/svelte-query';
  import { Card, CardContent } from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import { Alert, AlertDescription, AlertTitle } from '$lib/components/ui/alert';
  import { PageEmpty, PageError, PageLoading } from '$lib/components/shell';
  import CheckCircle from '~icons/fa6-solid/circle-check';
  import XCircle from '~icons/fa6-solid/circle-xmark';
  import Download from '~icons/fa6-solid/download';
  import Trash2 from '~icons/fa6-solid/trash';
  import Play from '~icons/fa6-solid/play';
  import Loader2 from '~icons/fa6-solid/spinner';
  import AlertCircle from '~icons/fa6-solid/circle-exclamation';
  import { sdkService } from '../services/sdkService';
  import { sdkConfigsQuery } from '../queries/sdkQueries';
  import {
    normalizeVersion,
    type RawSDKVersion,
    type SDKVersionRow,
  } from '../utils/normalizeVersions';
  import { queryKeys } from '$lib/domains/shared/query/keys';
  import { toast } from '$lib/utils/toast';
  import { confirmAction } from '$lib/utils/confirm';

  interface SDKGroup {
    sdkType: string;
    sdkName: string;
    /** The manager that owns this SDK's versions — activate/uninstall go through it. */
    manager: string | null;
    rows: SDKVersionRow[];
  }

  const queryClient = useQueryClient();

  // The SDK configs carry the managers, which the version actions need; they are
  // already cached by the sidebar, so this costs no extra probe pass.
  const configsQuery = createQuery(sdkConfigsQuery);

  const versionsQuery = createQuery(() => ({
    queryKey: queryKeys.sdk.versions('all'),
    enabled: !!configsQuery.data,
    queryFn: async (): Promise<{ groups: SDKGroup[]; failed: string[] }> => {
      const configs = configsQuery.data ?? [];
      const failed: string[] = [];
      const groups: SDKGroup[] = [];

      await Promise.all(
        configs.map(async (config) => {
          try {
            const raw = (await sdkService.fetchAvailableVersions(config.id)) as RawSDKVersion[];
            if (raw.length === 0) return; // no version feed for this SDK
            const manager = config.sdk_managers?.find(
              (m) => m.installed && m.supports_version_switching
            );
            groups.push({
              sdkType: config.id,
              sdkName: config.display_name,
              manager: manager?.id ?? null,
              rows: raw.map((v) => normalizeVersion(v, config.id, config.display_name)),
            });
          } catch {
            // A version feed being down must not make the SDK disappear silently.
            failed.push(config.display_name);
          }
        })
      );

      groups.sort((a, b) => a.sdkName.localeCompare(b.sdkName));
      return { groups, failed };
    },
    refetchOnWindowFocus: false,
  }));

  // Action state lives beside the query data, which TanStack owns.
  interface ActionState {
    busy?: 'install' | 'uninstall' | 'activate';
    error?: string;
  }
  let actions = $state<Record<string, ActionState>>({});

  const rowKey = (v: SDKVersionRow) => `${v.sdkType}@${v.version}`;

  const groups = $derived(versionsQuery.data?.groups ?? []);
  const failed = $derived(versionsQuery.data?.failed ?? []);
  const totalRows = $derived(groups.reduce((n, g) => n + g.rows.length, 0));

  function message(err: unknown, fallback: string) {
    return err instanceof Error ? err.message : String(err ?? fallback);
  }

  async function refresh() {
    await queryClient.invalidateQueries({ queryKey: ['sdk'] });
  }

  async function install(v: SDKVersionRow) {
    const key = rowKey(v);
    actions[key] = { busy: 'install' };
    try {
      await sdkService.downloadAndInstallVersion(v.sdkType, v.version, false);
      delete actions[key];
      toast.success(`${v.sdkName} ${v.version} installed`);
      await refresh();
    } catch (err) {
      const detail = message(err, 'Installation failed');
      actions[key] = { error: detail };
      toast.error(`Could not install ${v.sdkName} ${v.version}`, detail);
    }
  }

  async function uninstall(v: SDKVersionRow, manager: string) {
    const ok = await confirmAction(
      `Uninstall ${v.sdkName} ${v.version}? Projects pinned to this version will stop working until another one is active.`,
      `Uninstall ${v.sdkName} ${v.version}`,
      { confirmLabel: 'Uninstall', destructive: true }
    );
    if (!ok) return;

    const key = rowKey(v);
    actions[key] = { busy: 'uninstall' };
    try {
      await sdkService.uninstallVersionViaManager(manager, v.version);
      delete actions[key];
      toast.success(`${v.sdkName} ${v.version} uninstalled`);
      await refresh();
    } catch (err) {
      const detail = message(err, 'Uninstall failed');
      actions[key] = { error: detail };
      toast.error(`Could not uninstall ${v.sdkName} ${v.version}`, detail);
    }
  }

  async function activate(v: SDKVersionRow, manager: string) {
    const key = rowKey(v);
    actions[key] = { busy: 'activate' };
    try {
      await sdkService.switchVersionViaManager(manager, v.version);
      delete actions[key];
      toast.success(`${v.sdkName} ${v.version} is now active`);
      await refresh();
    } catch (err) {
      const detail = message(err, 'Could not switch version');
      actions[key] = { error: detail };
      toast.error(`Could not activate ${v.sdkName} ${v.version}`, detail);
    }
  }
</script>

{#if configsQuery.isPending || versionsQuery.isPending}
  <PageLoading message="Loading SDK versions..." />
{:else if configsQuery.error || versionsQuery.error}
  <PageError
    title="Failed to load SDK versions"
    message={message(configsQuery.error ?? versionsQuery.error, 'Unknown error')}
    onRetry={refresh}
  />
{:else}
  <div class="space-y-6">
    {#if failed.length > 0}
      <Alert variant="destructive">
        <AlertCircle class="h-4 w-4" />
        <AlertTitle>Some version lists could not be loaded</AlertTitle>
        <AlertDescription>
          {failed.join(', ')} — their releases could not be fetched, so no versions are listed below.
          Check your connection and refresh.
        </AlertDescription>
      </Alert>
    {/if}

    {#if totalRows === 0}
      <PageEmpty
        title="No SDK versions found"
        description="No installable versions were returned for any SDK."
        actionLabel="Refresh"
        onAction={refresh}
      />
    {:else}
      {#each groups as group (group.sdkType)}
        <section class="space-y-3">
          <div class="flex items-baseline gap-2">
            <h3 class="text-lg font-semibold">{group.sdkName}</h3>
            <span class="text-xs text-muted-foreground">
              {group.rows.filter((r) => r.installed).length} of {group.rows.length} installed
            </span>
          </div>

          <div class="space-y-2">
            {#each group.rows as version (version.version)}
              {@const state = actions[rowKey(version)]}
              <Card>
                <CardContent class="p-4">
                  <div class="flex flex-wrap items-center justify-between gap-3">
                    <div class="flex min-w-0 flex-wrap items-center gap-2">
                      <span class="font-mono text-sm font-medium">{version.version}</span>
                      {#if version.installed}
                        {#if version.active}
                          <Badge variant="default" class="bg-green-100 text-green-800">
                            <CheckCircle class="mr-1 h-3 w-3" />
                            Active
                          </Badge>
                        {:else}
                          <Badge variant="outline" class="text-green-600">
                            <CheckCircle class="mr-1 h-3 w-3" />
                            Installed
                          </Badge>
                        {/if}
                      {:else}
                        <Badge variant="outline" class="text-muted-foreground">
                          <XCircle class="mr-1 h-3 w-3" />
                          Not installed
                        </Badge>
                      {/if}
                      {#if version.releaseDate}
                        <span class="text-xs text-muted-foreground">
                          Released {version.releaseDate}
                        </span>
                      {/if}
                      {#if version.size}
                        <span class="text-xs text-muted-foreground">{version.size}</span>
                      {/if}
                    </div>

                    <div class="flex shrink-0 items-center gap-2">
                      {#if version.installed && !group.manager}
                        <span class="text-xs text-muted-foreground">
                          No version manager installed for {group.sdkName}
                        </span>
                      {:else if version.installed && group.manager}
                        {@const manager = group.manager}
                        {#if !version.active}
                          <Button
                            variant="outline"
                            size="sm"
                            disabled={!!state?.busy}
                            onclick={() => activate(version, manager)}
                          >
                            {#if state?.busy === 'activate'}
                              <Loader2 class="mr-1 h-4 w-4 animate-spin" />
                              Activating...
                            {:else}
                              <Play class="mr-1 h-4 w-4" />
                              Activate
                            {/if}
                          </Button>
                        {/if}
                        <Button
                          variant="outline"
                          size="sm"
                          disabled={!!state?.busy}
                          onclick={() => uninstall(version, manager)}
                        >
                          {#if state?.busy === 'uninstall'}
                            <Loader2 class="mr-1 h-4 w-4 animate-spin" />
                            Uninstalling...
                          {:else}
                            <Trash2 class="mr-1 h-4 w-4" />
                            Uninstall
                          {/if}
                        </Button>
                      {:else}
                        <Button size="sm" disabled={!!state?.busy} onclick={() => install(version)}>
                          {#if state?.busy === 'install'}
                            <Loader2 class="mr-1 h-4 w-4 animate-spin" />
                            Installing...
                          {:else}
                            <Download class="mr-1 h-4 w-4" />
                            {state?.error ? 'Retry install' : 'Install'}
                          {/if}
                        </Button>
                      {/if}
                    </div>
                  </div>

                  {#if state?.error}
                    <p class="mt-2 text-xs text-destructive">{state.error}</p>
                  {/if}
                </CardContent>
              </Card>
            {/each}
          </div>
        </section>
      {/each}
    {/if}
  </div>
{/if}
