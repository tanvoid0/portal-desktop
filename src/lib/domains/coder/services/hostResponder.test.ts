import { describe, expect, it, vi } from 'vitest';
import type { ThreadEventFrame } from '$lib/domains/threads/generated';

// The routing is pure; keep the terminal and Tauri modules out of the test.
vi.mock('./coderService.js', () => ({ coderService: {} }));
vi.mock('./coderTerminalCoordinator.js', () => ({}));
vi.mock('../state/coderTerminalStore.svelte.js', () => ({ coderTerminalStore: {} }));
vi.mock('../state/coderWorkspaceStore.svelte.js', () => ({ coderWorkspaceStore: {} }));
vi.mock('$lib/domains/threads/api/threadApi', () => ({ threadApi: {} }));
vi.mock('$lib/domains/mobile/browserShim', () => ({ isBrowserClient: false }));

import { hostCallOf } from './hostResponder';

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

  it('routes run_command and list_terminals in the desktop webview', () => {
    expect(hostCallOf(run, true)).toMatchObject({
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
    expect(hostCallOf(list, true)).toEqual({
      kind: 'list_terminals',
      threadId: 't1',
      callId: 'c2',
    });
  });

  it('answers nothing elsewhere, and ignores other requests and events', () => {
    expect(hostCallOf(run, false)).toBeNull();
    const ask = frame({ type: 'host_requested', call_id: 'c3', kind: 'ask_user', payload: {} });
    expect(hostCallOf(ask, true)).toBeNull();
    expect(hostCallOf(frame({ type: 'stop_requested' }), true)).toBeNull();
  });
});
