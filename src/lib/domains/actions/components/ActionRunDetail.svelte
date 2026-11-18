<!--
  One action run (a script_executions row): status, live output while it runs,
  and the controls — Stop, Re-run, Run in terminal, copy. Loads and polls by id,
  so it works in the project Actions tab and the global Action Runs rail alike.
-->
<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from '$lib/components/ui/card';
  import { PageLoading } from '$lib/components/shell';
  import Stop from '~icons/fa6-solid/stop';
  import Rerun from '~icons/fa6-solid/rotate-right';
  import TerminalIcon from '~icons/fa6-solid/terminal';
  import Copy from '~icons/fa6-solid/copy';
  import Robot from '~icons/fa6-solid/robot';
  import Close from '~icons/fa6-solid/xmark';
  import OpenPage from '~icons/fa6-solid/up-right-from-square';
  import EmbeddedTerminal from '$lib/domains/scripts/components/EmbeddedTerminal.svelte';
  import {
    scriptExecutionService,
    type ScriptExecutionInfo,
  } from '$lib/domains/scripts/services/scriptExecutionService';
  import {
    formatExecutionDuration,
    getExecutionStatusBadgeClass,
    getExecutionStatusBadgeVariant,
    getExecutionStatusColor,
    getExecutionStatusIcon,
  } from '$lib/domains/projects/pipelines/utils/executionDisplay';
  import { copyRunText, isActive, rerun, stopRun } from '../runActions';
  import { runRequester, toTerminalText } from '../runOutput';
  import { createWorkflowRunQuery } from '../queries/workflowRunQueries';
  import WorkflowTree from './WorkflowTree.svelte';

  interface Props {
    executionId: string;
    /** Show the interactive terminal for this command (bindable: closing it clears it). */
    terminal?: boolean;
    /** A new run started from here (Re-run) — the parent should select it. */
    onStarted?: (executionId: string) => void;
    /** A step node of the workflow tree was clicked — show that step's run. */
    onSelectExecution?: (executionId: string) => void;
    /** The run finished or was stopped — refresh any list showing it. */
    onChanged?: () => void;
    onClose?: () => void;
    /** Show the "Open page" link to /pipeline-runs/<id> (off on that page itself). */
    showPageLink?: boolean;
  }

  let {
    executionId,
    terminal = $bindable(false),
    onStarted,
    onSelectExecution,
    onChanged,
    onClose,
    showPageLink = true,
  }: Props = $props();

  let exec = $state<ScriptExecutionInfo | null>(null);
  let live = $state('');
  let loading = $state(true);
  let busy = $state(false);
  let outputEl = $state<HTMLPreElement | null>(null);
  let follow = $state(true);
  let terminalStartedAt: string | undefined;

  const runQuery = createWorkflowRunQuery(() => exec?.workflowRunId);
  const plan = $derived(exec?.workflowPlan ?? runQuery.data?.[0]?.workflowPlan ?? null);
  const stepId = $derived(
    runQuery.data?.find((r) => r.id === executionId)?.stepId ?? exec?.stepId ?? null
  );

  const active = $derived(exec ? isActive(exec) : false);
  const requester = $derived(runRequester(exec?.triggeredBy));
  const text = $derived(toTerminalText(active ? live : (exec?.output ?? '')));

  // Load the run, then poll its status + live buffer once a second while it runs.
  $effect(() => {
    const id = executionId;
    let cancelled = false;
    let timer: ReturnType<typeof setTimeout> | undefined;
    let wasActive = false;
    exec = null;
    live = '';
    loading = true;
    follow = true;

    const tick = async () => {
      try {
        const [info, out] = await Promise.all([
          scriptExecutionService.getExecution(id),
          scriptExecutionService.getLiveOutput(id),
        ]);
        if (cancelled) return;
        exec = info;
        live = out.join('');
        const nowActive = !!info && isActive(info);
        if (wasActive && !nowActive) onChanged?.();
        wasActive = nowActive;
        if (nowActive) timer = setTimeout(tick, 1000);
      } catch {
        if (!cancelled) timer = setTimeout(tick, 2000);
      } finally {
        if (!cancelled) loading = false;
      }
    };
    void tick();
    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  });

  // Stick to the bottom as output arrives, unless the user scrolled up.
  $effect(() => {
    void text;
    if (follow && outputEl) outputEl.scrollTop = outputEl.scrollHeight;
  });

  function onScroll() {
    if (!outputEl) return;
    follow = outputEl.scrollTop + outputEl.clientHeight >= outputEl.scrollHeight - 16;
  }

  async function stop() {
    if (!exec || busy) return;
    busy = true;
    // The poll is still running (the run is active) and picks up `cancelled`.
    await stopRun(exec);
    busy = false;
  }

  async function runAgain() {
    if (!exec || busy) return;
    busy = true;
    const id = await rerun(exec);
    busy = false;
    if (id) onStarted?.(id);
  }

  function openTerminal() {
    terminalStartedAt = new Date().toISOString();
    terminal = true;
  }

  async function terminalDone(exitCode: number | null) {
    if (!exec) return;
    try {
      await scriptExecutionService.recordInteractive({
        command: exec.command,
        workingDirectory: exec.workingDirectory ?? undefined,
        exitCode,
        startedAt: terminalStartedAt,
      });
      onChanged?.();
    } catch {
      // history is best-effort
    }
  }
