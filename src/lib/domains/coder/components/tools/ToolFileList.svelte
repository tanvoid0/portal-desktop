<script lang="ts">
  import { resultLines } from '../../utils/toolCallDisplay.js';
  import ToolOutput from './ToolOutput.svelte';
  import type { ToolBodyProps } from './types.js';

  // List entries and search hits: one mono row per result line.
  let { kind, result, failed, failedOutput }: ToolBodyProps = $props();

  const lines = $derived(result ? resultLines(result) : []);
</script>

{#if failed}
  <ToolOutput text={failedOutput} failed />
{:else if result !== null}
  <ul
    class="max-h-48 overflow-auto text-xs {kind === 'list'
      ? 'grid grid-cols-2 gap-x-2 gap-y-0.5 sm:grid-cols-3'
      : 'space-y-0.5'}"
  >
    {#each lines as line, i (i)}
      <li class="truncate font-mono text-muted-foreground" title={line}>{line}</li>
    {/each}
  </ul>
{/if}
