import { defaultTerminalConfig } from "../config/defaultTerminalConfig";
import { resolveShellMetadata } from "../services/terminalAiContext";
import type { TerminalConfig } from "../types";
import type { TerminalTab } from "../stores/terminalStore";

const INTEGRATION_FAMILIES = new Set([
  "powershell",
  "pwsh",
  "zsh",
  "bash",
]);

/** Shells that support OSC 133 command blocks (Warp-style UI). */
export function shellSupportsIntegration(shell?: string): boolean {
  return INTEGRATION_FAMILIES.has(resolveShellMetadata(shell).family);
}

/** Map a shell path/name to the executable the backend expects. */
export function canonicalShellExecutable(shell?: string): string {
  switch (resolveShellMetadata(shell).family) {
    case "powershell":
      return "powershell.exe";
    case "pwsh":
      return "pwsh.exe";
    case "bash":
      return "bash";
    case "zsh":
      return "zsh";
    case "fish":
      return "fish";
    case "cmd":
      return "cmd.exe";
    default:
      return shell?.trim() || defaultTerminalConfig.defaultShell;
  }
}

/**
 * Merge tab + workspace settings for a TerminalSession.
 * Prefers the workspace default when a persisted tab still references cmd.exe
 * but the host (e.g. ProjectTerminal) defaults to PowerShell.
 */
export function resolveSessionSettings(
  tab: TerminalTab,
  workspaceSettings: TerminalConfig,
): TerminalConfig {
  const tabShell = tab.shell?.trim() || "";
  const workspaceShell = workspaceSettings.defaultShell;

  let shell = tabShell || workspaceShell;
  if (
    !shellSupportsIntegration(shell) &&
    shellSupportsIntegration(workspaceShell)
  ) {
    shell = workspaceShell;
  }

  const workingDirectory =
    tab.workingDirectory?.trim() || workspaceSettings.workingDirectory;

  return {
    ...workspaceSettings,
    defaultShell: canonicalShellExecutable(shell),
    workingDirectory,
  };
}

/** Upgrade stale tabs (cmd / wrong cwd) when opening a project terminal. */
export function projectTabNeedsMigration(
  tab: TerminalTab,
  projectId: string,
  projectPath: string,
  preferredShell: string,
): boolean {
  if (tab.resourceName !== "project" || tab.resourceId !== projectId) {
    return false;
  }
  const shellStale =
    !shellSupportsIntegration(tab.shell) &&
    shellSupportsIntegration(preferredShell);
  const cwdStale =
    !!projectPath &&
    !!tab.workingDirectory &&
    tab.workingDirectory !== projectPath;
  return shellStale || cwdStale;
}

export function migratedProjectTabFields(
  projectPath: string,
  preferredShell: string,
): Pick<TerminalTab, "shell" | "workingDirectory"> {
  return {
    shell: canonicalShellExecutable(preferredShell),
    workingDirectory: projectPath,
  };
}
