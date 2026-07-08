<script lang="ts">
  import {
    Collapsible,
    CollapsibleContent,
    CollapsibleTrigger,
  } from "$lib/components/ui/collapsible";
  import { Button } from "$lib/components/ui/button";
  import Icon from "@iconify/svelte";
  import ChatMarkdown from "$lib/components/ui/chat-markdown/ChatMarkdown.svelte";

  interface Props {
    content: string;
    /** Character limit before showing expand/collapse (default: 500) */
    truncateAt?: number;
    /** Whether to start expanded (default: false) */
    defaultExpanded?: boolean;
    /** Custom class for the markdown content */
    class?: string;
  }

  let {
    content,
    truncateAt = 500,
    defaultExpanded = false,
    class: className = "",
  }: Props = $props();

  let isExpanded = $state(defaultExpanded);
  let shouldTruncate = $derived(content.length > truncateAt);

  const truncatedContent = $derived(
    shouldTruncate && !isExpanded
      ? content.substring(0, truncateAt) + "..."
      : content,
  );
</script>

<div class="space-y-2">
  {#if shouldTruncate}
    <Collapsible bind:open={isExpanded}>
      <ChatMarkdown content={truncatedContent} class={className} />
      <div class="mt-2">
        <CollapsibleTrigger>
          <Button
            variant="ghost"
            size="sm"
            class="h-auto p-0 text-muted-foreground hover:text-foreground"
          >
            {#if isExpanded}
              <Icon icon="lucide:chevron-up" class="mr-1 h-4 w-4" />
              Show less
            {:else}
              <Icon icon="lucide:chevron-down" class="mr-1 h-4 w-4" />
              Show more
            {/if}
          </Button>
        </CollapsibleTrigger>
      </div>
    </Collapsible>
  {:else}
    <ChatMarkdown content={truncatedContent} class={className} />
  {/if}
</div>
