<script lang="ts">
  import type { Snippet } from "svelte";
  import type { TerminalConfig, TerminalProcess } from "../../types";
  import Terminal from "./Terminal.svelte";
  import BlocksView from "./BlocksView.svelte";
  import CommandInputBar from "../widgets/CommandInputBar.svelte";
  import { TerminalService } from "../../services/terminalService";
  import {
    commandBlockStore,
    type CapturedCommand,
  } from "../../stores/commandBlockStore";
  import { commandHistoryStore } from "../../stores/commandHistoryStore";
  import { aiChatService } from "$lib/domains/ai/services/aiChatService";
  import {
    buildTerminalContext,
    buildExplainPrompt,
  } from "../../services/terminalAiContext";
  import { isTauriEnvironment } from "$lib/utils/tauri";

  export type SessionView = "blocks" | "terminal";

  interface Props {
    tabId: string;
    settings: TerminalConfig;
    showCommandInput?: boolean;
    initialCommand?: string;
    /** Initial view. Defaults to Warp-style blocks when the shell supports
     *  OSC 133 hooks (powershell/pwsh/zsh/bash), raw terminal otherwise. */
    defaultView?: SessionView;
    widgets?: Snippet<[{ tabId: string; process: TerminalProcess | null }]>;
  }

  let {
    tabId,
    settings,
    showCommandInput = true,
    initialCommand,
    defaultView,
    widgets,
  }: Props = $props();

  const INTEGRATION_SHELLS = /powershell|pwsh|zsh|bash/i;
  const shellSupportsBlocks = INTEGRATION_SHELLS.test(settings.defaultShell);

  let terminal = $state<ReturnType<typeof Terminal> | null>(null);
  let currentProcess = $state<TerminalProcess | null>(null);
  let view = $state<SessionView>(
    defaultView ?? (shellSupportsBlocks ? "blocks" : "terminal"),
  );
  const isTauri = isTauriEnvironment();

  async function handleCommandSubmit(command: string, isAIMode = false) {
    if (isAIMode) {
      await handleAIQuery(command);
      return;
    }

    if (!currentProcess || !isTauri) return;

    // With shell integration (OSC 133) active, blocks are created/completed
    // by the shell hooks — creating one here would duplicate it. Manual block
    // only for shells without hooks (e.g. cmd.exe).
    const hasIntegration = commandBlockStore.hasIntegration(currentProcess.id);
    const blockId = hasIntegration
      ? null
      : commandBlockStore.addBlock(tabId, {
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

    // Multi-line commands (AI-suggested scripts) via bracketed paste, so the
    // shell (PSReadLine, zsh, …) takes the block as one unit instead of
    // stalling at continuation prompts line by line.
    const lineEnding = navigator.userAgent.includes("Windows") ? "\r\n" : "\n";
    const payload = command.includes("\n")
      ? `\x1b[200~${command}\x1b[201~\r`
      : `${command}${lineEnding}`;

    try {
      await TerminalService.sendInput(currentProcess.id, payload, tabId);
    } catch {
      if (blockId) commandBlockStore.completeBlock(tabId, blockId, 1);
    }
  }

  async function handleAIQuery(query: string, blockLabel?: string) {
    const blockId = commandBlockStore.addBlock(tabId, {
      command: blockLabel ?? `/ai ${query}`,
      output: "",
      source: "ai",
      status: "running",
    });

    const context = buildTerminalContext(tabId, {
      shell: settings.defaultShell,
      workingDirectory:
        currentProcess?.working_directory || settings.workingDirectory,
    });
    const message = `${context}\n\n---\nUser request: ${query}`;

    // streamMessage both invokes onError AND rejects — guard so the error
    // isn't appended to the block twice.
    let failed = false;
    const fail = (msg: string) => {
      if (failed) return;
      failed = true;
      commandBlockStore.appendOutput(tabId, blockId, `\n\nError: ${msg}`);
      commandBlockStore.completeBlock(tabId, blockId, 1);
    };

    try {
      await aiChatService.streamMessage(message, [], {
        onChunk: (chunk) => {
          commandBlockStore.appendOutput(tabId, blockId, chunk);
        },
        onComplete: () => {
          commandBlockStore.completeBlock(tabId, blockId, 0);
        },
        onError: (error) => fail(error.message),
      });
    } catch (error) {
      fail(error instanceof Error ? error.message : "Unknown error");
    }
  }

  function handleExplain(block: CapturedCommand) {
    handleAIQuery(
      buildExplainPrompt(block, tabId, {
        shell: settings.defaultShell,
        workingDirectory:
          block.workingDirectory ||
          currentProcess?.working_directory ||
          settings.workingDirectory,
      }),
      `/ai explain: ${block.command}`.slice(0, 120),
    );
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

  export function askAI(query: string) {
    return handleAIQuery(query);
  }

  export function explainError(block: CapturedCommand) {
    handleExplain(block);
  }
</script>

<div class="flex h-full w-full">
  <div class="flex min-h-0 min-w-0 flex-1 flex-col">
    <div class="relative min-h-0 flex-1">
      <!-- xterm stays mounted in both views so the PTY keeps streaming and
           interactive apps survive view switches. visibility (not display):
           the element must keep its real size or FitAddon reports garbage
           cols/rows and the PTY wraps output at a few characters. -->
      <div
        class="absolute inset-0 bg-[var(--terminal-background)] p-2"
        style:visibility={view === "terminal" ? "visible" : "hidden"}
      >
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
      </div>
      {#if view === "blocks"}
        <div class="absolute inset-0 z-10">
          <BlocksView {tabId} onRerun={handleRerun} onExplain={handleExplain} />
        </div>
      {/if}
    </div>

    {#if showCommandInput && isTauri}
      <CommandInputBar
        {tabId}
        {view}
        onViewChange={(v) => (view = v)}
        onSubmit={handleCommandSubmit}
        onStop={handleStop}
      />
    {/if}
  </div>

  {#if widgets}
    <aside class="flex w-96 min-w-0 flex-col border-l border-border bg-card">
      {@render widgets({ tabId, process: currentProcess })}
    </aside>
  {/if}
</div>
