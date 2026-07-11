import { describe, expect, it } from "vitest";
import {
  buildExplainPrompt,
  buildTerminalContext,
  parseAiResponse,
  resolveShellMetadata,
} from "./terminalAiContext";

describe("resolveShellMetadata", () => {
  it("detects PowerShell 7", () => {
    expect(
      resolveShellMetadata("C:\\Program Files\\PowerShell\\7\\pwsh.exe").family,
    ).toBe("pwsh");
  });

  it("detects Windows PowerShell", () => {
    expect(
      resolveShellMetadata(
        "C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe",
      ).family,
    ).toBe("powershell");
  });

  it("detects zsh and bash", () => {
    expect(resolveShellMetadata("/bin/zsh").family).toBe("zsh");
    expect(resolveShellMetadata("/usr/bin/bash").family).toBe("bash");
  });
});

describe("buildTerminalContext", () => {
  it("includes shell family constraints and forbids cross-shell syntax", () => {
    const context = buildTerminalContext("tab-test", {
      shell: "/bin/zsh",
      workingDirectory: "/home/user/project",
    });
    expect(context).toContain("Active shell: Zsh");
    expect(context).toContain("Current directory: /home/user/project");
    expect(context).toContain("Do NOT suggest PowerShell");
    expect(context).toContain("```zsh");
  });
});

describe("buildExplainPrompt", () => {
  it("includes command, output, and fix instructions", () => {
    const prompt = buildExplainPrompt({
      command: "asdfasd",
      output: "command not found",
      exitCode: 1,
      workingDirectory: "C:\\Users\\test",
    });
    expect(prompt).toContain("asdfasd");
    expect(prompt).toContain("exit code 1");
    expect(prompt).toContain("C:\\Users\\test");
    expect(prompt).toContain("plain language");
    expect(prompt).toContain("fenced code block");
  });
});

describe("parseAiResponse", () => {
  it("splits text and fenced code segments", () => {
    const segments = parseAiResponse(
      "Use this:\n```powershell\nGet-ChildItem\n```\nThen check output.",
    );
    expect(segments).toEqual([
      { type: "text", content: "Use this:" },
      { type: "code", content: "Get-ChildItem", language: "powershell" },
      { type: "text", content: "Then check output." },
    ]);
  });

  it("handles responses without code blocks", () => {
    expect(parseAiResponse("plain answer")).toEqual([
      { type: "text", content: "plain answer" },
    ]);
  });

  it("handles unlabeled fences and multiple blocks", () => {
    const segments = parseAiResponse("```\nls\n```\n```\npwd\n```");
    expect(segments.filter((s) => s.type === "code").map((s) => s.content))
      .toEqual(["ls", "pwd"]);
  });
});
