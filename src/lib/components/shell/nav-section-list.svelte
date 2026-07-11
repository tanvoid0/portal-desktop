<script lang="ts">
  import { Badge } from "$lib/components/ui/badge";
  import {
    MenuAction as SidebarMenuAction,
    MenuButton as SidebarMenuButton,
    MenuItem as SidebarMenuItem,
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

  let submenuExpanded = $state<Record<string, boolean>>({});

  function isNavItemActive(url: string, path: string): boolean {
    if (path === url) return true;
    if (url === "/") return false;
    return path.startsWith(`${url}/`);
  }

  function isSubmenuActive(item: NavItem, path: string): boolean {
    if (isNavItemActive(item.url, path)) return true;
    return item.submenu?.some((sub) => isNavItemActive(sub.url, path)) ?? false;
  }

  function isSubmenuExpanded(item: NavItem): boolean {
    if (isSubmenuActive(item, currentPath)) return true;
    return submenuExpanded[item.url] ?? false;
  }

  function handleSubmenuOpenChange(item: NavItem, open: boolean) {
    if (isSubmenuActive(item, currentPath) && !open) return;
    submenuExpanded = { ...submenuExpanded, [item.url]: open };
  }

  function navigateToParent(item: NavItem) {
    onNavigate(item.url);
    submenuExpanded = { ...submenuExpanded, [item.url]: true };
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
          <Collapsible
            open={isSubmenuExpanded(item)}
            onOpenChange={(open) => handleSubmenuOpenChange(item, open)}
            class="group/collapsible"
          >
            <SidebarMenuItem>
              <SidebarMenuButton
                size="lg"
                isActive={isSubmenuActive(item, currentPath)}
                tooltipContent={item.title}
                onclick={() => navigateToParent(item)}
              >
                <NavIcon icon={item.icon} />
                <span class="group-data-[collapsible=icon]:hidden">{item.title}</span>
                {#if item.badge}
                  <Badge
                    variant="secondary"
                    class="ml-auto mr-6 text-xs group-data-[collapsible=icon]:hidden"
                  >
                    {item.badge}
                  </Badge>
                {/if}
              </SidebarMenuButton>
              <CollapsibleTrigger>
                {#snippet child({ props })}
                  <SidebarMenuAction
                    {...props}
                    aria-label="Toggle {item.title} submenu"
                  >
                    <ChevronDown
                      class="transition-transform group-data-[state=open]/collapsible:rotate-180"
                    />
                  </SidebarMenuAction>
                {/snippet}
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
            </SidebarMenuItem>
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
