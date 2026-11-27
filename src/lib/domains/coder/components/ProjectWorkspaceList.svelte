<script lang="ts">
  import { onMount, tick, untrack } from 'svelte';
  import { goto } from '$app/navigation';
  import { open } from '@tauri-apps/plugin-dialog';
  import { ScrollArea } from '$lib/components/ui/scroll-area/index.js';
  import { Button } from '$lib/components/ui/button';
  import { Skeleton } from '$lib/components/ui/skeleton/index.js';
  import FolderPlus from '~icons/fa6-solid/folder-plus';
  import Loader2 from '~icons/fa6-solid/spinner';
  import { projectService } from '$lib/domains/projects';
  import type { Project } from '$lib/domains/projects';
  import { toggleProjectStar } from '$lib/domains/projects/api/projectApi';
  import { queryClient } from '$lib/domains/shared/query/queryClient';
  import { queryKeys } from '$lib/domains/shared/query/keys';
  import { copyText } from '$lib/domains/shared/utils/clipboard';
  import { toast } from '$lib/utils/toast';
  import { buildWorkspaceMenu } from '../utils/threadMenu';
  import ProjectWorkspaceSection from './ProjectWorkspaceSection.svelte';
  import { coderUi, type CoderUiState } from '../state/coderUi.svelte.js';
  import {
    groupSessionsByProject,
    sortThreadsForWorkspace,
    filterAndSortSessions,
    arrangeWorkspaces,
    workspaceTier,
    movePath,
    UNKNOWN_FOLDER,
    type ProjectWorkspace,
    type SessionListFilters,
    type SessionSortKey,
  } from '../utils/sessionList.js';
  import type { SessionCardBadge } from '$lib/domains/ai/components/shared/AISessionCard.svelte';
  import type { CoderThread } from '../types.js';

  interface Props {
    threads: CoderThread[];
    filters: SessionListFilters;
    selectedThreadId?: string | null;
    runningThreadIds?: Set<string>;
    loading?: boolean;
    onThreadClick?: (thread: CoderThread) => void;
    onDeleteThread?: (thread: CoderThread) => void;
    /** Delete a workspace's whole history; `kept` counts the running sessions left out. */
    onDeleteThreads?: (threads: CoderThread[], label: string, kept: number) => void;
    onProjectSelect?: (path: string, projectId?: string) => void;
    queuedCountFor?: (threadId: string) => number;
    subAgentSummaryFor?: (threadId: string) => { running: number; total: number };
    badgeFor?: (threadId: string) => SessionCardBadge[];
    ui?: CoderUiState;
    onLaunchWorkspace?: ((thread: CoderThread) => void) | null;
  }

  let {
    threads,
    filters,
    selectedThreadId = null,
    runningThreadIds = new Set<string>(),
    loading = false,
    onThreadClick,
    onDeleteThread,
    onDeleteThreads,
    onProjectSelect,
    queuedCountFor,
    subAgentSummaryFor,
    badgeFor,
    ui = coderUi,
    onLaunchWorkspace,
  }: Props = $props();

  let projects = $state<Project[]>([]);
  let projectsLoading = $state(true);
  let importPath = $state<string | null>(null);
  let importing = $state(false);
  let importError = $state<string | null>(null);

  onMount(async () => {
    ui.initFromStorage();
    await loadProjects();
    ui.resolveProjectFromList(projects);
    if (!ui.activeProjectPath && projects.length > 0) {
      const first = projects[0];
      selectProject(first);
    } else if (ui.activeProjectPath) {
      const match = projects.find((p) => p.path === ui.activeProjectPath);
      onProjectSelect?.(ui.activeProjectPath, match?.id);
    }
  });

  async function loadProjects() {
    projectsLoading = true;
    try {
      projects = await projectService.loadProjects();
    } catch {
      projects = [];
    } finally {
      projectsLoading = false;
    }
  }

  function basename(p: string): string {
    const parts = p.split(/[/\\]/).filter(Boolean);
    return parts[parts.length - 1] ?? p;
  }

  async function browseFolder() {
    const dir = await open({ directory: true, multiple: false });
    if (typeof dir !== 'string') return;
    const match = projects.find((p) => p.path === dir);
    if (match) {
      selectProject(match);
      importPath = null;
    } else {
      importPath = dir;
      importError = null;
    }
  }

  async function confirmImport() {
    if (!importPath) return;
    importing = true;
    importError = null;
    try {
      const project = await projectService.createProject({
        name: basename(importPath),
        path: importPath,
      });
      projects = [...projects, project];
      selectProject(project);
      importPath = null;
    } catch (e) {
      importError = String(e);
    } finally {
      importing = false;
    }
  }

  function cancelImport() {
    importPath = null;
    importError = null;
  }

  function selectProjectByPath(path: string, projectId?: string) {
    ui.setActiveProject(path, projectId);
    onProjectSelect?.(path, projectId);
  }

  function selectProject(project: Project) {
    selectProjectByPath(project.path, project.id);
  }

  const isStarred = (w: ProjectWorkspace) =>
    w.projectId
      ? (projects.find((p) => p.id === w.projectId)?.starred ?? false)
      : ui.starred.includes(w.path);

  // A hand-set order only applies to the default sort; picking another sort overrides it.
  const manualSort = $derived(filters.sort === 'updated_desc');
  const workspaces = $derived(
    arrangeWorkspaces(
      groupSessionsByProject(threads, projects, filters.sort as SessionSortKey, runningThreadIds),
      isStarred,
      manualSort ? ui.order : []
    )
  );
  const searching = $derived(filters.search.trim() !== '');
  const hasMatches = (w: ProjectWorkspace) =>
    !searching || threadsForWorkspace(w.path, w.threads).length > 0;
  const anyVisible = $derived(workspaces.some(hasMatches));

  async function toggleStar(w: ProjectWorkspace) {
    if (!w.projectId) return ui.toggleStar(w.path);
    try {
      await toggleProjectStar(Number(w.projectId));
      projects = projects.map((p) => (p.id === w.projectId ? { ...p, starred: !p.starred } : p));
      void queryClient.invalidateQueries({ queryKey: queryKeys.projects.all });
    } catch {
      toast.error('Could not update the star');
    }
  }

  function deleteHistoryFor(w: ProjectWorkspace): (() => void) | undefined {
    const idle = w.threads.filter((t) => !runningThreadIds.has(t.id));
    if (!onDeleteThreads || idle.length === 0) return undefined;
    return () => onDeleteThreads(idle, w.label, w.threads.length - idle.length);
  }

  function menuFor(w: ProjectWorkspace, expanded: boolean) {
    const at = workspaces.indexOf(w);
    const tier = workspaceTier(w, isStarred(w));
    const sameTier = (other: ProjectWorkspace | undefined) =>
      !!other && workspaceTier(other, isStarred(other)) === tier;
    const canMove = manualSort && !searching;
    const moveTo = (to: number) => () => {
      const paths = workspaces.map((x) => x.path);
      ui.setOrder(movePath(paths, at, to));
    };
    const up = canMove && sameTier(workspaces[at - 1]);
    const real = w.path !== UNKNOWN_FOLDER;
    return buildWorkspaceMenu({
      starred: isStarred(w),
      expanded,
      onToggleStar: () => void toggleStar(w),
      onToggle: () => ui.toggleExpanded(w.path),
      onMoveTop: up ? moveTo(workspaces.findIndex(sameTier)) : undefined,
      onMoveUp: up ? moveTo(at - 1) : undefined,
      onMoveDown: canMove && sameTier(workspaces[at + 1]) ? moveTo(at + 1) : undefined,
      onResetOrder: canMove && ui.order.length ? () => ui.setOrder([]) : undefined,
      onOpenProject: w.projectId ? () => void goto(`/projects/${w.projectId}`) : undefined,
      onAddProject:
        !w.projectId && real
          ? () => {
              importPath = w.path;
              importError = null;
            }
          : undefined,
      onCopyPath: real ? () => copyText(w.path, 'Path copied') : undefined,
      onDeleteHistory: deleteHistoryFor(w),
    });
  }

  function threadsForWorkspace(path: string, raw: CoderThread[]): CoderThread[] {
    const scopedFilters = {
      ...filters,
      project: 'all' as const,
    };
    let scoped = filterAndSortSessions(raw, scopedFilters, runningThreadIds, queuedCountFor);
    // Stable sort: starred sessions float up, the chosen order holds within each half.
    return sortThreadsForWorkspace(scoped, filters.sort as SessionSortKey, runningThreadIds).sort(
      (a, b) => Number(ui.starred.includes(b.id)) - Number(ui.starred.includes(a.id))
    );
  }

  // Follow the open conversation once per selection: expand its workspace and scroll its row
  // into view. Repeating this on every list refresh dragged the list back while scrolling.
  let followedId: string | null = null;
  $effect(() => {
    const id = selectedThreadId;
    if (!id) {
      followedId = null;
      return;
    }
    if (id === followedId || projectsLoading) return;
    const ws = workspaces.find((w) => w.threads.some((t) => t.id === id));
    if (!ws) return;
    followedId = id;
    untrack(() => {
      if (!ui.isExpanded(ws.path)) ui.toggleExpanded(ws.path);
    });
    void tick().then(() => {
      const el = listEl?.querySelector(`[data-session-id="${CSS.escape(id)}"]`);
      el?.scrollIntoView({ block: 'nearest' });
    });
  });

  let listEl = $state<HTMLElement | null>(null);

  const showLoading = $derived(loading && threads.length === 0 && projectsLoading);
