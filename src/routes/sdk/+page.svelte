<!--
	SDK Overview Page
	Main SDK management interface with service and version management
-->

<script lang="ts">
  import { getContext } from 'svelte';
  import { useQueryClient } from '@tanstack/svelte-query';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import Database from '~icons/fa6-solid/database';
  import Code from '~icons/fa6-solid/code';
  import Globe from '~icons/fa6-solid/globe';
  import Container from '~icons/fa6-solid/cube';
  import Package from '~icons/fa6-solid/box-open';
  import Settings from '~icons/fa6-solid/gear';
  import Play from '~icons/fa6-solid/play';
  import Download from '~icons/fa6-solid/download';
  import AlertCircle from '~icons/fa6-solid/circle-exclamation';
  import CheckCircle from '~icons/fa6-solid/circle-check';
  import Activity from '~icons/fa6-solid/wave-square';
  import RefreshCw from '~icons/fa6-solid/rotate';
  import Zap from '~icons/fa6-solid/bolt';
  import Devicon from '$lib/components/ui/devicon.svelte';
  import { PageError, PageHeader, PageStats } from '$lib/components/shell';
  import { SDK_QUERIES_CONTEXT, type SdkQueries } from '$lib/domains/sdk/queries/sdkQueries';
  import { formatSdkVersion } from '$lib/domains/sdk/utils/normalizeVersions';
  import { goto } from '$app/navigation';

  // Read from the single pair of queries `/sdk/+layout.svelte` creates — not our own
  // `createQuery` calls. Two independent observers for the same key, both racing to resolve
  // on a cold mount, could diverge (one gets `.data`, the other gets stuck); sharing the
  // layout's observers with the sidebar makes that divergence impossible.
  const { configsQuery, managersQuery } = getContext<SdkQueries>(SDK_QUERIES_CONTEXT);
  const queryClient = useQueryClient();
  const pending = $derived(configsQuery.isPending || managersQuery.isPending);
  const queryError = $derived(configsQuery.error ?? managersQuery.error);
  const configs = $derived(configsQuery.data ?? []);
  const managerList = $derived(managersQuery.data ?? []);

  const isBusy = $derived(pending || configsQuery.isFetching || managersQuery.isFetching);

  // Every mutation changes what the sidebar and counters show.
  async function reload() {
    await queryClient.invalidateQueries({ queryKey: ['sdk'] });
  }

  // Statistics
  const serviceConfigs = $derived(configs.filter((c) => c.service_running != null));
  const totalServices = $derived(serviceConfigs.length);
  const runningServices = $derived(serviceConfigs.filter((c) => c.service_running).length);
  const totalSDKs = $derived(configs.length);
  const installedSDKCount = $derived(configs.filter((c) => c.sdk_installed).length);
  const totalManagers = $derived(managerList.length);
  const installedManagers = $derived(managerList.filter((m) => m.installed).length);

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

  interface QuickNavItem {
    id: string;
    displayName: string;
    icon: string;
    category: string;
    installed: boolean;
    version: string | null;
  }

  function getSDKRoute(sdkId: string, category: string): string {
    const normalizedId = sdkId.toLowerCase().trim();
    // SDK Managers go to /sdk/managers/[name]; everything else to /sdk/[sdk].
    return category === 'manager' ? `/sdk/managers/${normalizedId}` : `/sdk/${normalizedId}`;
  }

  // Organize SDKs by category
  const quickNavSDKs = $derived.by(() => {
    const categories: Record<string, QuickNavItem[]> = {
      manager: [],
      language: [],
      database: [],
      web: [],
      container: [],
    };

    const toItem = (c: (typeof configs)[number]): QuickNavItem => ({
      id: c.id,
      displayName: c.display_name,
      icon: getSDKIcon(c.id),
      category: c.category,
      // A running service proves the SDK is there even when binary detection misses it
      installed: c.sdk_installed || c.service_running === true,
      version: formatSdkVersion(c.sdk_version),
    });
    categories.manager = managerList.map((m) => ({
      id: m.id,
      displayName: m.display_name,
      icon: getSDKIcon(m.id),
      category: 'manager',
      installed: m.installed,
      version: formatSdkVersion(m.version),
    }));
    for (const c of configs) {
      if (c.category === 'database') categories.database.push(toItem(c));
      else if (c.category === 'server') categories.web.push(toItem(c));
      else if (c.category === 'container') categories.container.push(toItem(c));
      else if (c.category === 'language') categories.language.push(toItem(c));
    }

    return categories;
  });

  const quickNavEmpty = $derived(Object.values(quickNavSDKs).every((list) => list.length === 0));

  function handleSDKClick(sdk: QuickNavItem) {
    goto(getSDKRoute(sdk.id, sdk.category));
  }

  // Check if a service is running for an SDK
  function isServiceRunning(sdkId: string): boolean {
    return configs.some((c) => c.id === sdkId && c.service_running);
  }
