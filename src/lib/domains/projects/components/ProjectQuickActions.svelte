<!-- Quick-launch buttons (explorer, Claude) plus the ellipsis menu. Shared by the list row and the grid card. -->
<script lang="ts">
  import type { Project, ProjectStage } from '$lib/domains/projects/types';
  import { PROJECT_STAGES } from '$lib/domains/projects/types';
  import { Button } from '$lib/components/ui/button';
  import * as Dialog from '$lib/components/ui/dialog';
  import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuLabel,
    DropdownMenuRadioGroup,
    DropdownMenuRadioItem,
    DropdownMenuSeparator,
    DropdownMenuSub,
    DropdownMenuSubContent,
    DropdownMenuSubTrigger,
    DropdownMenuTrigger,
  } from '$lib/components/ui/dropdown-menu';
  import Select from '$lib/components/ui/select.svelte';
  import { cn } from '$lib/utils';
  import Archive from '~icons/fa6-solid/box-archive';
  import TruckMoving from '~icons/fa6-solid/truck-moving';
  import Ellipsis from '~icons/fa6-solid/ellipsis';
  import ExternalLink from '~icons/fa6-solid/arrow-up-right-from-square';
  import Flag from '~icons/fa6-solid/flag';
  import Folder from '~icons/fa6-solid/folder';
  import FolderTree from '~icons/fa6-solid/folder-tree';
  import GitCommitHorizontal from '~icons/fa6-solid/code-commit';
  import Star from '~icons/fa6-solid/star';
  import Trash2 from '~icons/fa6-solid/trash';
  import Unlink from '~icons/fa6-solid/link-slash';
  import WandSparkles from '~icons/fa6-solid/wand-magic-sparkles';
  import FaBolt from '~icons/fa6-solid/bolt';
  import {
    openInExplorer,
    launchInCoder,
    launchInAgents,
  } from '$lib/domains/projects/utils/launch';
  import { goto } from '$app/navigation';
  import GitCommitDialog from '$lib/domains/coder/components/GitCommitDialog.svelte';
  import MoveProjectDialog from './MoveProjectDialog.svelte';
  import { queryClient, invalidateProjectsList } from '$lib/domains/shared/query';
  import { createProjectMovesQuery } from '$lib/domains/projects/queries/projectMoveQueries';
  import { findActiveMove } from '$lib/domains/projects/api/projectMoveApi';

  interface Props {
    project: Project;
    eligibleParents?: Array<{ id: string; name: string }>;
    onDelete?: (project: Project) => void;
    onSetParent?: (projectId: string, parentId: number | null) => void;
    onToggleStar?: (project: Project) => void;
    onSetStage?: (project: Project, stage: ProjectStage | null) => void;
    onSetArchived?: (project: Project, archived: boolean) => void;
    class?: string;
  }

  let {
    project,
    eligibleParents = [],
    onDelete = () => {},
    onSetParent = () => {},
    onToggleStar = () => {},
    onSetStage = () => {},
    onSetArchived = () => {},
    class: className = '',
  }: Props = $props();

  let groupOpen = $state(false);
  let commitOpen = $state(false);
  let moveOpen = $state(false);

  const movesQuery = createProjectMovesQuery();
  const activeMove = $derived(findActiveMove(movesQuery.data, Number(project.id)));

  const needsAttention = $derived(
    project.has_uncommitted_changes || (project.unpushed_commits ?? 0) > 0
  );

  // Read-only evidence shown next to the stage picker — never sets anything itself.
  const stageHint = $derived.by(() => {
    const parts: string[] = [];
    if (project.activity_status) parts.push(project.activity_status);
    if (project.risk_level && project.risk_level !== 'none') {
      parts.push(`risk ${project.risk_level}`);
    }
    if (project.unpushed_commits) parts.push(`${project.unpushed_commits} unpushed`);
    if (project.has_uncommitted_changes) parts.push('uncommitted');
    return parts.length ? parts.join(' · ') : 'no activity signals yet';
  });

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
  <Button
    variant="ghost"
    size="icon"
    class="h-8 w-8 opacity-0 transition-opacity group-hover:opacity-100 focus-within:opacity-100"
    title="Run agent here"
    aria-label="Run agent on {project.name}"
    onclick={() => launchInAgents(project)}
  >
    <FaBolt class="h-4 w-4" />
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
      {#if needsAttention}
        <DropdownMenuItem onclick={() => (commitOpen = true)}>
          <GitCommitHorizontal class="mr-2 h-4 w-4" />
          Review & commit…
        </DropdownMenuItem>
      {/if}
      <DropdownMenuSub>
        <DropdownMenuSubTrigger>
          <Flag class="mr-2 h-4 w-4" />
          Stage
        </DropdownMenuSubTrigger>
        <DropdownMenuSubContent>
          <DropdownMenuLabel class="text-xs text-muted-foreground">{stageHint}</DropdownMenuLabel>
          <DropdownMenuRadioGroup
            value={project.stage ?? ''}
            onValueChange={(value) => onSetStage(project, value ? (value as ProjectStage) : null)}
          >
            {#each PROJECT_STAGES as stage (stage.value)}
              <DropdownMenuRadioItem value={stage.value}>{stage.label}</DropdownMenuRadioItem>
            {/each}
          </DropdownMenuRadioGroup>
          {#if project.stage}
            <DropdownMenuItem onclick={() => onSetStage(project, null)}>Clear</DropdownMenuItem>
          {/if}
        </DropdownMenuSubContent>
      </DropdownMenuSub>
      <DropdownMenuItem onclick={() => onSetArchived(project, !project.archived)}>
        <Archive class="mr-2 h-4 w-4" />
        {project.archived ? 'Unarchive' : 'Archive'}
      </DropdownMenuItem>
      <DropdownMenuItem disabled={!!activeMove} onclick={() => (moveOpen = true)}>
        <TruckMoving class="mr-2 h-4 w-4" />
        {activeMove ? 'Moving…' : 'Move folder…'}
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

<GitCommitDialog
  bind:open={commitOpen}
  workspaceRoot={project.path}
  onCommitted={() => invalidateProjectsList(queryClient)}
/>

<MoveProjectDialog bind:open={moveOpen} {project} />