</script>

<div class="flex min-h-0 flex-1 flex-col">
  <div class="divider-edge-b divider-edge-full flex items-center justify-between gap-2 px-3 py-2">
    <span class="text-xs font-semibold tracking-wide text-muted-foreground uppercase">
      Workspaces
    </span>
    <Button
      size="icon"
      variant="ghost"
      class="h-6 w-6"
      title="Add workspace folder"
      onclick={browseFolder}
      disabled={projectsLoading}
    >
      <FolderPlus class="h-3.5 w-3.5" />
    </Button>
  </div>

  {#if importPath}
    <div
      class="mx-2 mt-2 flex flex-col gap-1.5 rounded border border-border bg-muted/40 px-2 py-1.5 text-xs"
    >
      <span class="truncate font-mono" title={importPath}>{importPath}</span>
      <div class="flex gap-1">
        <Button size="sm" class="h-6 text-xs" onclick={confirmImport} disabled={importing}>
          {importing ? 'Importing…' : 'Import project'}
        </Button>
        <Button
          size="sm"
          variant="ghost"
          class="h-6 text-xs"
          onclick={cancelImport}
          disabled={importing}
        >
          Cancel
        </Button>
      </div>
      {#if importError}
        <span class="text-destructive">{importError}</span>
      {/if}
    </div>
  {/if}

  <ScrollArea class="min-h-0 flex-1">
    <div class="space-y-0.5 p-2" bind:this={listEl}>
      {#if showLoading}
        <div class="flex items-center justify-center gap-2 py-6 text-muted-foreground">
          <Loader2 class="h-4 w-4 animate-spin" />
          <span class="text-xs">Loading…</span>
        </div>
        {#each Array(3) as _, i (i)}
          <Skeleton class="h-8 w-full" />
        {/each}
      {:else if projectsLoading}
        {#each Array(3) as _, i (i)}
          <Skeleton class="h-8 w-full" />
        {/each}
      {:else if workspaces.length === 0}
        <p class="px-2 py-4 text-center text-xs text-muted-foreground">
          No workspaces yet. Add a project folder to get started.
        </p>
      {:else}
        {#if searching && !anyVisible}
          <p class="px-2 py-4 text-center text-xs text-muted-foreground">No matching sessions</p>
        {/if}
        {#each workspaces as workspace (workspace.path)}
          {@render workspaceRow(workspace)}
        {/each}
      {/if}
    </div>
  </ScrollArea>
</div>

{#snippet workspaceRow(workspace: ProjectWorkspace)}
  {@const isActive = workspace.path === ui.activeProjectPath}
  {@const expanded = searching || ui.isExpanded(workspace.path)}
  {@const wsThreads = expanded ? threadsForWorkspace(workspace.path, workspace.threads) : []}
  {#if !searching || wsThreads.length > 0}
    <ProjectWorkspaceSection
      {workspace}
      threads={wsThreads}
      {expanded}
      active={isActive}
      {selectedThreadId}
      {runningThreadIds}
      onToggle={() => ui.toggleExpanded(workspace.path)}
      onSelectProject={() => {
        const p = projects.find((proj) => proj.path === workspace.path);
        if (p) selectProject(p);
        else selectProjectByPath(workspace.path, workspace.projectId);
      }}
      {onThreadClick}
      {onDeleteThread}
      {queuedCountFor}
      {subAgentSummaryFor}
      {badgeFor}
      {onLaunchWorkspace}
      menuItems={menuFor(workspace, expanded)}
      starred={isStarred(workspace)}
      onDeleteHistory={deleteHistoryFor(workspace)}
      starredThreadIds={ui.starred}
      onToggleThreadStar={(t) => ui.toggleStar(t.id)}
    />
  {/if}
{/snippet}
