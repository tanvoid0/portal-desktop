<script lang="ts">
  import { ChevronRight, Folder } from "@lucide/svelte";
  import { Button } from "$lib/components/ui/button";
  import CoderSessionCard from "./CoderSessionCard.svelte";
  import type { CoderThread } from "../types.js";
  import type { ProjectWorkspace } from "../utils/sessionList.js";

  interface Props {
    workspace: ProjectWorkspace;
    expanded?: boolean;
    active?: boolean;
    selectedThreadId?: string | null;
    runningThreadIds?: Set<string>;
    threads?: CoderThread[];
    onToggle?: () => void;
    onSelectProject?: () => void;
    onThreadClick?: (thread: CoderThread) => void;
    onDeleteThread?: (thread: CoderThread) => void;
    queuedCountFor?: (threadId: string) => number;
    subAgentSummaryFor?: (threadId: string) => { running: number; total: number };
  }

  let {
    workspace,
    expanded = false,
    active = false,
    selectedThreadId = null,
    runningThreadIds = new Set<string>(),
    threads = workspace.threads,
    onToggle,
    onSelectProject,
    onThreadClick,
    onDeleteThread,
    queuedCountFor,
    subAgentSummaryFor,
  }: Props = $props();

  const sessionCount = $derived(threads.length);
  const hasRunning = $derived(
    threads.some((t) => runningThreadIds.has(t.id)),
  );
</script>

<div class="rounded-md {active ? 'bg-muted/50' : ''}">
  <div
    class="flex w-full items-center gap-1.5 rounded-md px-2 py-1.5 text-left text-sm transition-colors hover:bg-muted/60"
  >
    <Button
      type="button"
      variant="ghost"
      size="icon-sm"
      class="h-5 w-5 shrink-0"
      title={expanded ? "Collapse" : "Expand"}
      onclick={() => onToggle?.()}
    >
      <ChevronRight
        class="h-3.5 w-3.5 text-muted-foreground transition-transform {expanded
          ? 'rotate-90'
          : ''}"
      />
    </Button>
    <Button
      type="button"
      variant="ghost"
      class="h-auto min-w-0 flex-1 justify-start gap-1.5 px-0 text-left text-sm"
      onclick={() => {
        onSelectProject?.();
        if (!expanded) onToggle?.();
      }}
    >
      <Folder
        class="h-3.5 w-3.5 shrink-0 {active ? 'text-primary' : 'text-muted-foreground'}"
      />
      <span
        class="min-w-0 flex-1 truncate font-medium {active
          ? 'text-foreground'
          : 'text-foreground/90'}"
        title={workspace.path}
      >
        {workspace.label}
      </span>
      {#if hasRunning}
        <span
          class="h-1.5 w-1.5 shrink-0 animate-pulse rounded-full bg-primary"
          title="Running session"
        ></span>
      {/if}
      {#if sessionCount > 0}
        <span class="shrink-0 text-[10px] tabular-nums text-muted-foreground">
          {sessionCount}
        </span>
      {/if}
    </Button>
  </div>

  {#if expanded && threads.length > 0}
    <div class="space-y-1 pb-1 pl-3 pr-1 pt-0.5">
      {#each threads as t (t.id)}
        <CoderSessionCard
          thread={t}
          compact
          hideProject
          onClick={() => onThreadClick?.(t)}
          onDelete={onDeleteThread}
          isActive={selectedThreadId === t.id}
          isRunning={runningThreadIds.has(t.id)}
          queuedCount={queuedCountFor?.(t.id) ?? 0}
          subAgentRunning={subAgentSummaryFor?.(t.id).running ?? 0}
        />
      {/each}
    </div>
  {:else if expanded && sessionCount === 0}
    <p class="px-3 pb-2 pl-8 text-[11px] text-muted-foreground">
      No sessions — start one below
    </p>
  {/if}
</div>
