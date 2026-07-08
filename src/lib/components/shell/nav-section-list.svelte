<script lang="ts">
  import { Badge } from "$lib/components/ui/badge";
  import {
    MenuButton as SidebarMenuButton,
    MenuSub,
    MenuSubButton,
    MenuSubItem,
  } from "$lib/components/ui/sidebar";
  import {
    Collapsible,
    CollapsibleContent,
    CollapsibleTrigger,
  } from "$lib/components/ui/collapsible";
  import { ChevronDown } from "@lucide/svelte";
  import NavIcon from "./nav-icon.svelte";
  import type { NavItem, NavSection } from "./nav-types";

  interface Props {
    sections: NavSection[];
    currentPath: string;
    onNavigate: (url: string) => void;
  }

  let { sections = [], currentPath, onNavigate }: Props = $props();

  function isNavItemActive(url: string, path: string): boolean {
    if (path === url) return true;
    if (url === "/") return false;
    return path.startsWith(`${url}/`);
  }

  function isSubmenuActive(item: NavItem, path: string): boolean {
    if (isNavItemActive(item.url, path)) return true;
    return item.submenu?.some((sub) => isNavItemActive(sub.url, path)) ?? false;
  }

  function submenuOpen(item: NavItem): boolean {
    return isSubmenuActive(item, currentPath);
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
        {#if item.submenu && item.submenu.length > 0}
          <Collapsible open={submenuOpen(item)} class="group/collapsible">
            <CollapsibleTrigger class="w-full">
              <SidebarMenuButton
                size="lg"
                isActive={isSubmenuActive(item, currentPath)}
                tooltipContent={item.title}
                class="w-full"
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
                <ChevronDown
                  class="ml-auto h-4 w-4 shrink-0 transition-transform group-data-[collapsible=icon]:hidden group-data-[state=open]/collapsible:rotate-180"
                />
              </SidebarMenuButton>
            </CollapsibleTrigger>
            <CollapsibleContent>
              <MenuSub>
                {#each item.submenu as subItem}
                  <MenuSubItem>
                    <MenuSubButton
                      isActive={isNavItemActive(subItem.url, currentPath)}
                      onclick={() => onNavigate(subItem.url)}
                    >
                      <NavIcon icon={subItem.icon} />
                      <span>{subItem.title}</span>
                    </MenuSubButton>
                  </MenuSubItem>
                {/each}
              </MenuSub>
            </CollapsibleContent>
          </Collapsible>
        {:else}
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
        {/if}
      {/each}
    </div>
  </div>
{/each}
