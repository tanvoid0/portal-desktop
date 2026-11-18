<!--
	Software Installer Page
	Install and manage software via multiple package managers (winget, scoop, chocolatey, etc.)
-->

<script lang="ts">
  import { Card, CardContent } from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import { Tabs, TabsContent, TabsList, TabsTrigger } from '$lib/components/ui/tabs';
  import RefreshCw from '~icons/fa6-solid/rotate';
  import PackageIcon from '~icons/fa6-solid/box-open';
  import InlineEmptyState from '$lib/components/shared/InlineEmptyState.svelte';
  import PackageManagerPicker from '$lib/domains/sdk/components/installer/PackageManagerPicker.svelte';
  import PackageBrowseTab from '$lib/domains/sdk/components/installer/PackageBrowseTab.svelte';
  import PackageInstalledTab from '$lib/domains/sdk/components/installer/PackageInstalledTab.svelte';
  import PackageUpdatesTab from '$lib/domains/sdk/components/installer/PackageUpdatesTab.svelte';
  import PackageDetailsDialog from '$lib/domains/sdk/components/installer/PackageDetailsDialog.svelte';
  import { PackageInstaller } from '$lib/domains/sdk/state/packageInstaller.svelte';
  import { PageError, PageHeader, PageLoading } from '$lib/components/shell';
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { buildTabUrl, resolveUrlTab } from '$lib/utils/url-tabs';

  const INSTALLER_TABS = ['browse', 'installed', 'updates'] as const;
  type InstallerTab = (typeof INSTALLER_TABS)[number];

  const activeTab = $derived(resolveUrlTab(page.url.searchParams, INSTALLER_TABS, 'browse'));
  const inst = new PackageInstaller(() => activeTab);

  function setActiveTab(tab: InstallerTab) {
    goto(buildTabUrl(page.url.pathname, page.url.searchParams, tab), {
      replaceState: true,
      reset: false,
    });
  }

  // Load available managers on mount
  $effect(() => {
    inst.loadAvailableManagers();
  });

  // Load manager info when selection changes, then the visible tab's data
  $effect(() => {
    if (inst.selectedManager) {
      (async () => {
        await inst.loadManagerInfo();
        await inst.loadActiveTab();
      })();
    }
  });

  // Load data when tab changes
  $effect(() => {
    if (inst.selectedManager && inst.managerInfo?.available) {
      if (activeTab === 'installed') {
        inst.loadInstalled();
      } else if (activeTab === 'updates') {
        inst.loadUpdates();
      }
    }
  });
</script>

<svelte:head>
  <title>Software Installer - Portal Desktop</title>
</svelte:head>

<div class="space-y-6">
  {#if inst.loading && inst.availableManagers.length === 0}
    <PageLoading message="Loading package managers..." />
  {:else}
    <PageHeader
      title="Software Installer"
      description="Install and manage software via package managers"
    >
      {#snippet actions()}
        <Button
          variant="outline"
          size="sm"
          onclick={() => inst.loadAvailableManagers()}
          disabled={inst.loading}
        >
          <RefreshCw class="mr-2 h-4 w-4 {inst.loading ? 'animate-spin' : ''}" />
          Refresh
        </Button>
      {/snippet}
    </PageHeader>

    {#if inst.error}
      <PageError
        title="Failed to load package managers"
        message={inst.error}
        onRetry={() => inst.loadAvailableManagers()}
      />
    {/if}

    <PackageManagerPicker
      managers={inst.availableManagers}
      selected={inst.selectedManager}
      info={inst.managerInfo}
      loading={inst.loading}
      searching={inst.searching}
      bind:query={inst.searchQuery}
      onselect={(m) => inst.select(m)}
      onsearchinput={() => inst.onSearchInput()}
    />

    {#if !inst.selectedManager || !inst.managerInfo?.available}
      <Card>
        <CardContent class="pt-6">
          <InlineEmptyState
            icon={PackageIcon}
            title={inst.availableManagers.length === 0
              ? 'No package managers are available on your system.'
              : 'Please select an available package manager to get started.'}
          />
        </CardContent>
      </Card>
    {:else}
      <Tabs value={activeTab} onValueChange={(v) => setActiveTab(v as InstallerTab)}>
        <TabsList>
          <TabsTrigger value="browse">Browse ({inst.searchResults.length})</TabsTrigger>
          <TabsTrigger value="installed">Installed ({inst.installedPackages.length})</TabsTrigger>
          <TabsTrigger value="updates">Updates ({inst.availableUpdates.length})</TabsTrigger>
        </TabsList>

        <TabsContent value="browse" class="space-y-4">
          <PackageBrowseTab
            searching={inst.searching}
            query={inst.searchQuery}
            results={inst.searchResults}
            installingId={inst.installingPackage}
            oninstall={(p) => inst.install(p)}
            ondetails={(p) => inst.showDetails(p)}
          />
        </TabsContent>

        <TabsContent value="installed" class="space-y-4">
          <PackageInstalledTab
            packages={inst.installedPackages}
            loading={inst.loadingInstalled}
            upgradingId={inst.upgradingPackage}
            uninstallingId={inst.uninstallingPackage}
            onrefresh={() => inst.loadInstalled()}
            ondetails={(p) => inst.showDetails(p)}
            onupgrade={(p) => inst.upgrade(p)}
            onuninstall={(p) => inst.uninstall(p)}
          />
        </TabsContent>

        <TabsContent value="updates" class="space-y-4">
          <PackageUpdatesTab
            updates={inst.availableUpdates}
            loading={inst.loadingUpdates}
            upgradingId={inst.upgradingPackage}
            onrefresh={() => inst.loadUpdates()}
            onupgrade={(u) => inst.upgrade(u)}
          />
        </TabsContent>
      </Tabs>
    {/if}
  {/if}
</div>

<PackageDetailsDialog
  bind:open={inst.showDetailsDialog}
  loading={inst.loadingDetails}
  details={inst.selectedPackageDetails}
/>
