/**
 * Settings Service - Frontend business logic for settings management
 */

import { invoke } from "@tauri-apps/api/core";
import { logger } from "$lib/domains/shared";
import type {
  AppSettings,
  EditorSettings,
  GitHubIntegrationSettings,
  TerminalSettings,
  ThemeSettings,
} from "../types";

// Define missing types
export interface Settings {
  id: string;
  app: AppSettings;
  editor: EditorSettings;
  terminal: TerminalSettings;
  theme: ThemeSettings;
}

export interface SettingsUpdate {
  app?: Partial<AppSettings>;
  editor?: Partial<EditorSettings>;
  terminal?: Partial<TerminalSettings>;
  theme?: Partial<ThemeSettings>;
}

const DEFAULT_THEME_SETTINGS: ThemeSettings = {
  primaryColor: "#3b82f6",
  secondaryColor: "#64748b",
  accentColor: "#f59e0b",
  backgroundColor: "#ffffff",
  surfaceColor: "#f8fafc",
  textColor: "#1e293b",
  borderRadius: 8,
  shadowIntensity: 0.1,
  animationSpeed: "normal",
  customThemes: [],
  activeTheme: "default",
};

const DEFAULT_GITHUB_INTEGRATION_SETTINGS: GitHubIntegrationSettings = {
  clientId: "",
};

const DEFAULT_APP_SETTINGS: AppSettings = {
  theme: "system",
  language: "en",
  timezone: "UTC",
  dateFormat: "%Y-%m-%d",
  timeFormat: "24h",
  windowState: {
    width: 1200,
    height: 800,
    maximized: false,
    rememberPosition: true,
  },
  startupBehavior: {
    openLastSession: true,
    restoreWindows: true,
    showWelcomeScreen: false,
    minimizeToTray: false,
    startMinimized: false,
  },
  notifications: {
    enabled: true,
    desktopNotifications: true,
    soundEnabled: true,
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
    channel: "stable",
    notifyOnUpdate: true,
  },
  integrations: {
    github: DEFAULT_GITHUB_INTEGRATION_SETTINGS,
  },
};

type RawThemeSettings = Partial<ThemeSettings> & Record<string, unknown>;
type RawAppSettings = Partial<AppSettings> & Record<string, unknown>;

