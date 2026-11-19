<script lang="ts">
  import ToolOutput from './ToolOutput.svelte';
  import type { ToolBodyProps } from './types.js';

  let { args, result, failed, failedOutput }: ToolBodyProps = $props();

  const target = $derived(String(args.url || args.query || ''));
</script>

<div class="space-y-2">
  {#if target}
    <p class="truncate font-mono text-xs text-muted-foreground" title={target}>{target}</p>
  {/if}
  {#if result !== null}
    <ToolOutput text={failed ? failedOutput : result} {failed} />
  {/if}
</div>
