<script lang="ts">
  import { Card, CardContent } from "$lib/components/ui/card";
  import { Button } from "$lib/components/ui/button";
  import { FolderOpen, Clock, Trash2 } from "@lucide/svelte";
  import type { CoderThread } from "../types.js";

  interface Props {
    thread: CoderThread;
    onClick?: () => void;
    onDelete?: (thread: CoderThread) => void;
    isActive?: boolean;
    isRunning?: boolean;
    queuedCount?: number;
  }

  let { thread, onClick, onDelete, isActive = false, isRunning = false, queuedCount = 0 }: Props = $props();

  function handleDelete(e: MouseEvent) {
    e.stopPropagation();
    onDelete?.(thread);
  }

  const messageCount = $derived(
    thread.messages.filter((m) => m.role === "user" || m.role === "assistant")
      .length,
  );

  const workspaceName = $derived(
    thread.workspace_root.split(/[/\\]/).filter(Boolean).pop() ?? thread.workspace_root,
  );
</script>

<Card
  class="cursor-pointer transition-colors hover:bg-muted/50 {isActive
    ? 'border-primary bg-muted'
    : ''}"
  onclick={onClick}
>
  <CardContent class="p-3">
    <div class="mb-1.5 flex items-start justify-between gap-2">
      <h3 class="line-clamp-1 min-w-0 flex-1 text-sm font-medium">
        {thread.title}
      </h3>
      {#if isRunning}
        <span
          class="h-2 w-2 shrink-0 animate-pulse rounded-full bg-primary"
          title="Running"
        ></span>
      {/if}
      {#if queuedCount > 0}
        <span
          class="shrink-0 rounded-full bg-muted px-1.5 text-[10px] text-muted-foreground"
          title="{queuedCount} message(s) queued"
        >
          {queuedCount}
        </span>
      {/if}
      {#if onDelete}
        <Button
          variant="ghost"
          size="icon"
          class="h-6 w-6 shrink-0 text-muted-foreground hover:text-destructive"
          onclick={handleDelete}
          title="Delete session"
        >
          <Trash2 class="h-3.5 w-3.5" />
        </Button>
      {/if}
    </div>
    <div class="flex items-center gap-3 text-xs text-muted-foreground">
      <div class="flex min-w-0 items-center gap-1">
        <FolderOpen class="h-3 w-3 shrink-0" />
        <span class="truncate">{workspaceName}</span>
      </div>
      <div class="flex shrink-0 items-center gap-1">
        <Clock class="h-3 w-3" />
        <span>{new Date(thread.updated_at).toLocaleDateString()}</span>
      </div>
      {#if messageCount > 0}
        <span class="shrink-0">{messageCount} msgs</span>
      {/if}
    </div>
  </CardContent>
</Card>
