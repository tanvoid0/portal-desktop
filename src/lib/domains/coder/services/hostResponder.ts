/**
 * Answers the agent's `run_command` / `list_terminals` host requests with this window's
 * terminals. Only the hub's own desktop webview may: a browser or phone client sees the same
 * `host_requested` frames, but has no terminals to run in. The first answer wins in the hub.
 */
import type { UnlistenFn } from '@tauri-apps/api/event';
import { threadApi } from '$lib/domains/threads/api/threadApi';
import type { ThreadEventFrame } from '$lib/domains/threads/generated';
import { isBrowserClient } from '$lib/domains/mobile/browserShim';
import { isMobileShell } from '$lib/domains/mobile/shell';
import { isTauriEnvironment } from '$lib/utils/tauri';
import type { ChatMessage } from '../types.js';
import { coderService } from './coderService.js';
import { executeAgentCommand, mirrorCommandOutput } from './coderTerminalCoordinator.js';
import { coderTerminalStore } from '../state/coderTerminalStore.svelte.js';
import { coderWorkspaceStore } from '../state/coderWorkspaceStore.svelte.js';
import { formatFailedResult, isToolResultFailure } from '../utils/toolCallDisplay.js';

export type HostCall =
  | {
      kind: 'run_command';
      threadId: string;
      callId: string;
      command: string;
      workspaceRoot: string;
      terminalId?: string | null;
    }
  | { kind: 'list_terminals'; threadId: string; callId: string };

/** The host request behind a `host_requested` event, or null (`ask_user` is answered on its card). */
export function hostCall(
  threadId: string,
  callId: string,
  kind: string,
  payload: unknown
): HostCall | null {
  if (kind === 'list_terminals') return { kind, threadId, callId };
  if (kind !== 'run_command') return null;
  const p = (payload ?? {}) as {
    command?: string;
    workspace_root?: string;
    terminal_id?: string | null;
  };
  return {
    kind,
    threadId,
    callId,
    command: String(p.command ?? ''),
    workspaceRoot: String(p.workspace_root ?? ''),
    terminalId: p.terminal_id,
  };
}

export function hostCallOf(f: ThreadEventFrame): HostCall | null {
  const e = f.event;
  return e.type === 'host_requested' ? hostCall(f.thread_id, e.call_id, e.kind, e.payload) : null;
}

/** Only the hub's own desktop webview: not a browser client, and not the Android shell (a real Tauri runtime). */
export const canAnswerHost = (tauri: boolean, browserClient: boolean, mobileShell: boolean) =>
  tauri && !browserClient && !mobileShell;

/** Calls this window ran itself: their tool result is already in the terminal. */
const delegated = new Set<string>();

function parseArgs(raw: string): Record<string, unknown> {
  try {
    return JSON.parse(raw || '{}') as Record<string, unknown>;
  } catch {
    return {};
  }
}

/** Echo a `run_command` result that this window did not run (a background run) into the session terminal. */
export function mirrorToolResult(threadId: string, messages: ChatMessage[], m: ChatMessage) {
  if (m.role !== 'tool' || !m.tool_call_id || m.content == null) return;
  if (delegated.delete(`${threadId}:${m.tool_call_id}`)) return;
  const call = messages
    .flatMap((x) => (x.role === 'assistant' ? (x.tool_calls ?? []) : []))
    .find((c) => c.id === m.tool_call_id && c.function.name === 'run_command');
  if (!call) return;
  const failed = isToolResultFailure('run_command', m.content);
  const output = failed ? formatFailedResult('run_command', m.content) : m.content;
  mirrorCommandOutput(
    threadId,
    null,
    String(parseArgs(call.function.arguments).command ?? ''),
    output,
    failed
  );
}

/** Calls being answered. Some providers number calls per turn (`call_0`), so the key has the thread. */
const answering = new Set<string>();

/** Claim a call so it is answered once, live or from a snapshot; returns the release, or null if taken. */
export function claimCall(call: HostCall): (() => void) | null {
  const key = `${call.threadId}:${call.callId}`;
  if (answering.has(key)) return null;
  answering.add(key);
  return () => void answering.delete(key);
}

async function answer(call: HostCall, openTerminal: () => void) {
  const release = claimCall(call);
  if (!release) return;
  try {
    await run(call, openTerminal);
  } finally {
    release();
  }
}

async function run(call: HostCall, openTerminal: () => void) {
  if (call.kind === 'list_terminals') {
    const list = coderTerminalStore.listForAgent(call.threadId);
    await coderService.submitTerminalList(call.threadId, call.callId, JSON.stringify(list));
    return;
  }
  openTerminal();
  const tab = coderTerminalStore.ensureDefault(call.threadId, call.workspaceRoot);
  coderWorkspaceStore.openTerminal(call.threadId, tab.id, tab.label);
  delegated.add(`${call.threadId}:${call.callId}`);
  let result: string;
  try {
    result = await executeAgentCommand({ ...call, terminalId: call.terminalId });
  } catch (e) {
    result = `Error: ${e}`;
  }
  await coderService.submitCommandResult(call.threadId, call.callId, result);
}

/** Open requests of running Portal threads: a reload or a late start missed their frames. */
async function answerOpen(openTerminal: () => void) {
  const open = (await threadApi.list()).filter(
    (t) => t.provider === 'portal' && (t.status === 'running' || t.status === 'waiting')
  );
  for (const t of open) {
    const snap = await threadApi.snapshot(t.id);
    for (const h of snap?.state.host_requests ?? []) {
      const call = hostCall(t.id, h.call_id, h.kind, h.payload);
      if (call)
        void answer(call, openTerminal).catch((e) => console.error('coder: host answer failed', e));
    }
  }
}

/** Idempotent per app lifetime; `openTerminal` shows the session terminal before a command runs. */
let started: Promise<UnlistenFn | null> | null = null;
export function startHostResponder(openTerminal: () => void): Promise<UnlistenFn | null> {
  started ??= (async () => {
    if (!canAnswerHost(isTauriEnvironment(), isBrowserClient, isMobileShell)) return null;
    const un = await threadApi.onEvent((f) => {
      const call = hostCallOf(f);
      if (call)
        void answer(call, openTerminal).catch((e) => console.error('coder: host answer failed', e));
    });
    answerOpen(openTerminal).catch((e) => console.error('coder: open host requests failed', e));
    return un;
  })();
  return started;
}
