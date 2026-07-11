<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import Select from "$lib/components/ui/select.svelte";
  import { Plus, X, LayoutGrid } from "@lucide/svelte";
  import { formatCount } from "$lib/domains/shared/utils";
  import AISessionSidebar from "$lib/domains/ai/components/shared/AISessionSidebar.svelte";
  import ProjectWorkspaceList from "./ProjectWorkspaceList.svelte";
  import CoderSidebarFooter from "./CoderSidebarFooter.svelte";
  import { coderUi } from "../state/coderUi.svelte.js";
  import {
    DEFAULT_SESSION_FILTERS,
    SORT_OPTIONS,
    STATUS_OPTIONS,
    extractProviderOptions,
    filterAndSortSessions,
    hasActiveFilters,
    type SessionListFilters,
    type SessionSortKey,
    type SessionStatusFilter,
  } from "../utils/sessionList.js";
  import type { CoderThread } from "../types.js";

  interface Props {
    threads: CoderThread[];
    onThreadClick?: (thread: CoderThread) => void;
    onCreateNew?: () => void;
    onDeleteThread?: (thread: CoderThread) => void;
    onProjectSelect?: (path: string, projectId?: string) => void;
    selectedThreadId?: string | null;
    runningThreadIds?: Set<string>;
    queuedCountFor?: (threadId: string) => number;
    subAgentSummaryFor?: (threadId: string) => { running: number; total: number };
    loading?: boolean;
    showRules?: boolean;
    onToggleRules?: () => void;
  }

  let {
    threads,
    onThreadClick,
    onCreateNew,
    onDeleteThread,
    onProjectSelect,
    selectedThreadId = null,
    runningThreadIds = new Set<string>(),
    queuedCountFor,
    subAgentSummaryFor,
    loading = false,
    showRules = false,
    onToggleRules,
  }: Props = $props();

  let searchValue = $state("");
  let statusFilter = $state<SessionStatusFilter>(DEFAULT_SESSION_FILTERS.status);
  let sortFilter = $state<SessionSortKey>(DEFAULT_SESSION_FILTERS.sort);
  let providerFilter = $state(DEFAULT_SESSION_FILTERS.provider);

  const listFilters = $derived<SessionListFilters>({
    search: searchValue,
    project: DEFAULT_SESSION_FILTERS.project,
    projectId: DEFAULT_SESSION_FILTERS.projectId,
    status: statusFilter,
    provider: providerFilter,
    sort: sortFilter,
  });

  const displayThreads = $derived.by(() => {
    const byId = new Map<string, CoderThread>();
    for (const t of threads) {
      byId.set(t.id, t);
    }

    for (const id of runningThreadIds) {
      if (byId.has(id)) continue;
      byId.set(id, {
        id,
        title: "Running session",
        workspace_root: coderUi.activeProjectPath ?? "",
        project_id: coderUi.activeProjectId
          ? Number.parseInt(coderUi.activeProjectId, 10)
          : null,
        messages: [],
        created_at: "",
        updated_at: new Date().toISOString(),
      });
    }

    return [...byId.values()];
  });

  const providerOptions = $derived([
    { value: "all", label: "All providers" },
    ...extractProviderOptions(displayThreads),
  ]);

  const showProviderFilter = $derived(providerOptions.length > 2);

  const filteredCount = $derived(
    filterAndSortSessions(
      displayThreads,
      {
        ...listFilters,
        project: coderUi.showAllWorkspaces
          ? "all"
          : (coderUi.activeProjectPath ?? "all"),
        projectId: coderUi.showAllWorkspaces
          ? "all"
          : (coderUi.activeProjectId ?? "all"),
      },
      runningThreadIds,
      queuedCountFor,
    ).length,
  );

  const filtersActive = $derived(
    hasActiveFilters(listFilters) || coderUi.showAllWorkspaces,
  );

  const showLoadingSkeleton = $derived(loading && displayThreads.length === 0);

  function clearFilters() {
    searchValue = "";
    statusFilter = DEFAULT_SESSION_FILTERS.status;
    sortFilter = DEFAULT_SESSION_FILTERS.sort;
    providerFilter = DEFAULT_SESSION_FILTERS.provider;
    coderUi.showAllWorkspaces = false;
  }
</script>

<AISessionSidebar
  searchPlaceholder="Search sessions..."
  bind:searchValue={searchValue}
  {showLoadingSkeleton}
  internalScroll={false}
>
  {#snippet filters()}
    <div class="grid grid-cols-2 gap-2">
      <Select
        options={[...STATUS_OPTIONS]}
        value={statusFilter}
        onSelect={(v) => (statusFilter = v as SessionStatusFilter)}
        placeholder="All statuses"
        class="h-8 text-xs"
      />
      <Select
        options={[...SORT_OPTIONS]}
        value={sortFilter}
        onSelect={(v) => (sortFilter = v as SessionSortKey)}
        placeholder="Sort by"
        class="h-8 text-xs"
      />
    </div>

    {#if showProviderFilter}
      <Select
        options={providerOptions}
        value={providerFilter}
        onSelect={(v) => (providerFilter = v)}
        placeholder="All providers"
        class="h-8 text-xs"
      />
    {/if}
  {/snippet}

  {#snippet meta()}
    <div class="flex items-center justify-between gap-2">
      <p class="min-w-0 text-xs text-muted-foreground">
        {#if showLoadingSkeleton}
          Loading sessions…
        {:else}
          {formatCount(filteredCount, "session")}
        {/if}
      </p>
      <div class="flex shrink-0 items-center gap-1">
        <Button
          variant={coderUi.showAllWorkspaces ? "secondary" : "ghost"}
          size="sm"
          class="h-7 gap-1 px-2 text-xs"
          title={coderUi.showAllWorkspaces
            ? "Show active workspace only"
            : "Show all workspaces"}
          onclick={() => (coderUi.showAllWorkspaces = !coderUi.showAllWorkspaces)}
        >
          <LayoutGrid class="h-3 w-3" />
          All
        </Button>
        {#if filtersActive}
          <Button
            variant="ghost"
            size="sm"
            class="h-7 gap-1 px-2 text-xs text-muted-foreground"
            onclick={clearFilters}
            title="Clear filters"
          >
            <X class="h-3 w-3" />
          </Button>
        {/if}
        <Button onclick={onCreateNew} size="sm" class="h-7 gap-1 px-2" title="New session">
          <Plus class="h-3.5 w-3.5" />
          <span class="text-xs">New</span>
        </Button>
      </div>
    </div>
  {/snippet}

  {#snippet children()}
    <ProjectWorkspaceList
      threads={displayThreads}
      filters={listFilters}
      {selectedThreadId}
      {runningThreadIds}
      {loading}
      {onThreadClick}
      {onDeleteThread}
      {onProjectSelect}
      {queuedCountFor}
      {subAgentSummaryFor}
    />
  {/snippet}

  {#snippet footer()}
    <CoderSidebarFooter {showRules} {onToggleRules} />
  {/snippet}
</AISessionSidebar>