</script>

{#snippet sdkGroup(title: string, icon: typeof Code, items: QuickNavItem[])}
  {#if items.length > 0}
    {@const Icon = icon}
    <div class="space-y-3">
      <div class="flex items-center gap-2">
        <Icon class="h-4 w-4 text-muted-foreground" />
        <h3 class="text-sm font-medium">{title}</h3>
      </div>
      <div class="grid grid-cols-[repeat(auto-fill,minmax(min(100%,13rem),1fr))] gap-3">
        {#each items as sdk (sdk.id)}
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
              <span class="truncate text-xs text-muted-foreground">
                {sdk.version ?? (sdk.installed ? 'Installed' : 'Not installed')}
              </span>
              <div class="flex items-center gap-1">
                {#if isServiceRunning(sdk.id)}
                  <Badge variant="default" class="bg-blue-500 px-1 py-0 text-xs">Running</Badge>
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
{/snippet}

<div class="space-y-6">
  <PageHeader title="SDK Management" description="Development environment and SDK versions">
    {#snippet actions()}
      <Button variant="outline" size="sm" onclick={reload} disabled={isBusy}>
        <RefreshCw class="mr-2 h-4 w-4 {isBusy ? 'animate-spin' : ''}" />
        Refresh
      </Button>
    {/snippet}
  </PageHeader>

  {#if queryError}
    <PageError
      title="Failed to load SDK status"
      message={queryError instanceof Error ? queryError.message : String(queryError)}
      onRetry={reload}
    />
  {/if}

  <PageStats
    stats={[
      {
        label: 'Managers installed',
        value: pending ? '—' : `${installedManagers}/${totalManagers}`,
        icon: Package,
      },
      {
        label: 'Services running',
        value: pending ? '—' : `${runningServices}/${totalServices}`,
        icon: Database,
      },
      {
        label: 'SDKs installed',
        value: pending ? '—' : `${installedSDKCount}/${totalSDKs}`,
        icon: Code,
      },
      {
        label: 'Status',
        value: pending ? '—' : queryError ? 'Unknown' : runningServices > 0 ? 'Online' : 'Offline',
        icon: Globe,
      },
    ]}
  />

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
  {:else}
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
          <Button variant="outline" onclick={() => goto('/sdk/managers')}>
            <Settings class="mr-2 h-4 w-4" />
            SDK Managers
          </Button>
          <Button variant="outline" onclick={() => goto('/sdk/software-installer')}>
            <Package class="mr-2 h-4 w-4" />
            Software Installer
          </Button>
          <Button variant="outline" onclick={() => goto('/sdk/services')}>
            <Play class="mr-2 h-4 w-4" />
            Services
          </Button>
        </div>
      </CardContent>
    </Card>

    <!-- Quick Navigation Section -->
    <Card>
      <CardHeader>
        <CardTitle class="flex items-center gap-2">
          <Activity class="h-5 w-5" />
          Quick Navigation
        </CardTitle>
        <p class="mt-1 text-sm text-muted-foreground">
          Quick access to all SDKs and managers. Green indicates installed/active, gray indicates
          not installed.
        </p>
      </CardHeader>
      <CardContent>
        <div class="space-y-6">
          {@render sdkGroup('SDK Managers', Package, quickNavSDKs.manager)}
          {@render sdkGroup('Language & Runtime', Code, quickNavSDKs.language)}
          {@render sdkGroup('Database Server', Database, quickNavSDKs.database)}
          {@render sdkGroup('Web Server', Globe, quickNavSDKs.web)}
          {@render sdkGroup('Container Platform', Container, quickNavSDKs.container)}

          {#if quickNavEmpty}
            <div class="py-8 text-center text-muted-foreground">
              <AlertCircle class="mx-auto mb-2 h-12 w-12 opacity-50" />
              <p class="mb-2 text-sm">No SDKs detected.</p>
              <p class="text-xs">
                Click "Refresh" to detect installed SDKs, or install SDK managers to get started.
              </p>
            </div>
          {/if}
        </div>
      </CardContent>
    </Card>

    <!-- Quick Start Section -->
    {#if !queryError && installedSDKCount === 0}
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
              <Button onclick={() => goto('/sdk/versions')}>
                <Download class="mr-2 h-4 w-4" />
                Install SDK
              </Button>
              <Button variant="outline" onclick={() => goto('/sdk/services')}>
                <Play class="mr-2 h-4 w-4" />
                Start Service
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>
    {/if}
  {/if}
</div>
