<script lang="ts">
  import { ChevronRight } from "@lucide/svelte";
  import { Button } from "$lib/components/ui/button";
  import ChatMarkdown from "$lib/components/ui/chat-markdown/ChatMarkdown.svelte";
  import type { ChatMessage } from "../types.js";
  import { formatThoughtDuration } from "../utils/feedBlocks.js";

  interface Props {
    message: ChatMessage;
    durationMs?: number | null;
  }

  let { message, durationMs = null }: Props = $props();

  let open = $state(false);

  const label = $derived(formatThoughtDuration(durationMs));
</script>

<div class="py-0.5">
  <Button
    type="button"
    variant="ghost"
    class="h-auto gap-1.5 px-0 py-0.5 text-[11px] font-normal text-muted-foreground hover:bg-transparent hover:text-foreground"
    onclick={() => (open = !open)}
  >
    <ChevronRight
      class="h-3 w-3 shrink-0 transition-transform {open ? 'rotate-90' : ''}"
    />
    <span>{label}</span>
  </Button>

  {#if open && message.content}
    <div class="ml-4 mt-1 border-l border-border/60 pl-3">
      <ChatMarkdown
        content={message.content}
        variant="assistant"
        density="compact"
        class="text-muted-foreground"
      />
    </div>
  {/if}
</div>
