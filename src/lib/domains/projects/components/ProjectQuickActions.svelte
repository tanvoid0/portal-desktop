<!-- Quick-launch buttons (explorer, Claude) plus the ellipsis menu. Shared by the list row and the grid card. -->
<script lang="ts">
  import type { Project } from '$lib/domains/projects/types';
  import { Button } from '$lib/components/ui/button';
  import * as Dialog from '$lib/components/ui/dialog';
  import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuSeparator,
    DropdownMenuTrigger,
  } from '$lib/components/ui/dropdown-menu';
  import Select from '$lib/components/ui/select.svelte';
  import { cn } from '$lib/utils';
  import Ellipsis from '~icons/fa6-solid/ellipsis';
  import ExternalLink from '~icons/fa6-solid/arrow-up-right-from-square';
  import Folder from '~icons/fa6-solid/folder';
  import FolderTree from '~icons/fa6-solid/folder-tree';
  import Star from '~icons/fa6-solid/star';
  import Trash2 from '~icons/fa6-solid/trash';
  import Unlink from '~icons/fa6-solid/link-slash';
  import WandSparkles from '~icons/fa6-solid/wand-magic-sparkles';
  import { openInExplorer, launchInCoder } from '$lib/domains/projects/utils/launch';
  import { goto } from '$app/navigation';

  interface Props {
    project: Project;
    eligibleParents?: Array<{ id: string; name: string }>;
    onDelete?: (project: Project) => void;
    onSetParent?: (projectId: string, parentId: number | null) => void;
    onToggleStar?: (project: Project) => void;
    class?: string;
  }

  let {
    project,
    eligibleParents = [],
    onDelete = () => {},
    onSetParent = () => {},
    onToggleStar = () => {},
    class: className = '',
  }: Props = $props();

  let groupOpen = $state(false);

  // The row/card wraps the whole tile in a link; these buttons are siblings of
  // it, so no click-through guarding is needed here.
</script>

<div class={cn('flex shrink-0 items-center gap-0.5', className)}>
  <Button
    variant="ghost"
    size="icon"
    class="h-8 w-8 opacity-0 transition-opacity group-hover:opacity-100 focus-within:opacity-100"
    title="Open in file explorer"
    aria-label="Open {project.name} in file explorer"
    onclick={() => openInExplorer(project.path)}
  >
    <Folder class="h-4 w-4" />
  </Button>
  <Button
    variant="ghost"
    size="icon"
    class="h-8 w-8 opacity-0 transition-opacity group-hover:opacity-100 focus-within:opacity-100"
    title="Launch in Claude"
    aria-label="Launch {project.name} in Claude"
    onclick={() => launchInCoder(project)}
  >
    <WandSparkles class="h-4 w-4" />
  </Button>

  <DropdownMenu>
    <DropdownMenuTrigger>
      {#snippet child({ props })}
        <Button
          {...props}
          variant="ghost"
          size="icon"
          class="h-8 w-8 shrink-0 opacity-0 transition-opacity group-hover:opacity-100 focus-within:opacity-100 data-[state=open]:opacity-100"
          aria-label="Actions for {project.name}"
        >
          <Ellipsis class="h-4 w-4" />
        </Button>
      {/snippet}
    </DropdownMenuTrigger>
    <DropdownMenuContent align="end">
      <DropdownMenuItem onclick={() => goto(`/projects/${project.id}`)}>
        <ExternalLink class="mr-2 h-4 w-4" />
        Open
      </DropdownMenuItem>
      <DropdownMenuItem onclick={() => onToggleStar(project)}>
        <Star class="mr-2 h-4 w-4" />
        {project.starred ? 'Unstar' : 'Star'}
      </DropdownMenuItem>
      {#if eligibleParents.length > 0}
        <DropdownMenuItem onclick={() => (groupOpen = true)}>
          <FolderTree class="mr-2 h-4 w-4" />
          Group under…
        </DropdownMenuItem>
      {/if}
      {#if project.parent_id != null}
        <DropdownMenuItem onclick={() => onSetParent(project.id, null)}>
          <Unlink class="mr-2 h-4 w-4" />
          Remove from group
        </DropdownMenuItem>
      {/if}
      <DropdownMenuSeparator />
      <DropdownMenuItem variant="destructive" onclick={() => onDelete(project)}>
        <Trash2 class="mr-2 h-4 w-4" />
        Delete
      </DropdownMenuItem>
    </DropdownMenuContent>
  </DropdownMenu>
</div>

<Dialog.Root bind:open={groupOpen}>
  <Dialog.Content class="max-w-sm">
    <Dialog.Header>
      <Dialog.Title>Group “{project.name}” under…</Dialog.Title>
    </Dialog.Header>
    <Select
      defaultValue=""
      placeholder="Choose a parent project"
      options={eligibleParents.map((p) => ({ value: p.id, label: p.name }))}
      onSelect={(value) => {
        if (!value) return;
        onSetParent(project.id, Number(value));
        groupOpen = false;
      }}
    />
  </Dialog.Content>
</Dialog.Root>
