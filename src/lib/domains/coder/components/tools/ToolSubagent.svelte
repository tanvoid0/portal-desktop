<script lang="ts">
  import CoderSubAgentInline from '../CoderSubAgentInline.svelte';
  import ToolOutput from './ToolOutput.svelte';
  import type { ToolBodyProps } from './types.js';

  let {
    call,
    args,
    result,
    failed,
    failedOutput,
    subAgents,
    coordinatorId,
    onOpenSubAgent,
    onCancelSubAgent,
    onCleanupSubAgent,
  }: ToolBodyProps = $props();

  const spawn = $derived(call.function.name === 'spawn_parallel_tasks');
  const prompt = $derived(spawn ? '' : String(args.prompt || args.description || ''));
</script>

<div class="space-y-2">
  {#if spawn && subAgents.length > 0 && !failed}
    <CoderSubAgentInline
      {subAgents}
      {coordinatorId}
      onOpen={onOpenSubAgent}
      onCancel={onCancelSubAgent}
      onCleanup={onCleanupSubAgent}
      compact={true}
    />
  {:else}
    {#if prompt}
      <p class="text-xs whitespace-pre-wrap text-muted-foreground">{prompt}</p>
    {/if}
    {#if result !== null}
      <ToolOutput text={failed ? failedOutput : result} {failed} />
    {:else if spawn}
      <p class="text-xs text-muted-foreground">Spawning parallel sub-agents…</p>
    {/if}
  {/if}
</div>
