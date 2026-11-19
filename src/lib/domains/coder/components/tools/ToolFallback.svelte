<script lang="ts">
  import ToolOutput from './ToolOutput.svelte';
  import type { ToolBodyProps } from './types.js';

  let { args, result, failed, failedOutput }: ToolBodyProps = $props();

  // Objects are pretty-printed; scalars stay on one line.
  const entries = $derived(
    Object.entries(args)
      .filter(([, v]) => v !== undefined && v !== null && v !== '')
      .map(([k, v]) => [k, typeof v === 'object' ? JSON.stringify(v, null, 2) : String(v)] as const)
  );
</script>

<div class="space-y-2">
  <dl class="space-y-1 text-xs">
    {#each entries as [key, value] (key)}
      <div class="flex gap-2">
        <dt class="shrink-0 text-muted-foreground">{key}</dt>
        <dd class="min-w-0 font-mono break-all whitespace-pre-wrap">{value}</dd>
      </div>
    {/each}
  </dl>
  {#if result !== null}
    <ToolOutput text={failed ? failedOutput : result} {failed} />
  {/if}
</div>
