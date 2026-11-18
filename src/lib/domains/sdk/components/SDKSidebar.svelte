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
  import ArrowLeft from '~icons/fa6-solid/arrow-left';
  import { invoke } from '@tauri-apps/api/core';
  import { logger } from '$lib/domains/shared';
  import type { ProcessedSDKConfig } from '$lib/domains/sdk/services/sdkConfigService';
  import { sdkConfigsQuery, sdkManagersQuery } from '$lib/domains/sdk/queries/sdkQueries';
  import { createQuery, useQueryClient } from '@tanstack/svelte-query';
  import { queryKeys } from '$lib/domains/shared/query/keys';
  import { goto } from '$app/navigation';
  import { useSidebar, MenuButton as SidebarMenuButton } from '$lib/components/ui/sidebar';

  import { getSDKIconColor } from '../utils/sdkIcons';
  import { buildSDKCategories, type SDKCategory, type SDKManager } from '../utils/sdkSidebarItems';
  import SDKSidebarNav from './sidebar/SDKSidebarNav.svelte';
  import SDKCategorySection from './sidebar/SDKCategorySection.svelte';
  import type { SDKItem } from './sidebar/sdkSidebarTypes';

  interface Props {
    selectedSDK?: string;
    navigationItemIds?: string[];
    onSDKSelect?: (sdk: SDKItem) => void;
  }

  const { selectedSDK, navigationItemIds, onSDKSelect }: Props = $props();

  // State
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

  const configsQuery = createQuery(sdkConfigsQuery);
  const managersQuery = createQuery(sdkManagersQuery);
  const queryClient = useQueryClient();
  const sdkConfigs = $derived<ProcessedSDKConfig[]>(configsQuery.data ?? []);
  const sdkManagers = $derived<SDKManager[]>(managersQuery.data ?? []);
  const sidebarLoading = $derived(configsQuery.isPending || managersQuery.isPending);

  function refreshSDKConfigsSilent() {
    return queryClient.invalidateQueries({ queryKey: queryKeys.sdk.configs });
  }

  const categories = $derived(buildSDKCategories(sdkConfigs, sdkManagers));

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

  // If nothing is selected yet, open languages + managers once data arrives. Once only: re-running
  // looped forever when there were no languages (each write re-triggered it), and re-opened the
  // section every time the user collapsed it. Empty categories are hidden, so opening them is moot.
  let defaultSectionsOpened = false;
  $effect(() => {
    if (sidebarLoading || defaultSectionsOpened) return;
    defaultSectionsOpened = true;
    if (selectedItem) return;
    openSections = { ...openSections, language: true, manager: true };
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
      <SDKSidebarNav {navigationItemIds} />

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
