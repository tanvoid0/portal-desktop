<script lang="ts">
  import { cn } from "$lib/utils.js";

  interface Props {
    /** Optional status line beside the dots (e.g. "Thinking…"). */
    label?: string;
    class?: string;
    size?: "sm" | "md";
  }

  let { label, class: className, size = "md" }: Props = $props();

  const delays = ["0ms", "160ms", "320ms"];
</script>

<div
  class={cn(
    "inline-flex items-center gap-2 animate-fade-in",
    size === "sm"
      ? "py-0.5"
      : "gap-2.5 rounded-2xl border border-border/50 bg-muted/90 px-4 py-3 shadow-sm backdrop-blur-sm",
    size === "sm" && label ? "gap-1.5" : "",
    className,
  )}
  role="status"
  aria-live="polite"
  aria-label={label ?? "Assistant is typing"}
>
  <span
    class="flex items-center {size === 'sm' ? 'gap-0.5' : 'gap-1.5'}"
    aria-hidden="true"
  >
    {#each delays as delay}
      <span
        class={cn(
          "rounded-full animate-typing-bounce",
          size === "sm"
            ? "h-1 w-1 bg-muted-foreground/50"
            : "h-2 w-2 bg-muted-foreground/60",
        )}
        style="animation-delay: {delay}"
      ></span>
    {/each}
  </span>
  {#if label}
    <span class="{size === 'sm' ? 'text-[11px]' : 'text-xs'} text-muted-foreground">{label}</span>
  {/if}
</div>
