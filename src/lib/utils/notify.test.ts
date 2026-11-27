import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest';

vi.mock('@tauri-apps/plugin-notification', () => ({
  isPermissionGranted: vi.fn(),
  requestPermission: vi.fn(),
  onAction: vi.fn(),
  sendNotification: vi.fn(),
}));
vi.mock('@tauri-apps/api/window', () => ({ getCurrentWindow: vi.fn() }));
vi.mock('$app/navigation', () => ({ goto: vi.fn() }));
vi.mock('$lib/domains/mobile/shell', () => ({ isMobileShell: false }));
vi.mock('$lib/domains/settings/stores/settingsStore', async () => {
  const { writable } = await import('svelte/store');
  return { appSettings: writable<{ notifications: Record<string, boolean> } | null>(null) };
});

import {
  isPermissionGranted,
  requestPermission,
  onAction,
  sendNotification,
} from '@tauri-apps/plugin-notification';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { goto } from '$app/navigation';
import * as mobileShell from '$lib/domains/mobile/shell';
import { appSettings } from '$lib/domains/settings/stores/settingsStore';
import { toastActions } from '$lib/domains/shared/stores/toastStore';
import { notificationInbox } from '$lib/domains/notifications/state/notificationInbox.svelte';
import { notify, initNotify } from './notify';
import type { Writable } from 'svelte/store';

const flush = () => new Promise((r) => setTimeout(r, 0));

// The mocked module swaps the real derived (read-only) store for a writable
// one so tests can drive it — cast past the real, read-only type.
const settingsStore = appSettings as unknown as Writable<{
  notifications: Record<string, boolean>;
} | null>;

function setSettings(enabled: boolean, desktopNotifications: boolean, soundEnabled = false) {
  settingsStore.set({ notifications: { enabled, desktopNotifications, soundEnabled } });
}

