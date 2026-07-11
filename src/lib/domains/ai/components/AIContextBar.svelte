<script lang="ts">
  import * as Popover from "$lib/components/ui/popover/index.js";
  import { cn } from "$lib/utils.js";
  import type { ContextUsage, LlmUsage } from "../types/index.js";
  import {
    contextBarColor,
    contextCategoriesForDisplay,
    formatTokenCount,
  } from "../utils/contextUsage.js";

  interface Props {
    contextUsage?: ContextUsage | null;
    llmUsage?: LlmUsage | null;
    variant?: "bar" | "ring";
    class?: string;
  }

  let {
    contextUsage = null,
    llmUsage = null,
    variant = "bar",
    class: className = "",
  }: Props = $props();

  const hasData = $derived(!!contextUsage && contextUsage.context_window > 0);
  const percent = $derived(
    hasData ? Math.min(100, Math.max(0, contextUsage!.percent_used)) : 0,
  );
  const barColor = $derived(contextBarColor(percent));
  const categories = $derived(
    contextUsage ? contextCategoriesForDisplay(contextUsage) : [],
  );
  const summaryLabel = $derived(
    hasData
      ? `${formatTokenCount(contextUsage!.total_estimated)} / ${formatTokenCount(contextUsage!.context_window)}`
      : "Context",
  );
  const ringRadius = 7;
  const ringCircumference = 2 * Math.PI * ringRadius;
  const ringOffset = $derived(ringCircumference * (1 - percent / 100));
  const ringStroke = $derived(
    percent >= 90 ? "stroke-destructive" : percent >= 75 ? "stroke-amber-500" : "stroke-primary",
  );
</script>

{#if hasData}
  <Popover.Root>
    <Popover.Trigger
      type="button"
      class={cn(
        variant === "ring"
          ? "inline-flex shrink-0 items-center gap-1.5 rounded-md px-1 py-0.5 transition-colors hover:bg-muted/60"
          : "flex min-w-0 max-w-[220px] flex-1 items-center gap-2 rounded-md px-1 py-0.5 text-left transition-colors hover:bg-muted/60",
        className,
      )}
      title="Context window usage"
    >
      {#if variant === "ring"}
        <svg viewBox="0 0 18 18" class="h-4 w-4 shrink-0" aria-hidden="true">
          <circle
            cx="9"
            cy="9"
            r={ringRadius}
            fill="none"
            class="stroke-muted-foreground/25"
            stroke-width="2"
          />
          <circle
            cx="9"
            cy="9"
            r={ringRadius}
            fill="none"
            class={cn("transition-all", ringStroke)}
            stroke-width="2"
            stroke-linecap="round"
            stroke-dasharray={ringCircumference}
            stroke-dashoffset={ringOffset}
            transform="rotate(-90 9 9)"
          />
        </svg>
        <span class="text-[11px] tabular-nums">{percent.toFixed(0)}%</span>
      {:else}
        <span class="shrink-0 text-[10px] font-medium uppercase tracking-wide text-muted-foreground">
          Context
        </span>
        <div class="relative h-1.5 min-w-[72px] flex-1 overflow-hidden rounded-full bg-primary/20">
          <div
            class={cn("absolute inset-y-0 left-0 rounded-full transition-all", barColor)}
            style="width: {percent}%"
          ></div>
        </div>
        <span class="shrink-0 text-[10px] tabular-nums text-muted-foreground">
          {percent.toFixed(0)}%
        </span>
      {/if}
    </Popover.Trigger>
    <Popover.Content align="start" class="w-72 p-3">
      <div class="space-y-3">
        <div>
          <p class="text-xs font-medium">Context window</p>
          <p class="mt-0.5 text-sm tabular-nums">
            {summaryLabel}
            <span class="text-muted-foreground"> ({percent.toFixed(1)}%)</span>
          </p>
        </div>

        {#if categories.length > 0}
          <div class="space-y-1.5">
            <p class="text-xs font-medium text-muted-foreground">Breakdown</p>
            {#each categories as cat}
              <div class="flex items-center justify-between gap-2 text-xs">
                <span class="truncate text-muted-foreground">{cat.label}</span>
                <span class="shrink-0 tabular-nums">{formatTokenCount(cat.tokens)}</span>
              </div>
            {/each}
          </div>
        {/if}

        {#if llmUsage && llmUsage.total_tokens > 0}
          <div class="divider-edge-t divider-edge-full pt-2">
            <p class="text-xs font-medium text-muted-foreground">Last turn (LLM)</p>
            <p class="mt-0.5 text-xs tabular-nums">
              {formatTokenCount(llmUsage.total_tokens)} tokens
              {#if llmUsage.cost_usd > 0}
                <span class="text-muted-foreground">
                  · ${llmUsage.cost_usd.toFixed(4)}
                </span>
              {/if}
            </p>
          </div>
        {/if}

        <p class="text-[10px] leading-snug text-muted-foreground">
          Estimates may differ from provider billing. Output budget:
          {formatTokenCount(contextUsage!.reserved_output)} reserved.
        </p>
      </div>
    </Popover.Content>
  </Popover.Root>
{/if}
