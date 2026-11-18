<script lang="ts">
  import type { Snippet } from 'svelte';
  import ChevronRight from '~icons/fa6-solid/chevron-right';
  import {
    Collapsible,
    CollapsibleContent,
    CollapsibleTrigger,
  } from '$lib/components/ui/collapsible';
  import type { Project } from '$lib/domains/projects/types';
  import type { ProjectGroup } from '$lib/domains/projects/utils/groupProjects';
  import type { ViewMode } from '$lib/domains/projects/utils/viewPrefs';

  interface Props {
    groups: ProjectGroup[];
    viewMode: ViewMode;
    /** While a filter is on, every match stays expanded and the toggle is inert. */
    isFiltered: boolean;
    isGroupOpen: (group: ProjectGroup) => boolean;
    isLoneProject: (group: ProjectGroup) => boolean;
    onGroupOpenChange: (key: string, open: boolean) => void;
    row: Snippet<[Project, boolean, string]>;
    card: Snippet<[Project]>;
  }
  let {
    groups,
    viewMode,
    isFiltered,
    isGroupOpen,
    isLoneProject,
    onGroupOpenChange,
    row,
    card,
  }: Props = $props();
</script>

{#snippet cardGrid(items: ProjectGroup['items'])}
  <div class="grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-3">
    {#each items as item (item.project.id)}
      {@render card(item.project)}
      {#each item.children as child (child.id)}
        {@render card(child)}
      {/each}
    {/each}
  </div>
{/snippet}

<div class="space-y-6">
  {#each groups as group (group.key)}
    {#if isLoneProject(group)}
      {#if viewMode === 'grid'}
        {@render cardGrid(group.items)}
      {:else}
        {@render row(group.items[0].project, false, group.title)}
      {/if}
    {:else}
      <Collapsible
        bind:open={() => isGroupOpen(group), (open) => onGroupOpenChange(group.key, open)}
      >
        <section>
          <h2 class="mb-1 border-b px-3 pb-1">
            <CollapsibleTrigger
              disabled={isFiltered}
              class="group flex w-full items-baseline gap-2 rounded text-sm font-semibold outline-none focus-visible:ring-2 focus-visible:ring-ring"
            >
              <ChevronRight
                class="h-3 w-3 shrink-0 text-muted-foreground transition-transform group-data-[state=open]:rotate-90"
              />
              <span class="truncate">{group.title}</span>
              <span class="text-xs font-normal text-muted-foreground">{group.items.length}</span>
            </CollapsibleTrigger>
          </h2>
          <CollapsibleContent>
            {#if viewMode === 'grid'}
              {@render cardGrid(group.items)}
            {:else}
              {#each group.items as item (item.project.id)}
                {@render row(item.project, false, '')}
                {#each item.children as child (child.id)}
                  {@render row(child, true, '')}
                {/each}
              {/each}
            {/if}
          </CollapsibleContent>
        </section>
      </Collapsible>
    {/if}
  {/each}
</div>
