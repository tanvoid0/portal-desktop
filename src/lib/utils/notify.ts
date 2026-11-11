/** Desktop notification dispatcher: native OS banner when the window is
 * unfocused, in-app toast always (it's the backup once the banner expires
 * into the Action Center). Click-to-action on either surface navigates. */
import { get } from 'svelte/store';
import { goto } from '$app/navigation';
import { getCurrentWindow } from '@tauri-apps/api/window';
import {
  isPermissionGranted,
  requestPermission,
  onAction,
  sendNotification,
} from '@tauri-apps/plugin-notification';
import { toastActions, type Toast } from '$lib/domains/shared/stores/toastStore';
import { appSettings } from '$lib/domains/settings/stores/settingsStore';
import { createLogger } from '$lib/domains/shared/services/logger';

const log = createLogger('notify');

export interface NotifyPayload {
  title: string;
  body?: string;
  type?: Toast['type'];
  route?: string;
  actionLabel?: string;
  duration?: number;
}

export function notify(p: NotifyPayload): void {
  const focused = document.hasFocus();
  if (focused && p.route === location.pathname) return;

  const toastId = toastActions.show({
    title: p.title,
    description: p.body,
    type: p.type ?? 'info',
    duration: p.duration,
    action: p.route ? { label: p.actionLabel ?? 'Open', onClick: () => goto(p.route!) } : undefined,
  });

  // Settings not loaded yet (cold start): default to on rather than silently
  // dropping native notifications for the whole session.
  const n = get(appSettings)?.notifications ?? {
    enabled: true,
    desktopNotifications: true,
    soundEnabled: true,
  };
  if (!n.enabled || !n.desktopNotifications) return;
  if (focused) return;

  try {
    sendNotification({
      title: p.title,
      body: p.body,
      // ponytail: 'Default' is the Windows sound name; macOS/Linux stay
      // silent until per-OS sound names are added.
      sound: n.soundEnabled ? 'Default' : undefined,
      extra: { route: p.route, toastId },
    });
  } catch (err) {
    log.error('Failed to send native notification', err);
  }
}

export async function initNotify(): Promise<() => void> {
  try {
    const granted = (await isPermissionGranted()) || (await requestPermission()) === 'granted';
    if (!granted) log.error('Notification permission not granted');
  } catch (err) {
    log.error('Failed to request notification permission', err);
  }

  try {
    const listener = await onAction((e) => {
      const extra = e.notification?.extra as { route?: string; toastId?: string } | undefined;
      const win = getCurrentWindow();
      void win
        .unminimize()
        .then(() => win.show())
        .then(() => win.setFocus());
      if (extra?.toastId) toastActions.dismiss(extra.toastId);
      if (extra?.route) goto(extra.route);
    });
    return () => void listener.unregister();
  } catch (err) {
    log.error('Failed to register notification action listener', err);
    return () => {};
  }
}
