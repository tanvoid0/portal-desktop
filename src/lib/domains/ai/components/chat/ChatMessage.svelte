<script lang="ts">
  import type { Snippet } from "svelte";
  import { Card } from "$lib/components/ui/card";
  import { Bot, User } from "@lucide/svelte";
  import TypingIndicator from "$lib/components/ui/typing-indicator.svelte";
  import ChatMarkdown from "$lib/components/ui/chat-markdown/ChatMarkdown.svelte";
  import type { ChatMessage as ChatMessageType } from "../../types/index.js";

  interface Props {
    message: ChatMessageType;
    showLoader?: boolean;
    /** Show a blinking cursor after streamed content. */
    isStreaming?: boolean;
    responseLatencyMs?: number | null;
    children?: Snippet;
  }

  let {
    message,
    showLoader = false,
    isStreaming = false,
    responseLatencyMs = null,
    children,
  }: Props = $props();

  const isAssistant = $derived(message.role === "assistant");
  const isUser = $derived(message.role === "user");
  const formattedTimestamp = $derived(formatTimestamp(message.timestamp));
  const formattedLatency = $derived(formatLatency(responseLatencyMs));
  const showTypingBubble = $derived(
    showLoader && isAssistant && !message.content,
  );
  const hasBubbleContent = $derived(!!message.content || showTypingBubble);

  function formatTimestamp(value: Date | string | undefined): string {
    if (!value) return "";
    const date = value instanceof Date ? value : new Date(value);
    if (Number.isNaN(date.getTime())) return "";

    const now = new Date();
    const sameDay = now.toDateString() === date.toDateString();
    if (sameDay) {
      return date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
    }
    return date.toLocaleString([], {
      year: "numeric",
      month: "short",
      day: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
    });
  }

  function formatLatency(ms: number | null): string {
    if (ms == null || ms < 0) return "";
    if (ms < 1000) return "< 1s";
    const totalSeconds = Math.round(ms / 1000);
    if (totalSeconds < 60) return `${totalSeconds}s`;
    const minutes = Math.floor(totalSeconds / 60);
    const seconds = totalSeconds % 60;
    return seconds === 0 ? `${minutes}m` : `${minutes}m ${seconds}s`;
  }
</script>

<div class="flex flex-col {isUser ? 'items-end' : 'items-start'}">
  {#if showTypingBubble}
    <TypingIndicator size="sm" label="Thinking…" />
  {:else if hasBubbleContent}
    <Card
      class="max-w-[90%] px-3 py-2 {isUser
        ? 'bg-primary text-primary-foreground'
        : 'border-border/40 bg-muted/30 shadow-none'} {isStreaming && isAssistant
        ? 'ring-1 ring-primary/15'
        : ''}"
    >
      {#if formattedTimestamp || formattedLatency}
        <div
          class="mb-1.5 flex flex-wrap items-center gap-x-2 gap-y-0.5 text-[10px] {isUser
            ? 'text-primary-foreground/75'
            : 'text-muted-foreground'}"
        >
          {#if formattedTimestamp}
            <time datetime={typeof message.timestamp === 'string' ? message.timestamp : message.timestamp?.toISOString()}>
              {formattedTimestamp}
            </time>
          {/if}
          {#if formattedLatency}
            {#if formattedTimestamp}
              <span aria-hidden="true">•</span>
            {/if}
            <span>Replied in {formattedLatency}</span>
          {/if}
        </div>
      {/if}
      <div class="flex items-start gap-1.5">
        {#if isAssistant}
          <Bot class="mt-0.5 h-3.5 w-3.5 shrink-0 text-muted-foreground" />
        {:else}
          <User class="mt-0.5 h-3.5 w-3.5 shrink-0" />
        {/if}
        {#if message.content}
          {#if isUser}
            <p class="flex-1 whitespace-pre-wrap text-xs leading-relaxed">
              {message.content}
            </p>
          {:else}
            <ChatMarkdown
              content={message.content}
              variant="assistant"
              {isStreaming}
              density="compact"
              class="flex-1"
            />
          {/if}
        {/if}
      </div>
    </Card>
  {/if}
  {#if children}
    <div class="mt-1.5 w-full max-w-[90%] space-y-1.5">
      {@render children()}
    </div>
  {/if}
</div>
