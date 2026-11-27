/**
 * Answers the agent's `run_command` / `list_terminals` host requests with this window's
 * terminals. Only the hub's own desktop webview may: a browser or phone client sees the same
 * `host_requested` frames, but has no terminals to run in. The first answer wins in the hub.
 */
import type { UnlistenFn } from '@tauri-apps/api/event';
import { threadApi } from '$lib/domains/threads/api/threadApi';
import type { ThreadEventFrame } from '$lib/domains/threads/generated';
import { isBrowserClient } from '$lib/domains/mobile/browserShim';
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

/** Which host request a frame carries, or null (not one, or not ours to answer). */
export function hostCallOf(f: ThreadEventFrame, desktop: boolean): HostCall | null {
  const e = f.event;
  if (!desktop || e.type !== 'host_requested') return null;
  if (e.kind === 'list_terminals') {
    return { kind: 'list_terminals', threadId: f.thread_id, callId: e.call_id };
  }
  if (e.kind !== 'run_command') return null;
  const p = (e.payload ?? {}) as {
    command?: string;
    workspace_root?: string;
    terminal_id?: string | null;
  };
  return {
    kind: 'run_command',
    threadId: f.thread_id,
    callId: e.call_id,
    command: String(p.command ?? ''),
    workspaceRoot: String(p.workspace_root ?? ''),
    terminalId: p.terminal_id,
  };
}

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
  if (delegated.delete(m.tool_call_id)) return;
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

async function answer(call: HostCall, openTerminal: () => void) {
  if (call.kind === 'list_terminals') {
    const list = coderTerminalStore.listForAgent(call.threadId);
    await coderService.submitTerminalList(call.threadId, call.callId, JSON.stringify(list));
    return;
  }
  openTerminal();
  const tab = coderTerminalStore.ensureDefault(call.threadId, call.workspaceRoot);
  coderWorkspaceStore.openTerminal(call.threadId, tab.id, tab.label);
  delegated.add(call.callId);
  let result: string;
  try {
    result = await executeAgentCommand({ ...call, terminalId: call.terminalId });
  } catch (e) {
    result = `Error: ${e}`;
  }
  await coderService.submitCommandResult(call.threadId, call.callId, result);
}

/** Idempotent per app lifetime; `openTerminal` shows the session terminal before a command runs. */
let started: Promise<UnlistenFn | null> | null = null;
export function startHostResponder(openTerminal: () => void): Promise<UnlistenFn | null> {
  started ??= (async () => {
    if (!isTauriEnvironment() || isBrowserClient) return null;
    return threadApi.onEvent((f) => {
      const call = hostCallOf(f, true);
      if (call)
        void answer(call, openTerminal).catch((e) => console.error('coder: host answer failed', e));
    });
  })();
  return started;
}
