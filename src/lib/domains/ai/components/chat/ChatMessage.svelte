<script lang="ts">
  import type { Snippet } from 'svelte';
  import ChatUserBubble from './ChatUserBubble.svelte';
  import ChatAssistantTurn from './ChatAssistantTurn.svelte';
  import type { ChatMessage as ChatMessageType } from '../../types/index.js';

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

  const isUser = $derived(message.role === 'user');
  const stats = $derived(message.stats ?? null);

  const statChips = $derived(buildChips());

  function buildChips(): string[] {
    if (!stats) return [];
    const chips: string[] = [];
    if (stats.tokensPerSecond != null) chips.push(`${stats.tokensPerSecond.toFixed(2)} tok/sec`);
    if (stats.completionTokens != null) chips.push(`${stats.completionTokens} tokens`);
    if (stats.durationMs != null) chips.push(`${(stats.durationMs / 1000).toFixed(2)}s`);
    if (stats.stopReason) chips.push(`Stop reason: ${stats.stopReason}`);
    return chips;
  }
</script>

<!--
Assistant turns render unboxed so long answers read as a document; only user
turns get a bubble. Model id, stats and actions stay visible (LM Studio style).
-->
{#if isUser}
  <div class="group/msg flex flex-col items-end gap-1.5">
    {#if message.content}
      <ChatUserBubble content={message.content} />
    {/if}
    {#if children}
      {@render children()}
    {/if}
  </div>
{:else}
  <div class="group/msg">
    <ChatAssistantTurn
      content={message.content}
      label={modelLabel}
      {isStreaming}
      {showLoader}
      thinkingMs={stats?.thinkingMs ?? null}
    >
      {#if statChips.length}
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
    </ChatAssistantTurn>
  </div>
{/if}
