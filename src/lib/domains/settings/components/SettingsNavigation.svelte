<!--
	Settings Navigation - Sidebar navigation for settings sections
-->

<script lang="ts" module>
  import Settings from '~icons/fa6-solid/gear';
  import Code from '~icons/fa6-solid/code';
  import Terminal from '~icons/fa6-solid/terminal';
  import Palette from '~icons/fa6-solid/palette';
  import Laptop from '~icons/fa6-solid/laptop';
  import Brain from '~icons/fa6-solid/brain';
  import Bot from '~icons/fa6-solid/robot';
  import Download from '~icons/fa6-solid/download';
  import Languages from '~icons/fa6-solid/language';
  import Sparkles from '~icons/fa6-solid/wand-magic-sparkles';
  import FolderGit2 from '~icons/fa6-solid/folder-tree';

  // Sections with `subSections` are group labels, not links: their children are the pages.
  // `path` on a group is only used for the collapsed rail icon.
  export const sections = [
    {
      id: 'general' as const,
      label: 'General',
      icon: Settings,
      path: '/settings/general',
    },
    {
      id: 'updates' as const,
      label: 'Updates',
      icon: Download,
      path: '/settings/updates',
    },
    {
      id: 'editor' as const,
      label: 'Editor',
      icon: Code,
      path: '/settings/editor',
    },
    {
      id: 'terminal' as const,
      label: 'Terminal',
      icon: Terminal,
      path: '/settings/terminal',
    },
    {
      id: 'ides' as const,
      label: 'IDEs',
      icon: Laptop,
      path: '/settings/ides',
    },
    {
      id: 'frameworks-languages' as const,
      label: 'Frameworks & Languages',
      icon: Languages,
      path: '/settings/languages',
      subSections: [
        {
          id: 'languages' as const,
          label: 'Languages',
          path: '/settings/languages',
        },
        {
          id: 'package-managers' as const,
          label: 'Package Managers',
          path: '/settings/package-managers',
        },
        {
          id: 'frameworks' as const,
          label: 'Frameworks',
          path: '/settings/frameworks',
        },
      ],
    },
    {
      id: 'github' as const,
      label: 'GitHub',
      icon: FolderGit2,
      path: '/settings/github',
    },
    {
      id: 'theme' as const,
      label: 'Theme',
      icon: Palette,
      path: '/settings/theme',
    },
    {
      id: 'learning' as const,
      label: 'Learning',
      icon: Brain,
      path: '/settings/learning',
    },
    {
      id: 'ai' as const,
      label: 'AI',
      icon: Sparkles,
      path: '/settings/ai/providers',
      subSections: [
        {
          id: 'providers' as const,
          label: 'Providers',
          path: '/settings/ai/providers',
        },
        {
          id: 'training' as const,
          label: 'Training Data',
          path: '/settings/ai/training',
        },
      ],
    },
    {
      id: 'autonomy' as const,
      label: 'Autonomy',
      icon: Bot,
      path: '/settings/autonomy',
    },
  ];

  function matches(pathname: string, path: string): boolean {
    return pathname === path || pathname.startsWith(`${path}/`);
  }

  /** Label for the active page, e.g. "General" or "AI › Providers"; derived from `sections`. */
  export function activeSectionLabel(pathname: string): string | undefined {
    for (const section of sections) {
      if (!section.subSections) {
        if (matches(pathname, section.path)) return section.label;
        continue;
      }
      const sub = section.subSections.find((s) => matches(pathname, s.path));
      if (sub) return `${section.label} › ${sub.label}`;
    }
  }
</script>

<script lang="ts">
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { Button } from '$lib/components/ui/button';
  import { useSidebar, MenuButton as SidebarMenuButton } from '$lib/components/ui/sidebar';

  const sidebar = useSidebar();

  const activePath = $derived(page.url.pathname);

  function isActive(path: string): boolean {
    return activePath === path || activePath.startsWith(`${path}/`);
  }

  function isSectionActive(section: (typeof sections)[number]): boolean {
    return section.subSections
      ? section.subSections.some((sub) => isActive(sub.path))
      : isActive(section.path);
  }
</script>

<nav class="space-y-0.5">
  {#if sidebar.state === 'collapsed'}
    {#each sections as section}
      {@const Icon = section.icon}
      <SidebarMenuButton
        isActive={isSectionActive(section)}
        tooltipContent={section.label}
        onclick={() => goto(section.path)}
      >
        <Icon class="h-4 w-4 flex-shrink-0" />
      </SidebarMenuButton>
    {/each}
  {:else}
    {#each sections as section}
      {@const Icon = section.icon}
      {#if section.subSections}
        <div class="px-4 pt-3 pb-1">
          <div
            class="flex items-center gap-2 text-xs font-semibold tracking-wide text-muted-foreground uppercase"
          >
            <Icon class="h-4 w-4 flex-shrink-0" />
            {section.label}
          </div>
        </div>
        <div class="divider-edge-l ml-4 space-y-1 pl-4">
          {#each section.subSections as subSection}
            <Button
              type="button"
              variant="ghost"
              onclick={() => goto(subSection.path)}
              aria-current={isActive(subSection.path) ? 'page' : undefined}
              class="flex h-8 w-full items-center justify-start rounded-md px-3 text-sm transition-colors {isActive(
                subSection.path
              )
                ? 'bg-accent font-medium text-accent-foreground'
                : 'text-muted-foreground hover:bg-accent/50 hover:text-foreground'}"
            >
              <span class="text-sm">{subSection.label}</span>
            </Button>
          {/each}
        </div>
      {:else}
        <Button
          type="button"
          variant="ghost"
          onclick={() => goto(section.path)}
          aria-current={isActive(section.path) ? 'page' : undefined}
          class="h-8 w-full justify-start gap-3 rounded-md px-3 text-sm font-medium transition-colors {isActive(
            section.path
          )
            ? 'bg-accent text-accent-foreground'
            : 'text-muted-foreground hover:bg-accent/50 hover:text-foreground'}"
        >
          <Icon class="h-4 w-4 flex-shrink-0" />
          <span class="truncate">{section.label}</span>
        </Button>
      {/if}
    {/each}
  {/if}
</nav>
