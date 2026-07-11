<script lang="ts">
  import type { Snippet } from "svelte";
  import { Button } from "$lib/components/ui/button";
  import { Plus } from "@lucide/svelte";
  import { cn } from "$lib/utils.js";
  import AIComposerShell from "./AIComposerShell.svelte";

  interface Props {
    value: string;
    onValueChange?: (value: string) => void;
    onSend: () => void;
    placeholder?: string;
    disabled?: boolean;
    submitOn?: "enter" | "modifier-enter";
    rows?: number;
    class?: string;
    toolbar?: Snippet;
    hint?: string | null;
  }

  let {
    value = $bindable(""),
    onValueChange,
    onSend,
    placeholder = "Type your message...",
    disabled = false,
    submitOn = "enter",
    rows = 2,
    class: className = "",
    toolbar,
    hint = null,
  }: Props = $props();

  const hintText = $derived(
    hint ??
      (submitOn === "modifier-enter"
        ? "Ctrl/Cmd+Enter to send, Enter for new line"
        : "Enter to send, Shift+Enter for new line"),
  );

  $effect(() => {
    if (onValueChange) {
      onValueChange(value);
    }
  });
</script>

<AIComposerShell
  bind:value
  {onSend}
  {placeholder}
  {disabled}
  {submitOn}
  {rows}
  class={className}
>
  {#snippet leading()}
    <Button
      type="button"
      variant="ghost"
      size="icon"
      class="mb-0.5 h-8 w-8 shrink-0 rounded-full text-muted-foreground hover:bg-muted/80"
      title="Add context"
      disabled={disabled}
    >
      <Plus class="h-4 w-4" />
    </Button>
  {/snippet}

  {#snippet trailing()}
    {#if toolbar}
      {@render toolbar()}
    {/if}
  {/snippet}

  {#snippet footer()}
    {#if hintText}
      <p class={cn("text-xs text-muted-foreground", toolbar ? "px-1" : "text-center")}>
        {hintText}
      </p>
    {/if}
  {/snippet}
</AIComposerShell>
