<!-- Text truncated to `limit` characters with a Show more / Show less toggle -->
<script lang="ts">
  import { Button } from '$lib/components/ui/button';

  interface Props {
    text: string;
    limit?: number;
  }
  let { text, limit = 200 }: Props = $props();

  let expanded = $state(false);
  const linkClass =
    'ml-2 h-auto p-0 text-xs text-blue-600 underline hover:text-blue-700 dark:text-blue-400 dark:hover:text-blue-300';
</script>

{#if text.length > limit && !expanded}
  <div>
    <span>{text.slice(0, limit)}...</span>
    <Button variant="link" size="sm" onclick={() => (expanded = true)} class={linkClass}>
      Show more
    </Button>
  </div>
{:else}
  <div>
    {text}
    {#if text.length > limit && expanded}
      <Button variant="link" size="sm" onclick={() => (expanded = false)} class={linkClass}>
        Show less
      </Button>
    {/if}
  </div>
{/if}
