import { describe, expect, it, vi } from 'vitest';
import type { ThreadEventFrame } from '$lib/domains/threads/generated';

// The routing is pure; keep the terminal and Tauri modules out of the test.
vi.mock('./coderService.js', () => ({ coderService: {} }));
vi.mock('./coderTerminalCoordinator.js', () => ({}));
vi.mock('../state/coderTerminalStore.svelte.js', () => ({ coderTerminalStore: {} }));
vi.mock('../state/coderWorkspaceStore.svelte.js', () => ({ coderWorkspaceStore: {} }));
vi.mock('$lib/domains/threads/api/threadApi', () => ({ threadApi: {} }));
vi.mock('$lib/domains/mobile/browserShim', () => ({ isBrowserClient: false }));
vi.mock('$lib/domains/mobile/shell', () => ({ isMobileShell: false }));

import { canAnswerHost, claimCall, hostCallOf } from './hostResponder';

const frame = (event: ThreadEventFrame['event']): ThreadEventFrame => ({
  thread_id: 't1',
  provider: 'portal',
  seq: 1,
  at: '',
  actor: 'local',
  event,
});

describe('hostCallOf', () => {
  const run = frame({
    type: 'host_requested',
    call_id: 'c1',
    kind: 'run_command',
    payload: { command: 'ls', workspace_root: '/w', terminal_id: null },
  });

  it('routes run_command and list_terminals ', () => {
    expect(hostCallOf(run)).toMatchObject({
      kind: 'run_command',
      callId: 'c1',
      command: 'ls',
    });
    const list = frame({
      type: 'host_requested',
      call_id: 'c2',
      kind: 'list_terminals',
      payload: {},
    });
    expect(hostCallOf(list)).toEqual({
      kind: 'list_terminals',
      threadId: 't1',
      callId: 'c2',
    });
  });

  it('only the desktop webview answers: not a browser client, not the Android shell', () => {
    expect(canAnswerHost(true, false, false)).toBe(true);
    expect(canAnswerHost(true, true, false)).toBe(false);
    expect(canAnswerHost(true, false, true)).toBe(false);
    expect(canAnswerHost(false, false, false)).toBe(false);
  });

  it('claims a call once per thread and frees it after the answer', () => {
    const a = { kind: 'list_terminals' as const, threadId: 't1', callId: 'call_0' };
    const release = claimCall(a);
    expect(release).not.toBeNull();
    expect(claimCall(a)).toBeNull(); // the startup scan and the live frame
    expect(claimCall({ ...a, threadId: 't2' })).not.toBeNull(); // same id, other thread
    release?.();
    expect(claimCall(a)).not.toBeNull(); // a later request with the same id
  });

  it('ignores other requests and events', () => {
    const ask = frame({ type: 'host_requested', call_id: 'c3', kind: 'ask_user', payload: {} });
    expect(hostCallOf(ask)).toBeNull();
    expect(hostCallOf(frame({ type: 'stop_requested' }))).toBeNull();
  });
});
