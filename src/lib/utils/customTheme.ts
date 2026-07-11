/**
 * Applies saved ThemeSettings hex colors to CSS custom properties.
 * Values use the shadcn HSL token format: "H S% L%" (no hsl() wrapper).
 */

import type { ThemeSettings } from "$lib/domains/settings/types";

const DEFAULT_THEME_ID = "default";

const DEFAULT_COLORS = {
  primary: "#3b82f6",
  secondary: "#64748b",
  accent: "#f59e0b",
  background: "#ffffff",
  surface: "#f8fafc",
  text: "#1e293b",
} as const;

const MANAGED_CSS_VARS = [
  "--primary",
  "--primary-foreground",
  "--secondary",
  "--secondary-foreground",
  "--accent",
  "--accent-foreground",
  "--background",
  "--foreground",
  "--card",
  "--card-foreground",
  "--muted",
  "--muted-foreground",
  "--popover",
  "--popover-foreground",
  "--border",
  "--input",
  "--ring",
  "--radius",
  "--gradient-from",
  "--gradient-to",
  "--card-gradient-from",
  "--card-gradient-to",
] as const;

/** Surface tokens — keep CSS `.dark` values in dark mode; only apply custom hex in light mode */
const SURFACE_CSS_VARS = [
  "--background",
  "--foreground",
  "--card",
  "--card-foreground",
  "--muted",
  "--muted-foreground",
  "--popover",
  "--popover-foreground",
  "--border",
  "--input",
  "--gradient-from",
  "--gradient-to",
  "--card-gradient-from",
  "--card-gradient-to",
] as const;

function clearSurfaceOverrides(html: HTMLElement): void {
  for (const variable of SURFACE_CSS_VARS) {
    html.style.removeProperty(variable);
  }
}

function ensureHex(hex: string | undefined | null, fallback: string): string {
  return typeof hex === "string" && hex.length > 0 ? hex : fallback;
}