function normalizeAppSettings(
  raw: RawAppSettings | null | undefined,
): AppSettings {
  const app = raw ?? {};

  return {
    theme:
      (app.theme as AppSettings["theme"] | undefined) ??
      DEFAULT_APP_SETTINGS.theme,
    language:
      (app.language as string | undefined) ?? DEFAULT_APP_SETTINGS.language,
    timezone:
      (app.timezone as string | undefined) ?? DEFAULT_APP_SETTINGS.timezone,
    dateFormat:
      (app.dateFormat as string | undefined) ??
      (app.date_format as string | undefined) ??
      DEFAULT_APP_SETTINGS.dateFormat,
    timeFormat:
      (app.timeFormat as AppSettings["timeFormat"] | undefined) ??
      (app.time_format as AppSettings["timeFormat"] | undefined) ??
      DEFAULT_APP_SETTINGS.timeFormat,
    windowState: {
      ...DEFAULT_APP_SETTINGS.windowState,
      ...((app.windowState as Record<string, unknown> | undefined) ?? {}),
      ...((app.window_state as Record<string, unknown> | undefined) ?? {}),
      rememberPosition: Boolean(
        (app.windowState as Record<string, unknown> | undefined)
          ?.rememberPosition ??
          (app.window_state as Record<string, unknown> | undefined)
            ?.remember_position ??
          DEFAULT_APP_SETTINGS.windowState.rememberPosition,
      ),
    },
    startupBehavior: {
      ...DEFAULT_APP_SETTINGS.startupBehavior,
      ...((app.startupBehavior as Record<string, unknown> | undefined) ?? {}),
      ...((app.startup_behavior as Record<string, unknown> | undefined) ?? {}),
      openLastSession: Boolean(
        (app.startupBehavior as Record<string, unknown> | undefined)
          ?.openLastSession ??
          (app.startup_behavior as Record<string, unknown> | undefined)
            ?.open_last_session ??
          DEFAULT_APP_SETTINGS.startupBehavior.openLastSession,
      ),
      restoreWindows: Boolean(
        (app.startupBehavior as Record<string, unknown> | undefined)
          ?.restoreWindows ??
          (app.startup_behavior as Record<string, unknown> | undefined)
            ?.restore_windows ??
          DEFAULT_APP_SETTINGS.startupBehavior.restoreWindows,
      ),
      showWelcomeScreen: Boolean(
        (app.startupBehavior as Record<string, unknown> | undefined)
          ?.showWelcomeScreen ??
          (app.startup_behavior as Record<string, unknown> | undefined)
            ?.show_welcome_screen ??
          DEFAULT_APP_SETTINGS.startupBehavior.showWelcomeScreen,
      ),
      minimizeToTray: Boolean(
        (app.startupBehavior as Record<string, unknown> | undefined)
          ?.minimizeToTray ??
          (app.startup_behavior as Record<string, unknown> | undefined)
            ?.minimize_to_tray ??
          DEFAULT_APP_SETTINGS.startupBehavior.minimizeToTray,
      ),
      startMinimized: Boolean(
        (app.startupBehavior as Record<string, unknown> | undefined)
          ?.startMinimized ??
          (app.startup_behavior as Record<string, unknown> | undefined)
            ?.start_minimized ??
          DEFAULT_APP_SETTINGS.startupBehavior.startMinimized,
      ),
    },
    notifications:
      (app.notifications as AppSettings["notifications"] | undefined) ??
      DEFAULT_APP_SETTINGS.notifications,
    privacy:
      (app.privacy as AppSettings["privacy"] | undefined) ??
      DEFAULT_APP_SETTINGS.privacy,
    updates:
      (app.updates as AppSettings["updates"] | undefined) ??
      DEFAULT_APP_SETTINGS.updates,
    avatarEnabled:
      (app.avatarEnabled as boolean | undefined) ??
      (app.avatar_enabled as boolean | undefined),
    integrations: {
      github: {
        clientId:
          ((app.integrations as Record<string, unknown> | undefined)?.github as
            | Record<string, unknown>
            | undefined)?.clientId?.toString?.() ??
          ((app.integrations as Record<string, unknown> | undefined)?.github as
            | Record<string, unknown>
            | undefined)?.client_id?.toString?.() ??
          ((app.integrations as Record<string, unknown> | undefined)
            ?.github_client_id as string | undefined) ??
          DEFAULT_GITHUB_INTEGRATION_SETTINGS.clientId,
      },
    },
  };
}

function normalizeThemeSettings(
  raw: RawThemeSettings | null | undefined,
): ThemeSettings {
  const theme = raw ?? {};
  const animationSpeed = theme.animationSpeed ?? theme.animation_speed;
  const normalizedSpeed =
    animationSpeed === "slow" ||
    animationSpeed === "normal" ||
    animationSpeed === "fast"
      ? animationSpeed
      : DEFAULT_THEME_SETTINGS.animationSpeed;

  return {
    primaryColor:
      (theme.primaryColor as string | undefined) ??
      (theme.primary_color as string | undefined) ??
      DEFAULT_THEME_SETTINGS.primaryColor,
    secondaryColor:
      (theme.secondaryColor as string | undefined) ??
      (theme.secondary_color as string | undefined) ??
      DEFAULT_THEME_SETTINGS.secondaryColor,
    accentColor:
      (theme.accentColor as string | undefined) ??
      (theme.accent_color as string | undefined) ??
      DEFAULT_THEME_SETTINGS.accentColor,
    backgroundColor:
      (theme.backgroundColor as string | undefined) ??
      (theme.background_color as string | undefined) ??
      DEFAULT_THEME_SETTINGS.backgroundColor,
    surfaceColor:
      (theme.surfaceColor as string | undefined) ??
      (theme.surface_color as string | undefined) ??
      DEFAULT_THEME_SETTINGS.surfaceColor,
    textColor:
      (theme.textColor as string | undefined) ??
      (theme.text_color as string | undefined) ??
      DEFAULT_THEME_SETTINGS.textColor,
    borderRadius: Number(
      theme.borderRadius ?? theme.border_radius ?? DEFAULT_THEME_SETTINGS.borderRadius,
    ),
    shadowIntensity: Number(
      theme.shadowIntensity ??
        theme.shadow_intensity ??
        DEFAULT_THEME_SETTINGS.shadowIntensity,
    ),
    animationSpeed: normalizedSpeed,
    customThemes:
      (theme.customThemes as ThemeSettings["customThemes"] | undefined) ??
      (theme.custom_themes as ThemeSettings["customThemes"] | undefined) ??
      DEFAULT_THEME_SETTINGS.customThemes,
    activeTheme:
      (theme.activeTheme as string | undefined) ??
      (theme.active_theme as string | undefined) ??
      DEFAULT_THEME_SETTINGS.activeTheme,
  };
}

