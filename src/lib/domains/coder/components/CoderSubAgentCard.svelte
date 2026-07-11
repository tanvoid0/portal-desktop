<script lang="ts">
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import { ChevronRight, FolderGit2, Square, Trash2 } from "@lucide/svelte";
  import type { CoderSubAgent } from "../types.js";

  interface Props {
    subAgent: CoderSubAgent;
    onOpen?: (childThreadId: string, coordinatorId: string) => void;
    onCancel?: (subAgentId: string) => void;
    onCleanup?: (subAgentId: string) => void;
    coordinatorId?: string;
  }

  let {
    subAgent,
    onOpen,
    onCancel,
    onCleanup,
    coordinatorId = "",
  }: Props = $props();

  const tone = $derived.by(() => {
    switch (subAgent.status) {
      case "running":
        return "bg-primary/15 text-primary";
      case "completed":
        return "bg-emerald-500/15 text-emerald-600";
      case "failed":
        return "bg-destructive/10 text-destructive";
      case "cancelled":
        return "bg-muted text-muted-foreground";
      default:
        return "bg-muted text-muted-foreground";
    }
  });

  function openSession() {
    if (!onOpen) return;
    onOpen(
      subAgent.child_thread_id,
      coordinatorId || subAgent.coordinator_thread_id,
    );
  }
</script>

<div
  class="rounded-lg border border-border/60 bg-background px-3 py-2.5 {onOpen
    ? 'transition-colors hover:border-primary/30 hover:bg-muted/30'
    : ''}"
>
  <div class="flex items-start gap-3">
    <button
      type="button"
      class="min-w-0 flex-1 text-left {onOpen ? 'cursor-pointer' : 'cursor-default'}"
      disabled={!onOpen}
      onclick={openSession}
    >
      <div class="flex items-center gap-2">
        <p class="truncate text-sm font-medium">{subAgent.title}</p>
        <span class={`rounded-full px-2 py-0.5 text-[10px] font-medium ${tone}`}>
          {subAgent.status}
        </span>
      </div>
      <div class="mt-1 flex items-center gap-1 text-xs text-muted-foreground">
        <FolderGit2 class="h-3 w-3" />
        <span class="truncate">{subAgent.branch}</span>
      </div>
      {#if subAgent.result_summary}
        <p class="mt-1 line-clamp-2 text-xs text-muted-foreground">
          {subAgent.result_summary}
        </p>
      {:else if subAgent.error}
        <p class="mt-1 line-clamp-2 text-xs text-destructive">
          {subAgent.error}
        </p>
      {:else if onOpen}
        <p class="mt-1 text-xs text-muted-foreground">Open session</p>
      {/if}
    </button>
    <div class="flex shrink-0 items-center gap-1">
      {#if onOpen}
        <Button
          size="icon"
          variant="ghost"
          class="h-7 w-7"
          title="Open sub-agent session"
          onclick={openSession}
        >
          <ChevronRight class="h-3.5 w-3.5" />
        </Button>
      {/if}
      {#if subAgent.status === "running" || subAgent.status === "pending"}
        <Button
          size="icon"
          variant="ghost"
          class="h-7 w-7"
          title="Cancel sub-agent"
          onclick={() => onCancel?.(subAgent.id)}
        >
          <Square class="h-3.5 w-3.5" />
        </Button>
      {:else}
        <Button
          size="icon"
          variant="ghost"
          class="h-7 w-7"
          title="Clean up worktree"
          onclick={() => onCleanup?.(subAgent.id)}
        >
          <Trash2 class="h-3.5 w-3.5" />
        </Button>
      {/if}
    </div>
  </div>
</div>
