<script lang="ts">
  import type { HTMLAttributes } from 'svelte/elements';
  import { cn, type WithElementRef } from '$lib/utils.js';
  import Card from './card.svelte';
  import CardHeader from './card-header.svelte';
  import CardTitle from './card-title.svelte';
  import CardDescription from './card-description.svelte';
  import CardContent from './card-content.svelte';

  interface Props extends Omit<HTMLAttributes<HTMLDivElement>, 'onclick'> {
    ref?: HTMLDivElement | null;
    class?: string;
    children?: any;
    title?: string;
    description?: string;
    value?: string | number;
    icon?: any;
    trend?: 'up' | 'down' | 'neutral';
    trendValue?: string;
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
    onclick,
    onkeydown,
    ...restProps
  }: Props = $props();

  const trendConfig = {
    up: {
      color: 'text-success-600 dark:text-success-400',
      icon: '↑',
    },
    down: {
      color: 'text-error-600 dark:text-error-400',
      icon: '↓',
    },
    neutral: {
      color: 'text-muted-foreground',
      icon: '→',
    },
  };

  const trendStyles = trend ? trendConfig[trend] : null;
</script>

<Card
  bind:ref
  variant="surface"
  elevation="raised"
  {onclick}
  {onkeydown}
  class={cn('group relative overflow-hidden', className)}
  {...restProps}
>
  <div class="relative z-10">
    {#if title || description || icon}
      <CardHeader class="pb-2">
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
              class="flex size-9 flex-shrink-0 items-center justify-center rounded-md bg-primary/10"
            >
              <Icon class="size-5 text-primary" />
            </div>
          {/if}
        </div>
      </CardHeader>
    {/if}

    <CardContent>
      {#if value !== undefined}
        <div class="space-y-2">
          <div class="flex items-baseline gap-2">
            <span class="text-3xl font-semibold tracking-tight text-foreground">
              {value}
            </span>
            {#if trend && trendValue}
              <span class={cn('text-sm font-medium', trendStyles?.color)}>
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