function normalizeSettings(raw: Settings): Settings {
  return {
    ...raw,
    app: normalizeAppSettings(raw.app as RawAppSettings),
    theme: normalizeThemeSettings(raw.theme as RawThemeSettings),
  };
}

function serializeAppSettings(app: AppSettings): Record<string, unknown> {
  return {
    theme: app.theme,
    language: app.language,
    timezone: app.timezone,
    date_format: app.dateFormat,
    time_format: app.timeFormat,
    window_state: {
      width: app.windowState.width,
      height: app.windowState.height,
      x: app.windowState.x,
      y: app.windowState.y,
      maximized: app.windowState.maximized,
      remember_position: app.windowState.rememberPosition,
    },
    startup_behavior: {
      open_last_session: app.startupBehavior.openLastSession,
      restore_windows: app.startupBehavior.restoreWindows,
      show_welcome_screen: app.startupBehavior.showWelcomeScreen,
      minimize_to_tray: app.startupBehavior.minimizeToTray,
      start_minimized: app.startupBehavior.startMinimized,
    },
    notifications: app.notifications,
    privacy: app.privacy,
    updates: app.updates,
    avatar_enabled: app.avatarEnabled,
    integrations: {
      github: {
        client_id: app.integrations?.github?.clientId ?? "",
      },
    },
  };
}

function serializeThemeSettings(theme: ThemeSettings): Record<string, unknown> {
  return {
    primary_color: theme.primaryColor,
    secondary_color: theme.secondaryColor,
    accent_color: theme.accentColor,
    background_color: theme.backgroundColor,
    surface_color: theme.surfaceColor,
    text_color: theme.textColor,
    border_radius: theme.borderRadius,
    shadow_intensity: theme.shadowIntensity,
    animation_speed: theme.animationSpeed,
    custom_themes: theme.customThemes.map((customTheme) => ({
      id: customTheme.id,
      name: customTheme.name,
      description: customTheme.description,
      colors: customTheme.colors,
      created_at:
        customTheme.createdAt instanceof Date
          ? customTheme.createdAt.toISOString()
          : customTheme.createdAt,
      updated_at:
        customTheme.updatedAt instanceof Date
          ? customTheme.updatedAt.toISOString()
          : customTheme.updatedAt,
    })),
    active_theme: theme.activeTheme,
  };
}

function serializeSettingsForRust(
  settings: Settings & Record<string, unknown>,
): Record<string, unknown> {
  const now = new Date().toISOString();
  return {
    ...settings,
    app: serializeAppSettings(settings.app),
    theme: serializeThemeSettings(settings.theme),
    created_at: settings.created_at ?? settings.createdAt ?? now,
    updated_at: settings.updated_at ?? settings.updatedAt ?? now,
  };
}

function serializeSettingsUpdateForRust(
  updates: SettingsUpdate,
): Record<string, unknown> {
  const serialized: Record<string, unknown> = { ...updates };
  if (updates.theme) {
    serialized.theme = serializeThemeSettings(updates.theme);
  }
  return serialized;
}

export class SettingsService {
  private static instance: SettingsService;

  static getInstance(): SettingsService {
    if (!SettingsService.instance) {
      SettingsService.instance = new SettingsService();
    }
    return SettingsService.instance;
  }

  /**
   * Get current settings
   */
  async getSettings(): Promise<Settings> {
    try {
      logger.info("Getting settings", { context: "SettingsService" });

      const settings = await invoke<Settings>("get_settings_command");

      logger.info("Settings retrieved successfully", {
        context: "SettingsService",
        data: { settingsId: settings.id },
      });

      return normalizeSettings(settings);
    } catch (error) {
      logger.error("Failed to get settings", {
        context: "SettingsService",
        error,
      });
      throw error;
    }
  }

