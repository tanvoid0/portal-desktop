<script lang="ts">
  import type { Snippet } from "svelte";
  import { cn } from "$lib/utils.js";

  type PageContainerVariant = "readable" | "chat" | "full";

  interface Props {
    variant?: PageContainerVariant;
    class?: string;
    children: Snippet<[]>;
  }

  let {
    variant = "full",
    class: className = "",
    children,
  }: Props = $props();

  const variantClasses: Record<PageContainerVariant, string> = {
    readable:
      "mx-auto w-full max-w-readable px-[var(--content-gutter)]",
    chat: "mx-auto w-full max-w-chat px-[var(--content-gutter)]",
    full: "w-full px-[var(--content-gutter)]",
  };
</script>

<div class={cn(variantClasses[variant], className)}>
  {@render children()}
</div>
