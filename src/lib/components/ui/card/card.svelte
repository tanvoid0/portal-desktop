<script lang="ts">
  import type { HTMLAttributes } from "svelte/elements";
  import { cn, type WithElementRef } from "$lib/utils.js";

  interface Props extends HTMLAttributes<HTMLDivElement> {
    ref?: HTMLDivElement | null;
    class?: string;
    children?: any;
    variant?: "default" | "elevated" | "surface" | "flat";
    elevation?: "flat" | "raised" | "elevated";
    borderAccent?: "none" | "left" | "top" | "full";
    gradient?: boolean;
    glass?: boolean;
  }

  let {
    ref = $bindable(null),
    class: className,
    children,
    variant = "default",
    elevation = "raised",
    borderAccent = "none",
    gradient = false,
    glass = false,
    ...restProps
  }: Props = $props();

  const elevationClasses = {
    flat: "shadow-card-flat",
    raised: "shadow-card-raised",
    elevated: "shadow-card-elevated",
  };

  const borderAccentClasses = {
    none: "",
    left: "border-l-4 border-l-primary",
    top: "border-t-4 border-t-primary",
    full: "border-2 border-primary/20",
  };

  const variantClasses = {
    default: "bg-card",
    elevated: "bg-card-elevated",
    surface: "bg-card-surface",
    flat: "bg-card",
  };
</script>

<div
  bind:this={ref}
  data-slot="card"
  class={cn(
    "flex flex-col gap-6 rounded-2xl border py-6 text-card-foreground transition-shadow duration-200",
    variantClasses[variant],
    elevationClasses[elevation],
    borderAccentClasses[borderAccent],
    gradient && "gradient-premium",
    glass && "glass",
    !glass && "bg-card/95 backdrop-blur-sm",
    "hover:shadow-card-hover",
    "border-border/50",
    className,
  )}
  {...restProps}
>
  {@render children?.()}
</div>