</script>

{#if loading && !exec}
  <PageLoading message="Loading run…" />
{:else if !exec}
  <Card>
    <CardContent class="py-6 text-sm text-muted-foreground">
      Run not found — it may have been deleted.
    </CardContent>
  </Card>
{:else}
  {@const StatusIcon = getExecutionStatusIcon(exec.status)}
  <Card>
    <CardHeader class="space-y-3">
      <div class="flex items-start justify-between gap-2">
        <div class="min-w-0">
          <CardTitle class="text-base">Run details</CardTitle>
          <CardDescription class="break-all">
            {exec.workingDirectory ?? '—'}
          </CardDescription>
        </div>
        <div class="flex shrink-0 items-center gap-1">
          {#if requester}
            <Badge variant="outline" class="gap-1" title="Requested over MCP">
              <Robot class="h-3 w-3" />
              {requester}
            </Badge>
          {/if}
          <Badge
            variant={getExecutionStatusBadgeVariant(exec.status)}
            class="gap-1 {getExecutionStatusBadgeClass(exec.status)}"
          >
            <StatusIcon
              class="h-3.5 w-3.5 {getExecutionStatusColor(exec.status)}{active
                ? ' animate-spin'
                : ''}"
            />
            {exec.status}
            {#if exec.exitCode != null}· exit {exec.exitCode}{/if}
          </Badge>
          {#if showPageLink}
            <Button
              variant="ghost"
              size="sm"
              class="h-7 px-2"
              title="Open this run on its own page"
              href="/pipeline-runs/{exec.id}"
            >
              <OpenPage class="h-3.5 w-3.5" />
            </Button>
          {/if}
          {#if onClose}
            <Button variant="ghost" size="sm" class="h-7 px-2" title="Close" onclick={onClose}>
              <Close class="h-3.5 w-3.5" />
            </Button>
          {/if}
        </div>
      </div>
      <div class="flex flex-wrap gap-2">
        {#if active}
          <Button
            size="sm"
            variant="destructive"
            class="gap-1"
            disabled={busy}
            onclick={() => void stop()}
          >
            <Stop class="h-3.5 w-3.5" />
            Stop
          </Button>
        {:else}
          <Button size="sm" class="gap-1" disabled={busy} onclick={() => void runAgain()}>
            <Rerun class="h-3.5 w-3.5" />
            Re-run
          </Button>
        {/if}
        <Button
          size="sm"
          variant="outline"
          class="gap-1"
          disabled={terminal}
          title="Run this command in an interactive terminal (real TTY: colours, prompts, Ctrl+C)"
          onclick={openTerminal}
        >
          <TerminalIcon class="h-3.5 w-3.5" />
          Run in terminal
        </Button>
        <Button
          size="sm"
          variant="ghost"
          class="gap-1"
          onclick={() => exec && void copyRunText('command', exec)}
        >
          <Copy class="h-3.5 w-3.5" />
          Command
        </Button>
        <Button
          size="sm"
          variant="ghost"
          class="gap-1"
          onclick={() => exec && void copyRunText('output', exec)}
        >
          <Copy class="h-3.5 w-3.5" />
          Output
        </Button>
      </div>
    </CardHeader>
    <CardContent class="space-y-3">
      <code class="block rounded bg-muted px-2 py-1.5 text-xs break-all whitespace-pre-wrap">
        {exec.command}
      </code>
      <div class="flex flex-wrap gap-x-4 gap-y-1 text-xs text-muted-foreground">
        <span>Started {new Date(exec.startedAt).toLocaleString()}</span>
        <span>Duration: {formatExecutionDuration(exec.startedAt, exec.finishedAt)}</span>
      </div>

      {#if exec.workflowRunId && plan?.length}
        <WorkflowTree
          {plan}
          executions={runQuery.data ?? []}
          selectedStepId={stepId}
          onSelect={(_, id) => id && id !== executionId && onSelectExecution?.(id)}
        />
      {/if}

      {#if terminal}
        <div class="space-y-2">
          <div class="flex items-center justify-between gap-2">
            <p class="text-xs font-medium text-muted-foreground">Interactive terminal</p>
            <Button size="sm" variant="ghost" onclick={() => (terminal = false)}>Close</Button>
          </div>
          <EmbeddedTerminal
            command={exec.command}
            workingDirectory={exec.workingDirectory ?? undefined}
            onComplete={(code) => void terminalDone(code)}
          />
        </div>
      {/if}

      {#if exec.error}
        <pre
          class="max-h-40 overflow-auto rounded border border-destructive/30 bg-destructive/5 px-2 py-1.5 text-xs whitespace-pre-wrap text-destructive">{exec.error}</pre>
      {/if}

      {#if text.trim()}
        <div>
          <p class="mb-1 text-xs font-medium text-muted-foreground">
            Output{active ? ' (live)' : ''}
          </p>
          <pre
            bind:this={outputEl}
            onscroll={onScroll}
            class="max-h-[60vh] overflow-auto rounded bg-muted px-2 py-1.5 font-mono text-xs whitespace-pre-wrap">{text}</pre>
        </div>
      {:else if active}
        <p class="text-xs text-muted-foreground">Running… no output yet.</p>
      {:else if !exec.error}
        <p class="text-xs text-muted-foreground">No output captured.</p>
      {/if}
    </CardContent>
  </Card>
{/if}
