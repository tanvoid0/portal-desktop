<!--
	SDK Overview Page - FlyEnv-style dashboard
	Main SDK management interface with service and version management
-->

<script lang="ts">
  import { createQuery, useQueryClient } from '@tanstack/svelte-query';
  import { page } from '$app/state';
  import { sdkService } from '$lib/domains/sdk/services/sdkService';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Tabs, TabsContent, TabsList, TabsTrigger } from '$lib/components/ui/tabs';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import { Alert, AlertDescription } from '$lib/components/ui/alert';
  import Database from '~icons/fa6-solid/database';
  import Code from '~icons/fa6-solid/code';
  import Globe from '~icons/fa6-solid/globe';
  import Container from '~icons/fa6-solid/cube';
  import Package from '~icons/fa6-solid/box-open';
  import Download from '~icons/fa6-solid/download';
  import Settings from '~icons/fa6-solid/gear';
  import Play from '~icons/fa6-solid/play';
  import Square from '~icons/fa6-solid/square';
  import AlertCircle from '~icons/fa6-solid/circle-exclamation';
  import CheckCircle from '~icons/fa6-solid/circle-check';
  import Activity from '~icons/fa6-solid/wave-square';
  import TrendingUp from '~icons/fa6-solid/arrow-trend-up';
  import Clock from '~icons/fa6-solid/clock';
  import RefreshCw from '~icons/fa6-solid/rotate';
  import Plus from '~icons/fa6-solid/plus';
  import Zap from '~icons/fa6-solid/bolt';
  import VersionList from '$lib/domains/sdk/components/VersionList.svelte';
  import Devicon from '$lib/components/ui/devicon.svelte';
  import { PageError } from '$lib/components/shell';
  import { sdkConfigsQuery, sdkManagersQuery } from '$lib/domains/sdk/queries/sdkQueries';
  import { goto } from '$app/navigation';
  import { buildTabUrl, resolveUrlTab } from '$lib/utils/url-tabs';

  const SDK_OVERVIEW_TABS = ['overview', 'managers', 'services', 'versions'] as const;
  type SdkOverviewTab = (typeof SDK_OVERVIEW_TABS)[number];

  const activeTab = $derived(resolveUrlTab(page.url.searchParams, SDK_OVERVIEW_TABS, 'overview'));

  function setActiveTab(tab: SdkOverviewTab) {
    goto(buildTabUrl(page.url.pathname, page.url.searchParams, tab), {
      replaceState: true,
      reset: false,
    });
  }

  // State
  let loading = $state(false);
  let error = $state<string | null>(null);
  let versions = $state<any[]>([]);
  let versionsLoaded = false;

  // Overview counters, Quick Navigation, Managers and Services tabs read the same
  // queries as the sidebar.
  const queryClient = useQueryClient();
  const configsQuery = createQuery(sdkConfigsQuery);
  const managersQuery = createQuery(sdkManagersQuery);
  const pending = $derived(configsQuery.isPending || managersQuery.isPending);
  const queryError = $derived(configsQuery.error ?? managersQuery.error);
  const configs = $derived(configsQuery.data ?? []);
  const managerList = $derived(managersQuery.data ?? []);

  // The version lists hit the network for every SDK, so load them only when shown.
  $effect(() => {
    if (activeTab === 'versions' && !versionsLoaded) {
      versionsLoaded = true;
      void loadVersions();
    }
  });

  async function loadVersions() {
    loading = true;
    error = null;
    try {
      const availableData = await sdkService.getAllAvailableSDKs();
      const sdks: any[] = Array.isArray(availableData) ? availableData : [];
      const withVersions = await Promise.all(
        sdks.map(async (sdk: any) => {
          try {
            return { ...sdk, versions: await sdkService.fetchAvailableVersions(sdk.id) };
          } catch {
            return { ...sdk, versions: [] };
          }
        })
      );
      versions = withVersions.flatMap((sdk) =>
        sdk.versions.map((version: any) => ({ ...version, sdkType: sdk.id, sdkName: sdk.name }))
      );
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to load data';
      console.error('Failed to load SDK data:', err);
    } finally {
      loading = false;
    }
  }

  const isBusy = $derived(
    pending || configsQuery.isFetching || managersQuery.isFetching || loading
  );

  // Every mutation changes what the sidebar and counters show.
  async function reload() {
    await Promise.all([
      queryClient.invalidateQueries({ queryKey: ['sdk'] }),
      versionsLoaded ? loadVersions() : Promise.resolve(),
    ]);
  }

  // Version management
  async function installVersion(version: any) {
    version.downloading = true;
    version.progress = 0;

    try {
      await sdkService.downloadAndInstallVersion(version.sdkType, version.version, false);

      version.downloading = false;
      version.installed = true;
      await reload();
    } catch (err) {
      version.downloading = false;
      version.error = err instanceof Error ? err.message : 'Installation failed';
    }
  }

  async function uninstallVersion(version: any) {
    try {
      await sdkService.uninstallVersion({
        type: version.sdkType,
        version: version.version,
        manager: version.manager ?? version.sdkType,
      });
      version.installed = false;
      await reload();
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to uninstall version';
    }
  }

  async function setActiveVersion(version: any) {
    try {
      await sdkService.switchVersion({
        type: version.sdkType,
        version: version.version,
      });

      versions.forEach((v) => (v.active = false));
      version.active = true;
      await reload();
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to set active version';
    }
  }

  // Statistics
  const serviceConfigs = $derived(configs.filter((c) => c.service_running != null));
  let totalServices = $derived(serviceConfigs.length);
  let runningServices = $derived(serviceConfigs.filter((c) => c.service_running).length);
  let totalSDKs = $derived(configs.length);
  let installedSDKCount = $derived(configs.filter((c) => c.sdk_installed).length);
  let totalManagers = $derived(managerList.length);
  let installedManagers = $derived(managerList.filter((m) => m.installed).length);

  // Helper functions to organize SDKs by category (similar to sidebar)
  function getSDKIcon(sdkType: string): string {
    const iconMap: Record<string, string> = {
      java: 'devicon-java-plain',
      node: 'devicon-nodejs-plain',
      nodejs: 'devicon-nodejs-plain',
      python: 'devicon-python-plain',
      rust: 'devicon-rust-plain',
      go: 'devicon-go-plain',
      php: 'devicon-php-plain',
      ruby: 'devicon-ruby-plain',
      bun: 'devicon-bun-plain',
      deno: 'devicon-deno-plain',
      mysql: 'devicon-mysql-plain',
      postgresql: 'devicon-postgresql-plain',
      mongodb: 'devicon-mongodb-plain',
      mariadb: 'devicon-mariadb-plain',
      nginx: 'devicon-nginx-original',
      apache: 'devicon-apache-plain',
      docker: 'devicon-docker-plain',
      kubernetes: 'devicon-kubernetes-plain',
      nvm: 'devicon-nodejs-plain',
      pyenv: 'devicon-python-plain',
      rustup: 'devicon-rust-plain',
      sdkman: 'devicon-sdkman-plain',
      rbenv: 'devicon-ruby-plain',
      phpenv: 'devicon-php-plain',
    };
    return iconMap[sdkType.toLowerCase()] || 'devicon-devicon-plain';
  }

  function getSDKRoute(sdkId: string, category: string): string {
    const normalizedId = sdkId.toLowerCase().trim();

    // SDK Managers go to /sdk/manager/[name] (singular, matches route structure)
    if (category === 'manager') {
      return `/sdk/manager/${normalizedId}`;
    }

    // Use the new dynamic route for all SDKs (language, database, ai, server, container, etc.)
    // The dynamic route /sdk/[sdk]/+page.svelte handles all SDKs
    return `/sdk/${normalizedId}`;
  }

  // Organize SDKs by category
  let quickNavSDKs = $derived.by(() => {
    const categories: Record<string, any[]> = {
      manager: [],
      language: [],
      database: [],
      web: [],
      container: [],
    };

    const toItem = (c: (typeof configs)[number]) => ({
      id: c.id,
      displayName: c.display_name,
      icon: getSDKIcon(c.id),
      category: c.category,
      // A running service proves the SDK is there even when binary detection misses it
      installed: c.sdk_installed || c.service_running === true,
      version: c.sdk_version?.trim().replace(/^v/, '') || null,
    });
    categories.manager = managerList.map((m) => ({
      id: m.id,
      displayName: m.display_name,
      icon: getSDKIcon(m.id),
      category: 'manager',
      installed: m.installed,
      version: m.version?.trim().replace(/^v/, '') || null,
    }));
    for (const c of configs) {
      if (c.category === 'database') categories.database.push(toItem(c));
      else if (c.category === 'server') categories.web.push(toItem(c));
      else if (c.category === 'container') categories.container.push(toItem(c));
      else if (c.category === 'language') categories.language.push(toItem(c));
    }

    return categories;
  });

  function handleSDKClick(sdk: any) {
    const route = getSDKRoute(sdk.id, sdk.category);
    goto(route);
  }

  // Check if a service is running for an SDK
  function isServiceRunning(sdkId: string): boolean {
    return configs.some((c) => c.id === sdkId && c.service_running);
  }
