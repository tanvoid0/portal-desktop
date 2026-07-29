<!--
  ChatUserBubble — the one user turn treatment used by every AI surface
  (chat, coder feed, terminal assistant). Alignment is the caller's job.
-->
<script lang="ts">
  import type { Snippet } from 'svelte';
  import { cn } from '$lib/utils.js';

  interface Props {
    content?: string | null;
    /** `compact` matches the denser agent feeds. */
    density?: 'default' | 'compact';
    class?: string;
    /** Replaces the text body — used by the coder feed's inline editor. */
    children?: Snippet;
  }

  let { content = '', density = 'default', class: className = '', children }: Props = $props();
</script>

<div
  class={cn(
    'max-w-[85%] rounded-2xl bg-muted px-3.5 py-2 leading-relaxed text-foreground',
    density === 'compact' ? 'text-xs' : 'text-sm',
    className
  )}
>
  {#if children}
    {@render children()}
  {:else}
    <p class="whitespace-pre-wrap">{content}</p>
  {/if}
</div>
