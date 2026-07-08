<!--
	Settings Navigation - Sidebar navigation for settings sections
-->

<script lang="ts">
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { Button } from "$lib/components/ui/button";
  import {
    Collapsible,
    CollapsibleContent,
    CollapsibleTrigger,
  } from "$lib/components/ui/collapsible";
  import {
    useSidebar,
    MenuButton as SidebarMenuButton,
  } from "$lib/components/ui/sidebar";
  import {
    Settings,
    Code,
    Terminal,
    Palette,
    Laptop,
    Brain,
    Package,
    Bot,
    Download,
    Box,
    FileCode,
    ChevronDown,
    Languages,
    Sparkles,
    FolderGit2,
  } from "@lucide/svelte";

  type SettingsSectionType =
    | "general"
    | "editor"
    | "terminal"
    | "theme"
    | "ides"
    | "frameworks-languages"
    | "frameworks"
    | "package-managers"
    | "languages"
    | "learning"
    | "ai"
    | "github"
    | "autonomy"
    | "updates";

  interface Props {
    currentSection?: SettingsSectionType | "framework-ides" | "updates";
    className?: string;
  }

  let { currentSection, className = "" }: Props = $props();

  // Check if we're in the frameworks-languages section or any of its sub-sections
  const isFrameworksLanguagesSection = $derived(() => {
    const path = $page.url.pathname;
    return (
      path.startsWith("/settings/frameworks-languages") ||
      path.startsWith("/settings/frameworks") ||
      path.startsWith("/settings/package-managers") ||
      path.startsWith("/settings/languages")
    );
  });

  // State for collapsible open/close
  let collapsibleOpen = $state(false);

  $effect(() => {
    collapsibleOpen = isFrameworksLanguagesSection();
  });

  const sidebar = useSidebar();

  const sections = [
    {
      id: "general" as const,
      label: "General",
      description: "Application preferences",
      icon: Settings,
      path: "/settings/general",
    },
    {
      id: "updates" as const,
      label: "Updates",
      description: "Check for updates",
      icon: Download,
      path: "/settings/updates",
    },
    {
      id: "editor" as const,
      label: "Editor",
      description: "Code editor settings",
      icon: Code,
      path: "/settings/editor",
    },
    {
      id: "terminal" as const,
      label: "Terminal",
      description: "Terminal configuration",
      icon: Terminal,
      path: "/settings/terminal",
    },
    {
      id: "ides" as const,
      label: "IDEs",
      description: "Configure IDEs & framework mappings",
      icon: Laptop,
      path: "/settings/ides",
    },
    {
      id: "frameworks-languages" as const,
      label: "Frameworks & Languages",
      description: "Languages, frameworks & package managers",
      icon: Languages,
      path: "/settings/frameworks-languages",
      subSections: [
        {
          id: "languages" as const,
          label: "Languages",
          path: "/settings/languages",
        },
        {
          id: "package-managers" as const,
          label: "Package Managers",
          path: "/settings/package-managers",
        },
        {
          id: "frameworks" as const,
          label: "Frameworks",
          path: "/settings/frameworks",
        },
      ],
    },
    {
      id: "github" as const,
      label: "GitHub",
      description: "OAuth app and account connection",
      icon: FolderGit2,
      path: "/settings/github",
    },
    {
      id: "theme" as const,
      label: "Theme",
      description: "Appearance & colors",
      icon: Palette,
      path: "/settings/theme",
    },
    {
      id: "learning" as const,
      label: "Learning",
      description: "ML learning & AI settings",
      icon: Brain,
      path: "/settings/learning",
    },
    {
      id: "ai" as const,
      label: "AI Providers",
      description: "Agent platform & model connections",
      icon: Sparkles,
      path: "/settings/ai",
    },
    {
      id: "autonomy" as const,
      label: "Autonomy",
      description: "Autonomous action settings",
      icon: Bot,
      path: "/settings/autonomy",
    },
  ];

  // Derive current section from URL if not provided
  const activeSection = $derived((): SettingsSectionType => {
    if (currentSection) {
      // Map framework-ides to ides
      return currentSection === "framework-ides" ? "ides" : currentSection;
    }
    const path = $page.url.pathname;
    if (path === "/settings" || path === "/settings/") return "general";
    const section = path.replace("/settings/", "").replace(/\/$/, "");
    // Redirect framework-ides to ides
    if (section === "framework-ides") return "ides";
    // Map old paths to new structure
    if (
      section === "frameworks" ||
      section === "package-managers" ||
      section === "languages"
    ) {
      return "frameworks-languages";
    }
    const normalizedSection = section || "general";
    // Type guard to ensure we return a valid section
    if (
      [
        "general",
        "editor",
        "terminal",
        "theme",
        "ides",
        "frameworks-languages",
        "frameworks",
        "package-managers",
        "languages",
        "learning",
        "ai",
        "github",
        "autonomy",
        "updates",
      ].includes(normalizedSection)
    ) {
      return normalizedSection as SettingsSectionType;
    }
    return "general";
  });

  // Get active sub-section for frameworks-languages
  const activeSubSection = $derived(() => {
    if (!isFrameworksLanguagesSection()) return null;
    const path = $page.url.pathname;
    if (path.includes("/languages")) return "languages";
    if (path.includes("/package-managers")) return "package-managers";
    if (path.includes("/frameworks") && !path.includes("/frameworks-languages"))
      return "frameworks";
    // If on the main frameworks-languages page, check query param
    if (path.includes("/frameworks-languages")) {
      const tab = $page.url.searchParams.get("tab");
      if (tab) return tab;
    }
    return null;
  });

  function handleSectionClick(sectionId: string, path: string) {
    goto(path);
  }

  function handleSubSectionClick(path: string) {
    goto(path);
  }

  function handleCollapsedSectionClick(sectionId: string, path: string) {
    // Preserve the "tab" query param when collapsing from a sub-section.
    if (sectionId === "frameworks-languages") {
      const tab = activeSubSection();
      if (tab) {
        goto(`${path}?tab=${tab}`);
        return;
      }
    }

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
        onclick={() => handleCollapsedSectionClick(section.id, section.path)}
      >
        <Icon class="h-4 w-4 flex-shrink-0" />
      </SidebarMenuButton>
    {/each}
  {:else}
    {#each sections as section}
      {@const isActive = activeSection() === section.id}
      {#if section.subSections}
        <Collapsible bind:open={collapsibleOpen} class="w-full">
          <CollapsibleTrigger
            type="button"
            onclick={() => handleSectionClick(section.id, section.path)}
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
            <ChevronDown
              class="ml-2 mt-0.5 h-4 w-4 transition-transform {collapsibleOpen
                ? 'rotate-180'
                : ''}"
            />
          </CollapsibleTrigger>
          <CollapsibleContent>
            <div class="ml-4 mt-1 space-y-1 border-l-2 border-border pl-4">
              {#each section.subSections as subSection}
                {@const isSubActive = activeSubSection() === subSection.id}
                <Button
                  type="button"
                  variant="ghost"
                  onclick={() => handleSubSectionClick(subSection.path)}
                  class="flex h-auto w-full items-center justify-start rounded-md px-3 py-2 text-sm transition-colors {isSubActive
                    ? 'bg-accent font-medium text-accent-foreground'
                    : 'text-muted-foreground hover:bg-accent/50 hover:text-foreground'}"
                >
                  <span class="text-sm">{subSection.label}</span>
                </Button>
              {/each}
            </div>
          </CollapsibleContent>
        </Collapsible>
      {:else}
        <Button
          type="button"
          variant="ghost"
          onclick={() => handleSectionClick(section.id, section.path)}
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
      {/if}
    {/each}
  {/if}
</nav>
