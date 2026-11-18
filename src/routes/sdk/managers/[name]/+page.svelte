<!--
	SDK Manager Detail Page
	Shows details for a specific SDK manager with version management
-->

<script lang="ts">
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { invoke } from '@tauri-apps/api/core';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import CheckCircle from '~icons/fa6-solid/circle-check';
  import XCircle from '~icons/fa6-solid/circle-xmark';
  import { Badge } from '$lib/components/ui/badge';
  import Devicon from '$lib/components/ui/devicon.svelte';
  import { Tabs, TabsContent, TabsList, TabsTrigger } from '$lib/components/ui/tabs';
  import ManagerNotInstalledCard from '$lib/domains/sdk/components/managers/ManagerNotInstalledCard.svelte';
  import ManagerInstalledVersions from '$lib/domains/sdk/components/managers/ManagerInstalledVersions.svelte';
  import ManagerAvailableVersions from '$lib/domains/sdk/components/managers/ManagerAvailableVersions.svelte';
  import ManagerInfoCards from '$lib/domains/sdk/components/managers/ManagerInfoCards.svelte';
  import { getSDKIcon } from '$lib/domains/sdk/utils/sdkIcons';
  import type { SDKManager } from '$lib/domains/sdk/utils/sdkSidebarItems';
  import { PageError, PageHeader, PageLoading } from '$lib/components/shell';
  import { confirmAction } from '$lib/utils/confirm';
  import { toast } from '$lib/utils/toast';
  import { buildTabUrl, resolveUrlTab } from '$lib/utils/url-tabs';

  const MANAGER_TABS = ['installed', 'available'] as const;
  type ManagerTab = (typeof MANAGER_TABS)[number];

  const activeTab = $derived(resolveUrlTab(page.url.searchParams, MANAGER_TABS, 'installed'));

  function setActiveTab(tab: ManagerTab) {
    goto(buildTabUrl(page.url.pathname, page.url.searchParams, tab), {
      replaceState: true,
      reset: false,
    });
  }

  // Get manager name from URL
  let managerName = $derived(page.params.name);

  // State
  let loading = $state(true);
  let error = $state<string | null>(null);
  let manager = $state<SDKManager | null>(null);
  let installedVersions = $state<string[]>([]);
  let availableVersions = $state<string[]>([]);
  let currentVersion = $state<string | null>(null);
  let loadingVersions = $state(false);
  // Kept apart from `error`: a manager Portal can't drive yet must not hide its info card.
  let versionsError = $state<string | null>(null);
  let installingVersion = $state<string | null>(null);
  let switchingVersion = $state<string | null>(null);

  // Load manager details
  $effect(() => {
    if (managerName) {
      loadManagerDetails();
    }
  });

  async function loadManagerDetails() {
    loading = true;
    error = null;

    try {
      // Get manager from all SDK managers
      const allManagers = await invoke<SDKManager[]>('get_all_sdk_managers');
      const foundManager = allManagers.find((m) => m.id === managerName || m.name === managerName);

      if (!foundManager) {
        error = `Manager '${managerName}' not found`;
        return;
      }

      manager = foundManager;

      // Load versions if installed
      if (manager.installed) {
        await loadVersions();
      }
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to load manager details';
      console.error('Failed to load manager details:', err);
    } finally {
      loading = false;
    }
  }

  async function loadVersions() {
    if (!manager || !manager.installed) return;

    loadingVersions = true;
    versionsError = null;
    try {
      const [installed, available, current] = await Promise.allSettled([
        invoke<string[]>('get_manager_installed_versions', { managerName: manager.id }),
        invoke<string[]>('get_manager_available_versions', { managerName: manager.id }),
        invoke<string | null>('get_manager_current_version', { managerName: manager.id }),
      ]);
      // Installed versions must load (an unsupported manager says so instead of showing "0");
      // the available list needs the network, so its failure is only a warning.
      if (installed.status === 'rejected') throw installed.reason;
      if (available.status === 'rejected') {
        toast.warning(`Couldn't list available versions: ${String(available.reason)}`);
      }

      installedVersions = installed.value;
      availableVersions = available.status === 'fulfilled' ? available.value : [];
      currentVersion = current.status === 'fulfilled' ? current.value : null;
    } catch (err) {
      // Tauri rejects with the Rust error string, not an Error.
      versionsError = err instanceof Error ? err.message : String(err);
      console.error('Failed to load versions:', err);
    } finally {
      loadingVersions = false;
    }
  }

  async function installVersion(version: string) {
    if (!manager) return;

    installingVersion = version;
    try {
      await invoke('install_version_via_manager', {
        managerName: manager.id,
        version: version,
      });

      // Reload versions
      await loadVersions();
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to install version';
      console.error('Failed to install version:', err);
    } finally {
      installingVersion = null;
    }
  }

  async function switchVersion(version: string) {
    if (!manager) return;

    switchingVersion = version;
    try {
      await invoke('switch_version_via_manager', {
        managerName: manager.id,
        version: version,
      });

      // Reload versions to get updated current version
      await loadVersions();
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to switch version';
      console.error('Failed to switch version:', err);
    } finally {
      switchingVersion = null;
    }
  }

  async function uninstallVersion(version: string) {
    if (!manager) return;

    const confirmed = await confirmAction(
      `Are you sure you want to uninstall version ${version}?`,
      'Uninstall version'
    );
    if (!confirmed) return;

    try {
      await invoke('uninstall_version_via_manager', {
        managerName: manager.id,
        version: version,
      });

      // Reload versions
      await loadVersions();
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to uninstall version';
      console.error('Failed to uninstall version:', err);
    }
  }
