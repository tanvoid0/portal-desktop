<!--
  Terminal — the single reusable terminal primitive.

  One component backs every terminal surface in the app: the full workspace
  page, the project terminal tab, the embedded script runner, and any future
  one-off dialog. It is deliberately *bare*: no window chrome, no fixed height.
  The parent sizes it (`h-full min-h-0`) and wraps chrome around it if wanted.

  Modes:
    - "interactive" (default): long-lived shell.
    - "oneshot": runs `command` once via the backend and exits with its real code.
    - "display": read-only xterm buffer (no PTY).
-->
<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import "@xterm/xterm/css/xterm.css";
  import type { ITheme } from "@xterm/xterm";
  import type { TerminalConfig, TerminalProcess } from "../../types";
  import {
    XtermSession,
    type XtermSessionMode,
  } from "../../composables/useXtermSession";
  import { isTauriEnvironment } from "$lib/utils/tauri";
  import { logger } from "$lib/domains/shared";

  const log = logger.createScoped("Terminal");

  interface Props {
    /** Terminal appearance/shell config. */
    settings: TerminalConfig;
    /** Stable id used for session persistence + command routing. */
    tabId?: string;
    mode?: XtermSessionMode;
    /** Oneshot command (required when mode="oneshot"). */
    command?: string;
    /** Static buffer for mode="display". */
    displayContent?: string;
    /** Extra environment overrides forwarded to the PTY. */
    env?: Record<string, string>;
    /** Per-instance xterm theme override merged over the resolved app theme. */
    themeOverride?: Partial<ITheme>;
    /** Start the process on mount. When false, call `start()` yourself. */
    autoStart?: boolean;
    /** Kill the backend process on destroy (default: true unless oneshot). */
    killOnDestroy?: boolean;
    /** Extra classes for the host element (padding/background live on parent). */
    class?: string;
    onReady?: (process: TerminalProcess | null) => void;
    onData?: (chunk: string) => void;
    onExit?: (exitCode: number | null, buffer: string) => void;
  }

  let {
    settings,
    tabId,
    mode = "interactive",
    command,
    displayContent,
    env,
    themeOverride,
    autoStart = true,
    killOnDestroy,
    class: className = "",
    onReady,
    onData,
    onExit,
  }: Props = $props();

  const resolvedTabId = tabId ?? crypto.randomUUID();

  let containerEl = $state<HTMLDivElement | null>(null);
  let session = $state<XtermSession | null>(null);
  let started = false;

  function buildSession(): XtermSession {
    return new XtermSession({
      tabId: resolvedTabId,
      settings,
      mode,
      oneshotCommand: command,
      displayContent,
      env,
      themeOverride,
      killOnDestroy,
      onOutputChunk: onData,
      onComplete: onExit,
    });
  }

  /** Mount + start the terminal. Safe to call once; no-ops if already started. */
  export async function start(): Promise<void> {
    if (started || !containerEl) return;
    started = true;
    try {
      const s = buildSession();
      await s.mount(containerEl);
      session = s;
      onReady?.(s.currentProcess);
    } catch (e) {
      started = false;
      log.error("Terminal init failed", { e });
    }
  }

  /** Run a command in the current interactive session. */
  export async function run(cmd: string): Promise<void> {
    await session?.sendCommand(cmd);
  }

  export function write(data: string): void {
    session?.write(data);
  }

  export function clear(): void {
    session?.clear();
  }

  export function fit(): void {
    session?.fit();
  }

  export function focus(): void {
    session?.terminal?.focus();
  }

  export function getProcess(): TerminalProcess | null {
    return session?.currentProcess ?? null;
  }

  /** Underlying session — escape hatch for advanced callers (kill, etc.). */
  export function getSession(): XtermSession | null {
    return session;
  }

  export async function dispose(): Promise<void> {
    await session?.destroy();
    session = null;
    started = false;
  }

  onMount(() => {
    if (autoStart && (isTauriEnvironment() || mode === "display")) {
      void start();
    }
  });

  onDestroy(async () => {
    await dispose();
  });
</script>

<div
  bind:this={containerEl}
  class="terminal-host h-full min-h-0 w-full overflow-hidden {className}"
></div>

<style>
  .terminal-host :global(.xterm),
  .terminal-host :global(.xterm-screen),
  .terminal-host :global(.xterm-viewport) {
    height: 100% !important;
    width: 100% !important;
  }
</style>
