<script lang="ts">
  import TypingIndicator from "$lib/components/ui/typing-indicator.svelte";
  import ChatMarkdown from "$lib/components/ui/chat-markdown/ChatMarkdown.svelte";
  import type { ChatMessage } from "../types.js";
  import { formatWorkedDuration } from "../utils/feedBlocks.js";

  interface Props {
    message: ChatMessage;
    responseLatencyMs?: number | null;
    isStreaming?: boolean;
    showLoader?: boolean;
    /** Seconds waited on the current LLM step with no output yet (0 = not waiting). */
    waitingSeconds?: number;
  }

  let {
    message,
    responseLatencyMs = null,
    isStreaming = false,
    showLoader = false,
    waitingSeconds = 0,
  }: Props = $props();

  const workedLabel = $derived(formatWorkedDuration(responseLatencyMs));
  const showTyping = $derived(showLoader && !message.content);
  const typingLabel = $derived(
    waitingSeconds >= 5 ? `Waiting on model… ${Math.round(waitingSeconds)}s` : "Thinking…",
  );
</script>

<div class="space-y-1">
  {#if workedLabel}
    <p class="text-[10px] text-muted-foreground">{workedLabel}</p>
  {/if}

  {#if showTyping}
    <TypingIndicator size="sm" label={typingLabel} />
  {:else if message.content}
    <ChatMarkdown
      content={message.content}
      variant="assistant"
      {isStreaming}
      density="compact"
    />
  {/if}
</div>
