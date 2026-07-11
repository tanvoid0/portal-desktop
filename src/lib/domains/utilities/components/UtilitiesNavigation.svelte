<script lang="ts">
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { Button } from "$lib/components/ui/button";
  import {
    useSidebar,
    MenuButton as SidebarMenuButton,
  } from "$lib/components/ui/sidebar";
  import { HardDrive, Variable } from "@lucide/svelte";

  type UtilitySection = "disk" | "environment";

  interface Props {
    currentSection?: UtilitySection;
    className?: string;
  }

  let { currentSection, className = "" }: Props = $props();

  const sidebar = useSidebar();

  const sections = [
    {
      id: "disk" as const,
      label: "Disk Utility",
      description: "AI-assisted disk cleanup with human review",
      icon: HardDrive,
      path: "/utilities/disk",
    },
    {
      id: "environment" as const,
      label: "Environment Variables",
      description: "Edit user and system environment variables",
      icon: Variable,
      path: "/utilities/environment",
    },
  ];

  const activeSection = $derived((): UtilitySection => {
    if (currentSection) return currentSection;
    const path = $page.url.pathname;
    if (path.startsWith("/utilities/environment")) return "environment";
    if (path.startsWith("/utilities/disk")) return "disk";
    return "disk";
  });

  function handleSectionClick(path: string) {
    goto(path);
  }
</script>

<nav class="space-y-2 {className}">
  {#if sidebar.state === "collapsed"}
    {#each sections as section}
      {@const isActive = activeSection() === section.id}
      {@const Icon = section.icon}
      <SidebarMenuButton
        {isActive}
        tooltipContent={section.label}
        onclick={() => handleSectionClick(section.path)}
      >
        <Icon class="h-4 w-4 flex-shrink-0" />
      </SidebarMenuButton>
    {/each}
  {:else}
    {#each sections as section}
      {@const isActive = activeSection() === section.id}
      <Button
        type="button"
        variant="ghost"
        onclick={() => handleSectionClick(section.path)}
        class="flex h-auto w-full items-start justify-start whitespace-normal rounded-lg px-4 py-3 text-base font-medium transition-colors {isActive
          ? 'bg-accent text-accent-foreground'
          : 'text-muted-foreground hover:bg-accent/50 hover:text-foreground'}"
      >
        {@const Icon = section.icon}
        <Icon class="mr-3 mt-0.5 h-5 w-5 flex-shrink-0" />
        <div class="min-w-0 flex-1 text-left">
          <div class="text-base font-semibold leading-tight">
            {section.label}
          </div>
          <p
            class="mt-1 break-words text-sm leading-relaxed text-muted-foreground"
          >
            {section.description}
          </p>
        </div>
      </Button>
    {/each}
  {/if}
</nav>
