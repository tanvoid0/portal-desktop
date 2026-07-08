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
    children?: Snippet;
  }

  let {
    message,
    showLoader = false,
    isStreaming = false,
    children,
  }: Props = $props();

  const isAssistant = $derived(message.role === "assistant");
  const isUser = $derived(message.role === "user");
  const showTypingBubble = $derived(
    showLoader && isAssistant && !message.content,
  );
  const hasBubbleContent = $derived(!!message.content || showTypingBubble);
</script>

<div class="flex flex-col {isUser ? 'items-end' : 'items-start'}">
  {#if showTypingBubble}
    <div class="flex max-w-[85%] items-start gap-2.5">
      <div
        class="mt-0.5 flex h-7 w-7 shrink-0 items-center justify-center rounded-full bg-muted text-muted-foreground"
        aria-hidden="true"
      >
        <Bot class="h-4 w-4" />
      </div>
      <TypingIndicator />
    </div>
  {:else if hasBubbleContent}
    <Card
      class="max-w-[85%] px-4 py-2.5 {isUser
        ? 'bg-primary text-primary-foreground'
        : 'bg-muted/90 border-border/50 shadow-sm'} {isStreaming && isAssistant
        ? 'ring-1 ring-primary/20'
        : ''}"
    >
      <div class="flex items-start gap-2">
        {#if isAssistant}
          <Bot class="mt-0.5 h-4 w-4 shrink-0 text-muted-foreground" />
        {:else}
          <User class="mt-0.5 h-4 w-4 shrink-0" />
        {/if}
        {#if message.content}
          {#if isUser}
            <p class="flex-1 whitespace-pre-wrap text-sm leading-relaxed">
              {message.content}
            </p>
          {:else}
            <ChatMarkdown
              content={message.content}
              variant="assistant"
              {isStreaming}
              class="flex-1"
            />
          {/if}
        {/if}
      </div>
    </Card>
  {/if}
  {#if children}
    <div class="mt-2 w-full max-w-[85%] space-y-2">
      {@render children()}
    </div>
  {/if}
  {#if message.timestamp}
    <span class="mt-1 px-1 text-xs text-muted-foreground">
      {new Date(message.timestamp).toLocaleTimeString()}
    </span>
  {/if}
</div>
