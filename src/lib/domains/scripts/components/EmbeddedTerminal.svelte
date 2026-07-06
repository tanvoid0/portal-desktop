<script lang="ts">
  import Terminal from "$lib/domains/terminal/components/core/Terminal.svelte";
  import { defaultTerminalConfig } from "$lib/domains/terminal/config/defaultTerminalConfig";
  import type { TerminalConfig } from "$lib/domains/terminal/types";
  import { Copy, Square, Maximize2, Minimize2 } from "@lucide/svelte";
  import { Button } from "$lib/components/ui/button";
  import { toast } from "$lib/utils/toast";

  interface Props {
    command: string;
    workingDirectory?: string;
    onComplete?: (exitCode: number | null) => void;
    onStart?: (processId: string) => void;
  }

  let { command, workingDirectory, onComplete, onStart }: Props = $props();

  let terminal = $state<ReturnType<typeof Terminal> | null>(null);
  let isExpanded = $state(false);
  let isConnected = $state(true);
  let outputBuffer = $state("");

  const terminalId = `embedded-terminal-${Date.now()}`;

  const settings = $derived<TerminalConfig>({
    ...defaultTerminalConfig,
    workingDirectory: workingDirectory || defaultTerminalConfig.workingDirectory,
    scrollbackLines: 5000,
    fontSize: 13,
  });

  function handleStop() {
    terminal?.getSession()?.kill();
    isConnected = false;
    onComplete?.(null);
  }

  function handleCopy() {
    if (outputBuffer) {
      navigator.clipboard.writeText(outputBuffer);
      toast.success("Output copied to clipboard");
    }
  }

  function toggleExpand() {
    isExpanded = !isExpanded;
    setTimeout(() => terminal?.fit(), 150);
  }
</script>

<div
  class="embedded-terminal overflow-hidden rounded-lg border border-gray-700"
  class:expanded={isExpanded}
>
  <div
    class="flex items-center justify-between border-b border-gray-700 bg-gray-800 px-3 py-2"
  >
    <div class="flex items-center gap-2">
      <div
        class="h-2.5 w-2.5 rounded-full"
        class:bg-green-500={isConnected}
        class:bg-yellow-500={!isConnected}
      ></div>
      <span class="text-xs text-gray-400">
        {isConnected ? "Running" : "Stopped"}
      </span>
    </div>
    <div class="flex items-center gap-1">
      <Button
        variant="ghost"
        size="sm"
        onclick={handleCopy}
        class="h-7 px-2"
        title="Copy output"
      >
        <Copy class="h-3.5 w-3.5" />
      </Button>
      <Button
        variant="ghost"
        size="sm"
        onclick={toggleExpand}
        class="h-7 px-2"
        title={isExpanded ? "Collapse" : "Expand"}
      >
        {#if isExpanded}
          <Minimize2 class="h-3.5 w-3.5" />
        {:else}
          <Maximize2 class="h-3.5 w-3.5" />
        {/if}
      </Button>
      {#if isConnected}
        <Button
          variant="ghost"
          size="sm"
          onclick={handleStop}
          class="h-7 px-2 text-red-400 hover:text-red-300"
          title="Stop process"
        >
          <Square class="h-3.5 w-3.5" />
        </Button>
      {/if}
    </div>
  </div>

  <div class="terminal-content" class:h-64={!isExpanded} class:h-96={isExpanded}>
    <Terminal
      bind:this={terminal}
      tabId={terminalId}
      {settings}
      mode="oneshot"
      {command}
      onReady={(p) => {
        if (p) onStart?.(p.id);
      }}
      onData={(chunk) => {
        outputBuffer += chunk;
      }}
      onExit={(code) => {
        isConnected = false;
        onComplete?.(code);
      }}
    />
  </div>
</div>

<style>
  .embedded-terminal {
    background: #0c0c0c;
  }

  .terminal-content {
    width: 100%;
    height: 100%;
    overflow: hidden;
  }

  .terminal-content :global(.xterm) {
    height: 100% !important;
    width: 100% !important;
  }

  .h-64 {
    height: 16rem;
  }

  .h-96 {
    height: 24rem;
  }
</style>
