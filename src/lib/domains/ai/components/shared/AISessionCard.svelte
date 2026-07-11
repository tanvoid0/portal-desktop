<script lang="ts">
  import type { Component } from "svelte";
  import { Button, buttonVariants } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import { Clock, MessageSquare, Trash2 } from "@lucide/svelte";
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
    compact?: boolean;
    onClick?: () => void;
    onDelete?: () => void;
    deleteTitle?: string;
    updatedAt?: string | Date | null;
    messageCount?: number;
    subtitle?: string | null;
    subtitleTitle?: string;
    subtitleIcon?: Component<{ class?: string }>;
    hideSubtitle?: boolean;
    inlineBadges?: SessionCardBadge[];
    trailingBadges?: SessionCardBadge[];
    queuedCount?: number;
  }

  let {
    title,
    isActive = false,
    isRunning = false,
    compact = false,
    onClick,
    onDelete,
    deleteTitle = "Delete",
    updatedAt = null,
    messageCount = 0,
    subtitle = null,
    subtitleTitle,
    subtitleIcon,
    hideSubtitle = false,
    inlineBadges = [],
    trailingBadges = [],
    queuedCount = 0,
  }: Props = $props();

  function handleDelete(e: MouseEvent) {
    e.stopPropagation();
    onDelete?.();
  }

  const updatedIso = $derived(
    updatedAt instanceof Date ? updatedAt.toISOString() : updatedAt,
  );
  const updatedLabel = $derived(formatSessionDateTime(updatedIso));
  const updatedFull = $derived(formatSessionDateTimeFull(updatedIso));
  const SubtitleIcon = $derived(subtitleIcon);

  function handleActivate() {
    onClick?.();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      onClick?.();
    }
  }
</script>

<div
  role="button"
  tabindex="0"
  class={cn(
    buttonVariants({ variant: "ghost" }),
    "group relative h-auto w-full justify-start rounded-lg border bg-background text-left shadow-sm transition-colors",
    compact ? "px-2.5 py-2" : "px-3 py-2.5",
    isActive
      ? "border-primary/40 bg-primary/5 ring-1 ring-primary/20"
      : "border-border/60 hover:border-border hover:bg-muted/40",
  )}
  onclick={handleActivate}
  onkeydown={handleKeydown}
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
          class="line-clamp-2 min-w-0 flex-1 font-medium leading-snug text-foreground {compact
            ? 'text-xs'
            : 'text-[13px]'}"
          {title}
        >
          {title}
        </p>
        {#each inlineBadges as badge (badge.label)}
          <Badge
            variant={badge.variant ?? "secondary"}
            class="shrink-0 text-[10px] {badge.class ?? ''}"
          >
            {badge.label}
          </Badge>
        {/each}
      </div>

      {#if !hideSubtitle && subtitle}
        <div
          class="flex items-center gap-1 text-xs text-muted-foreground"
          title={subtitleTitle ?? subtitle}
        >
          {#if SubtitleIcon}
            <SubtitleIcon class="h-3.5 w-3.5 shrink-0 opacity-70" />
          {/if}
          <span class="truncate">{subtitle}</span>
        </div>
      {/if}

      <div class="flex flex-wrap items-center gap-x-2.5 gap-y-1 text-xs text-muted-foreground">
        {#if updatedLabel}
          <span class="inline-flex items-center gap-1" title={updatedFull}>
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
      {#each trailingBadges as badge (badge.label)}
        <Badge
          variant={badge.variant ?? "outline"}
          class="h-5 px-1.5 text-[10px] {badge.class ?? ''}"
        >
          {badge.label}
        </Badge>
      {/each}
      {#if onDelete}
        <Button
          variant="ghost"
          size="icon"
          class="h-6 w-6 text-muted-foreground opacity-0 transition-opacity hover:text-destructive group-hover:opacity-100 {isActive
            ? 'opacity-100'
            : ''}"
          onclick={handleDelete}
          title={deleteTitle}
        >
          <Trash2 class="h-3.5 w-3.5" />
        </Button>
      {/if}
    </div>
  </div>
</div>
