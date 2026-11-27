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
import {
  notificationInbox,
  type NotificationContext,
} from '$lib/domains/notifications/state/notificationInbox.svelte';

import { isMobileShell } from '$lib/domains/mobile/shell';

const log = createLogger('notify');

// Mobile's native notification (`Mobile::notify` in mobile/src-tauri/src/lib.rs) is posted
// from Rust, which has no way to know the id of the toast this same call creates below — so
// it can only carry a route (#642). Track the latest toast per route so a tap can still find
// and dismiss it. ponytail: grows one entry per distinct route notified for the session; same
// tradeoff as `notified` in lib.rs — revisit with an LRU cap if that ever matters.
const toastByRoute = new Map<string, string>();

export interface NotifyPayload extends NotificationContext {
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

  // History first: the toast and the banner both expire, the inbox doesn't.
  const type = p.type ?? 'info';
  const { title, body, route, source, runId, requestId, projectId, facts } = p;
  notificationInbox.add({
    title,
    body,
    type,
    route,
    source,
    runId,
    requestId,
    projectId,
    facts,
  });

  const open = () => {
    notificationInbox.markRouteRead(p.route!);
    goto(p.route!);
  };
  const toastId = toastActions.show({
    title: p.title,
    description: p.body,
    type,
    duration: p.duration,
    action: p.route ? { label: p.actionLabel ?? 'Open', onClick: open } : undefined,
  });
  if (p.route) toastByRoute.set(p.route, toastId);

  // Settings not loaded yet (cold start): default to on rather than silently
  // dropping native notifications for the whole session.
  const n = get(appSettings)?.notifications ?? {
    enabled: true,
    desktopNotifications: true,
    soundEnabled: true,
  };
  if (isMobileShell || !n.enabled || !n.desktopNotifications) return;
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
      // Desktop only: tapping a native notification there doesn't refocus the
      // window on its own. On mobile the tap already foregrounds the app
      // (that's how Android delivers it), and these window commands aren't
      // granted in mobile/src-tauri/capabilities (#641).
      if (!isMobileShell) {
        const win = getCurrentWindow();
        void win
          .unminimize()
          .then(() => win.show())
          .then(() => win.setFocus());
      }
      const dismissId =
        extra?.toastId ?? (extra?.route ? toastByRoute.get(extra.route) : undefined);
      if (dismissId) toastActions.dismiss(dismissId);
      if (extra?.route) {
        toastByRoute.delete(extra.route);
        notificationInbox.markRouteRead(extra.route);
        goto(extra.route);
      }
    });
    return () => void listener.unregister();
  } catch (err) {
    log.error('Failed to register notification action listener', err);
    return () => {};
  }
}
