<script lang="ts">
  import { onMount } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
  import { Button } from "$lib/components/ui/button";
  import { Skeleton } from "$lib/components/ui/skeleton/index.js";
  import { FolderPlus, Loader2 } from "@lucide/svelte";
  import { projectService } from "$lib/domains/projects";
  import type { Project } from "$lib/domains/projects";
  import ProjectWorkspaceSection from "./ProjectWorkspaceSection.svelte";
  import { coderUi } from "../state/coderUi.svelte.js";
  import {
    groupSessionsByProject,
    sortThreadsForWorkspace,
    filterAndSortSessions,
    type SessionListFilters,
    type SessionSortKey,
  } from "../utils/sessionList.js";
  import type { CoderThread } from "../types.js";

  interface Props {
    threads: CoderThread[];
    filters: SessionListFilters;
    selectedThreadId?: string | null;
    runningThreadIds?: Set<string>;
    loading?: boolean;
    onThreadClick?: (thread: CoderThread) => void;
    onDeleteThread?: (thread: CoderThread) => void;
    onProjectSelect?: (path: string, projectId?: string) => void;
    queuedCountFor?: (threadId: string) => number;
    subAgentSummaryFor?: (threadId: string) => { running: number; total: number };
  }

  let {
    threads,
    filters,
    selectedThreadId = null,
    runningThreadIds = new Set<string>(),
    loading = false,
    onThreadClick,
    onDeleteThread,
    onProjectSelect,
    queuedCountFor,
    subAgentSummaryFor,
  }: Props = $props();

  let projects = $state<Project[]>([]);
  let projectsLoading = $state(true);
  let importPath = $state<string | null>(null);
  let importing = $state(false);
  let importError = $state<string | null>(null);

  onMount(async () => {
    coderUi.initFromStorage();
    await loadProjects();
    coderUi.resolveProjectFromList(projects);
    if (!coderUi.activeProjectPath && projects.length > 0) {
      const first = projects[0];
      selectProject(first);
    } else if (coderUi.activeProjectPath) {
      const match = projects.find((p) => p.path === coderUi.activeProjectPath);
      onProjectSelect?.(coderUi.activeProjectPath, match?.id);
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
    if (typeof dir !== "string") return;
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
    coderUi.setActiveProject(path, projectId);
    onProjectSelect?.(path, projectId);
  }

  function selectProject(project: Project) {
    selectProjectByPath(project.path, project.id);
  }

  const workspaces = $derived(groupSessionsByProject(threads, projects));

  function threadsForWorkspace(path: string, raw: CoderThread[]): CoderThread[] {
    const scopedFilters = {
      ...filters,
      project: "all" as const,
    };
    let scoped = filterAndSortSessions(
      raw,
      scopedFilters,
      runningThreadIds,
      queuedCountFor,
    );
    return sortThreadsForWorkspace(
      scoped,
      filters.sort as SessionSortKey,
      runningThreadIds,
    );
  }

  const showLoading = $derived(loading && threads.length === 0 && projectsLoading);
</script>

<div class="flex min-h-0 flex-1 flex-col">
  <div
    class="flex items-center justify-between gap-2 border-b px-3 py-2"
  >
    <span class="text-[11px] font-semibold uppercase tracking-wide text-muted-foreground">
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
          {importing ? "Importing…" : "Import project"}
        </Button>
        <Button size="sm" variant="ghost" class="h-6 text-xs" onclick={cancelImport} disabled={importing}>
          Cancel
        </Button>
      </div>
      {#if importError}
        <span class="text-destructive">{importError}</span>
      {/if}
    </div>
  {/if}

  <ScrollArea class="min-h-0 flex-1">
    <div class="space-y-0.5 p-2">
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
        {#each workspaces as workspace (workspace.path)}
          {@const isActive = workspace.path === coderUi.activeProjectPath}
          {@const expanded =
            coderUi.showAllWorkspaces
              ? coderUi.isExpanded(workspace.path)
              : isActive}
          {@const wsThreads = expanded
            ? threadsForWorkspace(workspace.path, workspace.threads)
            : []}
          <ProjectWorkspaceSection
            {workspace}
            threads={wsThreads}
            {expanded}
            active={isActive}
            {selectedThreadId}
            {runningThreadIds}
            onToggle={() => coderUi.toggleExpanded(workspace.path)}
            onSelectProject={() => {
              const p = projects.find((proj) => proj.path === workspace.path);
              if (p) selectProject(p);
              else selectProjectByPath(workspace.path, workspace.projectId);
            }}
            {onThreadClick}
            {onDeleteThread}
            {queuedCountFor}
            {subAgentSummaryFor}
          />
        {/each}
      {/if}
    </div>
  </ScrollArea>
</div>
