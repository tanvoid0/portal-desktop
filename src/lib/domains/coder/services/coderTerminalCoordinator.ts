/**
 * Runs agent commands in coder session terminals — reuse the open interactive
 * shell when available; fall back to a oneshot PTY only when the tab is not
 * mounted yet.
 */

import Terminal from "$lib/domains/terminal/components/core/Terminal.svelte";
import { defaultTerminalConfig } from "$lib/domains/terminal/config/defaultTerminalConfig";
import {
  createTerminalProcess,
  getProcessExitCode,
  subscribeProcessOutput,
} from "$lib/domains/terminal/composables/useTerminalProcess";
import { coderTerminalStore } from "../state/coderTerminalStore.svelte.js";
import { isTauriEnvironment } from "$lib/utils/tauri";

type TerminalRef = ReturnType<typeof Terminal>;

const refs = new Map<string, TerminalRef>();
const abortControllers = new Map<string, AbortController>();

/** Max wait for an interactive terminal ref before falling back to oneshot. */
const REF_WAIT_MS = 1500;

export class CommandAbortedError extends Error {
  constructor() {
    super("cancelled");
    this.name = "CommandAbortedError";
  }
}

function commandKey(threadId: string, callId: string): string {
  return `${threadId}:${callId}`;
}

function refKey(threadId: string, terminalId: string): string {
  return `${threadId}:${terminalId}`;
}

/** Abort in-flight agent command executions for a thread. */
export function abortAgentCommands(threadId: string) {
  for (const [key, ac] of abortControllers.entries()) {
    if (key.startsWith(`${threadId}:`)) {
      ac.abort();
      abortControllers.delete(key);
    }
  }
}

export function registerCoderTerminal(
  threadId: string,
  terminalId: string,
  ref: TerminalRef | null,
) {
  const key = refKey(threadId, terminalId);
  if (ref) refs.set(key, ref);
  else refs.delete(key);
}

export function getCoderTerminalRef(
  threadId: string,
  terminalId: string,
): TerminalRef | undefined {
  return refs.get(refKey(threadId, terminalId));
}

function throwIfAborted(signal: AbortSignal) {
  if (signal.aborted) throw new CommandAbortedError();
}

function sleep(ms: number, signal: AbortSignal): Promise<void> {
  throwIfAborted(signal);
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      signal.removeEventListener("abort", onAbort);
      resolve();
    }, ms);
    const onAbort = () => {
      clearTimeout(timer);
      reject(new CommandAbortedError());
    };
    signal.addEventListener("abort", onAbort, { once: true });
  });
}

async function waitForRef(
  threadId: string,
  terminalId: string,
  signal: AbortSignal,
  timeoutMs = REF_WAIT_MS,
): Promise<TerminalRef | null> {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    throwIfAborted(signal);
    const ref = getCoderTerminalRef(threadId, terminalId);
    if (ref?.getSession()) return ref;
    await sleep(50, signal);
  }
  const ref = getCoderTerminalRef(threadId, terminalId);
  return ref?.getSession() ? ref : null;
}

async function runInInteractive(
  threadId: string,
  terminalId: string,
  ref: TerminalRef,
  command: string,
  signal: AbortSignal,
  timeoutMs = 120_000,
): Promise<{ output: string; exitCode: number | null }> {
  throwIfAborted(signal);
  const session = ref.getSession();
  if (!session) {
    return { output: "Error: terminal session not ready", exitCode: 1 };
  }

  const startLen = session.outputBuffer.length;
  coderTerminalStore.setRunning(threadId, terminalId, true);

  try {
    await ref.run(command);
    const deadline = Date.now() + timeoutMs;
    let lastLen = startLen;
    let stable = 0;

    while (Date.now() < deadline) {
      throwIfAborted(signal);
      await sleep(200, signal);
      const len = session.outputBuffer.length;
      if (len === lastLen) {
        stable += 1;
        if (stable >= 5) break;
      } else {
        stable = 0;
        lastLen = len;
      }
    }

    const output = session.outputBuffer.slice(startLen).trim();
    return { output: output || "(no output)", exitCode: 0 };
  } catch (e) {
    if (e instanceof CommandAbortedError) throw e;
    return {
      output: `Error: ${e instanceof Error ? e.message : String(e)}`,
      exitCode: 1,
    };
  } finally {
    coderTerminalStore.setRunning(threadId, terminalId, false);
  }
}

