<!-- Tail of an agent run row: approval hint, platform launcher when down, link to the Coder thread. -->
<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { aiPlatform } from '$lib/domains/ai/state/aiPlatform.svelte.js';

  interface Props {
    waiting?: boolean;
    unavailable?: boolean;
    threadId?: string;
    /** Link text while the agent is still working vs. once it is done. */
    busy: boolean;
    onOpenThread: (id: string) => void;
  }

  let { waiting, unavailable, threadId, busy, onOpenThread }: Props = $props();
</script>

{#if waiting}
  <span class="text-xs text-amber-600 dark:text-amber-400">
    Waiting for your approval in Coder
  </span>
{/if}
{#if unavailable && aiPlatform.status === 'unreachable'}
  <Button
    type="button"
    size="sm"
    variant="outline"
    disabled={aiPlatform.starting}
    onclick={() => void aiPlatform.start()}
  >
    {aiPlatform.starting ? 'Launching…' : 'Launch Agent Platform'}
  </Button>
{/if}
{#if threadId}
  <Button type="button" size="sm" variant="link" onclick={() => onOpenThread(threadId)}>
    {busy ? 'Open in Coder' : 'Review changes in Coder'}
  </Button>
{/if}
