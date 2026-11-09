<!--
	SDK Sidebar - FlyEnv-style sidebar with language and database categories
	Shows all available SDKs with toggle switches and selection states

	One tree for both sidebar states: the collapsed rail is expressed in CSS with
	`group-data-[collapsible=icon]:*`, the same mechanism as
	src/lib/components/layout/app-shell.svelte.
-->

<script lang="ts">
  import { ScrollArea } from '$lib/components/ui/scroll-area';
  import { Separator } from '$lib/components/ui/separator';
  import Settings from '~icons/fa6-solid/gear';
  import Database from '~icons/fa6-solid/database';
  import Code from '~icons/fa6-solid/code';
  import Globe from '~icons/fa6-solid/globe';
  import Container from '~icons/fa6-solid/cube';
  import Package from '~icons/fa6-solid/box-open';
  import Download from '~icons/fa6-solid/download';
  import ArrowLeft from '~icons/fa6-solid/arrow-left';
  import { getContext } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { logger } from '$lib/domains/shared';
  import type { ProcessedSDKConfig } from '$lib/domains/sdk/services/sdkConfigService';
  import { SDK_QUERIES_CONTEXT, type SdkQueries } from '$lib/domains/sdk/queries/sdkQueries';
  import { useQueryClient } from '@tanstack/svelte-query';
  import { queryKeys } from '$lib/domains/shared/query/keys';
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import { useSidebar, MenuButton as SidebarMenuButton } from '$lib/components/ui/sidebar';

  import SDKCategorySection from './sidebar/SDKCategorySection.svelte';
  import type { SDKItem } from './sidebar/sdkSidebarTypes';

  interface Props {
    selectedSDK?: string;
    navigationItemIds?: string[];
    onSDKSelect?: (sdk: SDKItem) => void;
  }

  interface SDKManager {
    id: string;
    name: string;
    display_name: string;
    installed: boolean;
    version: string | null;
    supports_installation: boolean;
    supports_version_switching: boolean;
    install_command: string | null;
    website: string | null;
  }

  const { selectedSDK, navigationItemIds, onSDKSelect }: Props = $props();

  // State
  const currentPath = $derived(page.url.pathname);
  const sidebar = useSidebar();
  // The rail only exists on the desktop sidebar; the mobile sheet is always full width.
  const railMode = $derived(sidebar.state === 'collapsed' && !sidebar.isMobile);
  let serviceToggleInFlight = $state<Set<string>>(new Set());
  let openSections = $state<Record<string, boolean>>({
    manager: false,
    language: false,
    database: false,
    web: false,
    container: false,
    ai: false,
  });
  let lastSelectedItemId = $state<string | null>(null);

  const { configsQuery, managersQuery } = getContext<SdkQueries>(SDK_QUERIES_CONTEXT);
  const queryClient = useQueryClient();
  const sdkConfigs = $derived<ProcessedSDKConfig[]>(configsQuery.data ?? []);
  const sdkManagers = $derived<SDKManager[]>(managersQuery.data ?? []);
  const sidebarLoading = $derived(configsQuery.isPending || managersQuery.isPending);

  function refreshSDKConfigsSilent() {
    return queryClient.invalidateQueries({ queryKey: queryKeys.sdk.configs });
  }

  // Navigation items - using actual routes
  type NavigationItem = {
    id: string;
    name: string;
    icon: typeof Settings;
    description: string;
  };

  const navigationItems: NavigationItem[] = [
    {
      id: '/sdk',
      name: 'Overview',
      icon: Settings,
      description: 'SDK overview and statistics',
    },
    {
      id: '/sdk/managers',
      name: 'SDK Managers',
      icon: Settings,
      description: 'Manage SDK version managers',
    },
    {
      id: '/sdk/services',
      name: 'Services',
      icon: Package,
      description: 'Start and stop SDK services',
    },
    {
      id: '/sdk/versions',
      name: 'Versions',
      icon: Download,
      description: 'Install and switch SDK versions',
    },
  ];

  // Allow the parent layout to trim the sidebar navigation per-section.
  // This keeps the sidebar reusable and avoids page-specific nav markup.
  const resolvedNavigationItems = $derived.by((): NavigationItem[] =>
    navigationItemIds?.length
      ? navigationItems.filter((item) => navigationItemIds.includes(item.id))
      : navigationItems
  );

  function isNavigationItemActive(itemId: string) {
    const normalizedCurrentPath =
      currentPath.endsWith('/') && currentPath !== '/' ? currentPath.slice(0, -1) : currentPath;

    const normalizedItemId = itemId.endsWith('/') && itemId !== '/' ? itemId.slice(0, -1) : itemId;

    // Mark the parent route as active for nested SDK-manager pages.
    if (normalizedItemId === '/sdk/managers') {
      return (
        normalizedCurrentPath === normalizedItemId ||
        normalizedCurrentPath.startsWith(`${normalizedItemId}/`)
      );
    }

    return normalizedCurrentPath === normalizedItemId;
  }

  /**
   * Convert SDK configs to SDKItem format. Languages are the one category that
   * can report a version through an installed SDK manager.
   */
  function toSDKItems(category: string, viaManagers = false): SDKItem[] {
    return sdkConfigs
      .filter((config) => config.category === category)
      .map((config) => {
        const rawVersion =
          config.sdk_version ||
          (viaManagers ? config.sdk_managers.find((m) => m.installed)?.version : null);

        return {
          id: config.id,
          name: config.name,
          displayName: config.display_name,
          icon: config.icon,
          category,
          installed:
            config.sdk_installed ||
            (viaManagers && config.sdk_managers.some((m) => m.installed)) ||
            false,
          enabled: true,
          version: (rawVersion ?? '').trim().replace(/^v/, '') || undefined,
          description: config.description,
          hasToggle: true,
          hasService: config.service_config != null,
          serviceRunning: config.service_running ?? null,
          port: config.service_port ?? null,
        } satisfies SDKItem;
      });
  }

  type SDKCategory = { key: string; title: string; icon: typeof Settings; items: SDKItem[] };

  const categories = $derived.by((): SDKCategory[] => [
    {
      key: 'manager',
      title: 'SDK Managers',
      icon: Settings,
      items: sdkManagers.map(
        (m) =>
          ({
            id: m.id,
            name: m.name ?? m.id,
            displayName: m.display_name ?? m.name ?? m.id,
            icon: getSDKIcon(m.id),
            category: 'manager',
            installed: m.installed,
            enabled: true,
            version: (m.version ?? '').trim().replace(/^v/, '') || undefined,
            description: undefined,
            hasToggle: false,
            hasService: false,
            serviceRunning: null,
            port: null,
          }) satisfies SDKItem
      ),
    },
    {
      key: 'language',
      title: 'Language & Runtime',
      icon: Code,
      items: toSDKItems('language', true),
    },
    { key: 'database', title: 'Database Server', icon: Database, items: toSDKItems('database') },
    { key: 'web', title: 'Web Server', icon: Globe, items: toSDKItems('server') },
    {
      key: 'container',
      title: 'Container Platform',
      icon: Container,
      items: toSDKItems('container'),
    },
    { key: 'ai', title: 'AI SDKs', icon: Code, items: toSDKItems('ai') },
  ]);

  const visibleCategories = $derived(categories.filter((c) => c.items.length > 0));
  const allSDKs = $derived(categories.flatMap((c) => c.items));

  function isServiceToggleDisabled(sdk: SDKItem) {
    return (
      !sdk.hasService ||
      !sdk.installed ||
      sdk.serviceRunning == null ||
      serviceToggleInFlight.has(sdk.id)
    );
  }

  function toggleSection(category: SDKCategory) {
    // The selected SDK's category stays open.
    if (category.items.some((s) => s.id === selectedItem)) return;
    openSections = { ...openSections, [category.key]: !openSections[category.key] };
  }

  async function setServiceRunning(sdk: SDKItem, running: boolean) {
    if (isServiceToggleDisabled(sdk)) return;

    try {
      // Clone to trigger reactivity.
      serviceToggleInFlight = new Set(serviceToggleInFlight);
      serviceToggleInFlight.add(sdk.id);

      const cmd = running ? 'start_service' : 'stop_service';
      await invoke(cmd, { sdkType: sdk.id });
    } catch (error) {
      logger.error('Failed to toggle service', {
        context: 'SDKSidebar',
        data: { sdkId: sdk.id, running },
        error,
      });
    } finally {
      serviceToggleInFlight = new Set(serviceToggleInFlight);
      serviceToggleInFlight.delete(sdk.id);
      await refreshSDKConfigsSilent();
    }
  }

  let selectedItem = $state<string | null>(null);

  $effect(() => {
    selectedItem = selectedSDK || null;
  });

  // Keep the currently selected SDK's category expanded. `categories` is still empty while
  // the SDK queries are pending, so don't commit `lastSelectedItemId` until a category
  // actually matched — otherwise this short-circuits on the next run (once the data has
  // arrived) without ever having opened anything.
  $effect(() => {
    if (!selectedItem) return;
    if (selectedItem === lastSelectedItemId) return;

    const matched = categories.find((category) =>
      category.items.some((s) => s.id === selectedItem)
    );
    if (!matched) return;

    openSections = { ...openSections, [matched.key]: true };
    lastSelectedItemId = selectedItem;
  });

  // If nothing is selected yet, show something by default.
  $effect(() => {
    if (sidebarLoading) return;
    if (selectedItem) return;
    if (openSections.language) return;
    openSections = {
      ...openSections,
      language: categories.some((c) => c.key === 'language' && c.items.length > 0),
      manager: categories.some((c) => c.key === 'manager' && c.items.length > 0),
    };
  });

  // Map SDK types to their route paths - use dynamic route
  function getSDKRoute(sdkId: string, category: string): string {
    // Normalize SDK ID for routing
    const normalizedId = sdkId.toLowerCase().trim();

    // SDK Managers go to /sdk/managers/[name]
    if (category === 'manager') {
      return `/sdk/managers/${normalizedId}`;
    }

    // Use the new dynamic route for all SDKs (language, database, ai, server, container, etc.)
    return `/sdk/${normalizedId}`;
  }

  function handleSDKClick(sdk: SDKItem) {
    selectedItem = sdk.id;
    onSDKSelect?.(sdk);

    // Get the appropriate route for this SDK
    const route = getSDKRoute(sdk.id, sdk.category);

    // Navigate to the appropriate route
    goto(route);

    logger.info('SDK selected', {
      context: 'SDKSidebar',
      data: {
        sdkId: sdk.id,
        sdkName: sdk.displayName,
        category: sdk.category,
        route,
      },
    });
  }

  function getSDKIcon(sdkType: string): string {
    const iconMap: Record<string, string> = {
      // Language & Runtime
      java: 'devicon-java-plain',
      node: 'devicon-nodejs-plain',
      python: 'devicon-python-plain',
      rust: 'devicon-rust-plain',
      go: 'devicon-go-plain',
      php: 'devicon-php-plain',
      ruby: 'devicon-ruby-plain',
      bun: 'devicon-bun-plain',
      deno: 'devicon-deno-plain',
      gradle: 'devicon-gradle-plain',
      kotlin: 'devicon-kotlin-plain',
      scala: 'devicon-scala-plain',
      erlang: 'devicon-erlang-plain',
      perl: 'devicon-perl-plain',

      // Database
      mysql: 'devicon-mysql-plain',
      postgresql: 'devicon-postgresql-plain',
      mongodb: 'devicon-mongodb-plain',
      mariadb: 'devicon-mariadb-plain',

      // Web Server
      nginx: 'devicon-nginx-original',
      apache: 'devicon-apache-plain',
      caddy: 'devicon-caddy-plain',

      // Container
      docker: 'devicon-docker-plain',
      kubernetes: 'devicon-kubernetes-plain',
      podman: 'devicon-podman-plain',

      // Package Managers
      npm: 'devicon-npm-original-wordmark',
      yarn: 'devicon-yarn-plain',
      pip: 'devicon-python-plain',
      cargo: 'devicon-rust-plain',
      composer: 'devicon-composer-plain',
      gem: 'devicon-ruby-plain',

      // SDK Managers
      nvm: 'devicon-nodejs-plain',
      pyenv: 'devicon-python-plain',
      rustup: 'devicon-rust-plain',
      sdkman: 'devicon-sdkman-plain',
      goenv: 'devicon-go-plain',
      rbenv: 'devicon-ruby-plain',
      phpenv: 'devicon-php-plain',
    };

    return iconMap[sdkType.toLowerCase()] || 'devicon-devicon-plain';
  }

  function getSDKIconColor(sdkId: string): string {
    const colorMap: Record<string, string> = {
      // Language & Runtime - colored icons
      java: 'text-orange-600',
      node: 'text-green-600',
      nodejs: 'text-green-600',
      python: 'text-blue-600',
      rust: 'text-orange-600',
      go: 'text-blue-500',
      php: 'text-purple-600',
      ruby: 'text-red-600',
      bun: 'text-yellow-600',
      deno: 'text-gray-800',
      gradle: 'text-blue-500',
      kotlin: 'text-purple-600',
      scala: 'text-red-600',
      erlang: 'text-red-500',
      perl: 'text-blue-700',

      // Database
      mysql: 'text-blue-600',
      postgresql: 'text-blue-700',
      mongodb: 'text-green-600',
      mariadb: 'text-blue-500',

      // Web Server
      nginx: 'text-green-600',
      apache: 'text-red-600',
      caddy: 'text-blue-600',

      // Container
      docker: 'text-blue-500',
      kubernetes: 'text-blue-600',
      podman: 'text-blue-600',

      // Package Managers
      npm: 'text-red-600',
      yarn: 'text-blue-600',
      pip: 'text-blue-600',
      cargo: 'text-orange-600',
      composer: 'text-gray-700',
      gem: 'text-red-600',

      // SDK Managers
      nvm: 'text-green-600',
      pyenv: 'text-blue-600',
      rustup: 'text-orange-600',
      sdkman: 'text-blue-600',
      goenv: 'text-blue-500',
      rbenv: 'text-red-600',
      phpenv: 'text-purple-600',
    };

    return colorMap[sdkId.toLowerCase()] || '';
  }
