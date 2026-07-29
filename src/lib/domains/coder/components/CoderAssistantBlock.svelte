<script lang="ts">
  import ChatAssistantTurn from '$lib/domains/ai/components/chat/ChatAssistantTurn.svelte';
  import type { ChatMessage } from '../types.js';
  import { formatWorkedDuration } from '../utils/feedBlocks.js';

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
  const typingLabel = $derived(
    waitingSeconds >= 5 ? `Waiting on model… ${Math.round(waitingSeconds)}s` : 'Thinking…'
  );
</script>

<ChatAssistantTurn
  content={message.content}
  label={workedLabel || null}
  {isStreaming}
  {showLoader}
  loaderLabel={typingLabel}
  density="compact"
/>