</script>

<div class="space-y-6">
  <!-- Header -->
  <div class="flex items-center justify-between">
    <div>
      <h1 class="text-3xl font-bold">SDK Management</h1>
      <p class="text-muted-foreground">Manage your development environment and SDK versions</p>
    </div>
    <div class="flex items-center gap-2">
      <Button variant="outline" onclick={reload} disabled={isBusy}>
        <RefreshCw class="mr-2 h-4 w-4" />
        Refresh
      </Button>
      <Button>
        <Settings class="mr-2 h-4 w-4" />
        Settings
      </Button>
    </div>
  </div>

  <!-- Error Alert -->
  {#if error}
    <Alert variant="destructive">
      <AlertCircle class="h-4 w-4" />
      <AlertDescription>{error}</AlertDescription>
    </Alert>
  {/if}

  {#if queryError}
    <PageError
      title="Failed to load SDK status"
      message={queryError instanceof Error ? queryError.message : String(queryError)}
      onRetry={reload}
    />
  {/if}

  <!-- Statistics Cards -->
  <div class="grid grid-cols-1 gap-4 md:grid-cols-4">
    <Card>
      <CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle class="text-sm font-medium">Managers</CardTitle>
        <Package class="h-4 w-4 text-muted-foreground" />
      </CardHeader>
      <CardContent>
        <div class="text-2xl font-bold">
          {pending ? '—' : `${installedManagers}/${totalManagers}`}
        </div>
        <p class="text-xs text-muted-foreground">
          {installedManagers} installed
        </p>
      </CardContent>
    </Card>

    <Card>
      <CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle class="text-sm font-medium">Services</CardTitle>
        <Database class="h-4 w-4 text-muted-foreground" />
      </CardHeader>
      <CardContent>
        <div class="text-2xl font-bold">
          {pending ? '—' : `${runningServices}/${totalServices}`}
        </div>
        <p class="text-xs text-muted-foreground">
          {runningServices} running
        </p>
      </CardContent>
    </Card>

    <Card>
      <CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle class="text-sm font-medium">SDKs</CardTitle>
        <Code class="h-4 w-4 text-muted-foreground" />
      </CardHeader>
      <CardContent>
        <div class="text-2xl font-bold">
          {pending ? '—' : `${installedSDKCount}/${totalSDKs}`}
        </div>
        <p class="text-xs text-muted-foreground">
          {installedSDKCount} installed
        </p>
      </CardContent>
    </Card>

    <Card>
      <CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle class="text-sm font-medium">Status</CardTitle>
        <Globe class="h-4 w-4 text-muted-foreground" />
      </CardHeader>
      <CardContent>
        <div class="text-2xl font-bold">
          {pending ? '—' : runningServices > 0 ? 'Online' : 'Offline'}
        </div>
        <p class="text-xs text-muted-foreground">Environment status</p>
      </CardContent>
    </Card>
  </div>

  <!-- Main Content Tabs -->
  <Tabs value={activeTab} onValueChange={(v) => setActiveTab(v as SdkOverviewTab)} class="w-full">
    <TabsList class="grid w-full grid-cols-4">
      <TabsTrigger value="overview">Overview</TabsTrigger>
      <TabsTrigger value="managers">SDK Managers</TabsTrigger>
      <TabsTrigger value="services">Services</TabsTrigger>
      <TabsTrigger value="versions">Versions</TabsTrigger>
    </TabsList>

    <TabsContent value="overview" class="space-y-6">
      <!-- Loading State -->
      {#if pending}
        <Card>
          <CardContent class="py-8">
            <div class="flex flex-col items-center justify-center space-y-4">
              <RefreshCw class="h-8 w-8 animate-spin text-muted-foreground" />
              <p class="text-sm text-muted-foreground">Loading SDK data...</p>
            </div>
          </CardContent>
        </Card>
      {/if}

      {#if !pending}
        <Card>
          <CardHeader>
            <CardTitle class="flex items-center gap-2">
              <Settings class="h-5 w-5 text-muted-foreground" />
              SDK Tools
            </CardTitle>
            <p class="mt-1 text-sm text-muted-foreground">Jump to common SDK sections.</p>
          </CardHeader>
          <CardContent>
            <div class="grid grid-cols-1 gap-2 md:grid-cols-3">
              <Button variant="outline" onclick={() => setActiveTab('managers')}>
                <Settings class="mr-2 h-4 w-4" />
                SDK Managers
              </Button>
              <Button variant="outline" onclick={() => goto('/sdk/software-installer')}>
                <Package class="mr-2 h-4 w-4" />
                Software Installer
              </Button>
              <Button variant="outline" onclick={() => goto('/sdk/installations')}>
                <Download class="mr-2 h-4 w-4" />
                Installations
              </Button>
            </div>
          </CardContent>
        </Card>
      {/if}

      <!-- Quick Navigation Section -->
      {#if !pending}
        <Card>
          <CardHeader>
            <CardTitle class="flex items-center gap-2">
              <Activity class="h-5 w-5" />
              Quick Navigation
            </CardTitle>
            <p class="mt-1 text-sm text-muted-foreground">
              Quick access to all SDKs and managers. Green indicates installed/active, gray
              indicates not installed.
            </p>
          </CardHeader>
          <CardContent>
            <div class="space-y-6">
              <!-- SDK Managers -->
              {#if quickNavSDKs.manager.length > 0}
                <div class="space-y-3">
                  <div class="flex items-center gap-2">
                    <Package class="h-4 w-4 text-muted-foreground" />
                    <h3 class="text-sm font-medium">SDK Managers</h3>
                  </div>
                  <div class="grid grid-cols-[repeat(auto-fill,minmax(min(100%,13rem),1fr))] gap-3">
                    {#each quickNavSDKs.manager as sdk}
                      <Button
                        variant="outline"
                        onclick={() => handleSDKClick(sdk)}
                        class="relative h-auto w-full min-w-0 flex-col items-start overflow-hidden rounded-lg border p-3 text-left transition-all hover:shadow-md
													{sdk.installed
                          ? 'border-green-500/50 bg-green-50/50 dark:bg-green-950/20'
                          : 'border-border bg-muted/30'}"
                      >
                        <div class="mb-2 flex w-full min-w-0 items-center gap-2">
                          <Devicon icon={sdk.icon} size="sm" />
                          <span class="truncate text-sm font-medium">{sdk.displayName}</span>
                        </div>
                        <div class="flex w-full min-w-0 items-center justify-between gap-2">
                          {#if sdk.version}
                            <span class="truncate text-xs text-muted-foreground">{sdk.version}</span
                            >
                          {:else}
                            <span class="text-xs text-muted-foreground"
                              >{sdk.installed ? 'Installed' : 'Not installed'}</span
                            >
                          {/if}
                          {#if sdk.installed}
                            <CheckCircle class="h-3 w-3 flex-shrink-0 text-green-600" />
                          {:else}
                            <AlertCircle class="h-3 w-3 flex-shrink-0 text-muted-foreground" />
                          {/if}
                        </div>
                      </Button>
                    {/each}
                  </div>
                </div>
              {/if}

              <!-- Language & Runtime -->
              {#if quickNavSDKs.language.length > 0}
                <div class="space-y-3">
                  <div class="flex items-center gap-2">
                    <Code class="h-4 w-4 text-muted-foreground" />
                    <h3 class="text-sm font-medium">Language & Runtime</h3>
                  </div>
                  <div class="grid grid-cols-[repeat(auto-fill,minmax(min(100%,13rem),1fr))] gap-3">
                    {#each quickNavSDKs.language as sdk}
                      <Button
                        variant="outline"
                        onclick={() => handleSDKClick(sdk)}
                        class="relative h-auto w-full min-w-0 flex-col items-start overflow-hidden rounded-lg border p-3 text-left transition-all hover:shadow-md
													{sdk.installed
                          ? 'border-green-500/50 bg-green-50/50 dark:bg-green-950/20'
                          : 'border-border bg-muted/30'}"
                      >
                        <div class="mb-2 flex w-full min-w-0 items-center gap-2">
                          <Devicon icon={sdk.icon} size="sm" />
                          <span class="truncate text-sm font-medium">{sdk.displayName}</span>
                        </div>
                        <div class="flex w-full min-w-0 items-center justify-between gap-2">
                          {#if sdk.version}
                            <span class="truncate text-xs text-muted-foreground">{sdk.version}</span
                            >
                          {:else}
                            <span class="text-xs text-muted-foreground"
                              >{sdk.installed ? 'Installed' : 'Not installed'}</span
                            >
                          {/if}
                          <div class="flex items-center gap-1">
                            {#if isServiceRunning(sdk.id)}
                              <Badge variant="default" class="bg-blue-500 px-1 py-0 text-xs"
                                >Running</Badge
                              >
                            {/if}
                            {#if sdk.installed}
                              <CheckCircle class="h-3 w-3 flex-shrink-0 text-green-600" />
                            {:else}
                              <AlertCircle class="h-3 w-3 flex-shrink-0 text-muted-foreground" />
                            {/if}
                          </div>
                        </div>
                      </Button>
                    {/each}
                  </div>
                </div>
              {/if}

              <!-- Database Server -->
              {#if quickNavSDKs.database.length > 0}
                <div class="space-y-3">
                  <div class="flex items-center gap-2">
                    <Database class="h-4 w-4 text-muted-foreground" />
                    <h3 class="text-sm font-medium">Database Server</h3>
                  </div>
                  <div class="grid grid-cols-[repeat(auto-fill,minmax(min(100%,13rem),1fr))] gap-3">
                    {#each quickNavSDKs.database as sdk}
                      <Button
                        variant="outline"
                        onclick={() => handleSDKClick(sdk)}
                        class="relative h-auto w-full min-w-0 flex-col items-start overflow-hidden rounded-lg border p-3 text-left transition-all hover:shadow-md
													{sdk.installed
                          ? 'border-green-500/50 bg-green-50/50 dark:bg-green-950/20'
                          : 'border-border bg-muted/30'}"
                      >
                        <div class="mb-2 flex w-full min-w-0 items-center gap-2">
                          <Devicon icon={sdk.icon} size="sm" />
                          <span class="truncate text-sm font-medium">{sdk.displayName}</span>
                        </div>
                        <div class="flex w-full min-w-0 items-center justify-between gap-2">
                          {#if sdk.version}
                            <span class="truncate text-xs text-muted-foreground">{sdk.version}</span
                            >
                          {:else}
                            <span class="text-xs text-muted-foreground"
                              >{sdk.installed ? 'Installed' : 'Not installed'}</span
                            >
                          {/if}
                          <div class="flex items-center gap-1">
                            {#if isServiceRunning(sdk.id)}
                              <Badge variant="default" class="bg-blue-500 px-1 py-0 text-xs"
                                >Running</Badge
                              >
                            {/if}
                            {#if sdk.installed}
                              <CheckCircle class="h-3 w-3 flex-shrink-0 text-green-600" />
                            {:else}
                              <AlertCircle class="h-3 w-3 flex-shrink-0 text-muted-foreground" />
                            {/if}
                          </div>
                        </div>
                      </Button>
                    {/each}
                  </div>
                </div>
              {/if}

              <!-- Web Server -->
              {#if quickNavSDKs.web.length > 0}
                <div class="space-y-3">
                  <div class="flex items-center gap-2">
                    <Globe class="h-4 w-4 text-muted-foreground" />
                    <h3 class="text-sm font-medium">Web Server</h3>
                  </div>
                  <div class="grid grid-cols-[repeat(auto-fill,minmax(min(100%,13rem),1fr))] gap-3">
                    {#each quickNavSDKs.web as sdk}
                      <Button
                        variant="outline"
                        onclick={() => handleSDKClick(sdk)}
                        class="relative h-auto w-full min-w-0 flex-col items-start overflow-hidden rounded-lg border p-3 text-left transition-all hover:shadow-md
													{sdk.installed
                          ? 'border-green-500/50 bg-green-50/50 dark:bg-green-950/20'
                          : 'border-border bg-muted/30'}"
                      >
                        <div class="mb-2 flex w-full min-w-0 items-center gap-2">
                          <Devicon icon={sdk.icon} size="sm" />
                          <span class="truncate text-sm font-medium">{sdk.displayName}</span>
                        </div>
                        <div class="flex w-full min-w-0 items-center justify-between gap-2">
                          {#if sdk.version}
                            <span class="truncate text-xs text-muted-foreground">{sdk.version}</span
                            >
                          {:else}
                            <span class="text-xs text-muted-foreground"
                              >{sdk.installed ? 'Installed' : 'Not installed'}</span
                            >
                          {/if}
                          <div class="flex items-center gap-1">
                            {#if isServiceRunning(sdk.id)}
                              <Badge variant="default" class="bg-blue-500 px-1 py-0 text-xs"
                                >Running</Badge
                              >
                            {/if}
                            {#if sdk.installed}
                              <CheckCircle class="h-3 w-3 flex-shrink-0 text-green-600" />
                            {:else}
                              <AlertCircle class="h-3 w-3 flex-shrink-0 text-muted-foreground" />
                            {/if}
                          </div>
                        </div>
                      </Button>
                    {/each}
                  </div>
                </div>
              {/if}

              <!-- Container Platform -->
              {#if quickNavSDKs.container.length > 0}
                <div class="space-y-3">
                  <div class="flex items-center gap-2">
                    <Container class="h-4 w-4 text-muted-foreground" />
                    <h3 class="text-sm font-medium">Container Platform</h3>
                  </div>
                  <div class="grid grid-cols-[repeat(auto-fill,minmax(min(100%,13rem),1fr))] gap-3">
                    {#each quickNavSDKs.container as sdk}
                      <Button
                        variant="outline"
                        onclick={() => handleSDKClick(sdk)}
                        class="relative h-auto w-full min-w-0 flex-col items-start overflow-hidden rounded-lg border p-3 text-left transition-all hover:shadow-md
													{sdk.installed
                          ? 'border-green-500/50 bg-green-50/50 dark:bg-green-950/20'
                          : 'border-border bg-muted/30'}"
                      >
                        <div class="mb-2 flex w-full min-w-0 items-center gap-2">
                          <Devicon icon={sdk.icon} size="sm" />
                          <span class="truncate text-sm font-medium">{sdk.displayName}</span>
                        </div>
                        <div class="flex w-full min-w-0 items-center justify-between gap-2">
                          {#if sdk.version}
                            <span class="truncate text-xs text-muted-foreground">{sdk.version}</span
                            >
                          {:else}
                            <span class="text-xs text-muted-foreground"
                              >{sdk.installed ? 'Installed' : 'Not installed'}</span
                            >
                          {/if}
                          <div class="flex items-center gap-1">
                            {#if isServiceRunning(sdk.id)}
                              <Badge variant="default" class="bg-blue-500 px-1 py-0 text-xs"
                                >Running</Badge
                              >
                            {/if}
                            {#if sdk.installed}
                              <CheckCircle class="h-3 w-3 flex-shrink-0 text-green-600" />
                            {:else}
                              <AlertCircle class="h-3 w-3 flex-shrink-0 text-muted-foreground" />
                            {/if}
                          </div>
                        </div>
                      </Button>
                    {/each}
                  </div>
                </div>
              {/if}

              <!-- Empty State -->
              {#if quickNavSDKs.manager.length === 0 && quickNavSDKs.language.length === 0 && quickNavSDKs.database.length === 0 && quickNavSDKs.web.length === 0 && quickNavSDKs.container.length === 0}
                <div class="py-8 text-center text-muted-foreground">
                  <AlertCircle class="mx-auto mb-2 h-12 w-12 opacity-50" />
                  <p class="mb-2 text-sm">No SDKs detected.</p>
                  <p class="text-xs">
                    Click "Refresh" to detect installed SDKs, or install SDK managers to get
                    started.
                  </p>
                </div>
              {/if}
            </div>
          </CardContent>
        </Card>
      {/if}

      <!-- Quick Start Section -->
      {#if !pending && !queryError && installedSDKCount === 0}
        <Card class="border-dashed">
          <CardHeader>
            <CardTitle class="flex items-center gap-2">
              <Zap class="h-5 w-5 text-primary" />
              Quick Start
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div class="space-y-4">
              <p class="text-muted-foreground">
                Get started with SDK management. Install your first SDK or start a service.
              </p>
              <div class="flex gap-2">
                <Button onclick={() => setActiveTab('versions')}>
                  <Download class="mr-2 h-4 w-4" />
                  Install SDK
                </Button>
                <Button variant="outline" onclick={() => setActiveTab('services')}>
                  <Play class="mr-2 h-4 w-4" />
                  Start Service
                </Button>
              </div>
            </div>
          </CardContent>
        </Card>
      {/if}

      <!-- System Status Card -->
      {#if !pending}
        <Card>
          <CardHeader>
            <CardTitle class="flex items-center gap-2">
              <Activity class="h-5 w-5" />
              System Status
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div class="grid gap-4 md:grid-cols-3">
              <div class="flex items-center gap-3">
                <div class="h-3 w-3 rounded-full bg-green-500"></div>
                <div>
                  <div class="font-medium">System Health</div>
                  <div class="text-sm text-muted-foreground">All systems operational</div>
                </div>
              </div>
              <div class="flex items-center gap-3">
                <div class="h-3 w-3 rounded-full bg-blue-500"></div>
                <div>
                  <div class="font-medium">Services</div>
                  <div class="text-sm text-muted-foreground">
                    {runningServices} running
                  </div>
                </div>
              </div>
              <div class="flex items-center gap-3">
                <div class="h-3 w-3 rounded-full bg-purple-500"></div>
                <div>
                  <div class="font-medium">SDK Managers</div>
                  <div class="text-sm text-muted-foreground">
                    {installedManagers} installed
                  </div>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>

        <!-- Quick Actions -->
        <Card>
          <CardHeader>
            <CardTitle class="flex items-center gap-2">
              <Plus class="h-5 w-5" />
              Quick Actions
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div class="grid grid-cols-2 gap-4 md:grid-cols-4">
              <Button
                variant="outline"
                class="flex h-20 flex-col gap-2"
                onclick={() => setActiveTab('services')}
              >
                <Play class="h-6 w-6" />
                <span>Start All</span>
              </Button>
              <Button
                variant="outline"
                class="flex h-20 flex-col gap-2"
                onclick={() => setActiveTab('services')}
              >
                <Square class="h-6 w-6" />
                <span>Stop All</span>
              </Button>
              <Button
                variant="outline"
                class="flex h-20 flex-col gap-2"
                onclick={() => setActiveTab('versions')}
              >
                <Download class="h-6 w-6" />
                <span>Install Latest</span>
              </Button>
              <Button
                variant="outline"
                class="flex h-20 flex-col gap-2"
                onclick={() => setActiveTab('versions')}
              >
                <Settings class="h-6 w-6" />
                <span>Configure</span>
              </Button>
            </div>
          </CardContent>
        </Card>
      {/if}
    </TabsContent>

    <TabsContent value="managers" class="space-y-6">
      <div class="grid grid-cols-[repeat(auto-fill,minmax(min(100%,18rem),1fr))] gap-4">
        {#each managerList as manager (manager.id)}
          <Card class="min-w-0">
            <CardHeader class="flex flex-row items-center justify-between gap-2 space-y-0">
              <CardTitle class="min-w-0 text-base break-words">{manager.display_name}</CardTitle>
              {#if manager.installed}
                <Badge variant="default" class="shrink-0 bg-green-100 text-green-800">
                  <CheckCircle class="mr-1 h-3 w-3" />
                  Installed
                </Badge>
              {:else}
                <Badge variant="outline" class="shrink-0 text-gray-500">Not Installed</Badge>
              {/if}
            </CardHeader>
            <CardContent class="space-y-3">
              {#if manager.version}
                <p class="text-xs text-muted-foreground">{manager.version}</p>
              {/if}
              <Button
                variant="outline"
                size="sm"
                onclick={() => goto(`/sdk/manager/${manager.id}`)}
              >
                View Details
              </Button>
            </CardContent>
          </Card>
        {/each}
      </div>
      {#if !pending && managerList.length === 0}
        <p class="py-8 text-center text-sm text-muted-foreground">No SDK managers found.</p>
      {/if}
    </TabsContent>

    <TabsContent value="services" class="space-y-6">
      <div class="grid gap-3">
        {#each serviceConfigs as c (c.id)}
          <Card>
            <CardContent class="flex items-center justify-between gap-4 p-4">
              <div class="min-w-0">
                <div class="font-medium">{c.display_name}</div>
                {#if c.service_port}
                  <div class="text-sm text-muted-foreground">Port {c.service_port}</div>
                {/if}
              </div>
              <div class="flex shrink-0 items-center gap-2">
                {#if c.service_running}
                  <Badge variant="default" class="bg-green-100 text-green-800">Running</Badge>
                {:else}
                  <Badge variant="outline" class="text-gray-500">Stopped</Badge>
                {/if}
                <Button variant="outline" size="sm" onclick={() => goto(`/sdk/${c.id}`)}>
                  Manage
                </Button>
              </div>
            </CardContent>
          </Card>
        {/each}
      </div>
      {#if !pending && serviceConfigs.length === 0}
        <p class="py-8 text-center text-sm text-muted-foreground">No SDK services found.</p>
      {/if}
    </TabsContent>

    <TabsContent value="versions" class="space-y-6">
      <VersionList
        {versions}
        onInstall={installVersion}
        onUninstall={uninstallVersion}
        onSetActive={setActiveVersion}
        {loading}
      />
    </TabsContent>
  </Tabs>
</div>
