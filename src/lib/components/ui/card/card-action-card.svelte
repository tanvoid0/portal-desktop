<script lang="ts">
  import type { HTMLAttributes } from "svelte/elements";
  import { cn, type WithElementRef } from "$lib/utils.js";
  import Card from "./card.svelte";
  import CardHeader from "./card-header.svelte";
  import CardTitle from "./card-title.svelte";
  import CardDescription from "./card-description.svelte";
  import CardContent from "./card-content.svelte";
  import CardFooter from "./card-footer.svelte";
  import { Button } from "$lib/components/ui/button";

  interface Props extends HTMLAttributes<HTMLDivElement> {
    ref?: HTMLDivElement | null;
    class?: string;
    children?: any;
    title?: string;
    description?: string;
    ctaLabel?: string;
    ctaAction?: () => void;
    onClick?: () => void;
    gradient?: boolean;
  }

  let {
    ref = $bindable(null),
    class: className,
    children,
    title,
    description,
    ctaLabel,
    ctaAction,
    onClick,
    gradient = true,
    ...restProps
  }: Props = $props();

  function handleClick() {
    if (onClick) {
      onClick();
    }
  }

  function handleCtaClick(e: MouseEvent) {
    e.stopPropagation();
    if (ctaAction) {
      ctaAction();
    }
  }
</script>

<Card
  bind:ref
  elevation="elevated"
  {gradient}
  glass={true}
  onclick={handleClick}
  class={cn(
    "group relative cursor-pointer overflow-hidden",
    "hover:shadow-card-hover",
    "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2",
    onClick && "transition-shadow duration-200",
    className,
  )}
  {...restProps}
>
  <!-- Hover gradient effect -->
  <div
    class="group-hover:from-primary/8 group-hover:via-primary/4 pointer-events-none absolute inset-0 bg-gradient-to-br from-primary/0 via-primary/0 to-primary/0 transition-opacity duration-200 group-hover:to-transparent"
  ></div>
  {#if title || description}
    <CardHeader>
      {#if title}
        <CardTitle class="transition-colors group-hover:text-primary"
          >{title}</CardTitle
        >
      {/if}
      {#if description}
        <CardDescription
          class="transition-colors group-hover:text-foreground/80"
          >{description}</CardDescription
        >
      {/if}
    </CardHeader>
  {/if}

  {#if children}
    <CardContent>
      {@render children?.()}
    </CardContent>
  {/if}

  {#if ctaLabel}
    <CardFooter class="pt-4">
      <Button
        onclick={handleCtaClick}
        class="w-full transition-transform group-hover:scale-105"
        variant="default"
      >
        {ctaLabel}
      </Button>
    </CardFooter>
  {/if}
</Card>
