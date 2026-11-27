<!--
  ChatThinkingBlock — collapsed reasoning disclosure shared by the chat page
  (inline <think> blocks) and the coder feed (separate thought messages).
-->
<script lang="ts">
  import * as Collapsible from '$lib/components/ui/collapsible';
  import ChevronRight from '~icons/fa6-solid/chevron-right';
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

<Collapsible.Root
  {open}
  class="rounded-lg border border-border/60 data-[state=closed]:border-transparent data-[state=open]:bg-muted/20"
>
  <Collapsible.Trigger
    class="group/think flex w-full cursor-pointer items-center gap-1.5 px-2 py-1.5 text-xs text-muted-foreground hover:text-foreground"
  >
    <ChevronRight
      class="h-3.5 w-3.5 transition-transform group-data-[state=open]/think:rotate-90"
    />
    {label}
  </Collapsible.Trigger>
  {#if content}
    <Collapsible.Content class="px-3 pb-2 text-muted-foreground/70">
      <ChatMarkdown {content} variant="assistant" density="compact" />
    </Collapsible.Content>
  {/if}
</Collapsible.Root>
