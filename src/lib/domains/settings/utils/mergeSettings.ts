import type { AppSettings } from '../types';

export const defaultSettings: AppSettings = {
  theme: 'system',
  language: 'en',
  timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
  dateFormat: 'YYYY-MM-DD',
  timeFormat: '24h',
  windowState: {
    width: 1280,
    height: 720,
    maximized: false,
    rememberPosition: true,
  },
  startupBehavior: {
    openLastSession: false,
    restoreWindows: true,
    showWelcomeScreen: true,
    minimizeToTray: false,
    startMinimized: false,
  },
  notifications: {
    enabled: true,
    desktopNotifications: true,
    soundEnabled: false,
    showInTaskbar: true,
    types: {
      success: true,
      info: true,
      warning: true,
      error: true,
      updates: true,
      security: true,
    },
  },
  privacy: {
    analytics: false,
    crashReports: true,
    telemetry: false,
    usageData: false,
    marketing: false,
  },
  updates: {
    autoCheck: true,
    autoDownload: false,
    autoInstall: false,
    checkInterval: 24,
    channel: 'stable',
    notifyOnUpdate: true,
  },
};

/** Merge stored settings over the defaults so every nested property exists. */
export function mergeSettings(s: AppSettings | null): AppSettings {
  if (!s) return defaultSettings;
  const d = defaultSettings;
  return {
    ...d,
    ...s,
    windowState: { ...d.windowState, ...(s.windowState || {}) },
    startupBehavior: { ...d.startupBehavior, ...(s.startupBehavior || {}) },
    notifications: {
      ...d.notifications,
      ...(s.notifications || {}),
      types: { ...d.notifications.types, ...(s.notifications?.types || {}) },
    },
    privacy: { ...d.privacy, ...(s.privacy || {}) },
    updates: { ...d.updates, ...(s.updates || {}) },
  };
}