async function runOneshotPty(
  threadId: string,
  terminalId: string,
  workspaceRoot: string,
  command: string,
  signal: AbortSignal,
): Promise<{ output: string; exitCode: number | null }> {
  throwIfAborted(signal);
  const tab = coderTerminalStore.getTab(threadId, terminalId);
  if (!tab) {
    return { output: "Error: terminal tab not found", exitCode: 1 };
  }
  if (!isTauriEnvironment()) {
    return { output: "Error: terminal requires desktop app", exitCode: 1 };
  }

  coderTerminalStore.setRunning(threadId, terminalId, true);

  try {
    const shell = defaultTerminalConfig.defaultShell;
    const process = await createTerminalProcess({
      tabId: tab.tabId,
      shell,
      workingDirectory: workspaceRoot || defaultTerminalConfig.workingDirectory,
      command,
    });
    if (!process) {
      return { output: "Error: failed to spawn terminal process", exitCode: 1 };
    }

    let buffer = "";
    return await new Promise((resolve, reject) => {
      const onAbort = () => reject(new CommandAbortedError());
      signal.addEventListener("abort", onAbort, { once: true });

      void subscribeProcessOutput(process.id, (output) => {
        if (signal.aborted) return;
        buffer += output.content;
        const ref = getCoderTerminalRef(threadId, terminalId);
        ref?.write(output.content);

        if (output.output_type === "exit") {
          signal.removeEventListener("abort", onAbort);
          void getProcessExitCode(process.id).then((code) => {
            const text = buffer.trim() || "(no output)";
            mirrorCommandOutput(threadId, terminalId, command, text, code !== 0);
            resolve({ output: text, exitCode: code });
          });
        }
      });
    });
  } finally {
    coderTerminalStore.setRunning(threadId, terminalId, false);
  }
}

export interface AgentCommandRequest {
  threadId: string;
  callId: string;
  command: string;
  workspaceRoot: string;
  terminalId?: string | null;
}

/**
 * Resolve terminal for an agent command:
 * - omit / null → default interactive tab (create if needed)
 * - "new" → new interactive tab
 * - existing id → reuse that tab when mounted
 */
export async function executeAgentCommand(
  req: AgentCommandRequest,
): Promise<string> {
  const key = commandKey(req.threadId, req.callId);
  abortAgentCommands(req.threadId);
  const ac = new AbortController();
  abortControllers.set(key, ac);
  const signal = ac.signal;

  try {
    const { threadId, command, workspaceRoot } = req;
    let terminalId = req.terminalId?.trim() || null;

    if (terminalId === "new") {
      const tab = coderTerminalStore.createTab(threadId, {
        workspaceRoot,
        createdBy: "agent",
        kind: "interactive",
      });
      terminalId = tab.id;
    } else if (!terminalId) {
      const tab = coderTerminalStore.ensureDefault(threadId, workspaceRoot);
      terminalId = tab.id;
    } else if (!coderTerminalStore.getTab(threadId, terminalId)) {
      coderTerminalStore.createTab(threadId, {
        workspaceRoot,
        createdBy: "agent",
        id: terminalId,
        kind: "interactive",
      });
    }

    coderTerminalStore.setActive(threadId, terminalId);
    const tab = coderTerminalStore.getTab(threadId, terminalId)!;

    let result: { output: string; exitCode: number | null };

    if (tab.kind === "oneshot") {
      result = await runOneshotPty(
        threadId,
        terminalId,
        workspaceRoot,
        command,
        signal,
      );
    } else {
      const ref = await waitForRef(threadId, terminalId, signal);
      if (ref) {
        result = await runInInteractive(
          threadId,
          terminalId,
          ref,
          command,
          signal,
        );
      } else {
        result = await runOneshotPty(
          threadId,
          terminalId,
          workspaceRoot,
          command,
          signal,
        );
      }
    }

    if (result.exitCode != null && result.exitCode !== 0) {
      return `exit ${result.exitCode}\n${result.output}`;
    }
    return result.output;
  } catch (e) {
    if (e instanceof CommandAbortedError) return "Error: cancelled";
    throw e;
  } finally {
    abortControllers.delete(key);
  }
}

/** Mirror completed tool output into an interactive terminal (display only). */
export function mirrorCommandOutput(
  threadId: string,
  terminalId: string | null,
  command: string,
  output: string,
  failed = false,
) {
  const id =
    terminalId && coderTerminalStore.getTab(threadId, terminalId)
      ? terminalId
      : coderTerminalStore.activeId(threadId);
  if (!id) return;

  const ref = getCoderTerminalRef(threadId, id);
  if (!ref) return;

  const prompt = failed ? "\x1b[31m" : "\x1b[90m";
  const body = failed ? "\x1b[31m" : "";
  const reset = "\x1b[0m";
  const header = `${prompt}$ ${command}${reset}\r\n`;
  const text = output.replace(/\n/g, "\r\n");
  ref.write(`${header}${body}${text}${reset}\r\n\r\n`);
  ref.fit();
}
