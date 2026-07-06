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
  defaultShell: isWindows ? "cmd.exe" : "zsh",
  workingDirectory: isWindows ? "C:\\" : "/home/tan",
};
