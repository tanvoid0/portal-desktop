<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Trash2 } from "@lucide/svelte";
  import { cn } from "$lib/utils";
  import {
    formatCount,
    formatSessionDateTime,
    formatSessionDateTimeFull,
  } from "$lib/domains/shared/utils";

  export interface SessionCardBadge {
    label: string;
    variant?: "default" | "secondary" | "outline" | "destructive";
    class?: string;
  }

  interface Props {
    title: string;
    isActive?: boolean;
    isRunning?: boolean;
    onClick?: () => void;
    onDelete?: () => void;
    deleteTitle?: string;
    updatedAt?: string | Date | null;
    messageCount?: number;
    /** Context for the row tooltip — model, workspace, whatever fits. */
    subtitle?: string | null;
    inlineBadges?: SessionCardBadge[];
    trailingBadges?: SessionCardBadge[];
    queuedCount?: number;
  }

  let {
    title,
    isActive = false,
    isRunning = false,
    onClick,
    onDelete,
    deleteTitle = "Delete",
    updatedAt = null,
    messageCount = 0,
    subtitle = null,
    inlineBadges = [],
    trailingBadges = [],
    queuedCount = 0,
  }: Props = $props();

  const updatedIso = $derived(
    updatedAt instanceof Date ? updatedAt.toISOString() : updatedAt,
  );
  const updatedLabel = $derived(
    updatedIso ? formatSessionDateTime(updatedIso) : "",
  );

  /** Everything that used to take a second line now lives in the tooltip. */
  const tooltip = $derived(
    [title, subtitle, updatedIso ? formatSessionDateTimeFull(updatedIso) : null]
      .filter(Boolean)
      .join("\n"),
  );

  /** Right-hand metric, mirroring LM Studio's per-row token count. */
  const metric = $derived(
    messageCount > 0 ? formatCount(messageCount, "message") : updatedLabel,
  );

  function handleDelete(e: MouseEvent) {
    e.stopPropagation();
    onDelete?.();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      onClick?.();
    }
  }
</script>

<!-- One flat line per session: title left, metric right, actions on hover. -->
<div
  role="button"
  tabindex="0"
  title={tooltip}
  class={cn(
    "group flex w-full cursor-pointer items-center gap-1.5 rounded-md px-2 py-1.5 text-left text-[13px] transition-colors",
    isActive
      ? "bg-primary font-medium text-primary-foreground"
      : "text-foreground hover:bg-muted/60",
  )}
  onclick={() => onClick?.()}
  onkeydown={handleKeydown}
>
  {#if isRunning}
    <span
      class={cn(
        "h-1.5 w-1.5 shrink-0 animate-pulse rounded-full",
        isActive ? "bg-primary-foreground" : "bg-primary",
      )}
      aria-label="Running"
    ></span>
  {/if}

  <span class="min-w-0 flex-1 truncate">{title}</span>

  {#each inlineBadges as badge (badge.label)}
    <span
      class={cn(
        "shrink-0 rounded px-1 text-[10px]",
        isActive ? "bg-primary-foreground/20" : "bg-muted text-muted-foreground",
      )}
    >
      {badge.label}
    </span>
  {/each}

  {#if queuedCount > 0}
    <span
      class={cn(
        "shrink-0 rounded px-1 text-[10px] tabular-nums",
        isActive ? "bg-primary-foreground/20" : "bg-muted text-muted-foreground",
      )}
      title="{formatCount(queuedCount, 'message')} queued"
    >
      {queuedCount} queued
    </span>
  {/if}

  {#each trailingBadges as badge (badge.label)}
    <span
      class={cn(
        "shrink-0 rounded px-1 text-[10px]",
        isActive ? "bg-primary-foreground/20" : "bg-muted text-muted-foreground",
      )}
    >
      {badge.label}
    </span>
  {/each}

  {#if metric}
    <span
      class={cn(
        "shrink-0 text-[11px] tabular-nums",
        isActive ? "text-primary-foreground/70" : "text-muted-foreground",
        onDelete && "group-hover:hidden",
      )}
    >
      {metric}
    </span>
  {/if}

  {#if onDelete}
    <Button
      variant="ghost"
      size="icon"
      class={cn(
        "hidden h-5 w-5 shrink-0 group-hover:flex",
        isActive
          ? "text-primary-foreground hover:bg-primary-foreground/20"
          : "text-muted-foreground hover:text-destructive",
      )}
      onclick={handleDelete}
      title={deleteTitle}
    >
      <Trash2 class="h-3 w-3" />
    </Button>
  {/if}
</div>
