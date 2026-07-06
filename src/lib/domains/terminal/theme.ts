/**
 * xterm.js theme resolution.
 *
 * Replaces the hardcoded `#0c0c0c`/`#cccccc` colors that used to live inline in
 * the terminal session. Colors are read from the app's CSS custom properties so
 * the terminal tracks the active design theme; each var has a safe fallback for
 * SSR / non-browser contexts.
 */

import type { ITheme } from "@xterm/xterm";
import type { TerminalConfig } from "./types";

/** Read a CSS custom property from :root, falling back when unavailable. */
function cssVar(name: string, fallback: string): string {
  if (typeof document === "undefined") return fallback;
  const value = getComputedStyle(document.documentElement)
    .getPropertyValue(name)
    .trim();
  return value || fallback;
}

const DARK_FALLBACK: ITheme = {
  background: "#0c0c0c",
  foreground: "#cccccc",
  cursor: "#ffffff",
  cursorAccent: "#0c0c0c",
  selectionBackground: "#ffffff40",
};

const LIGHT_FALLBACK: ITheme = {
  background: "#ffffff",
  foreground: "#1e1e1e",
  cursor: "#1e1e1e",
  cursorAccent: "#ffffff",
  selectionBackground: "#00000022",
};

/**
 * Build the xterm `ITheme` for the given terminal config.
 *
 * When the app exposes `--terminal-*` CSS variables they win; otherwise the
 * config's `theme` ("dark" | "light" | "auto") selects a sensible fallback.
 * A per-instance `override` is merged last for callers that need a specific look.
 */
export function resolveXtermTheme(
  config: Pick<TerminalConfig, "theme">,
  override?: Partial<ITheme>,
): ITheme {
  let prefersDark = config.theme !== "light";
  if (config.theme === "auto" && typeof window !== "undefined") {
    prefersDark = window.matchMedia?.("(prefers-color-scheme: dark)").matches ?? true;
  }
  const base = prefersDark ? DARK_FALLBACK : LIGHT_FALLBACK;

  const resolved: ITheme = {
    background: cssVar("--terminal-background", base.background!),
    foreground: cssVar("--terminal-foreground", base.foreground!),
    cursor: cssVar("--terminal-cursor", base.cursor!),
    cursorAccent: cssVar("--terminal-cursor-accent", base.cursorAccent!),
    selectionBackground: cssVar(
      "--terminal-selection",
      base.selectionBackground!,
    ),
  };

  return { ...resolved, ...override };
}
