import { describe, expect, it } from "vitest";
import {
  normalizeTerminalIcon,
  resolveShellIcon,
  resolveTabIcon,
  TERMINAL_ICONS,
} from "./shellIcons";

describe("resolveShellIcon", () => {
  it("maps common shells to iconify ids", () => {
    expect(resolveShellIcon("powershell.exe")).toBe("codicon:terminal-powershell");
    expect(resolveShellIcon("pwsh.exe")).toBe("codicon:terminal-powershell");
    expect(resolveShellIcon("/bin/zsh")).toBe("logos:zsh");
    expect(resolveShellIcon("bash")).toBe("codicon:terminal-bash");
    expect(resolveShellIcon("cmd.exe")).toBe("codicon:terminal-cmd");
    expect(resolveShellIcon("fish")).toBe("simple-icons:fishshell");
  });

  it("maps profile names and container commands", () => {
    expect(resolveShellIcon("PowerShell")).toBe("codicon:terminal-powershell");
    expect(resolveShellIcon("PowerShell Core")).toBe("codicon:terminal-powershell");
    expect(resolveShellIcon("Command Prompt")).toBe("codicon:terminal-cmd");
    expect(resolveShellIcon("Git Bash")).toBe("codicon:terminal-bash");
    expect(resolveShellIcon("docker exec -it abc bash")).toBe(
      TERMINAL_ICONS.container,
    );
    expect(resolveShellIcon("wsl.exe")).toBe(TERMINAL_ICONS.wsl);
  });
});

describe("resolveTabIcon", () => {
  it("prefers explicit iconify icons", () => {
    expect(
      resolveTabIcon({
        icon: "logos:docker-icon",
        shell: "bash",
      }),
    ).toBe("logos:docker-icon");
  });

  it("migrates legacy emoji icons", () => {
    expect(resolveTabIcon({ icon: "🐳" })).toBe(TERMINAL_ICONS.container);
    expect(resolveTabIcon({ icon: "💙" })).toBe("codicon:terminal-powershell");
  });

  it("falls back to shell and tab type", () => {
    expect(resolveTabIcon({ shell: "/bin/zsh" })).toBe("logos:zsh");
    expect(resolveTabIcon({ type: "editor" })).toBe(
      "mdi:file-document-edit-outline",
    );
    expect(resolveTabIcon({ resourceName: "container" })).toBe(
      TERMINAL_ICONS.container,
    );
  });
});

describe("normalizeTerminalIcon", () => {
  it("passes through iconify ids unchanged", () => {
    expect(normalizeTerminalIcon("logos:zsh")).toBe("logos:zsh");
  });
});