  /**
   * Save settings
   */
  async saveSettings(settings: Settings): Promise<void> {
    try {
      logger.info("Saving settings", {
        context: "SettingsService",
        data: { settingsId: settings.id },
      });

      await invoke("save_settings_command", {
        settings: serializeSettingsForRust(
          settings as Settings & Record<string, unknown>,
        ),
      });

      logger.info("Settings saved successfully", {
        context: "SettingsService",
        data: { settingsId: settings.id },
      });
    } catch (error) {
      logger.error("Failed to save settings", {
        context: "SettingsService",
        error,
        data: { settingsId: settings.id },
      });
      throw error;
    }
  }

  /**
   * Update settings
   */
  async updateSettings(
    currentSettings: Settings,
    updates: SettingsUpdate,
  ): Promise<Settings> {
    try {
      logger.info("Updating settings", {
        context: "SettingsService",
        data: { settingsId: currentSettings.id, updates },
      });

      const updatedSettings = await invoke<Settings>(
        "update_settings_command",
        {
          settings: serializeSettingsForRust(
            currentSettings as Settings & Record<string, unknown>,
          ),
          updates: serializeSettingsUpdateForRust(updates),
        },
      );

      logger.info("Settings updated successfully", {
        context: "SettingsService",
        data: { settingsId: updatedSettings.id },
      });

      return normalizeSettings(updatedSettings);
    } catch (error) {
      logger.error("Failed to update settings", {
        context: "SettingsService",
        error,
        data: { settingsId: currentSettings.id },
      });
      throw error;
    }
  }

  /**
   * Reset settings to defaults
   */
  async resetSettings(): Promise<Settings> {
    try {
      logger.info("Resetting settings", { context: "SettingsService" });

      const defaultSettings = await invoke<Settings>("reset_settings_command");

      logger.info("Settings reset successfully", {
        context: "SettingsService",
        data: { settingsId: defaultSettings.id },
      });

      return normalizeSettings(defaultSettings);
    } catch (error) {
      logger.error("Failed to reset settings", {
        context: "SettingsService",
        error,
      });
      throw error;
    }
  }

  /**
   * Export settings
   */
  async exportSettings(settings: Settings): Promise<string> {
    try {
      logger.info("Exporting settings", {
        context: "SettingsService",
        data: { settingsId: settings.id },
      });

      const exportedSettings = await invoke<string>("export_settings_command", {
        settings: serializeSettingsForRust(
          settings as Settings & Record<string, unknown>,
        ),
      });

      logger.info("Settings exported successfully", {
        context: "SettingsService",
        data: { settingsId: settings.id },
      });

      return exportedSettings;
    } catch (error) {
      logger.error("Failed to export settings", {
        context: "SettingsService",
        error,
        data: { settingsId: settings.id },
      });
      throw error;
    }
  }

  /**
   * Import settings
   */
  async importSettings(settingsJson: string): Promise<Settings> {
    try {
      logger.info("Importing settings", { context: "SettingsService" });

      const importedSettings = await invoke<Settings>(
        "import_settings_command",
        {
          settingsJson,
        },
      );

      logger.info("Settings imported successfully", {
        context: "SettingsService",
        data: { settingsId: importedSettings.id },
      });

      return normalizeSettings(importedSettings);
    } catch (error) {
      logger.error("Failed to import settings", {
        context: "SettingsService",
        error,
      });
      throw error;
    }
  }

  /**
   * Validate settings
   */
  validateSettings(settings: Partial<Settings>): string[] {
    const errors: string[] = [];

    // Validate app settings
    if (settings.app) {
      if (
        !settings.app.theme ||
        !["light", "dark", "system"].includes(settings.app.theme)
      ) {
        errors.push("Invalid theme value");
      }
      if (!settings.app.language || settings.app.language.length !== 2) {
        errors.push("Invalid language code");
      }
      if (settings.app.windowState && settings.app.windowState.width < 400) {
        errors.push("Window width must be at least 400px");
      }
      if (settings.app.windowState && settings.app.windowState.height < 300) {
        errors.push("Window height must be at least 300px");
      }
    }

    // Validate editor settings
    if (settings.editor) {
      if (
        settings.editor.fontSize &&
        (settings.editor.fontSize < 8 || settings.editor.fontSize > 72)
      ) {
        errors.push("Font size must be between 8 and 72");
      }
      if (
        settings.editor.tabSize &&
        (settings.editor.tabSize < 1 || settings.editor.tabSize > 8)
      ) {
        errors.push("Tab size must be between 1 and 8");
      }
    }

    // Validate terminal settings
    if (settings.terminal) {
      if (
        settings.terminal.fontSize &&
        (settings.terminal.fontSize < 8 || settings.terminal.fontSize > 72)
      ) {
        errors.push("Terminal font size must be between 8 and 72");
      }
      if (
        settings.terminal.scrollback &&
        (settings.terminal.scrollback < 100 ||
          settings.terminal.scrollback > 10000)
      ) {
        errors.push("Terminal scrollback must be between 100 and 10000");
      }
    }

    return errors;
  }

