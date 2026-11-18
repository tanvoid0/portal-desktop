<!-- Top navigation block of the SDK sidebar (Overview / Managers / Services / Versions). -->
<script lang="ts">
  import Settings from '~icons/fa6-solid/gear';
  import Package from '~icons/fa6-solid/box-open';
  import Download from '~icons/fa6-solid/download';
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import { MenuButton as SidebarMenuButton } from '$lib/components/ui/sidebar';

  let { navigationItemIds }: { navigationItemIds?: string[] } = $props();

  const currentPath = $derived(page.url.pathname);

  const navigationItems = [
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
  const resolvedNavigationItems = $derived(
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
</script>

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
