<!--
	SDK Managers Page
	Shows all SDK managers and their installation status
-->

<script lang="ts">
  import { goto } from '$app/navigation';
  import { createQuery, useQueryClient } from '@tanstack/svelte-query';
  import { sdkManagersQuery } from '$lib/domains/sdk/queries/sdkQueries';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import CheckCircle from '~icons/fa6-solid/circle-check';
  import XCircle from '~icons/fa6-solid/circle-xmark';
  import Download from '~icons/fa6-solid/download';
  import ExternalLink from '~icons/fa6-solid/arrow-up-right-from-square';
  import RefreshCw from '~icons/fa6-solid/rotate';
  import ArrowRight from '~icons/fa6-solid/arrow-right';
  import Loader2 from '~icons/fa6-solid/spinner';
  import Devicon from '$lib/components/ui/devicon.svelte';
  import { PageLoading, PageError, PageEmpty } from '$lib/components/shell';
  import { toast } from '$lib/utils/toast';
  import { confirmAction } from '$lib/utils/confirm';
  import {
    installSDKManager,
    uninstallSDKManager,
    type SDKManagerInfo,
  } from '$lib/domains/sdk/services/sdkManagerService';

  const queryClient = useQueryClient();
  const managersQ = createQuery(sdkManagersQuery);
  const managers = $derived<SDKManagerInfo[]>(managersQ.data ?? []);
  const loading = $derived(managersQ.isPending);
  const error = $derived(
    managersQ.error
      ? managersQ.error instanceof Error
        ? managersQ.error.message
        : 'Failed to load SDK managers'
      : null
  );

  // Installs change the sidebar's data too, so refetch everything under `sdk`.
  function loadManagers() {
    return queryClient.invalidateQueries({ queryKey: ['sdk'] });
  }

  let installingManager = $state<string | null>(null);
  let uninstallingManager = $state<string | null>(null);
  let installError = $state<Record<string, string>>({});

  async function installManager(manager: SDKManagerInfo) {
    if (!manager.install_available) {
      installError[manager.id] =
        manager.install_unavailable_reason ||
        'Automatic installation is not available for this manager.';
      toast.error('Install unavailable', {
        description: installError[manager.id],
      });
      return;
    }

    installingManager = manager.id;
    installError[manager.id] = '';

    try {
      const result = await installSDKManager(manager.id);
      toast.success('Manager installed', {
        description: result,
      });
      await loadManagers();
    } catch (err) {
      installError[manager.id] = err instanceof Error ? err.message : 'Installation failed';
      toast.error('Installation failed', {
        description: installError[manager.id],
      });
    } finally {
      installingManager = null;
    }
  }

  async function uninstallManager(manager: SDKManagerInfo) {
    if (!manager.uninstall_available) {
      installError[manager.id] =
        manager.uninstall_unavailable_reason ||
        'Automatic uninstall is not available for this manager.';
      toast.error('Uninstall unavailable', {
        description: installError[manager.id],
      });
      return;
    }

    const confirmed = await confirmAction(
      `Are you sure you want to uninstall ${manager.display_name}?`,
      'Uninstall SDK manager'
    );
    if (!confirmed) return;

    uninstallingManager = manager.id;
    installError[manager.id] = '';

    try {
      const result = await uninstallSDKManager(manager.id);
      toast.success('Manager uninstalled', {
        description: result,
      });
      await loadManagers();
    } catch (err) {
      installError[manager.id] = err instanceof Error ? err.message : 'Failed to uninstall manager';
      toast.error('Uninstall failed', {
        description: installError[manager.id],
      });
    } finally {
      uninstallingManager = null;
    }
  }

  function getManagerIcon(managerId: string): string {
    const iconMap: Record<string, string> = {
      nvm: 'devicon-nodejs-plain',
      pyenv: 'devicon-python-plain',
      rustup: 'devicon-rust-plain',
      sdkman: 'devicon-sdkman-plain',
      goenv: 'devicon-go-plain',
      rbenv: 'devicon-ruby-plain',
      phpenv: 'devicon-php-plain',
      fnm: 'devicon-nodejs-plain',
    };
    return iconMap[managerId.toLowerCase()] || 'devicon-devicon-plain';
  }

  function navigateToManagerDetails(managerId: string) {
    goto(`/sdk/manager/${managerId}`);
  }
</script>

<svelte:head>
  <title>SDK Managers - Portal Desktop</title>
</svelte:head>

