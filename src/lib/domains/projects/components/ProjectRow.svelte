<!-- Compact list row: the whole row is a link; the trailing menu holds the actions. -->
<script lang="ts">
  import type { Project, ProjectModule, ProjectStage } from '$lib/domains/projects/types';
  import { PROJECT_STAGES } from '$lib/domains/projects/types';
  import { formatRelativeTime } from '$lib/domains/shared/utils';
  import { Badge } from '$lib/components/ui/badge';
  import TechIcon from '$lib/components/ui/tech-icon.svelte';
  import { cn } from '$lib/utils';
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
  import ProjectQuickActions from './ProjectQuickActions.svelte';
  import ContextMenuList from '$lib/components/menu/ContextMenuList.svelte';
  import { buildProjectMenu, copyProjectReference } from '$lib/domains/projects/utils/projectMenu';
  import { launchInCoder, launchInAgents } from '$lib/domains/projects/utils/launch';
  import { createProjectMovesQuery } from '$lib/domains/projects/queries/projectMoveQueries';
  import { findActiveMove } from '$lib/domains/projects/api/projectMoveApi';

  interface Props {
    project: Project;
    modules?: ProjectModule[];
    /** Rendered under a parent row. */
    nested?: boolean;
    /** Area path shown before the name when the row stands in for its own group. */
    pathPrefix?: string;
    /** Bump after the icon registry loads so stack badges re-resolve. */
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
    nested = false,
    pathPrefix = '',
    registryReady = false,
    eligibleParents = [],
    onDelete = () => {},
    onSetParent = () => {},
    onToggleStar = () => {},
    onSetStage = () => {},
    onSetArchived = () => {},
  }: Props = $props();

  const stageLabel = $derived(PROJECT_STAGES.find((s) => s.value === project.stage)?.label);

  // 2 full badges + a "+N" chip reads cleaner at row width than 3 badges that
  // get clipped mid-word by the row's overflow.
  const MAX_BADGES = 2;
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
  // Deduped by name: older rows can hold a "Rust" framework beside the Rust language,
  // and a duplicate key would throw in the keyed each below.
  const allBadges = $derived.by(() => {
    if (!registryReady) return [];
    const all = [
      ...projectIconRegistry.resolveFrameworks(project),
      ...projectIconRegistry.resolveLanguages(project),
    ];
    return all.filter((t, i) => all.findIndex((o) => o.name === t.name) === i);
  });
  const movesQuery = createProjectMovesQuery();
  const isMoving = $derived(!!findActiveMove(movesQuery.data, Number(project.id)));
  const badges = $derived(allBadges.slice(0, MAX_BADGES));
  const extraBadges = $derived(Math.max(0, allBadges.length - MAX_BADGES));
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
    onLaunchAgents: launchInAgents,
    onDelete,
  })}
>
  {#snippet trigger(menuProps)}
    <div
      {...menuProps}
      class={cn(
        'group flex items-center gap-1 rounded-md pr-1 focus-within:bg-muted/60 hover:bg-muted/60',
        nested && 'ml-8 border-l pl-2'
      )}
    >
      <a
        href="/projects/{project.id}"
        class="flex min-w-0 flex-1 items-center gap-3 rounded-md px-3 py-2 outline-none focus-visible:ring-2 focus-visible:ring-ring"
      >
        <div class="flex h-8 w-8 shrink-0 items-center justify-center">
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

        <div class="min-w-0 shrink">
          <div class="flex items-center gap-1.5">
            {#if pathPrefix}
              <span class="max-w-[14rem] truncate text-sm text-muted-foreground">{pathPrefix}/</span
              >
            {/if}
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
              <Badge class={cn('shrink-0 font-normal', getStatusColor('archived'))}>Archived</Badge>
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
          {#if description}
            <p class="truncate text-xs text-muted-foreground">{description}</p>
          {/if}
        </div>

        <!-- Badges sit beside the name rather than in a wide fixed column, so the row has
         no empty middle at 1280 and the name keeps its width at 1024. Capped at
         MAX_BADGES with a "+N" chip instead of clipping a badge mid-word. -->
        <div class="hidden shrink-0 items-center gap-1.5 md:flex">
          {#if badges.length > 0}
            {#each badges as item (item.name)}
              <Badge variant="outline" class="shrink-0 gap-1.5 font-normal">
                <TechIcon icon={item.icon} iconType={item.icon_type} size="xs" alt={item.name} />
                {item.name}
              </Badge>
            {/each}
            {#if extraBadges > 0}
              <Badge
                variant="outline"
                class="shrink-0 font-normal text-muted-foreground"
                title={allBadges
                  .slice(MAX_BADGES)
                  .map((b) => b.name)
                  .join(', ')}
              >
                +{extraBadges}
              </Badge>
            {/if}
          {:else if modules.length > 0}
            <span class="truncate text-xs text-muted-foreground">{describeModules(modules)}</span>
          {/if}
        </div>

        <div
          class="ml-auto flex w-24 shrink-0 items-center gap-1.5 text-xs text-muted-foreground"
          title="{activity}{lastSeen ? ` · last activity ${when}` : ''}"
        >
          <span class="h-2 w-2 shrink-0 rounded-full {DOT[activity] ?? DOT.stale}"></span>
          <span class="truncate">{when}</span>
        </div>

        <div
          class="hidden w-36 shrink-0 items-center gap-1.5 text-xs text-muted-foreground xl:flex"
        >
          {#if branch}
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
          {/if}
        </div>
      </a>

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
  {/snippet}
</ContextMenuList>