describe('notify', () => {
  let showSpy: ReturnType<typeof vi.spyOn>;
  let dismissSpy: ReturnType<typeof vi.spyOn>;

  beforeEach(() => {
    showSpy = vi.spyOn(toastActions, 'show');
    dismissSpy = vi.spyOn(toastActions, 'dismiss').mockImplementation(() => {});
    vi.stubGlobal('document', { hasFocus: () => true });
    vi.stubGlobal('location', { pathname: '/' });
  });

  afterEach(() => {
    showSpy.mockRestore();
    dismissSpy.mockRestore();
    vi.clearAllMocks();
    vi.unstubAllGlobals();
    settingsStore.set(null);
    (mobileShell as { isMobileShell: boolean }).isMobileShell = false;
  });

  it('keeps an unread history entry and marks it read when the toast is opened', () => {
    notificationInbox.clear();
    notify({ title: 'hi', route: '/devices' });
    expect(notificationInbox.unread).toBe(1);

    showSpy.mock.calls[0]![0].action!.onClick();
    expect(notificationInbox.unread).toBe(0);
    expect(goto).toHaveBeenCalledWith('/devices');
  });

  it('does nothing when focused and already on the target route', () => {
    (document as unknown as { hasFocus: () => boolean }).hasFocus = () => true;
    vi.stubGlobal('location', { pathname: '/devices' });
    notify({ title: 'hi', route: '/devices' });
    expect(showSpy).not.toHaveBeenCalled();
    expect(sendNotification).not.toHaveBeenCalled();
  });

  it('toasts (no native) when focused on a different route', () => {
    vi.stubGlobal('location', { pathname: '/' });
    setSettings(true, true);
    notify({ title: 'hi', route: '/devices' });
    expect(showSpy).toHaveBeenCalled();
    expect(sendNotification).not.toHaveBeenCalled();
  });

  it('toasts and sends a native notification when unfocused and enabled', () => {
    vi.stubGlobal('document', { hasFocus: () => false });
    setSettings(true, true);
    notify({ title: 'hi', body: 'body', route: '/devices' });
    const toastId = showSpy.mock.results[0]!.value;
    expect(sendNotification).toHaveBeenCalledWith(
      expect.objectContaining({ extra: { route: '/devices', toastId } })
    );
  });

  it('skips the native call when desktopNotifications is off', () => {
    vi.stubGlobal('document', { hasFocus: () => false });
    setSettings(true, false);
    notify({ title: 'hi' });
    expect(showSpy).toHaveBeenCalled();
    expect(sendNotification).not.toHaveBeenCalled();
  });

  it('skips the native call when the master toggle is off', () => {
    vi.stubGlobal('document', { hasFocus: () => false });
    setSettings(false, true);
    notify({ title: 'hi' });
    expect(showSpy).toHaveBeenCalled();
    expect(sendNotification).not.toHaveBeenCalled();
  });

  it('passes the sound name only when soundEnabled is on', () => {
    vi.stubGlobal('document', { hasFocus: () => false });
    setSettings(true, true, true);
    notify({ title: 'hi' });
    expect(sendNotification).toHaveBeenCalledWith(expect.objectContaining({ sound: 'Default' }));

    setSettings(true, true, false);
    notify({ title: 'hi' });
    expect(sendNotification).toHaveBeenLastCalledWith(
      expect.objectContaining({ sound: undefined })
    );
  });

  it('initNotify raises the window, dismisses the toast and navigates on action', async () => {
    (isPermissionGranted as ReturnType<typeof vi.fn>).mockResolvedValue(true);
    let capturedCb: (e: unknown) => void = () => {};
    (onAction as ReturnType<typeof vi.fn>).mockImplementation((cb) => {
      capturedCb = cb;
      return Promise.resolve({ unregister: vi.fn().mockResolvedValue(undefined) });
    });
    const win = {
      unminimize: vi.fn().mockResolvedValue(undefined),
      show: vi.fn().mockResolvedValue(undefined),
      setFocus: vi.fn().mockResolvedValue(undefined),
    };
    (getCurrentWindow as ReturnType<typeof vi.fn>).mockReturnValue(win);

    await initNotify();
    capturedCb({ notification: { extra: { route: '/devices', toastId: 't1' } } });
    await flush();

    expect(win.unminimize).toHaveBeenCalled();
    expect(win.show).toHaveBeenCalled();
    expect(win.setFocus).toHaveBeenCalled();
    expect(dismissSpy).toHaveBeenCalledWith('t1');
    expect(goto).toHaveBeenCalledWith('/devices');
  });

  it('skips raising the window on mobile but still dismisses the toast and navigates on action (#641)', async () => {
    (isPermissionGranted as ReturnType<typeof vi.fn>).mockResolvedValue(true);
    let capturedCb: (e: unknown) => void = () => {};
    (onAction as ReturnType<typeof vi.fn>).mockImplementation((cb) => {
      capturedCb = cb;
      return Promise.resolve({ unregister: vi.fn().mockResolvedValue(undefined) });
    });
    (mobileShell as { isMobileShell: boolean }).isMobileShell = true;

    await initNotify();
    capturedCb({ notification: { extra: { route: '/devices', toastId: 't1' } } });
    await flush();

    expect(getCurrentWindow).not.toHaveBeenCalled();
    expect(dismissSpy).toHaveBeenCalledWith('t1');
    expect(goto).toHaveBeenCalledWith('/devices');
  });

  it('dismisses the toast by route when the action carries no toastId (mobile, #642)', async () => {
    (isPermissionGranted as ReturnType<typeof vi.fn>).mockResolvedValue(true);
    let capturedCb: (e: unknown) => void = () => {};
    (onAction as ReturnType<typeof vi.fn>).mockImplementation((cb) => {
      capturedCb = cb;
      return Promise.resolve({ unregister: vi.fn().mockResolvedValue(undefined) });
    });
    (mobileShell as { isMobileShell: boolean }).isMobileShell = true;

    // Mobile's native notification is posted by Rust, which never learns the
    // JS-generated toastId — only the route both surfaces agree on.
    vi.stubGlobal('document', { hasFocus: () => false });
    notify({ title: 'Agent needs your approval', route: '/ai/agents?session=r1', duration: 0 });
    const toastId = showSpy.mock.results[0]!.value;

    await initNotify();
    capturedCb({ notification: { extra: { route: '/ai/agents?session=r1' } } });
    await flush();

    expect(dismissSpy).toHaveBeenCalledWith(toastId);
    expect(goto).toHaveBeenCalledWith('/ai/agents?session=r1');
  });

  it('requests permission when not already granted', async () => {
    (isPermissionGranted as ReturnType<typeof vi.fn>).mockResolvedValue(false);
    (requestPermission as ReturnType<typeof vi.fn>).mockResolvedValue('granted');
    (onAction as ReturnType<typeof vi.fn>).mockResolvedValue({
      unregister: vi.fn().mockResolvedValue(undefined),
    });
    await initNotify();
    expect(requestPermission).toHaveBeenCalled();
  });
});
