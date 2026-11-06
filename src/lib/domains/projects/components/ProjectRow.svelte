<!-- Compact list row: the whole row is a link; the trailing menu holds the actions. -->
<script lang="ts">
  import type { Project, ProjectModule } from '$lib/domains/projects/types';
  import { formatRelativeTime } from '$lib/domains/shared/utils';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import * as Dialog from '$lib/components/ui/dialog';
  import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuSeparator,
    DropdownMenuTrigger,
  } from '$lib/components/ui/dropdown-menu';
  import Select from '$lib/components/ui/select.svelte';
  import TechIcon from '$lib/components/ui/tech-icon.svelte';
  import { cn } from '$lib/utils';
  import Ellipsis from '~icons/fa6-solid/ellipsis';
  import ExternalLink from '~icons/fa6-solid/arrow-up-right-from-square';
  import FolderOpen from '~icons/fa6-solid/folder-open';
  import FolderTree from '~icons/fa6-solid/folder-tree';
  import GitBranch from '~icons/fa6-solid/code-branch';
  import Star from '~icons/fa6-solid/star';
  import Trash2 from '~icons/fa6-solid/trash';
  import Unlink from '~icons/fa6-solid/link-slash';
  import { getProjectGitBranch, readableDescription } from '$lib/domains/projects/utils/display';
  import { describeModules } from '$lib/domains/projects/utils/moduleStacks';
  import { projectIconRegistry } from '$lib/domains/projects/utils/iconRegistry';
  import { goto } from '$app/navigation';

  interface Props {
    project: Project;
    modules?: ProjectModule[];
    /** Rendered under a parent row. */
    nested?: boolean;
    /** Bump after the icon registry loads so stack badges re-resolve. */
    registryReady?: boolean;
    eligibleParents?: Array<{ id: string; name: string }>;
    onDelete?: (project: Project) => void;
    onSetParent?: (projectId: string, parentId: number | null) => void;
    onToggleStar?: (project: Project) => void;
  }

  let {
    project,
    modules = [],
    nested = false,
    registryReady = false,
    eligibleParents = [],
    onDelete = () => {},
    onSetParent = () => {},
    onToggleStar = () => {},
  }: Props = $props();

  const MAX_BADGES = 3;
  const DOT: Record<string, string> = {
    active: 'bg-green-500',
    recent: 'bg-blue-500',
    dormant: 'bg-amber-500',
    stale: 'bg-muted-foreground/40',
  };

  let groupOpen = $state(false);

  const primary = $derived(
    registryReady
      ? (projectIconRegistry.resolvePrimaryFramework(project) ??
          projectIconRegistry.resolveLanguages(project)[0])
      : undefined
  );
  // Deduped by name: older rows can hold a "Rust" framework beside the Rust language,
  // and a duplicate key would throw in the keyed each below.
  const badges = $derived.by(() => {
    if (!registryReady) return [];
    const all = [
      ...projectIconRegistry.resolveFrameworks(project),
      ...projectIconRegistry.resolveLanguages(project),
    ];
    return all.filter((t, i) => all.findIndex((o) => o.name === t.name) === i).slice(0, MAX_BADGES);
  });
  const branch = $derived(getProjectGitBranch(project));
  const description = $derived(readableDescription(project.description));
  const lastSeen = $derived(project.last_activity ?? project.last_opened);
  const activity = $derived(project.activity_status ?? 'stale');
  const unpushed = $derived(project.unpushed_commits ?? 0);
  const when = $derived(lastSeen ? formatRelativeTime(new Date(lastSeen)) : 'never');
</script>

<div
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
        <TechIcon icon={primary.icon} iconType={primary.icon_type} size="lg" alt={primary.name} />
      {:else}
        <FolderOpen class="h-5 w-5 text-muted-foreground" />
      {/if}
    </div>

    <div class="min-w-0 flex-1">
      <div class="flex items-center gap-1.5">
        <span class="truncate text-sm font-medium">{project.name}</span>
        {#if project.starred}
          <Star
            class="h-3.5 w-3.5 shrink-0 fill-yellow-400 text-yellow-400"
            aria-label="Starred project"
          />
        {/if}
      </div>
      {#if description}
        <p class="truncate text-xs text-muted-foreground">{description}</p>
      {/if}
    </div>

    <div class="hidden w-72 shrink-0 items-center gap-1.5 overflow-hidden md:flex">
      <!-- The project's own stack first; the module summary only when none is recorded. -->
      {#if badges.length > 0}
        {#each badges as item (item.name)}
          <Badge variant="outline" class="shrink-0 gap-1.5 font-normal">
            <TechIcon icon={item.icon} iconType={item.icon_type} size="xs" alt={item.name} />
            {item.name}
          </Badge>
        {/each}
      {:else if modules.length > 0}
        <span class="truncate text-xs text-muted-foreground">{describeModules(modules)}</span>
      {/if}
    </div>

    <div
      class="flex w-24 shrink-0 items-center gap-1.5 text-xs text-muted-foreground"
      title="{activity}{lastSeen ? ` · last activity ${when}` : ''}"
    >
      <span class="h-2 w-2 shrink-0 rounded-full {DOT[activity] ?? DOT.stale}"></span>
      <span class="truncate">{when}</span>
    </div>

    <div class="hidden w-40 shrink-0 items-center gap-1.5 text-xs text-muted-foreground lg:flex">
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

  <DropdownMenu>
    <DropdownMenuTrigger>
      {#snippet child({ props })}
        <Button
          {...props}
          variant="ghost"
          size="icon"
          class="h-8 w-8 shrink-0"
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
