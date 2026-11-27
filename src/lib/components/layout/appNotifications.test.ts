import { describe, expect, it, vi } from 'vitest';

const notify = vi.fn();
let emit: (frame: unknown) => void;

vi.mock('@tauri-apps/api/event', () => ({ listen: async () => () => {} }));
vi.mock('$lib/utils/notify', () => ({ notify, initNotify: async () => () => {} }));
vi.mock('$lib/domains/notifications/utils/facts', () => ({
  permissionFact: () => [],
  pipelineFacts: () => [],
}));
vi.mock('$lib/domains/agents/state/agentRuns.svelte.js', () => ({
  agentRuns: { ensureInit: async () => {} },
}));
vi.mock('$lib/domains/shared/stores/toastStore', () => ({ toastActions: { dismiss: () => {} } }));
vi.mock('$lib/domains/threads/api/threadApi', () => ({
  threadApi: {
    onEvent: async (cb: (frame: unknown) => void) => {
      emit = cb;
      return () => {};
    },
  },
}));
vi.mock('$lib/config/agents-tabs', () => ({
  PORTAL_ENGINE_TAB: { url: '/ai/coder' },
  agentTabPath: () => '/agents',
}));
vi.mock('$lib/domains/mobile/shell', () => ({ isMobileShell: false }));

const { startAppNotifications } = await import('./appNotifications');

describe('agent failure toasts', () => {
  const send = (event: Record<string, unknown>) =>
    emit({ thread_id: 't', provider: 'claude-code', event });

  it('reports a failed turn once, and a crash by its exit code', () => {
    startAppNotifications();
    // Error result with the process still alive, then a non-zero exit.
    send({ type: 'turn_ended', outcome: 'error', error: 'boom' });
    send({ type: 'process_exited', code: 1 });
    expect(notify.mock.calls.map(([n]) => n.body)).toEqual(['boom']);

    // A later run of the same thread crashes mid-turn.
    notify.mockClear();
    send({ type: 'turn_ended', outcome: 'error', error: 'boom' });
    send({ type: 'turn_started' });
    send({ type: 'turn_ended', outcome: 'interrupted' });
    send({ type: 'process_exited', code: 2 });
    expect(notify.mock.calls.map(([n]) => n.body)).toEqual(['boom', 'Exited with code 2']);
  });
});
