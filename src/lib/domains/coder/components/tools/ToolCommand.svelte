<script lang="ts">
  import { formatCommandCwd } from '../../utils/toolCallDisplay.js';
  import CoderCommandOutput from '../CoderCommandOutput.svelte';
  import ToolOutput from './ToolOutput.svelte';
  import type { ToolBodyProps } from './types.js';

  let { call, args, result, failed, failedOutput, workspaceRoot, threadId }: ToolBodyProps =
    $props();

  const command = $derived(String(args.command ?? ''));
  const commandCwd = $derived(formatCommandCwd(workspaceRoot));
</script>

<div class="space-y-2">
  {#if result !== null && threadId}
    <CoderCommandOutput
      {threadId}
      callId={call.id}
      {command}
      output={failed ? failedOutput : result}
      {failed}
      {workspaceRoot}
    />
  {:else}
    {#if commandCwd}
      <div class="truncate font-mono text-xs text-muted-foreground" title={workspaceRoot}>
        {commandCwd}
      </div>
    {/if}
    <div
      class="rounded px-2 py-1 font-mono text-xs {failed
        ? 'bg-destructive/10 text-destructive'
        : 'bg-terminal text-terminal-foreground'}"
    >
      <span class={failed ? 'text-destructive/70' : 'text-terminal-foreground/50'}>$</span>
      {command}
    </div>
    {#if result !== null}
      <ToolOutput text={failed ? failedOutput : result} {failed} />
    {/if}
  {/if}
</div>
