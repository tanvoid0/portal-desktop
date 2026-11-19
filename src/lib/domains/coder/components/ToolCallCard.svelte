<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import ChevronRight from '~icons/fa6-solid/chevron-right';
  import type { CoderSubAgent, ToolCall } from '../types.js';
  import {
    formatFailedResult,
    getCompactToolLine,
    getToolResultFailureSummary,
    getToolResultStatus,
  } from '../utils/toolCallDisplay.js';
  import { normalizeTool } from '../utils/toolKinds.js';
  import { toolBodies } from './tools/index.js';

  interface Props {
    call: ToolCall;
    result?: string | null;
    workspaceRoot?: string;
    threadId?: string;
    subAgents?: CoderSubAgent[];
    coordinatorId?: string;
    onOpenSubAgent?: (childThreadId: string, coordinatorId: string) => void;
    onCancelSubAgent?: (subAgentId: string) => void;
    onCleanupSubAgent?: (subAgentId: string) => void;
    /** Hide outer padding when nested inside activity summary. */
    nested?: boolean;
  }

  let {
    call,
    result = null,
    workspaceRoot = '',
    threadId,
    subAgents = [],
    coordinatorId = '',
    onOpenSubAgent,
    onCancelSubAgent,
    onCleanupSubAgent,
    nested = false,
  }: Props = $props();

  let open = $state(false);

  const rawArgs = $derived.by(() => {
    try {
      return JSON.parse(call.function.arguments || '{}') as Record<string, unknown>;
    } catch {
      return {} as Record<string, unknown>;
    }
  });

  const tool = $derived(call.function.name);
  const normalized = $derived(normalizeTool(tool, rawArgs));
  const line = $derived(getCompactToolLine(tool, rawArgs));
  const status = $derived(getToolResultStatus(tool, result));
  const failed = $derived(status === 'failed');
  const failureSummary = $derived(
    failed && result ? getToolResultFailureSummary(tool, result) : ''
  );
  const failedOutput = $derived(failed && result ? formatFailedResult(tool, result) : '');
  // Full path/command behind the (basename-only) chip.
  const chipTitle = $derived(
    normalized.kind === 'search'
      ? line.target
      : String(normalized.args.path ?? normalized.args.command ?? line.target)
  );

  const expandable = $derived(Object.keys(rawArgs).length > 0 || result != null);
  const Body = $derived(toolBodies[normalized.kind]);
</script>

<div class={nested ? '' : 'py-0.5'}>
  <Button
    type="button"
    variant="ghost"
    class="h-7 w-full justify-start gap-1.5 px-0 py-0 text-left font-normal hover:bg-transparent {failed
      ? 'text-destructive'
      : 'text-muted-foreground hover:text-foreground'}"
    disabled={!expandable}
    onclick={() => (open = !open)}
  >
    {#if expandable}
      <ChevronRight class="h-3 w-3 shrink-0 transition-transform {open ? 'rotate-90' : ''}" />
    {:else}
      <span class="inline-block h-3 w-3 shrink-0" aria-hidden="true"></span>
    {/if}

    <span class="flex min-w-0 flex-1 items-baseline gap-1.5 text-xs leading-snug">
      <span class="shrink-0 text-muted-foreground">{line.verb}</span>
      {#if line.target}
        <span
          class="min-w-0 truncate rounded bg-muted/60 px-1 font-mono text-foreground/90"
          title={chipTitle}>{line.target}</span
        >
      {/if}
      {#if line.additions || line.deletions}
        <span class="shrink-0 tabular-nums">
          {#if line.additions}<span class="text-diff-add-foreground">+{line.additions}</span>{/if}
          {#if line.deletions}<span class="text-diff-del-foreground">−{line.deletions}</span>{/if}
        </span>
      {/if}
      {#if status === 'pending'}
        <span class="shrink-0 text-muted-foreground/70">…</span>
      {/if}
      {#if failed && failureSummary}
        <span class="min-w-0 truncate text-destructive">— {failureSummary}</span>
      {/if}
    </span>
  </Button>

  {#if open}
    <div class="mt-1 ml-4 space-y-2 border-l border-border/60 pb-1 pl-3">
      <Body
        {call}
        kind={normalized.kind}
        args={normalized.args}
        {result}
        {failed}
        {failedOutput}
        {workspaceRoot}
        {threadId}
        {subAgents}
        {coordinatorId}
        {onOpenSubAgent}
        {onCancelSubAgent}
        {onCleanupSubAgent}
      />
    </div>
  {/if}
</div>
