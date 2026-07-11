<!--
  CommandBlock — Warp-style block: command header with status, cwd, duration,
  hover actions (copy / rerun / AI explain), live-streaming output body.
  Used by BlocksView (main surface) and CommandBlocksPanel (side rail).
-->
<script lang="ts">
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import {
    ChevronDown,
    ChevronRight,
    Check,
    Copy,
    Clock,
    Folder,
    Loader,
    RotateCw,
    Sparkles,
    CheckCircle,
    XCircle,
  } from "@lucide/svelte";
  import type { CapturedCommand } from "../stores/commandBlockStore";
  import { stripForDisplay } from "../utils/textUtils";
  import AiResponse from "./ai/AiResponse.svelte";
  import ErrorOutput from "./ErrorOutput.svelte";

  interface Props {
    block: CapturedCommand;
    onRerun?: (command: string) => void;
    onExplain?: (block: CapturedCommand) => void;
    onclick?: (() => void) | undefined;
  }

  let { block, onRerun, onExplain, onclick = undefined }: Props = $props();

  let isExpanded = $state(block.isExpanded ?? true);
  let copied = $state<"command" | "output" | null>(null);
  let outputEl = $state<HTMLPreElement | null>(null);

  const isRunning = $derived(block.status === "running");
  const isFailed = $derived(
    block.status === "failed" ||
      (block.exitCode !== undefined && block.exitCode !== 0),
  );
  const isAI = $derived(block.source === "ai");
  const displayOutput = $derived(
    isAI ? block.output : stripForDisplay(block.output),
  );

  // Auto-follow live output while running.
  $effect(() => {
    void displayOutput;
    if (isRunning && outputEl) {
      outputEl.scrollTop = outputEl.scrollHeight;
    }
  });

  function formatDuration(ms: number): string {
    if (ms < 1000) return `${ms}ms`;
    if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
    return `${(ms / 60000).toFixed(1)}m`;
  }

  async function copy(text: string, what: "command" | "output") {
    try {
      await navigator.clipboard.writeText(text);
      copied = what;
      setTimeout(() => (copied = null), 1500);
    } catch {
      // clipboard unavailable — ignore
    }
  }
</script>

<div
  class="command-block group rounded-lg border bg-card {isFailed
    ? 'border-status-error/50'
    : isRunning
      ? 'border-status-info/50'
      : 'border-border'} {onclick ? 'cursor-pointer' : ''}"
  onclick={() => onclick && onclick()}
  role={onclick ? "button" : undefined}
  tabindex={onclick ? 0 : undefined}
  onkeydown={(e) => {
    if (onclick && (e.key === "Enter" || e.key === " ")) {
      e.preventDefault();
      onclick();
    }
  }}
>
  <!-- Header -->
  <div class="flex items-center gap-2 px-3 py-2">
    <div class="flex-shrink-0">
      {#if isRunning}
        <Loader class="h-3.5 w-3.5 animate-spin text-status-info" />
      {:else if isAI}
        <Sparkles class="h-3.5 w-3.5 text-primary" />
      {:else if isFailed}
        <XCircle class="h-3.5 w-3.5 text-status-error" />
      {:else}
        <CheckCircle class="h-3.5 w-3.5 text-status-success" />
      {/if}
    </div>

    <code class="min-w-0 flex-1 truncate font-mono text-sm text-foreground" title={block.command}>
      {block.command || "(command running…)"}
    </code>

    <!-- Metadata -->
    <div class="flex flex-shrink-0 items-center gap-1.5 text-xs text-muted-foreground">
      {#if block.workingDirectory}
        <span class="hidden items-center gap-1 md:flex" title={block.workingDirectory}>
          <Folder class="h-3 w-3" />
          <span class="max-w-40 truncate">{block.workingDirectory}</span>
        </span>
      {/if}
      {#if block.duration !== undefined}
        <Badge variant="outline" class="h-5 gap-1 px-1.5 text-[10px]">
          <Clock class="h-2.5 w-2.5" />
          {formatDuration(block.duration)}
        </Badge>
      {/if}
      {#if block.exitCode !== undefined && block.exitCode !== 0}
        <Badge variant="destructive" class="h-5 px-1.5 text-[10px]">
          exit {block.exitCode}
        </Badge>
      {/if}
    </div>

    <!-- Hover actions -->
    <div
      class="flex flex-shrink-0 items-center gap-0.5 opacity-0 transition-opacity group-hover:opacity-100"
    >
      <Button
        variant="ghost"
        size="sm"
        class="h-6 w-6 p-0"
        title="Copy command"
        onclick={(e: MouseEvent) => {
          e.stopPropagation();
          copy(block.command, "command");
        }}
      >
        {#if copied === "command"}<Check class="h-3 w-3 text-status-success" />{:else}<Copy class="h-3 w-3" />{/if}
      </Button>
      {#if onRerun && !isAI}
        <Button
          variant="ghost"
          size="sm"
          class="h-6 w-6 p-0"
          title="Run again"
          onclick={(e: MouseEvent) => {
            e.stopPropagation();
            onRerun(block.command);
          }}
        >
          <RotateCw class="h-3 w-3" />
        </Button>
      {/if}
      {#if onExplain && isFailed && !isAI}
        <Button
          variant="ghost"
          size="sm"
          class="h-6 gap-1 px-1.5 text-[10px] text-primary"
          title="Ask AI to explain this error"
          onclick={(e: MouseEvent) => {
            e.stopPropagation();
            onExplain(block);
          }}
        >
          <Sparkles class="h-3 w-3" />
          Explain
        </Button>
      {/if}
      <Button
        variant="ghost"
        size="sm"
        class="h-6 w-6 p-0"
        title={isExpanded ? "Collapse output" : "Expand output"}
        onclick={(e: MouseEvent) => {
          e.stopPropagation();
          isExpanded = !isExpanded;
        }}
      >
        {#if isExpanded}<ChevronDown class="h-3 w-3" />{:else}<ChevronRight class="h-3 w-3" />{/if}
      </Button>
    </div>
  </div>

  <!-- Output -->
  {#if isExpanded && isAI && displayOutput}
    <div class="border-t border-border/60 px-3 py-2">
      <AiResponse content={displayOutput} onRunCommand={onRerun} />
    </div>
  {:else if isExpanded && (displayOutput || isRunning)}
    <div class="relative border-t border-border/60">
      {#if isFailed && displayOutput && !isRunning}
        <ErrorOutput
          output={displayOutput}
          command={block.command}
          exitCode={block.exitCode}
          onAskAI={onExplain ? () => onExplain(block) : undefined}
        />
      {:else}
        <pre
          bind:this={outputEl}
          class="max-h-96 overflow-auto whitespace-pre-wrap break-words bg-muted/20 px-3 py-2 font-mono text-xs leading-relaxed text-foreground/90"
        >{displayOutput || " "}</pre>
        {#if displayOutput}
          <Button
            variant="ghost"
            size="sm"
            class="absolute right-1.5 top-1.5 h-6 w-6 bg-card/80 p-0 opacity-0 transition-opacity group-hover:opacity-100"
            title="Copy output"
            onclick={(e: MouseEvent) => {
              e.stopPropagation();
              copy(displayOutput, "output");
            }}
          >
            {#if copied === "output"}<Check class="h-3 w-3 text-status-success" />{:else}<Copy class="h-3 w-3" />{/if}
          </Button>
        {/if}
      {/if}
    </div>
  {/if}
</div>
