<script lang="ts">
  import type { Snippet } from "svelte";
  import type { TerminalConfig, TerminalProcess } from "../../types";
  import Terminal from "./Terminal.svelte";
  import CommandInputBar from "../widgets/CommandInputBar.svelte";
  import { TerminalService } from "../../services/terminalService";
  import { commandBlockStore } from "../../stores/commandBlockStore";
  import { commandHistoryStore } from "../../stores/commandHistoryStore";
  import { aiChatService } from "$lib/domains/ai/services/aiChatService";
  import { isTauriEnvironment } from "$lib/utils/tauri";

  interface Props {
    tabId: string;
    settings: TerminalConfig;
    showCommandInput?: boolean;
    initialCommand?: string;
    widgets?: Snippet<[{ tabId: string; process: TerminalProcess | null }]>;
  }

  let {
    tabId,
    settings,
    showCommandInput = true,
    initialCommand,
    widgets,
  }: Props = $props();

  let terminal = $state<ReturnType<typeof Terminal> | null>(null);
  let currentProcess = $state<TerminalProcess | null>(null);
  const isTauri = isTauriEnvironment();

  async function handleCommandSubmit(command: string, isAIMode = false) {
    if (isAIMode) {
      await handleAIQuery(command);
      return;
    }

    if (!currentProcess || !isTauri) return;

    const blockId = commandBlockStore.addBlock(tabId, {
      command,
      output: "",
      source: "pty",
      status: "running",
      processId: currentProcess.id,
    });

    TerminalService.startCommandTracking(command, tabId);
    commandHistoryStore.addEntry(tabId, {
      command,
      output: "",
      duration: 0,
    });

    const lineEnding = navigator.userAgent.includes("Windows") ? "\r\n" : "\n";

    try {
      await TerminalService.sendInput(
        currentProcess.id,
        `${command}${lineEnding}`,
        tabId,
      );
    } catch {
      commandBlockStore.completeBlock(tabId, blockId, 1);
    }
  }

  async function handleAIQuery(query: string) {
    const blockId = commandBlockStore.addBlock(tabId, {
      command: `/ai ${query}`,
      output: "",
      source: "ai",
      status: "running",
    });

    try {
      await aiChatService.streamMessage(query, [], {
        onChunk: (chunk) => {
          commandBlockStore.appendOutput(tabId, blockId, chunk);
        },
        onComplete: () => {
          commandBlockStore.completeBlock(tabId, blockId, 0);
        },
        onError: (error) => {
          commandBlockStore.appendOutput(
            tabId,
            blockId,
            `\n\nError: ${error.message}`,
          );
          commandBlockStore.completeBlock(tabId, blockId, 1);
        },
      });
    } catch (error) {
      const message =
        error instanceof Error ? error.message : "Unknown error";
      commandBlockStore.appendOutput(tabId, blockId, `\n\nError: ${message}`);
      commandBlockStore.completeBlock(tabId, blockId, 1);
    }
  }

  function handleRerun(command: string) {
    handleCommandSubmit(command, false);
  }

  function handleStop() {
    if (currentProcess) {
      TerminalService.sendInput(currentProcess.id, "\x03", tabId).catch(
        () => {},
      );
    }
  }

  export function rerunCommand(command: string) {
    handleCommandSubmit(command, false);
  }
</script>

<div class="flex h-full w-full">
  <div class="flex min-h-0 min-w-0 flex-1 flex-col">
    <Terminal
      bind:this={terminal}
      {tabId}
      {settings}
      onReady={(p) => {
        currentProcess = p;
        if (initialCommand && p) {
          handleCommandSubmit(initialCommand, false);
        }
      }}
    />

    {#if showCommandInput && isTauri}
      <CommandInputBar
        {tabId}
        onSubmit={handleCommandSubmit}
        onStop={handleStop}
      />
    {/if}
  </div>

  {#if widgets}
    <aside class="flex w-96 min-w-0 flex-col border-l border-gray-700 bg-gray-800">
      {@render widgets({ tabId, process: currentProcess })}
    </aside>
  {/if}
</div>
