import type { TerminalConfig } from "../types";

const isWindows =
  typeof window !== "undefined" && navigator.userAgent.includes("Windows");

export const defaultTerminalConfig: TerminalConfig = {
  theme: "dark",
  fontSize: 14,
  fontFamily: 'Monaco, Consolas, "Courier New", monospace',
  cursorStyle: "block",
  scrollbackLines: 1000,
  bellSound: false,
  autoClose: true,
  confirmClose: true,
  // PowerShell (not cmd) on Windows: cmd has no hooks for OSC 133 command
  // tracking, so blocks/AI capture only work in powershell/pwsh/zsh/bash.
  defaultShell: isWindows ? "powershell.exe" : "zsh",
  // Empty → backend falls back to a sensible directory (home).
  workingDirectory: "",
};
