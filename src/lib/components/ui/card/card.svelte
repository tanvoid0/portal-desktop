<script lang="ts">
  import type { HTMLAttributes } from 'svelte/elements';
  import { cn, type WithElementRef } from '$lib/utils.js';

  interface Props extends HTMLAttributes<HTMLDivElement> {
    ref?: HTMLDivElement | null;
    class?: string;
    children?: any;
    variant?: 'default' | 'elevated' | 'surface' | 'flat';
    elevation?: 'flat' | 'raised' | 'elevated';
    borderAccent?: 'none' | 'left' | 'top' | 'full';
  }

  let {
    ref = $bindable(null),
    class: className,
    children,
    variant = 'default',
    elevation = 'raised',
    borderAccent = 'none',
    ...restProps
  }: Props = $props();

  const elevationClasses = {
    flat: 'shadow-none',
    raised: 'shadow-xs',
    elevated: 'shadow-sm',
  };

  const borderAccentClasses = {
    none: '',
    left: 'border-l-2 border-l-primary',
    top: 'border-t-2 border-t-primary',
    full: 'border border-primary/30',
  };

  const variantClasses = {
    default: 'bg-card',
    elevated: 'bg-card-elevated',
    surface: 'bg-card-surface',
    flat: 'bg-card',
  };
</script>

<div
  bind:this={ref}
  data-slot="card"
  class={cn(
    // No outline: cards separate from the page by fill + hairline shadow.
    'flex flex-col gap-4 rounded-lg py-4 text-card-foreground transition-shadow',
    variantClasses[variant],
    elevationClasses[elevation],
    borderAccentClasses[borderAccent],
    className
  )}
  {...restProps}
>
  {@render children?.()}
</div>
