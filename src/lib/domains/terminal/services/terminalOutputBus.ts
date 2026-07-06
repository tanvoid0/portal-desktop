/**
 * Singleton terminal output event bus — survives Vite HMR without duplicate listeners.
 */

import { isTauriEnvironment } from "$lib/utils/tauri";
import type { TerminalOutput } from "../types";

type OutputHandler = (output: TerminalOutput) => void;

interface TerminalOutputBusGlobal {
  // A process can have more than one subscriber (e.g. the xterm view and a
  // history/onData tracker). A Set per processId avoids the previous bug where
  // a second subscribe() silently replaced the first handler.
  callbacks?: Map<string, Set<OutputHandler>>;
  unsubscribe?: () => void;
  setupPromise?: Promise<void>;
}

const GLOBAL_KEY = "__portal_terminal_output_bus__";

function getGlobal(): TerminalOutputBusGlobal {
  const g = globalThis as unknown as Record<
    string,
    TerminalOutputBusGlobal | undefined
  >;
  if (!g[GLOBAL_KEY]) {
    g[GLOBAL_KEY] = {};
  }
  return g[GLOBAL_KEY]!;
}

function getCallbacks(): Map<string, Set<OutputHandler>> {
  const global = getGlobal();
  if (!global.callbacks) {
    global.callbacks = new Map();
  }
  return global.callbacks;
}

function emit(output: TerminalOutput): void {
  const handlers = getCallbacks().get(output.process_id);
  if (!handlers) return;
  for (const handler of handlers) handler(output);
}

function normalizeOutput(raw: TerminalOutput & { processId?: string }): TerminalOutput {
  return {
    ...raw,
    process_id: raw.process_id ?? raw.processId ?? "",
    output_type:
      raw.output_type ??
      (raw as { outputType?: string }).outputType ??
      "stdout",
    content: raw.content ?? "",
    timestamp: raw.timestamp ?? new Date().toISOString(),
  };
}

async function ensureListener(): Promise<void> {
  const global = getGlobal();
  if (global.setupPromise) {
    await global.setupPromise;
    return;
  }

  global.setupPromise = (async () => {
    if (!isTauriEnvironment()) return;

    global.unsubscribe?.();
    global.unsubscribe = undefined;

    const { listen } = await import("@tauri-apps/api/event");
    global.unsubscribe = await listen<TerminalOutput>("terminal-output", (event) => {
      emit(
        normalizeOutput(event.payload as TerminalOutput & { processId?: string }),
      );
    });
  })();

  await global.setupPromise;
}

export async function subscribeTerminalOutput(
  processId: string,
  handler: OutputHandler,
): Promise<() => void> {
  await ensureListener();
  const callbacks = getCallbacks();
  let handlers = callbacks.get(processId);
  if (!handlers) {
    handlers = new Set();
    callbacks.set(processId, handlers);
  }
  handlers.add(handler);
  return () => {
    const set = getCallbacks().get(processId);
    if (!set) return;
    set.delete(handler);
    if (set.size === 0) getCallbacks().delete(processId);
  };
}

export function dispatchTerminalOutput(output: TerminalOutput): void {
  emit(output);
}

export async function ensureTerminalOutputListener(): Promise<void> {
  await ensureListener();
}

/** Tear down the global Tauri listener and clear all subscribers. */
export function disposeTerminalOutputBus(): void {
  const global = getGlobal();
  global.unsubscribe?.();
  global.unsubscribe = undefined;
  global.setupPromise = undefined;
  global.callbacks?.clear();
}

// Drop the singleton listener on HMR so reloads don't stack duplicate listeners.
if (import.meta.hot) {
  import.meta.hot.dispose(() => disposeTerminalOutputBus());
}
