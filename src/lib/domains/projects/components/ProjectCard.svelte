<!-- Grid card: richer than the list row — full stack badges, description, and quick actions always visible. -->
<script lang="ts">
  import type { Project, ProjectModule, ProjectStage } from '$lib/domains/projects/types';
  import { PROJECT_STAGES } from '$lib/domains/projects/types';
  import { formatRelativeTime } from '$lib/domains/shared/utils';
  import { Badge } from '$lib/components/ui/badge';
  import { Card } from '$lib/components/ui/card';
  import TechIcon from '$lib/components/ui/tech-icon.svelte';
  import FolderOpen from '~icons/fa6-solid/folder-open';
  import GitBranch from '~icons/fa6-solid/code-branch';
  import Star from '~icons/fa6-solid/star';
  import {
    getProjectGitBranch,
    getStatusColor,
    readableDescription,
  } from '$lib/domains/projects/utils/display';
  import { describeModules } from '$lib/domains/projects/utils/moduleStacks';
  import { projectIconRegistry } from '$lib/domains/projects/utils/iconRegistry';
  import { cn } from '$lib/utils';
  import ProjectQuickActions from './ProjectQuickActions.svelte';
  import ContextMenuList from '$lib/components/menu/ContextMenuList.svelte';
  import { buildProjectMenu, copyProjectReference } from '$lib/domains/projects/utils/projectMenu';
  import { launchInCoder } from '$lib/domains/projects/utils/launch';
  import { createProjectMovesQuery } from '$lib/domains/projects/queries/projectMoveQueries';
  import { findActiveMove } from '$lib/domains/projects/api/projectMoveApi';

  interface Props {
    project: Project;
    modules?: ProjectModule[];
    registryReady?: boolean;
    eligibleParents?: Array<{ id: string; name: string }>;
    onDelete?: (project: Project) => void;
    onSetParent?: (projectId: string, parentId: number | null) => void;
    onToggleStar?: (project: Project) => void;
    onSetStage?: (project: Project, stage: ProjectStage | null) => void;
    onSetArchived?: (project: Project, archived: boolean) => void;
  }

  let {
    project,
    modules = [],
    registryReady = false,
    eligibleParents = [],
    onDelete = () => {},
    onSetParent = () => {},
    onToggleStar = () => {},
    onSetStage = () => {},
    onSetArchived = () => {},
  }: Props = $props();

  const stageLabel = $derived(PROJECT_STAGES.find((s) => s.value === project.stage)?.label);

  const DOT: Record<string, string> = {
    active: 'bg-green-500',
    recent: 'bg-blue-500',
    dormant: 'bg-amber-500',
    stale: 'bg-muted-foreground/40',
  };

  const primary = $derived(
    registryReady
      ? (projectIconRegistry.resolvePrimaryFramework(project) ??
          projectIconRegistry.resolveLanguages(project)[0])
      : undefined
  );
  const badges = $derived.by(() => {
    if (!registryReady) return [];
    const all = [
      ...projectIconRegistry.resolveFrameworks(project),
      ...projectIconRegistry.resolveLanguages(project),
    ];
    return all.filter((t, i) => all.findIndex((o) => o.name === t.name) === i);
  });
  const movesQuery = createProjectMovesQuery();
  const isMoving = $derived(!!findActiveMove(movesQuery.data, Number(project.id)));
  const branch = $derived(getProjectGitBranch(project));
  const description = $derived(readableDescription(project.description));
  const lastSeen = $derived(project.last_activity ?? project.last_opened);
  const activity = $derived(project.activity_status ?? 'stale');
  const unpushed = $derived(project.unpushed_commits ?? 0);
  const when = $derived(lastSeen ? formatRelativeTime(new Date(lastSeen)) : 'never');
</script>

<ContextMenuList
  items={buildProjectMenu(project, {
    onToggleStar,
    onSetArchived,
    onCopyReference: (p) => copyProjectReference(p, branch),
    onLaunchCoder: launchInCoder,
    onDelete,
  })}
>
  {#snippet trigger(menuProps)}
    <Card
      {...menuProps}
      class="group relative flex flex-col gap-3 p-4 transition-colors hover:bg-muted/40"
    >
      <!-- The whole card (minus the corner actions) is one link, same as the row. -->
      <a
        href="/projects/{project.id}"
        class="flex flex-1 flex-col gap-3 rounded-xl outline-none focus-visible:ring-2 focus-visible:ring-ring"
      >
        <div class="flex items-start gap-2.5 pr-8">
          <div class="flex h-9 w-9 shrink-0 items-center justify-center">
            {#if primary}
              <TechIcon
                icon={primary.icon}
                iconType={primary.icon_type}
                size="lg"
                alt={primary.name}
              />
            {:else}
              <FolderOpen class="h-5 w-5 text-muted-foreground" />
            {/if}
          </div>
          <div class="min-w-0">
            <div class="flex items-center gap-1.5">
              <span class="truncate text-sm font-medium">{project.name}</span>
              {#if project.starred}
                <Star
                  class="h-3.5 w-3.5 shrink-0 fill-yellow-400 text-yellow-400"
                  aria-label="Starred project"
                />
              {/if}
              {#if stageLabel}
                <Badge variant="secondary" class="shrink-0 font-normal">{stageLabel}</Badge>
              {/if}
              {#if project.archived}
                <Badge class={cn('shrink-0 font-normal', getStatusColor('archived'))}
                  >Archived</Badge
                >
              {/if}
              {#if isMoving}
                <Badge
                  variant="outline"
                  class="shrink-0 font-normal text-amber-600 dark:text-amber-400"
                >
                  Moving…
                </Badge>
              {/if}
            </div>
            <div
              class="flex items-center gap-1.5 text-xs text-muted-foreground"
              title="{activity}{lastSeen ? ` · last activity ${when}` : ''}"
            >
              <span class="h-1.5 w-1.5 shrink-0 rounded-full {DOT[activity] ?? DOT.stale}"></span>
              <span class="truncate">{when}</span>
            </div>
          </div>
        </div>

        {#if description}
          <p class="line-clamp-2 text-xs text-muted-foreground">{description}</p>
        {/if}

        <div class="flex flex-wrap items-center gap-1.5">
          {#if badges.length > 0}
            {#each badges as item (item.name)}
              <Badge variant="outline" class="gap-1.5 font-normal">
                <TechIcon icon={item.icon} iconType={item.icon_type} size="xs" alt={item.name} />
                {item.name}
              </Badge>
            {/each}
          {:else if modules.length > 0}
            <span class="truncate text-xs text-muted-foreground">{describeModules(modules)}</span>
          {/if}
        </div>

        {#if branch}
          <div class="mt-auto flex items-center gap-1.5 text-xs text-muted-foreground">
            <GitBranch class="h-3.5 w-3.5 shrink-0" />
            <span class="truncate">{branch}</span>
            {#if project.has_uncommitted_changes}
              <span
                class="h-2 w-2 shrink-0 rounded-full bg-amber-500"
                title="Uncommitted changes"
                aria-label="Uncommitted changes"
              ></span>
            {/if}
            {#if unpushed > 0}
              <span class="shrink-0" title="{unpushed} unpushed commits">↑{unpushed}</span>
            {/if}
          </div>
        {/if}
      </a>

      <div class="absolute top-2.5 right-2.5 z-10">
        <ProjectQuickActions
          {project}
          {eligibleParents}
          {onDelete}
          {onSetParent}
          {onToggleStar}
          {onSetStage}
          {onSetArchived}
        />
      </div>
    </Card>
  {/snippet}
</ContextMenuList>