<div class="space-y-6">
  <!-- Header -->
  <div class="flex items-center gap-4">
    <div class="flex-1">
      <h1 class="text-3xl font-bold">SDK Managers</h1>
      <p class="text-muted-foreground">
        Install and manage SDK version managers for different programming languages
      </p>
    </div>
    <div class="flex items-center gap-2">
      <Button variant="outline" onclick={loadManagers} disabled={managersQ.isFetching}>
        <RefreshCw class="mr-2 h-4 w-4 {managersQ.isFetching ? 'animate-spin' : ''}" />
        Refresh
      </Button>
    </div>
  </div>

  <!-- Content -->
  {#if loading}
    <PageLoading message="Loading SDK managers..." />
  {:else if error}
    <PageError title="Failed to load SDK managers" message={error} onRetry={loadManagers} />
  {:else if managers.length === 0}
    <PageEmpty
      title="No SDK managers found"
      description="SDK managers will appear here once detected on your system."
      actionLabel="Refresh"
      onAction={loadManagers}
    />
  {:else}
    <!-- Managers Grid -->
    <div class="grid grid-cols-[repeat(auto-fill,minmax(min(100%,20rem),1fr))] gap-6">
      {#each managers as manager}
        <Card class="relative min-w-0">
          <CardHeader>
            <div class="flex flex-wrap items-center justify-between gap-2">
              <Button
                type="button"
                variant="ghost"
                class="h-auto min-w-0 flex-1 justify-start gap-3 px-0 text-left whitespace-normal hover:opacity-80"
                onclick={() => navigateToManagerDetails(manager.id)}
                onkeydown={(e) => e.key === 'Enter' && navigateToManagerDetails(manager.id)}
              >
                <Devicon icon={getManagerIcon(manager.id)} class="h-10 w-10 shrink-0" />
                <div class="min-w-0">
                  <CardTitle class="text-lg break-words">{manager.display_name}</CardTitle>
                  {#if manager.version}
                    <p class="mt-1 text-xs text-muted-foreground">
                      {manager.version}
                    </p>
                  {/if}
                </div>
              </Button>
              <div class="flex shrink-0 items-center gap-2">
                {#if manager.installed}
                  <Badge variant="default" class="bg-green-100 text-green-800">
                    <CheckCircle class="mr-1 h-3 w-3" />
                    Installed
                  </Badge>
                {:else}
                  <Badge variant="outline" class="text-gray-500">
                    <XCircle class="mr-1 h-3 w-3" />
                    Not Installed
                  </Badge>
                {/if}
              </div>
            </div>
          </CardHeader>
          <CardContent>
            <!-- Features -->
            <div class="mb-4">
              <div class="flex flex-wrap gap-1">
                {#if manager.supports_installation}
                  <Badge variant="secondary" class="text-xs">Installation</Badge>
                {/if}
                {#if manager.supports_version_switching}
                  <Badge variant="secondary" class="text-xs">Version Switching</Badge>
                {/if}
              </div>
            </div>

            <!-- Actions -->
            <div class="flex flex-wrap items-center gap-2">
              <Button
                variant="default"
                size="sm"
                onclick={() => navigateToManagerDetails(manager.id)}
                class="flex-1"
              >
                View Details
                <ArrowRight class="ml-1 h-4 w-4" />
              </Button>

              {#if manager.installed}
                <Button
                  variant="outline"
                  size="sm"
                  onclick={() => uninstallManager(manager)}
                  disabled={uninstallingManager === manager.id ||
                    installingManager === manager.id ||
                    !manager.uninstall_available}
                >
                  {#if uninstallingManager === manager.id}
                    <Loader2 class="mr-1 h-4 w-4 animate-spin" />
                    Uninstalling...
                  {:else}
                    <XCircle class="mr-1 h-4 w-4" />
                    Uninstall
                  {/if}
                </Button>
              {:else if manager.supports_installation && manager.install_command}
                <Button
                  variant="outline"
                  size="sm"
                  onclick={() => installManager(manager)}
                  disabled={installingManager === manager.id || !manager.install_available}
                >
                  {#if installingManager === manager.id}
                    <Loader2 class="mr-1 h-4 w-4 animate-spin" />
                    Installing...
                  {:else}
                    <Download class="mr-1 h-4 w-4" />
                    Install
                  {/if}
                </Button>
              {/if}

              {#if manager.website}
                <Button
                  variant="ghost"
                  size="sm"
                  onclick={() => window.open(manager.website!, '_blank')}
                >
                  <ExternalLink class="mr-1 h-4 w-4" />
                  Website
                </Button>
              {/if}
            </div>

            <!-- Installation Command -->
            {#if manager.install_command && !manager.installed}
              <div class="mt-4">
                <p class="mb-1 text-xs font-medium text-muted-foreground">Installation Command:</p>
                <code class="block rounded bg-muted p-2 font-mono text-xs break-all">
                  {manager.install_command}
                </code>
              </div>
            {/if}

            {#if !manager.installed && manager.install_unavailable_reason}
              <div class="mt-3 rounded-md border border-amber-200 bg-amber-50 p-3">
                <p class="text-xs text-amber-700">
                  {manager.install_unavailable_reason}
                </p>
              </div>
            {/if}

            {#if manager.installed && manager.uninstall_unavailable_reason}
              <div class="mt-3 rounded-md border border-amber-200 bg-amber-50 p-3">
                <p class="text-xs text-amber-700">
                  {manager.uninstall_unavailable_reason}
                </p>
              </div>
            {/if}

            <!-- Installation Error -->
            {#if installError[manager.id]}
              <div class="mt-3 rounded-md border border-red-200 bg-red-50 p-3">
                <p class="text-xs text-red-600">
                  {installError[manager.id]}
                </p>
              </div>
            {/if}
          </CardContent>
        </Card>
      {/each}
    </div>
  {/if}
</div>
