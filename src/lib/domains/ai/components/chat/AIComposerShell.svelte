<script lang="ts">
  import type { Snippet } from "svelte";
  import { Button } from "$lib/components/ui/button";
  import { Textarea } from "$lib/components/ui/textarea";
  import { ArrowUp, ListOrdered, Square } from "@lucide/svelte";
  import { cn } from "$lib/utils.js";

  interface Props {
    value: string;
    onSend: () => void;
    onStop?: () => void;
    placeholder?: string;
    disabled?: boolean;
    running?: boolean;
    submitOn?: "enter" | "modifier-enter";
    rows?: number;
    /** Show queue icon on send when running with text */
    queueSend?: boolean;
    sendTitle?: string;
    onModeCycle?: () => void;
    class?: string;
    leading?: Snippet;
    trailing?: Snippet;
    above?: Snippet;
    footer?: Snippet;
  }

  let {
    value = $bindable(""),
    onSend,
    onStop,
    placeholder = "Type your message...",
    disabled = false,
    running = false,
    submitOn = "modifier-enter",
    rows = 2,
    queueSend = false,
    sendTitle,
    class: className = "",
    leading,
    trailing,
    above,
    footer,
    onModeCycle,
  }: Props = $props();

  let textareaEl = $state<HTMLTextAreaElement | null>(null);

  const resolvedSendTitle = $derived(
    sendTitle ??
      (running
        ? value.trim()
          ? "Queue message (Ctrl/Cmd+Enter)"
          : "Stop agent"
        : submitOn === "modifier-enter"
          ? "Send (Ctrl/Cmd+Enter)"
          : "Send (Enter)"),
  );

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Tab" && event.shiftKey) {
      event.preventDefault();
      onModeCycle?.();
      return;
    }

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

  function autoResize() {
    if (!textareaEl) return;
    textareaEl.style.height = "auto";
    textareaEl.style.height = `${Math.min(textareaEl.scrollHeight, 160)}px`;
  }

  $effect(() => {
    value;
    autoResize();
  });
</script>

<div class={cn("px-4 pb-3 pt-2", className)}>
  <div class="mx-auto w-full max-w-3xl lg:max-w-4xl xl:max-w-5xl 2xl:max-w-6xl">
    {#if above}
      {@render above()}
    {/if}

    <div
      class="flex items-end gap-1 rounded-[26px] border border-border/80 bg-background px-2 py-2 shadow-sm ring-1 ring-black/[0.03] dark:ring-white/[0.04]"
    >
      {#if leading}
        {@render leading()}
      {/if}

      <Textarea
        bind:ref={textareaEl}
        bind:value
        {rows}
        {placeholder}
        {disabled}
        onkeydown={handleKeydown}
        oninput={autoResize}
        class="max-h-40 min-h-[36px] flex-1 resize-none border-0 bg-transparent px-1 py-1.5 text-xs leading-relaxed shadow-none outline-none placeholder:text-muted-foreground/70 focus-visible:ring-0"
      />

      <div class="mb-0.5 flex shrink-0 items-center gap-1">
        {#if trailing}
          {@render trailing()}
        {/if}

        {#if running && !value.trim() && onStop}
          <Button
            type="button"
            onclick={onStop}
            size="icon"
            variant="destructive"
            class="h-8 w-8 rounded-full"
            title="Stop agent"
          >
            <Square class="h-3.5 w-3.5 fill-current" />
          </Button>
        {:else}
          <Button
            type="button"
            onclick={onSend}
            disabled={!value.trim() || disabled}
            size="icon"
            class="h-8 w-8 rounded-full"
            title={resolvedSendTitle}
          >
            {#if queueSend && running && value.trim()}
              <ListOrdered class="h-4 w-4" />
            {:else}
              <ArrowUp class="h-4 w-4" />
            {/if}
          </Button>
        {/if}
      </div>
    </div>

    {#if footer}
      <div class="mt-2">
        {@render footer()}
      </div>
    {/if}
  </div>
</div>