</script>

<div class="flex h-full min-h-0 flex-col overflow-hidden">
  <!-- Header -->
  <div
    class="divider-edge-b divider-edge-full flex-shrink-0 bg-background p-3 group-data-[collapsible=icon]:p-1"
  >
    <SidebarMenuButton
      size="sm"
      class="h-auto"
      tooltipContent="Back to Portal Desktop"
      onclick={() => goto('/')}
    >
      <ArrowLeft class="h-4 w-4" />
      <span class="min-w-0 flex-1 group-data-[collapsible=icon]:hidden">
        <span class="block truncate text-base font-semibold">SDK Manager</span>
        <span class="block truncate text-xs text-muted-foreground"
          >Manage your development environment</span
        >
      </span>
    </SidebarMenuButton>
  </div>

  <!-- Sidebar Content -->
  <ScrollArea class="min-h-0 flex-1 overflow-hidden">
    <div
      class="space-y-4 p-3 group-data-[collapsible=icon]:space-y-3 group-data-[collapsible=icon]:p-1"
    >
      <!-- Navigation Section -->
      <div class="space-y-2">
        <h3 class="text-xs font-medium text-muted-foreground group-data-[collapsible=icon]:hidden">
          Navigation
        </h3>
        <div class="space-y-0.5">
          {#each resolvedNavigationItems as item}
            {@const Icon = item.icon}
            <SidebarMenuButton
              size="sm"
              class="h-auto"
              isActive={isNavigationItemActive(item.id)}
              tooltipContent={item.name}
              onclick={() => goto(item.id)}
            >
              <Icon class="h-4 w-4" />
              <span class="min-w-0 flex-1 group-data-[collapsible=icon]:hidden">
                <span class="block truncate text-xs font-medium">{item.name}</span>
                <span class="block truncate text-xs text-muted-foreground">{item.description}</span>
              </span>
            </SidebarMenuButton>
          {/each}
        </div>
      </div>

      <Separator />

      {#if sidebarLoading}
        <div class="py-4 text-center text-muted-foreground">
          <div class="mb-2 text-xl">⏳</div>
          <h3 class="mb-1 text-sm font-medium">Loading SDKs...</h3>
        </div>
      {:else if visibleCategories.length === 0}
        <div class="py-4 text-center text-muted-foreground">
          <div class="mb-2 text-xl">🔍</div>
          <h3 class="mb-1 text-sm font-medium">No SDKs Detected</h3>
          <p class="text-xs group-data-[collapsible=icon]:hidden">
            No SDK managers or SDKs were found on your system. Install SDK managers like NVM, Pyenv,
            or SDKMAN to get started.
          </p>
        </div>
      {:else}
        {#each visibleCategories as category, index (category.key)}
          {#if index > 0}
            <Separator />
          {/if}
          <SDKCategorySection
            variant={railMode ? 'collapsed' : 'expanded'}
            title={category.title}
            iconComponent={category.icon}
            items={category.items}
            selectedItemId={selectedItem}
            collapsible={true}
            isOpen={openSections[category.key]}
            onToggle={() => toggleSection(category)}
            {getSDKIconColor}
            {isServiceToggleDisabled}
            onSDKClick={(sdk) => handleSDKClick(sdk)}
            onServiceToggle={(sdk, next) => setServiceRunning(sdk, next)}
          />
        {/each}
      {/if}
    </div>
  </ScrollArea>

  <!-- Footer -->
  <div class="divider-edge-t divider-edge-full p-3 group-data-[collapsible=icon]:hidden">
    <div class="space-y-1 text-center text-xs text-muted-foreground">
      <div>
        {#if sidebarLoading}
          Loading SDKs...
        {:else}
          {allSDKs.filter((sdk) => sdk.installed).length} of {allSDKs.length} SDKs
        {/if}
      </div>
    </div>
  </div>
</div>
