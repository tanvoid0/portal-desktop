<script lang="ts">
  import FaBoxArchive from '~icons/fa6-solid/box-archive';
  import FaLayerGroup from '~icons/fa6-solid/layer-group';
  import FaFolder from '~icons/fa6-solid/folder';
  import FaInbox from '~icons/fa6-solid/inbox';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';

  interface Props {
    projects: { id: string; name: string; open: number }[];
    /** Selected project id: a project id, 'all' or 'none'. */
    selected: string;
    archiveActive: boolean;
    openTotal: number;
    noProjectOpen: number;
    showNoProject: boolean;
    archivedCount: number;
    onSelect: (project: string) => void;
    onArchive: () => void;
  }
  let {
    projects,
    selected,
    archiveActive,
    openTotal,
    noProjectOpen,
    showNoProject,
    archivedCount,
    onSelect,
    onArchive,
  }: Props = $props();
</script>

{#snippet navItem(label: string, count: number, active: boolean, onclick: () => void, Ico: any)}
  <Button
    {onclick}
    variant={active ? 'secondary' : 'ghost'}
    size="sm"
    class="h-7 w-full justify-between gap-2 px-2"
    aria-current={active ? 'true' : undefined}
  >
    <span class="flex min-w-0 items-center gap-2">
      <Ico class="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
      <span class="truncate" title={label}>{label}</span>
    </span>
    <Badge variant="outline" class="shrink-0 px-1.5">{count}</Badge>
  </Button>
{/snippet}

<nav class="flex flex-col gap-1 p-3" aria-label="Task projects">
  {@render navItem(
    'All projects',
    openTotal,
    selected === 'all' && !archiveActive,
    () => onSelect('all'),
    FaLayerGroup
  )}
  <div class="divider-edge-t divider-edge-full my-1"></div>
  {#each projects as p (p.id)}
    {@render navItem(
      p.name,
      p.open,
      selected === p.id && !archiveActive,
      () => onSelect(p.id),
      FaFolder
    )}
  {/each}
  {#if showNoProject}
    {@render navItem(
      'No project',
      noProjectOpen,
      selected === 'none' && !archiveActive,
      () => onSelect('none'),
      FaInbox
    )}
  {/if}
  <div class="divider-edge-t divider-edge-full my-1"></div>
  {@render navItem('Archive', archivedCount, archiveActive, onArchive, FaBoxArchive)}
</nav>
