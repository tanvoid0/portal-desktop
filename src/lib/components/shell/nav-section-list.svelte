<script lang="ts">
  import { Badge } from "$lib/components/ui/badge";
  import { MenuButton as SidebarMenuButton } from "$lib/components/ui/sidebar";
  import NavIcon from "./nav-icon.svelte";
  import type { NavSection } from "./nav-types";

  interface Props {
    sections: NavSection[];
    currentPath: string;
    onNavigate: (url: string) => void;
  }

  let { sections = [], currentPath, onNavigate }: Props = $props();
  function isNavItemActive(url: string, currentPath: string): boolean {
    if (currentPath === url) return true;
    if (url === "/") return false;
    return currentPath.startsWith(`${url}/`);
  }
</script>

{#each sections as section}
  <div class="px-3 py-2">
    <h2
      class="mb-1 px-2 text-xs font-semibold uppercase tracking-wider text-sidebar-foreground/60 group-data-[collapsible=icon]:hidden"
    >
      {section.title}
    </h2>
    <div class="space-y-1">
      {#each section.items as item}
        <SidebarMenuButton
          size="lg"
          isActive={isNavItemActive(item.url, currentPath)}
          tooltipContent={item.title}
          onclick={() => onNavigate(item.url)}
        >
          <NavIcon icon={item.icon} />
          <span class="group-data-[collapsible=icon]:hidden">{item.title}</span>
          {#if item.badge}
            <Badge
              variant="secondary"
              class="ml-auto text-xs group-data-[collapsible=icon]:hidden"
            >
              {item.badge}
            </Badge>
          {/if}
        </SidebarMenuButton>
      {/each}
    </div>
  </div>
{/each}
