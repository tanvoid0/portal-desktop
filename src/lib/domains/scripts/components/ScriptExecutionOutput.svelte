<!-- Non-interactive run result: status header, console output, error and exit code. -->
<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import Copy from '~icons/fa6-solid/copy';
  import Terminal from '~icons/fa6-solid/terminal';
  import TerminalOutputBox from '$lib/components/shared/TerminalOutputBox.svelte';
  import { toast } from '$lib/utils/toast';
  import type { ScriptExecutionInfo } from '../services';
  import { getStatusIcon, getStatusColor, formatDuration } from '../utils/executionStatus';

  interface Props {
    execution: ScriptExecutionInfo;
  }

  let { execution }: Props = $props();

  const StatusIcon = $derived(getStatusIcon(execution.status));

  function handleCopyOutput() {
    if (!execution.output) return;
    navigator.clipboard.writeText(execution.output);
    toast.success('Output copied to clipboard');
  }
</script>

<div class="divider-edge-t divider-edge-full space-y-3 pt-4">
  <div class="flex items-center justify-between">
    <div class="flex items-center gap-2">
      <Terminal class="h-4 w-4" />
      <span class="font-medium">Output</span>
      {#if execution.output}
        <Button variant="ghost" size="sm" onclick={handleCopyOutput} class="h-6 px-2">
          <Copy class="mr-1 h-3 w-3" />
          Copy
        </Button>
      {/if}
    </div>
    <div class="flex items-center gap-2">
      <StatusIcon class="h-4 w-4 {getStatusColor(execution.status)}" />
      <Badge
        variant={execution.status === 'success'
          ? 'default'
          : execution.status === 'failed'
            ? 'destructive'
            : 'secondary'}
      >
        {execution.status}
      </Badge>
      {#if execution.finishedAt}
        <span class="text-xs text-muted-foreground">
          {formatDuration(execution.startedAt, execution.finishedAt)}
        </span>
      {/if}
    </div>
  </div>
  <TerminalOutputBox class="max-h-80 rounded-lg">
    {#if execution.output}
      <pre class="whitespace-pre-wrap">{execution.output}</pre>
    {:else if ['pending', 'queued', 'running'].includes(execution.status)}
      <span class="animate-pulse">Waiting for output...</span>
    {:else}
      <span class="text-gray-500">No output</span>
    {/if}
  </TerminalOutputBox>
  {#if execution.error}
    <div class="rounded-lg border border-red-500/30 bg-red-500/10 p-4">
      <p class="text-sm font-medium text-red-500">Error</p>
      <p class="mt-1 text-sm text-red-400">
        {execution.error}
      </p>
    </div>
  {/if}
  {#if execution.exitCode !== null}
    <p class="text-sm text-muted-foreground">
      Exit code: <code class="rounded bg-muted px-1">{execution.exitCode}</code>
    </p>
  {/if}
</div>
