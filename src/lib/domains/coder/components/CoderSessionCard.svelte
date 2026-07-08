<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import { FolderOpen, Clock, MessageSquare, Trash2 } from "@lucide/svelte";
  import {
    formatCount,
    formatSessionDateTime,
    formatSessionDateTimeFull,
  } from "$lib/domains/shared/utils";
  import type { CoderThread } from "../types.js";

  interface Props {
    thread: CoderThread;
    onClick?: () => void;
    onDelete?: (thread: CoderThread) => void;
    isActive?: boolean;
    isRunning?: boolean;
    queuedCount?: number;
  }

  let {
    thread,
    onClick,
    onDelete,
    isActive = false,
    isRunning = false,
    queuedCount = 0,
  }: Props = $props();

  function handleDelete(e: MouseEvent) {
    e.stopPropagation();
    onDelete?.(thread);
  }

  const messageCount = $derived(
    thread.message_count ??
      thread.messages.filter((m) => m.role === "user" || m.role === "assistant")
        .length,
  );

  const workspaceName = $derived(
    (thread.workspace_root ?? "")
      .split(/[/\\]/)
      .filter(Boolean)
      .pop() ?? thread.workspace_root ?? "No project",
  );

  const updatedLabel = $derived(formatSessionDateTime(thread.updated_at));
  const updatedTitle = $derived(formatSessionDateTimeFull(thread.updated_at));
</script>

<button
  type="button"
  class="group relative w-full rounded-lg border bg-background px-3 py-2.5 text-left shadow-sm transition-colors
    {isActive
    ? 'border-primary/40 bg-primary/5 ring-1 ring-primary/20'
    : 'border-border/60 hover:border-border hover:bg-muted/40'}"
  onclick={onClick}
>
  {#if isActive}
    <span
      class="absolute bottom-2 left-0 top-2 w-0.5 rounded-full bg-primary"
      aria-hidden="true"
    ></span>
  {/if}

  <div class="flex items-start gap-2">
    <div class="min-w-0 flex-1 space-y-1.5">
      <div class="flex items-start gap-1.5">
        {#if isRunning}
          <span
            class="mt-1.5 h-2 w-2 shrink-0 animate-pulse rounded-full bg-primary"
            title="Running"
          ></span>
        {/if}
        <p
          class="line-clamp-2 min-w-0 flex-1 text-[13px] font-medium leading-snug text-foreground"
          title={thread.title}
        >
          {thread.title}
        </p>
        {#if thread.thread_kind === "coordinator"}
          <Badge variant="secondary" class="shrink-0 text-[10px]">Coordinator</Badge>
        {/if}
      </div>

      {#if workspaceName && workspaceName !== "No project"}
        <div
          class="flex items-center gap-1 text-xs text-muted-foreground"
          title={thread.workspace_root}
        >
          <FolderOpen class="h-3.5 w-3.5 shrink-0 opacity-70" />
          <span class="truncate">{workspaceName}</span>
        </div>
      {/if}

      <div class="flex flex-wrap items-center gap-x-2.5 gap-y-1 text-xs text-muted-foreground">
        {#if updatedLabel}
          <span class="inline-flex items-center gap-1" title={updatedTitle}>
            <Clock class="h-3.5 w-3.5 shrink-0 opacity-70" />
            <span class="tabular-nums">{updatedLabel}</span>
          </span>
        {/if}
        <span class="inline-flex items-center gap-1">
          <MessageSquare class="h-3.5 w-3.5 shrink-0 opacity-70" />
          <span>{formatCount(messageCount, "message")}</span>
        </span>
      </div>
    </div>

    <div class="flex shrink-0 flex-col items-end gap-1">
      {#if queuedCount > 0}
        <Badge
          variant="secondary"
          class="h-5 px-1.5 text-[10px] font-normal tabular-nums"
          title="{formatCount(queuedCount, 'message')} queued"
        >
          {queuedCount} queued
        </Badge>
      {/if}
      {#if isRunning}
        <Badge variant="outline" class="h-5 border-primary/30 px-1.5 text-[10px] text-primary">
          Active
        </Badge>
      {/if}
      {#if onDelete}
        <Button
          variant="ghost"
          size="icon"
          class="h-6 w-6 text-muted-foreground opacity-0 transition-opacity hover:text-destructive group-hover:opacity-100 {isActive
            ? 'opacity-100'
            : ''}"
          onclick={handleDelete}
          title="Delete session"
        >
          <Trash2 class="h-3.5 w-3.5" />
        </Button>
      {/if}
    </div>
  </div>
</button>
