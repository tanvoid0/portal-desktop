import { TerminalService } from "../services/terminalService";
import type { TerminalOutput, TerminalProcess } from "../types";
import { isTauriEnvironment } from "$lib/utils/tauri";

export interface ProcessConfig {
  tabId: string;
  shell: string;
  workingDirectory: string;
  cols?: number;
  rows?: number;
  /** Oneshot: run this single command and exit with its real code. */
  command?: string;
  /** Extra environment overrides forwarded to the PTY. */
  env?: Record<string, string>;
}

export async function createTerminalProcess(
  config: ProcessConfig,
): Promise<TerminalProcess | null> {
  if (!isTauriEnvironment()) return null;

  return TerminalService.createProcess(config.tabId, {
    shell: config.shell,
    working_directory: config.workingDirectory,
    cols: config.cols ?? 80,
    rows: config.rows ?? 24,
    ...(config.command ? { command: config.command } : {}),
    ...(config.env ? { environment: config.env } : {}),
  });
}

export async function getProcessExitCode(
  processId: string,
): Promise<number | null> {
  if (!isTauriEnvironment()) return null;
  return TerminalService.getProcessExitCode(processId);
}

export async function subscribeProcessOutput(
  processId: string,
  callback: (output: TerminalOutput) => void,
): Promise<() => void> {
  return TerminalService.subscribeToOutput(processId, callback);
}

export async function resizeTerminalProcess(
  processId: string,
  cols: number,
  rows: number,
): Promise<void> {
  await TerminalService.resizeTerminal(processId, cols, rows);
}

export async function sendProcessInput(
  processId: string,
  input: string,
  tabId?: string,
): Promise<void> {
  await TerminalService.sendInput(processId, input, tabId);
}

export async function killTerminalProcess(processId: string): Promise<void> {
  await TerminalService.killProcess(processId);
}
