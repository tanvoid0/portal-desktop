<!--
  ChatAssistantTurn — the one assistant turn treatment: unboxed markdown so long
  answers read as a document, an optional meta line above (model id, "Worked
  for 8s"), reasoning folded into a disclosure, and a typing indicator while the
  first token is outstanding. Shared by the chat page, coder feed and terminal.
-->
<script lang="ts">
  import type { Snippet } from 'svelte';
  import TypingIndicator from '$lib/components/ui/typing-indicator.svelte';
  import ChatMarkdown from '$lib/components/ui/chat-markdown/ChatMarkdown.svelte';
  import ChatThinkingBlock from './ChatThinkingBlock.svelte';
  import { splitThinking, formatThoughtLabel } from '../../utils/thinking.js';

  interface Props {
    content?: string | null;
    /** Small line above the answer — model id, "Worked for 8s", … */
    label?: string | null;
    isStreaming?: boolean;
    showLoader?: boolean;
    loaderLabel?: string;
    density?: 'default' | 'compact';
    /** Time spent inside the think block, once known. */
    thinkingMs?: number | null;
    /** Stat chips, action rows — rendered under the answer. */
    children?: Snippet;
  }

  let {
    content = '',
    label = null,
    isStreaming = false,
    showLoader = false,
    loaderLabel = 'Thinking…',
    density = 'default',
    thinkingMs = null,
    children,
  }: Props = $props();

  const parts = $derived(splitThinking(content ?? ''));
  const showTyping = $derived(showLoader && !content);
  /** Open while the model is still inside the think block, collapsed after. */
  const thinkingOpen = $derived(parts.reasoning != null && !parts.closed);
  const thinkingLabel = $derived(parts.closed ? formatThoughtLabel(thinkingMs) : 'Thinking…');
</script>

<div class="flex flex-col gap-1.5">
  {#if label}
    <p class="text-[11px] text-muted-foreground">{label}</p>
  {/if}

  {#if showTyping}
    <TypingIndicator size="sm" label={loaderLabel} />
  {:else if content}
    {#if parts.reasoning != null}
      <ChatThinkingBlock content={parts.reasoning} label={thinkingLabel} open={thinkingOpen} />
    {/if}
    {#if parts.answer}
      <ChatMarkdown content={parts.answer} variant="assistant" {isStreaming} {density} />
    {/if}
  {/if}

  {#if children}
    {@render children()}
  {/if}
</div>
