<script lang="ts">
  import type { HTMLAttributes } from "svelte/elements";
  import { cn, type WithElementRef } from "$lib/utils.js";
  import Card from "./card.svelte";
  import CardHeader from "./card-header.svelte";
  import CardTitle from "./card-title.svelte";
  import CardDescription from "./card-description.svelte";
  import CardContent from "./card-content.svelte";

  interface Props extends Omit<HTMLAttributes<HTMLDivElement>, "onclick"> {
    ref?: HTMLDivElement | null;
    class?: string;
    children?: any;
    title?: string;
    description?: string;
    value?: string | number;
    icon?: any;
    trend?: "up" | "down" | "neutral";
    trendValue?: string;
    gradient?: boolean;
    onclick?: () => void;
    onkeydown?: (e: KeyboardEvent) => void;
  }

  let {
    ref = $bindable(null),
    class: className,
    children,
    title,
    description,
    value,
    icon,
    trend,
    trendValue,
    gradient = true,
    onclick,
    onkeydown,
    ...restProps
  }: Props = $props();

  const trendConfig = {
    up: {
      color: "text-success-600 dark:text-success-400",
      icon: "↑",
    },
    down: {
      color: "text-error-600 dark:text-error-400",
      icon: "↓",
    },
    neutral: {
      color: "text-muted-foreground",
      icon: "→",
    },
  };

  const trendStyles = trend ? trendConfig[trend] : null;
</script>

<Card
  bind:ref
  variant="surface"
  elevation="elevated"
  {gradient}
  glass={true}
  {onclick}
  {onkeydown}
  class={cn("group relative overflow-hidden", className)}
  {...restProps}
>
  <!-- Modern gradient overlay -->
  <div
    class="gradient-modern pointer-events-none absolute inset-0 opacity-50"
  ></div>
  <div
    class="from-primary/8 via-primary/4 pointer-events-none absolute inset-0 bg-gradient-to-br to-transparent"
  ></div>

  <div class="relative z-10">
    {#if title || description || icon}
      <CardHeader class="pb-3">
        <div class="flex items-start justify-between">
          <div class="flex-1">
            {#if title}
              <CardTitle class="text-lg">{title}</CardTitle>
            {/if}
            {#if description}
              <CardDescription class="mt-1">{description}</CardDescription>
            {/if}
          </div>
          {#if icon}
            {@const Icon = icon}
            <div
              class="flex h-12 w-12 flex-shrink-0 items-center justify-center rounded-lg bg-primary/10"
            >
              <Icon class="h-6 w-6 text-primary" />
            </div>
          {/if}
        </div>
      </CardHeader>
    {/if}

    <CardContent>
      {#if value !== undefined}
        <div class="space-y-2">
          <div class="flex items-baseline gap-2">
            <span
              class="bg-gradient-to-r from-primary via-primary/85 to-primary/70 bg-clip-text text-4xl font-extrabold text-transparent"
            >
              {value}
            </span>
            {#if trend && trendValue}
              <span class={cn("text-sm font-medium", trendStyles?.color)}>
                {trendStyles?.icon}
                {trendValue}
              </span>
            {/if}
          </div>
        </div>
      {/if}

      {#if children}
        <div class="mt-4">
          {@render children?.()}
        </div>
      {/if}
    </CardContent>
  </div>
</Card>