  /**
   * Get theme colors
   */
  getThemeColors(theme: string): Record<string, string> {
    const themes: Record<string, Record<string, string>> = {
      light: {
        primary: "#3b82f6",
        secondary: "#64748b",
        accent: "#f59e0b",
        background: "#ffffff",
        surface: "#f8fafc",
        text: "#1e293b",
        border: "#e2e8f0",
      },
      dark: {
        primary: "#3b82f6",
        secondary: "#64748b",
        accent: "#f59e0b",
        background: "#0f172a",
        surface: "#1e293b",
        text: "#f1f5f9",
        border: "#334155",
      },
      system: {
        primary: "#3b82f6",
        secondary: "#64748b",
        accent: "#f59e0b",
        background: "var(--background)",
        surface: "var(--surface)",
        text: "var(--text)",
        border: "var(--border)",
      },
    };

    return themes[theme] || themes.system;
  }

  /**
   * Get available themes
   */
  getAvailableThemes(): Array<{
    id: string;
    name: string;
    description: string;
  }> {
    return [
      {
        id: "light",
        name: "Light",
        description: "Light theme with bright colors",
      },
      { id: "dark", name: "Dark", description: "Dark theme with dark colors" },
      {
        id: "system",
        name: "System",
        description: "Follow system theme preference",
      },
    ];
  }

  /**
   * Get available languages
   */
  getAvailableLanguages(): Array<{ id: string; name: string; native: string }> {
    return [
      { id: "en", name: "English", native: "English" },
      { id: "es", name: "Spanish", native: "Español" },
      { id: "fr", name: "French", native: "Français" },
      { id: "de", name: "German", native: "Deutsch" },
      { id: "it", name: "Italian", native: "Italiano" },
      { id: "pt", name: "Portuguese", native: "Português" },
      { id: "ru", name: "Russian", native: "Русский" },
      { id: "ja", name: "Japanese", native: "日本語" },
      { id: "ko", name: "Korean", native: "한국어" },
      { id: "zh", name: "Chinese", native: "中文" },
    ];
  }

  /**
   * Get available fonts
   */
  getAvailableFonts(): Array<{ id: string; name: string; category: string }> {
    return [
      { id: "monaco", name: "Monaco", category: "Monospace" },
      { id: "consolas", name: "Consolas", category: "Monospace" },
      { id: "courier-new", name: "Courier New", category: "Monospace" },
      { id: "fira-code", name: "Fira Code", category: "Monospace" },
      { id: "jetbrains-mono", name: "JetBrains Mono", category: "Monospace" },
      { id: "source-code-pro", name: "Source Code Pro", category: "Monospace" },
      { id: "roboto-mono", name: "Roboto Mono", category: "Monospace" },
      { id: "ubuntu-mono", name: "Ubuntu Mono", category: "Monospace" },
    ];
  }

  /**
   * Get available terminal themes
   */
  getAvailableTerminalThemes(): Array<{
    id: string;
    name: string;
    colors: Record<string, string>;
  }> {
    return [
      {
        id: "default",
        name: "Default",
        colors: {
          background: "#1e1e1e",
          foreground: "#d4d4d4",
          cursor: "#ffffff",
          selection: "#264f78",
        },
      },
      {
        id: "solarized-dark",
        name: "Solarized Dark",
        colors: {
          background: "#002b36",
          foreground: "#839496",
          cursor: "#93a1a1",
          selection: "#073642",
        },
      },
      {
        id: "solarized-light",
        name: "Solarized Light",
        colors: {
          background: "#fdf6e3",
          foreground: "#586e75",
          cursor: "#93a1a1",
          selection: "#eee8d5",
        },
      },
      {
        id: "monokai",
        name: "Monokai",
        colors: {
          background: "#272822",
          foreground: "#f8f8f2",
          cursor: "#f8f8f0",
          selection: "#49483e",
        },
      },
    ];
  }
}

export const settingsService = SettingsService.getInstance();