</script>

<svelte:head>
  <title>{manager?.display_name || 'SDK Manager'} - Portal Desktop</title>
</svelte:head>

<div class="space-y-6">
  {#if loading}
    <PageLoading message="Loading SDK manager..." />
  {:else if error}
    <PageError title="Failed to load SDK manager" message={error} onRetry={loadManagerDetails} />
  {:else if manager}
    <div class="space-y-6">
      <div class="flex items-center gap-3">
        <Devicon icon={getSDKIcon(manager.id)} class="h-8 w-8 shrink-0" />
        <div class="min-w-0 flex-1">
          <PageHeader
            title={manager.display_name}
            description={`${manager.id} — ${
              manager.supports_installation ? 'Supports installation' : 'Read-only'
            }`}
          >
            {#snippet actions()}
              {#if manager?.installed}
                <Badge variant="default" class="bg-green-100 text-green-800">
                  <CheckCircle class="mr-1 h-3 w-3" />
                  Installed
                  {#if manager.version}
                    <span class="ml-1">({manager.version})</span>
                  {/if}
                </Badge>
              {:else}
                <Badge variant="outline" class="text-gray-500">
                  <XCircle class="mr-1 h-3 w-3" />
                  Not Installed
                </Badge>
              {/if}
            {/snippet}
          </PageHeader>
        </div>
      </div>

      {#if !manager.installed}
        <ManagerNotInstalledCard {manager} />
      {:else}
        <!-- Current Version -->
        {#if currentVersion}
          <Card>
            <CardHeader>
              <CardTitle>Current Active Version</CardTitle>
            </CardHeader>
            <CardContent>
              <div class="flex items-center justify-between">
                <span class="text-2xl font-bold">{currentVersion}</span>
                <Badge variant="default" class="bg-blue-100 text-blue-800">Active</Badge>
              </div>
            </CardContent>
          </Card>
        {/if}

        <!-- Version Management Tabs -->
        <Tabs value={activeTab} onValueChange={(v) => setActiveTab(v as ManagerTab)}>
          <TabsList>
            <TabsTrigger value="installed">
              Installed ({installedVersions.length})
            </TabsTrigger>
            <TabsTrigger value="available">
              Available ({availableVersions.length})
            </TabsTrigger>
          </TabsList>

          <TabsContent value="installed" class="space-y-4">
            <ManagerInstalledVersions
              versions={installedVersions}
              current={currentVersion}
              loading={loadingVersions}
              error={versionsError}
              {switchingVersion}
              onrefresh={loadVersions}
              onswitch={switchVersion}
              onuninstall={uninstallVersion}
            />
          </TabsContent>

          <TabsContent value="available" class="space-y-4">
            <ManagerAvailableVersions
              versions={availableVersions}
              installed={installedVersions}
              supportsInstallation={manager.supports_installation}
              loading={loadingVersions}
              {installingVersion}
              {switchingVersion}
              onrefresh={loadVersions}
              oninstall={installVersion}
              onswitch={switchVersion}
            />
          </TabsContent>
        </Tabs>

        <ManagerInfoCards {manager} />
      {/if}
    </div>
  {/if}
</div>
