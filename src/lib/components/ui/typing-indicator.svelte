<script lang="ts">
  import { cn } from "$lib/utils.js";

  interface Props {
    /** Optional status line beside the dots (e.g. "Thinking…"). */
    label?: string;
    class?: string;
    size?: "sm" | "md";
  }

  let { label, class: className, size = "md" }: Props = $props();

  const dotSize = size === "sm" ? "h-1.5 w-1.5" : "h-2 w-2";
  const delays = ["0ms", "160ms", "320ms"];
</script>

<div
  class={cn(
    "inline-flex items-center gap-2.5 rounded-2xl border border-border/50 bg-muted/90 shadow-sm backdrop-blur-sm animate-fade-in",
    size === "sm" ? "px-3 py-2" : "px-4 py-3",
    className,
  )}
  role="status"
  aria-live="polite"
  aria-label={label ?? "Assistant is typing"}
>
  <span class="flex items-center gap-1.5" aria-hidden="true">
    {#each delays as delay}
      <span
        class={cn(
          "rounded-full bg-muted-foreground/60 animate-typing-bounce",
          dotSize,
        )}
        style="animation-delay: {delay}"
      ></span>
    {/each}
  </span>
  {#if label}
    <span class="text-xs text-muted-foreground">{label}</span>
  {/if}
</div>