export function hexToHslComponents(hex: string | undefined | null): string {
  if (typeof hex !== "string" || hex.length === 0) {
    return "0 0% 50%";
  }

  const normalized = hex.replace(/^#/, "");
  const expanded =
    normalized.length === 3
      ? normalized
          .split("")
          .map((c) => c + c)
          .join("")
      : normalized;

  if (expanded.length !== 6) {
    return "0 0% 50%";
  }

  const r = parseInt(expanded.slice(0, 2), 16) / 255;
  const g = parseInt(expanded.slice(2, 4), 16) / 255;
  const b = parseInt(expanded.slice(4, 6), 16) / 255;

  const max = Math.max(r, g, b);
  const min = Math.min(r, g, b);
  let h = 0;
  let s = 0;
  const l = (max + min) / 2;

  if (max !== min) {
    const d = max - min;
    s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
    switch (max) {
      case r:
        h = ((g - b) / d + (g < b ? 6 : 0)) / 6;
        break;
      case g:
        h = ((b - r) / d + 2) / 6;
        break;
      case b:
        h = ((r - g) / d + 4) / 6;
        break;
    }
  }

  return `${Math.round(h * 360)} ${Math.round(s * 100)}% ${Math.round(l * 100)}%`;
}

function getRelativeLuminance(hex: string | undefined | null): number {
  const safeHex = ensureHex(hex, DEFAULT_COLORS.primary);
  const normalized = safeHex.replace(/^#/, "");
  const expanded =
    normalized.length === 3
      ? normalized
          .split("")
          .map((c) => c + c)
          .join("")
      : normalized;

  const channels = [
    parseInt(expanded.slice(0, 2), 16) / 255,
    parseInt(expanded.slice(2, 4), 16) / 255,
    parseInt(expanded.slice(4, 6), 16) / 255,
  ].map((c) => (c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4));

  return 0.2126 * channels[0] + 0.7152 * channels[1] + 0.0722 * channels[2];
}

function contrastingForeground(hex: string | undefined | null): string {
  return getRelativeLuminance(hex) > 0.5 ? "0 0% 9%" : "0 0% 98%";
}

function resolveColors(settings: ThemeSettings) {
  const activeTheme = settings.activeTheme ?? DEFAULT_THEME_ID;
  if (activeTheme !== DEFAULT_THEME_ID) {
    const custom = settings.customThemes?.find(
      (theme) => theme.id === activeTheme,
    );
    if (custom) {
      return {
        primary: ensureHex(custom.colors.primary, DEFAULT_COLORS.primary),
        secondary: ensureHex(custom.colors.secondary, DEFAULT_COLORS.secondary),
        accent: ensureHex(custom.colors.accent, DEFAULT_COLORS.accent),
        background: ensureHex(custom.colors.background, DEFAULT_COLORS.background),
        surface: ensureHex(custom.colors.surface, DEFAULT_COLORS.surface),
        text: ensureHex(custom.colors.text, DEFAULT_COLORS.text),
      };
    }
  }

  return {
    primary: ensureHex(settings.primaryColor, DEFAULT_COLORS.primary),
    secondary: ensureHex(settings.secondaryColor, DEFAULT_COLORS.secondary),
    accent: ensureHex(settings.accentColor, DEFAULT_COLORS.accent),
    background: ensureHex(settings.backgroundColor, DEFAULT_COLORS.background),
    surface: ensureHex(settings.surfaceColor, DEFAULT_COLORS.surface),
    text: ensureHex(settings.textColor, DEFAULT_COLORS.text),
  };
}

function usesDefaultPalette(settings: ThemeSettings): boolean {
  const activeTheme = settings.activeTheme ?? DEFAULT_THEME_ID;
  if (activeTheme !== DEFAULT_THEME_ID) {
    return false;
  }

  const colors = resolveColors(settings);
  return (
    colors.primary === DEFAULT_COLORS.primary &&
    colors.secondary === DEFAULT_COLORS.secondary &&
    colors.accent === DEFAULT_COLORS.accent &&
    colors.background === DEFAULT_COLORS.background &&
    colors.surface === DEFAULT_COLORS.surface &&
    colors.text === DEFAULT_COLORS.text
  );
}

export function resetCustomTheme(): void {
  if (typeof document === "undefined") return;

  const html = document.documentElement;
  for (const variable of MANAGED_CSS_VARS) {
    html.style.removeProperty(variable);
  }
}

export function applyCustomTheme(settings: ThemeSettings): void {
  if (typeof document === "undefined") return;

  if (usesDefaultPalette(settings)) {
    resetCustomTheme();
    return;
  }

  const colors = resolveColors(settings);
  const html = document.documentElement;

  const primary = hexToHslComponents(colors.primary);
  const secondary = hexToHslComponents(colors.secondary);
  const accent = hexToHslComponents(colors.accent);
  const background = hexToHslComponents(colors.background);
  const foreground = hexToHslComponents(colors.text);
  const surface = hexToHslComponents(colors.surface);

  html.style.setProperty("--primary", primary);
  html.style.setProperty(
    "--primary-foreground",
    contrastingForeground(colors.primary),
  );
  html.style.setProperty("--secondary", secondary);
  html.style.setProperty(
    "--secondary-foreground",
    contrastingForeground(colors.secondary),
  );
  html.style.setProperty("--accent", accent);
  html.style.setProperty(
    "--accent-foreground",
    contrastingForeground(colors.accent),
  );

  const isDark = html.classList.contains("dark");
  if (isDark) {
    // Brand colors only — structural dark tokens come from app.css `.dark`
    clearSurfaceOverrides(html);
    return;
  }

  html.style.setProperty("--background", background);
  html.style.setProperty("--foreground", foreground);
  html.style.setProperty("--card", surface);
  html.style.setProperty("--card-foreground", foreground);
  html.style.setProperty("--muted", secondary);
  html.style.setProperty("--muted-foreground", foreground);
  html.style.setProperty("--popover", surface);
  html.style.setProperty("--popover-foreground", foreground);
  html.style.setProperty("--border", secondary);
  html.style.setProperty("--input", secondary);
  html.style.setProperty("--ring", primary);
  html.style.setProperty(
    "--radius",
    `${(settings.borderRadius ?? 8) / 16}rem`,
  );
  html.style.setProperty("--gradient-from", background);
  html.style.setProperty("--gradient-to", surface);
  html.style.setProperty("--card-gradient-from", surface);
  html.style.setProperty("--card-gradient-to", background);
}
