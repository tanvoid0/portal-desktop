<script lang="ts">
  import type { Snippet } from "svelte";
  import { Button } from "$lib/components/ui/button";
  import { Plus, Wrench } from "@lucide/svelte";
  import { cn } from "$lib/utils.js";
  import AIComposerShell from "./AIComposerShell.svelte";
  import type { ContextUsage } from "../../types/index.js";

  interface Props {
    value: string;
    onValueChange?: (value: string) => void;
    onSend: () => void;
    onStop?: () => void;
    running?: boolean;
    placeholder?: string;
    disabled?: boolean;
    submitOn?: "enter" | "modifier-enter";
    rows?: number;
    class?: string;
    toolbar?: Snippet;
    hint?: string | null;
    /** Feeds the `used / window` counter next to the send button. */
    contextUsage?: ContextUsage | null;
    /** Shows a wrench that opens the model-parameters panel. */
    onOpenSettings?: () => void;
    settingsOpen?: boolean;
  }

  let {
    value = $bindable(""),
    onValueChange,
    onSend,
    onStop,
    running = false,
    placeholder = "Type your message...",
    disabled = false,
    submitOn = "enter",
    rows = 2,
    class: className = "",
    toolbar,
    hint = null,
    contextUsage = null,
    onOpenSettings,
    settingsOpen = false,
  }: Props = $props();

  const hintText = $derived(
    hint ??
      (submitOn === "modifier-enter"
        ? "Ctrl/Cmd+Enter to send, Enter for new line"
        : "Enter to send, Shift+Enter for new line"),
  );

  const tokenCounter = $derived(
    contextUsage
      ? `${contextUsage.total_estimated.toLocaleString()}/${contextUsage.context_window.toLocaleString()}`
      : null,
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
  {onStop}
  {running}
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
      class="h-7 w-7 shrink-0 rounded-md text-muted-foreground hover:bg-muted"
      title="Add context"
      {disabled}
    >
      <Plus class="h-4 w-4" />
    </Button>
    {#if onOpenSettings}
      <Button
        type="button"
        variant="ghost"
        size="icon"
        class={cn(
          "h-7 w-7 shrink-0 rounded-md text-muted-foreground hover:bg-muted",
          settingsOpen && "bg-muted text-foreground",
        )}
        title="Model parameters"
        onclick={onOpenSettings}
      >
        <Wrench class="h-4 w-4" />
      </Button>
    {/if}
  {/snippet}

  {#snippet trailing()}
    {#if toolbar}
      {@render toolbar()}
    {/if}
  {/snippet}

  {#snippet meta()}
    {#if tokenCounter}
      <span
        class="font-mono text-[11px] tabular-nums text-muted-foreground/70"
        title="Estimated context used / context window"
      >
        {tokenCounter}
      </span>
      <div class="h-4 w-px bg-border" aria-hidden="true"></div>
    {/if}
  {/snippet}

  {#snippet footer()}
    {#if hintText}
      <p class="px-1 text-[11px] text-muted-foreground/70">
        {hintText}
      </p>
    {/if}
  {/snippet}
</AIComposerShell>
