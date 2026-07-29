<script lang="ts">
  import ChatThinkingBlock from '$lib/domains/ai/components/chat/ChatThinkingBlock.svelte';
  import type { ChatMessage } from '../types.js';
  import { formatThoughtDuration } from '../utils/feedBlocks.js';

  interface Props {
    message: ChatMessage;
    durationMs?: number | null;
  }

  let { message, durationMs = null }: Props = $props();
</script>

<ChatThinkingBlock content={message.content ?? ''} label={formatThoughtDuration(durationMs)} />
