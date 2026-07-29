<!--
  ChatThinkingBlock — collapsed reasoning disclosure shared by the chat page
  (inline <think> blocks) and the coder feed (separate thought messages).
-->
<script lang="ts">
  import { ChevronRight } from '@lucide/svelte';
  import ChatMarkdown from '$lib/components/ui/chat-markdown/ChatMarkdown.svelte';

  interface Props {
    content: string;
    /** Summary text, e.g. "Thinking…" or "Thought for 4s". */
    label: string;
    /** Force-open while reasoning is still streaming. */
    open?: boolean;
  }

  let { content, label, open = false }: Props = $props();
</script>

<details
  class="group/think rounded-lg border border-border/60 open:bg-muted/20 [&:not([open])]:border-transparent"
  {open}
>
  <summary
    class="flex cursor-pointer list-none items-center gap-1.5 px-2 py-1.5 text-xs text-muted-foreground marker:content-none hover:text-foreground"
  >
    <ChevronRight class="h-3.5 w-3.5 transition-transform group-open/think:rotate-90" />
    {label}
  </summary>
  {#if content}
    <div class="px-3 pb-2 text-muted-foreground/70">
      <ChatMarkdown {content} variant="assistant" density="compact" />
    </div>
  {/if}
</details>
