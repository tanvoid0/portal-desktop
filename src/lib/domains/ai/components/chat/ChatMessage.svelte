<script lang="ts">
  import type { Snippet } from "svelte";
  import { ChevronRight } from "@lucide/svelte";
  import TypingIndicator from "$lib/components/ui/typing-indicator.svelte";
  import ChatMarkdown from "$lib/components/ui/chat-markdown/ChatMarkdown.svelte";
  import type { ChatMessage as ChatMessageType } from "../../types/index.js";
  import { splitThinking } from "../../utils/thinking.js";

  interface Props {
    message: ChatMessageType;
    showLoader?: boolean;
    /** Show a blinking cursor after streamed content. */
    isStreaming?: boolean;
    /** Label above an assistant turn, e.g. the model id. */
    modelLabel?: string | null;
    children?: Snippet;
  }

  let {
    message,
    showLoader = false,
    isStreaming = false,
    modelLabel = null,
    children,
  }: Props = $props();

  const isAssistant = $derived(message.role === "assistant");
  const isUser = $derived(message.role === "user");
  const parts = $derived(splitThinking(message.content));
  const stats = $derived(message.stats ?? null);
  const showTypingBubble = $derived(
    showLoader && isAssistant && !message.content,
  );
  /** Open while the model is still inside the think block, collapsed after. */
  const thinkingOpen = $derived(parts.reasoning != null && !parts.closed);

  const statChips = $derived(buildChips());

  function buildChips(): string[] {
    if (!stats) return [];
    const chips: string[] = [];
    if (stats.tokensPerSecond != null)
      chips.push(`${stats.tokensPerSecond.toFixed(2)} tok/sec`);
    if (stats.completionTokens != null)
      chips.push(`${stats.completionTokens} tokens`);
    if (stats.durationMs != null)
      chips.push(`${(stats.durationMs / 1000).toFixed(2)}s`);
    if (stats.stopReason) chips.push(`Stop reason: ${stats.stopReason}`);
    return chips;
  }

  function formatSeconds(ms: number): string {
    return `${(ms / 1000).toFixed(2)} seconds`;
  }
</script>

<!--
Assistant turns render unboxed so long answers read as a document; only user
turns get a bubble. Model id, stats and actions stay visible (LM Studio style).
-->
<div class="group/msg flex flex-col gap-1.5 {isUser ? 'items-end' : 'items-stretch'}">
  {#if isAssistant && modelLabel}
    <div class="font-mono text-[11px] text-muted-foreground">{modelLabel}</div>
  {/if}

  {#if showTypingBubble}
    <TypingIndicator size="sm" label="Thinking…" />
  {:else if message.content}
    {#if isUser}
      <div
        class="max-w-[85%] rounded-2xl bg-muted px-3.5 py-2 text-sm leading-relaxed text-foreground"
      >
        <p class="whitespace-pre-wrap">{message.content}</p>
      </div>
    {:else}
      {#if parts.reasoning != null}
        <details
          class="group/think rounded-lg border border-border/60 open:bg-muted/20 [&:not([open])]:border-transparent"
          open={thinkingOpen}
        >
          <summary
            class="flex cursor-pointer list-none items-center gap-1.5 px-2 py-1.5 text-xs text-muted-foreground marker:content-none hover:text-foreground"
          >
            <ChevronRight
              class="h-3.5 w-3.5 transition-transform group-open/think:rotate-90"
            />
            {#if !parts.closed}
              Thinking…
            {:else if stats?.thinkingMs != null}
              Thought for {formatSeconds(stats.thinkingMs)}
            {:else}
              Thoughts
            {/if}
          </summary>
          <div class="px-3 pb-2 text-muted-foreground/70">
            <ChatMarkdown
              content={parts.reasoning}
              variant="assistant"
              density="compact"
            />
          </div>
        </details>
      {/if}

      {#if parts.answer}
        <ChatMarkdown
          content={parts.answer}
          variant="assistant"
          {isStreaming}
        />
      {/if}
    {/if}
  {/if}

  {#if isAssistant && statChips.length}
    <div class="flex flex-wrap items-center gap-1.5">
      {#each statChips as chip}
        <span
          class="rounded-md bg-muted/60 px-1.5 py-0.5 text-[10px] tabular-nums text-muted-foreground"
        >
          {chip}
        </span>
      {/each}
    </div>
  {/if}

  {#if children}
    {@render children()}
  {/if}
</div>
