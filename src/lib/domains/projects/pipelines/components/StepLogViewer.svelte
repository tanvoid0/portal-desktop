<!--
	Step Log Viewer - Monospace log panel with live streaming
-->
<script lang="ts">
  import { tick } from "svelte";
  import { Copy, Check } from "@lucide/svelte";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import { toast } from "$lib/utils/toast";
  import type { StepExecution } from "../types";

  interface Props {
    step: StepExecution | null;
    liveLines?: string[];
    autoScroll?: boolean;
  }

  let {
    step,
    liveLines = [],
    autoScroll = true,
  }: Props = $props();

  let scrollContainer = $state<HTMLElement | null>(null);
  let userAutoScroll = $state(true);
  let copied = $state(false);

  const displayLines = $derived.by(() => {
    if (liveLines.length > 0) {
      return liveLines;
    }
    if (step?.logs?.length) {
      return step.logs;
    }
    if (step?.output) {
      return step.output.split("\n").filter((l) => l.length > 0);
    }
    return [];
  });

  const lineCount = $derived(displayLines.length);

  $effect(() => {
    if (autoScroll && userAutoScroll && displayLines.length > 0) {
      tick().then(() => {
        if (scrollContainer) {
          scrollContainer.scrollTop = scrollContainer.scrollHeight;
        }
      });
    }
  });

  function isStderrLine(line: string): boolean {
    return line.startsWith("[stderr]");
  }

  function formatDuration(ms: number | undefined): string {
    if (!ms) return "";
    if (ms < 1000) return `${ms}ms`;
    return `${(ms / 1000).toFixed(1)}s`;
  }

  async function copyLogs() {
    if (displayLines.length === 0) return;

    try {
      await navigator.clipboard.writeText(displayLines.join("\n"));
      copied = true;
      toast.success("Logs copied to clipboard");
      setTimeout(() => {
        copied = false;
      }, 2000);
    } catch {
      toast.error("Failed to copy logs");
    }
  }
</script>

<div class="flex h-full min-h-[400px] flex-col rounded-lg border bg-zinc-950">
  <div
    class="flex items-center justify-between border-b border-zinc-800 px-4 py-2"
  >
    <div class="flex items-center gap-2">
      <span class="text-sm font-medium text-zinc-100">
        {step?.stepName ?? "Select a step"}
      </span>
      {#if step}
        <Badge variant="outline" class="border-zinc-700 text-zinc-300">
          {step.status}
        </Badge>
        {#if step.exitCode !== undefined && step.exitCode !== null}
          <Badge variant="outline" class="border-zinc-700 text-zinc-400">
            exit {step.exitCode}
          </Badge>
        {/if}
      {/if}
    </div>
    <div class="flex items-center gap-2 text-xs text-zinc-500">
      {#if step?.duration}
        <span>{formatDuration(step.duration)}</span>
      {/if}
      <span>{lineCount} lines</span>
      {#if displayLines.length > 0}
        <Button
          variant="ghost"
          size="sm"
          class="h-7 text-zinc-400 hover:text-zinc-100"
          onclick={copyLogs}
          title="Copy logs"
        >
          {#if copied}
            <Check class="mr-1 h-3.5 w-3.5" />
            Copied
          {:else}
            <Copy class="mr-1 h-3.5 w-3.5" />
            Copy
          {/if}
        </Button>
      {/if}
      <Button
        variant="ghost"
        size="sm"
        class="h-7 text-zinc-400 hover:text-zinc-100"
        onclick={() => (userAutoScroll = !userAutoScroll)}
      >
        {userAutoScroll ? "Auto-scroll on" : "Auto-scroll off"}
      </Button>
    </div>
  </div>

  <ScrollArea class="flex-1">
    <div
      bind:this={scrollContainer}
      class="max-h-[calc(100vh-280px)] min-h-[360px] overflow-y-auto p-4 font-mono text-xs leading-relaxed"
    >
      {#if !step}
        <p class="text-zinc-500">Select a step to view logs</p>
      {:else if displayLines.length === 0}
        <p class="text-zinc-500">
          {step.status === "running"
            ? "Waiting for output..."
            : "No log output for this step"}
        </p>
      {:else}
        {#each displayLines as line, i (i)}
          <div
            class="whitespace-pre-wrap break-all {isStderrLine(line)
              ? 'text-red-400'
              : 'text-zinc-300'}"
          >
            {line}
          </div>
        {/each}
      {/if}
    </div>
  </ScrollArea>
</div>
