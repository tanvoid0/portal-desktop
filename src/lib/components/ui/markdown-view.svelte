<script lang="ts">
  import FaChevronUp from '~icons/fa6-solid/chevron-up';
  import FaChevronDown from '~icons/fa6-solid/chevron-down';
  import {
    Collapsible,
    CollapsibleContent,
    CollapsibleTrigger,
  } from '$lib/components/ui/collapsible';
  import { Button } from '$lib/components/ui/button';
  import ChatMarkdown from '$lib/components/ui/chat-markdown/ChatMarkdown.svelte';

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
    class: className = '',
  }: Props = $props();

  let isExpanded = $state(defaultExpanded);
  let shouldTruncate = $derived(content.length > truncateAt);

  const truncatedContent = $derived(
    shouldTruncate && !isExpanded ? content.substring(0, truncateAt) + '...' : content
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
              <FaChevronUp class="mr-1 h-4 w-4" />
              Show less
            {:else}
              <FaChevronDown class="mr-1 h-4 w-4" />
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
