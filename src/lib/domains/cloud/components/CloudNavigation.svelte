<!--
	Cloud Navigation - Sidebar navigation for cloud resources, generated from RESOURCE_DESCRIPTORS
-->

<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { MenuButton as SidebarMenuButton } from '$lib/components/ui/sidebar';
  import NamespaceSelector from '$lib/domains/cloud/components/NamespaceSelector.svelte';
  import { useKeyboardShortcuts } from '$lib/domains/k8s-navigation';
  import { RESOURCE_TYPE_SHORTCUTS } from '$lib/domains/k8s-navigation/utils/keyboardConstants';
  import { cloudStore } from '$lib/domains/cloud/stores';
  import {
    RESOURCE_DESCRIPTORS,
    type ResourceDescriptor,
  } from '$lib/domains/cloud/core/resourceDescriptors';

  interface NavItem {
    id: string;
    label: string;
    description: string;
    icon: string;
    route: string;
    /** Extra path prefixes that keep this item highlighted — detail and create pages. */
    prefixes: string[];
  }

  interface NavGroup {
    title: string;
    items: NavItem[];
  }

  const GROUP_ORDER = ['Workloads', 'Configuration', 'Networking'] as const;

  function toNavItem(descriptor: ResourceDescriptor): NavItem {
    return {
      id: descriptor.slug,
      label: descriptor.label,
      description: descriptor.description,
      icon: descriptor.icon,
      route: `/cloud/${descriptor.slug}`,
      prefixes: [`/cloud/${descriptor.slug}/`, `${descriptor.detailBase}/`],
    };
  }

  const navGroups: NavGroup[] = [
    {
      title: 'Cloud (Kubernetes)',
      items: [
        {
          id: 'overview',
          label: 'Overview',
          description: 'Cluster overview and metrics',
          icon: '☸️',
          route: '/cloud/workloads',
          prefixes: [],
        },
      ],
    },
    ...GROUP_ORDER.map((title) => ({
      title,
      items: RESOURCE_DESCRIPTORS.filter((descriptor) => descriptor.group === title).map(toNavItem),
    })),
  ];

  function isActiveTab(item: NavItem): boolean {
    const currentPath = page.url.pathname;
    return (
      currentPath === item.route || item.prefixes.some((prefix) => currentPath.startsWith(prefix))
    );
  }

  function handleItemClick(route: string) {
    goto(route);
  }

  // Resource type navigation shortcuts — the digit keys in RESOURCE_TYPE_SHORTCUTS
  const resourceTypeShortcuts = useKeyboardShortcuts(
    [
      {
        key: RESOURCE_TYPE_SHORTCUTS.OVERVIEW,
        description: 'Go to Overview',
        action: () => goto('/cloud/workloads'),
      },
      ...RESOURCE_DESCRIPTORS.flatMap((descriptor) => {
        const key = (RESOURCE_TYPE_SHORTCUTS as Record<string, string | undefined>)[
          descriptor.slug.toUpperCase()
        ];
        return key
          ? [
              {
                key,
                description: `Go to ${descriptor.label}`,
                action: () => goto(`/cloud/${descriptor.slug}`),
              },
            ]
          : [];
      }),
    ],
    { enabled: $cloudStore.connection.isConnected }
  );

  function handleKeydown(event: KeyboardEvent) {
    resourceTypeShortcuts.handleKeydown(event);
  }

  onMount(() => {
    window.addEventListener('keydown', handleKeydown);
    return () => {
      window.removeEventListener('keydown', handleKeydown);
    };
  });
</script>

<div class="space-y-6">
  <!-- Namespace Selector -->
  <div class="divider-edge-b divider-edge-full pb-4 group-data-[collapsible=icon]:hidden">
    <NamespaceSelector />
  </div>

  <!-- Navigation Groups -->
  <nav class="space-y-6">
    {#each navGroups as group}
      <div>
        <h3
          class="mb-2 px-3 text-xs font-semibold tracking-wider text-muted-foreground uppercase group-data-[collapsible=icon]:hidden"
        >
          {group.title}
        </h3>
        <div class="space-y-1">
          {#each group.items as item}
            {@const isActive = isActiveTab(item)}
            <SidebarMenuButton
              size="default"
              {isActive}
              tooltipContent={item.label}
              onclick={() => handleItemClick(item.route)}
              class="items-start"
            >
              <span class="flex-shrink-0 text-lg">{item.icon}</span>
              <div class="min-w-0 flex-1 text-left group-data-[collapsible=icon]:hidden">
                <div class="text-sm leading-tight font-medium">
                  {item.label}
                </div>
                <p class="mt-0.5 text-xs leading-relaxed opacity-70">
                  {item.description}
                </p>
              </div>
            </SidebarMenuButton>
          {/each}
        </div>
      </div>
      {#if group !== navGroups[navGroups.length - 1]}
        <div class="divider-line-h group-data-[collapsible=icon]:hidden"></div>
      {/if}
    {/each}
  </nav>
</div>
