import {
  resolveShellMetadata,
  type ShellFamily,
} from "../services/terminalAiContext";

const SHELL_ICONS: Record<ShellFamily, string> = {
  powershell: "codicon:terminal-powershell",
  pwsh: "codicon:terminal-powershell",
  bash: "codicon:terminal-bash",
  zsh: "logos:zsh",
  fish: "simple-icons:fishshell",
  cmd: "codicon:terminal-cmd",
  sh: "codicon:terminal-bash",
  unknown: "codicon:terminal",
};

const TAB_TYPE_ICONS: Record<string, string> = {
  editor: "mdi:file-document-edit-outline",
  file: "mdi:file-outline",
  custom: "codicon:terminal",
};

export const TERMINAL_ICONS = {
  container: "logos:docker-icon",
  project: "mdi:folder-outline",
  default: "codicon:terminal",
  wsl: "mdi:linux",
} as const;

/** Legacy emoji icons persisted in localStorage before SVG migration. */
const LEGACY_EMOJI_ICONS: Record<string, string> = {
  "💙": SHELL_ICONS.powershell,
  "🐧": SHELL_ICONS.bash,
  "⚡": SHELL_ICONS.zsh,
  "🐠": SHELL_ICONS.fish,
  "🖥️": SHELL_ICONS.cmd,
  "💻": TERMINAL_ICONS.default,
  "🐳": TERMINAL_ICONS.container,
  "📝": TAB_TYPE_ICONS.editor,
  "📄": TAB_TYPE_ICONS.file,
  "📋": TAB_TYPE_ICONS.custom,
};

export function isIconifyIcon(icon: string): boolean {
  return icon.includes(":");
}

export function normalizeTerminalIcon(icon?: string): string | undefined {
  if (!icon) return undefined;
  if (isIconifyIcon(icon)) return icon;
  return LEGACY_EMOJI_ICONS[icon] ?? icon;
}

/** Resolve a shell executable, profile name, or backend icon key to an Iconify id. */
export function resolveShellIcon(shellOrProfile?: string): string {
  if (!shellOrProfile?.trim()) return TERMINAL_ICONS.default;

  const lower = shellOrProfile.toLowerCase();

  if (lower.includes("docker") || lower.includes("container")) {
    return TERMINAL_ICONS.container;
  }
  if (lower.includes("wsl")) {
    return TERMINAL_ICONS.wsl;
  }
  if (
    lower.includes("command prompt") ||
    /\bcmd\b/.test(lower)
  ) {
    return SHELL_ICONS.cmd;
  }
  if (lower.includes("git bash") || lower.includes("bash")) {
    return SHELL_ICONS.bash;
  }
  if (lower.includes("powershell") || lower.includes("pwsh")) {
    return SHELL_ICONS.powershell;
  }

  const knownKeys: Record<string, string> = {
    powershell: SHELL_ICONS.powershell,
    pwsh: SHELL_ICONS.pwsh,
    bash: SHELL_ICONS.bash,
    zsh: SHELL_ICONS.zsh,
    fish: SHELL_ICONS.fish,
    cmd: SHELL_ICONS.cmd,
    sh: SHELL_ICONS.sh,
  };

  const keyMatch = knownKeys[lower];
  if (keyMatch) return keyMatch;

  return SHELL_ICONS[resolveShellMetadata(shellOrProfile).family];
}

export function resolveTabIcon(tab: {
  icon?: string;
  shell?: string;
  type?: string;
  resourceName?: string;
}): string {
  const normalizedIcon = normalizeTerminalIcon(tab.icon);
  if (normalizedIcon && isIconifyIcon(normalizedIcon)) {
    return normalizedIcon;
  }

  if (tab.resourceName === "container") return TERMINAL_ICONS.container;
  if (tab.shell) return resolveShellIcon(tab.shell);
  if (tab.type && TAB_TYPE_ICONS[tab.type]) return TAB_TYPE_ICONS[tab.type];
  return TERMINAL_ICONS.default;
}
