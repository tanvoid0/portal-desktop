<script lang="ts">
  import type { Snippet } from "svelte";
  import { Textarea } from "$lib/components/ui/textarea";
  import { Button } from "$lib/components/ui/button";
  import { ArrowUp, Send } from "@lucide/svelte";
  import { cn } from "$lib/utils.js";

  interface Props {
    value: string;
    onValueChange?: (value: string) => void;
    onSend: () => void;
    placeholder?: string;
    disabled?: boolean;
    variant?: "default" | "floating";
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
    variant = "default",
    submitOn = "enter",
    rows = 3,
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

  function handleKeydown(event: KeyboardEvent) {
    if (event.key !== "Enter") return;

    if (submitOn === "modifier-enter") {
      if (event.ctrlKey || event.metaKey) {
        event.preventDefault();
        onSend();
      }
      return;
    }

    if (!event.shiftKey) {
      event.preventDefault();
      onSend();
    }
  }

  $effect(() => {
    if (onValueChange) {
      onValueChange(value);
    }
  });
</script>

{#if variant === "floating"}
  <div class={cn("px-4 pb-4 pt-2", className)}>
    <div
      class="mx-auto w-full max-w-3xl rounded-2xl border border-border bg-background shadow-lg"
    >
      <Textarea
        bind:value
        {placeholder}
        {rows}
        class="min-h-[52px] resize-none border-0 bg-transparent px-4 pt-4 text-sm shadow-none focus-visible:ring-0"
        onkeydown={handleKeydown}
        {disabled}
      />
      <div class="flex items-center justify-between gap-2 px-3 pb-3">
        <div class="flex min-w-0 flex-1 items-center gap-2">
          {#if toolbar}
            {@render toolbar()}
          {/if}
        </div>
        <Button
          onclick={onSend}
          disabled={!value.trim() || disabled}
          size="icon"
          class="h-8 w-8 shrink-0 rounded-full"
        >
          <ArrowUp class="h-4 w-4" />
        </Button>
      </div>
    </div>
    {#if hintText}
      <p class="mt-2 text-center text-xs text-muted-foreground">{hintText}</p>
    {/if}
  </div>
{:else}
  <div class={cn("border-t p-4", className)}>
    <div class="flex gap-2">
      <Textarea
        bind:value
        {placeholder}
        {rows}
        class="resize-none"
        onkeydown={handleKeydown}
        {disabled}
      />
      <Button
        onclick={onSend}
        disabled={!value.trim() || disabled}
        class="self-end"
        size="sm"
      >
        <Send class="h-4 w-4" />
      </Button>
    </div>
    {#if hintText}
      <p class="mt-2 text-xs text-muted-foreground">{hintText}</p>
    {/if}
  </div>
{/if}
