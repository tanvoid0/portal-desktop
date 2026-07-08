<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
  import Select from "$lib/components/ui/select.svelte";
  import CoderSessionCard from "./CoderSessionCard.svelte";
  import { Skeleton } from "$lib/components/ui/skeleton/index.js";
  import { Plus, Bot, Search, X, Loader2 } from "@lucide/svelte";
  import { formatCount } from "$lib/domains/shared/utils";
  import {
    DEFAULT_SESSION_FILTERS,
    SORT_OPTIONS,
    STATUS_OPTIONS,
    extractProjectOptions,
    extractProviderOptions,
    filterAndSortSessions,
    hasActiveFilters,
    type SessionListFilters,
  } from "../utils/sessionList.js";
  import type { CoderThread } from "../types.js";

  interface Props {
    threads: CoderThread[];
    onThreadClick?: (thread: CoderThread) => void;
    onCreateNew?: () => void;
    onDeleteThread?: (thread: CoderThread) => void;
    selectedThreadId?: string | null;
    runningThreadIds?: Set<string>;
    queuedCountFor?: (threadId: string) => number;
    loading?: boolean;
  }

  let {
    threads,
    onThreadClick,
    onCreateNew,
    onDeleteThread,
    selectedThreadId = null,
    runningThreadIds = new Set<string>(),
    queuedCountFor,
    loading = false,
  }: Props = $props();

  let filters = $state<SessionListFilters>({ ...DEFAULT_SESSION_FILTERS });

  /** Merge persisted threads with any in-flight runs not yet in the list. */
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
        workspace_root: "",
        messages: [],
        created_at: "",
        updated_at: new Date().toISOString(),
      });
    }

    return [...byId.values()];
  });

  const projectOptions = $derived([
    { value: "all", label: "All projects" },
    ...extractProjectOptions(displayThreads).map((p) => ({
      value: p.value,
      label: p.label,
    })),
  ]);

  const providerOptions = $derived([
    { value: "all", label: "All providers" },
    ...extractProviderOptions(displayThreads),
  ]);

  const showProviderFilter = $derived(providerOptions.length > 2);

  const filteredThreads = $derived(
    filterAndSortSessions(
      displayThreads,
      filters,
      runningThreadIds,
      queuedCountFor,
    ),
  );

  const filtersActive = $derived(hasActiveFilters(filters));

  const showLoadingSkeleton = $derived(loading && displayThreads.length === 0);

  function clearFilters() {
    filters = { ...DEFAULT_SESSION_FILTERS };
  }
</script>

<div class="flex h-full flex-col">
  <div
    class="space-y-2 border-b px-3 py-2.5 transition-opacity {showLoadingSkeleton
      ? 'pointer-events-none opacity-50'
      : ''}"
  >
    <div class="relative">
      <Search
        class="pointer-events-none absolute left-2.5 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-muted-foreground"
      />
      <Input
        placeholder="Search sessions..."
        bind:value={filters.search}
        class="h-8 pl-8 text-sm"
      />
    </div>

    <Select
      options={projectOptions}
      bind:value={filters.project}
      placeholder="All projects"
      class="h-8 text-xs"
    />

    <div class="grid grid-cols-2 gap-2">
      <Select
        options={[...STATUS_OPTIONS]}
        bind:value={filters.status}
        placeholder="All statuses"
        class="h-8 text-xs"
      />
      <Select
        options={[...SORT_OPTIONS]}
        bind:value={filters.sort}
        placeholder="Sort by"
        class="h-8 text-xs"
      />
    </div>

    {#if showProviderFilter}
      <Select
        options={providerOptions}
        bind:value={filters.provider}
        placeholder="All providers"
        class="h-8 text-xs"
      />
    {/if}

    <div class="flex items-center justify-between gap-2">
      <p class="min-w-0 text-xs text-muted-foreground">
        {#if showLoadingSkeleton}
          Loading sessions…
        {:else}
          {formatCount(filteredThreads.length, "session")}
          {#if filtersActive && filteredThreads.length !== displayThreads.length}
            <span> of {formatCount(displayThreads.length, "session")}</span>
          {/if}
        {/if}
      </p>
      <div class="flex shrink-0 items-center gap-1">
        {#if filtersActive}
          <Button
            variant="ghost"
            size="sm"
            class="h-7 gap-1 px-2 text-xs text-muted-foreground"
            onclick={clearFilters}
            title="Clear filters"
          >
            <X class="h-3 w-3" />
            Clear
          </Button>
        {/if}
        <Button onclick={onCreateNew} size="sm" class="h-7 gap-1 px-2" title="New session">
          <Plus class="h-3.5 w-3.5" />
          <span class="text-xs">New</span>
        </Button>
      </div>
    </div>
  </div>

  <ScrollArea class="flex-1">
    <div class="flex flex-col gap-2 p-2.5">
      {#if showLoadingSkeleton}
        <div class="flex items-center justify-center gap-2 py-4 text-muted-foreground">
          <Loader2 class="h-4 w-4 animate-spin" />
          <p class="text-sm">Loading sessions…</p>
        </div>
        {#each Array(4) as _, i (i)}
          <div
            class="rounded-lg border border-border/60 bg-background px-3 py-2.5 shadow-sm"
            aria-hidden="true"
          >
            <div class="space-y-2">
              <Skeleton class="h-4 w-3/4" />
              <Skeleton class="h-3 w-1/2" />
              <div class="flex gap-3">
                <Skeleton class="h-3 w-16" />
                <Skeleton class="h-3 w-20" />
              </div>
            </div>
          </div>
        {/each}
      {:else if displayThreads.length === 0}
        <div class="py-8 text-center text-muted-foreground">
          <Bot class="mx-auto mb-2 h-8 w-8 opacity-40" />
          <p class="text-sm font-medium">No sessions yet</p>
          <p class="mt-1 text-xs">Start a new session to begin coding</p>
        </div>
      {:else if filteredThreads.length === 0}
        <div class="py-8 text-center text-muted-foreground">
          <Bot class="mx-auto mb-2 h-8 w-8 opacity-40" />
          <p class="text-sm font-medium">No matching sessions</p>
          <p class="mt-1 text-xs">Try adjusting your search or filters</p>
          <Button
            variant="outline"
            size="sm"
            class="mt-3 h-7 text-xs"
            onclick={clearFilters}
          >
            Clear filters
          </Button>
        </div>
      {:else}
        {#each filteredThreads as t (t.id)}
          <CoderSessionCard
            thread={t}
            onClick={() => onThreadClick?.(t)}
            onDelete={onDeleteThread}
            isActive={selectedThreadId === t.id}
            isRunning={runningThreadIds.has(t.id)}
            queuedCount={queuedCountFor?.(t.id) ?? 0}
          />
        {/each}
      {/if}
    </div>
  </ScrollArea>
</div>
