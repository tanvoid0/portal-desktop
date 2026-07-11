/**
 * Builds AI prompt context from terminal state (Warp-style): recent command
 * blocks, cwd, shell family, OS — so the AI answers about *this* session and
 * suggests commands in the correct shell syntax.
 */

import { commandBlockStore } from "../stores/commandBlockStore";
import { formatCommandOutput } from "../utils/textUtils";

const MAX_BLOCKS = 5;
const MAX_OUTPUT_CHARS = 1500;

export type ShellFamily =
  | "powershell"
  | "pwsh"
  | "bash"
  | "zsh"
  | "cmd"
  | "fish"
  | "sh"
  | "unknown";

export interface ShellMetadata {
  family: ShellFamily;
  displayName: string;
  executable: string;
  fenceLanguage: string;
}

export interface TerminalContextOptions {
  shell?: string;
  workingDirectory?: string;
}

/** Normalize a shell executable/path into a family the AI can reason about. */
export function resolveShellMetadata(shell?: string): ShellMetadata {
  const executable = shell?.trim() || "unknown";
  const lower = executable.toLowerCase();

  if (lower.includes("pwsh") || /powershell.*7/i.test(lower)) {
    return {
      family: "pwsh",
      displayName: "PowerShell 7+",
      executable,
      fenceLanguage: "powershell",
    };
  }
  if (/powershell/i.test(lower)) {
    return {
      family: "powershell",
      displayName: "Windows PowerShell",
      executable,
      fenceLanguage: "powershell",
    };
  }
  if (/zsh/i.test(lower)) {
    return {
      family: "zsh",
      displayName: "Zsh",
      executable,
      fenceLanguage: "zsh",
    };
  }
  if (/fish/i.test(lower)) {
    return {
      family: "fish",
      displayName: "Fish",
      executable,
      fenceLanguage: "fish",
    };
  }
  if (/bash/i.test(lower)) {
    return {
      family: "bash",
      displayName: "Bash",
      executable,
      fenceLanguage: "bash",
    };
  }
  if (/cmd\.exe/i.test(lower) || /(^|[\\/])cmd$/i.test(lower)) {
    return {
      family: "cmd",
      displayName: "Windows Command Prompt (cmd.exe)",
      executable,
      fenceLanguage: "cmd",
    };
  }
  if (/\/sh$/i.test(lower) || /\bsh$/i.test(lower)) {
    return {
      family: "sh",
      displayName: "POSIX shell",
      executable,
      fenceLanguage: "sh",
    };
  }

  return {
    family: "unknown",
    displayName: executable || "unknown shell",
    executable,
    fenceLanguage: "text",
  };
}

function shellConstraintLines(meta: ShellMetadata, os: string): string[] {
  const lines: string[] = [
    `Shell family: ${meta.family}`,
    `You MUST only suggest commands valid for ${meta.displayName}.`,
  ];

  if (meta.family === "powershell" || meta.family === "pwsh") {
    lines.push(
      `Do NOT suggest bash, zsh, or cmd syntax.`,
      `Use PowerShell cmdlets (Get-ChildItem, Set-Location, Remove-Item), $env:VAR for env vars, and ; to chain commands.`,
      `Label runnable commands with a \`\`\`${meta.fenceLanguage} fence.`,
    );
  } else if (
    meta.family === "bash" ||
    meta.family === "zsh" ||
    meta.family === "fish" ||
    meta.family === "sh"
  ) {
    lines.push(
      `Do NOT suggest PowerShell or cmd syntax.`,
      `Use Unix paths, $VAR / export for environment variables, and standard ${meta.displayName} syntax.`,
      `Label runnable commands with a \`\`\`${meta.fenceLanguage} fence.`,
    );
  } else if (meta.family === "cmd") {
    lines.push(
      `Do NOT suggest PowerShell or bash/zsh syntax.`,
      `Use cmd.exe builtins (dir, cd, del, set).`,
      `Label runnable commands with a \`\`\`cmd fence.`,
    );
  } else {
    lines.push(
      `Infer syntax from recent commands in the history below.`,
      `Label runnable commands with an appropriate fenced code block.`,
    );
  }

  lines.push(
    os === "Windows"
      ? `Platform notes: Windows paths use backslashes (\\); line endings are CRLF.`
      : `Platform notes: Unix paths use forward slashes (/); line endings are LF.`,
  );

  return lines;
}

export function buildTerminalContext(
  tabId: string,
  opts: TerminalContextOptions = {},
): string {
  const os = navigator.userAgent.includes("Windows")
    ? "Windows"
    : navigator.userAgent.includes("Mac")
      ? "macOS"
      : "Linux";

  const shellMeta = resolveShellMetadata(opts.shell);
  const recent = commandBlockStore
    .getBlocksForTab(tabId)
    .filter((b) => b.source !== "ai")
    .slice(0, MAX_BLOCKS);

  // Prefer cwd from the latest real command (shell integration) over the
  // process start directory, which can be stale after cd.
  const cwd =
    recent[0]?.workingDirectory || opts.workingDirectory;

  const lines: string[] = [
    `You are an AI assistant embedded in a desktop terminal.`,
    `OS: ${os}.`,
    `Active shell: ${shellMeta.displayName}`,
  ];
  if (shellMeta.executable && shellMeta.executable !== "unknown") {
    lines.push(`Shell executable: ${shellMeta.executable}`);
  }
  if (cwd) lines.push(`Current directory: ${cwd}`);
  lines.push(...shellConstraintLines(shellMeta, os));

  if (recent.length > 0) {
    lines.push(``, `Recent commands (newest first):`);
    for (const b of recent) {
      lines.push(
        `$ ${b.command}` +
          (b.exitCode !== undefined ? ` (exit ${b.exitCode})` : " (running)") +
          (b.workingDirectory ? ` [cwd: ${b.workingDirectory}]` : ""),
      );
      const out = formatCommandOutput(b.output, MAX_OUTPUT_CHARS);
      if (out) lines.push(out);
    }
  }

  lines.push(
    ``,
    `When you suggest a command the user should run, put it alone in a fenced` +
      ` \`\`\`${shellMeta.fenceLanguage} block so it can be executed with one click.`,
  );
  return lines.join("\n");
}

/** Prompt for the "Explain" action on a failed block — includes session context. */
export function buildExplainPrompt(
  block: {
    command: string;
    output: string;
    exitCode?: number;
    workingDirectory?: string;
  },
  tabId?: string,
  opts: TerminalContextOptions = {},
): string {
  const context = tabId ? buildTerminalContext(tabId, opts) : "";
  const output = formatCommandOutput(block.output, MAX_OUTPUT_CHARS) || "(no output)";

  return [
    context,
    context ? `---` : ``,
    `The following command failed${block.exitCode !== undefined ? ` with exit code ${block.exitCode}` : ""}:`,
    "```",
    block.command,
    "```",
    block.workingDirectory ? `Working directory: ${block.workingDirectory}` : ``,
    `Error output:`,
    "```",
    output,
    "```",
    ``,
    `Using the recent terminal history above, explain in plain language what went wrong.`,
    `Suggest a concrete fix and provide a corrected command in a fenced code block the user can run with one click.`,
  ]
    .filter(Boolean)
    .join("\n");
}

export interface AiResponseSegment {
  type: "text" | "code";
  content: string;
  language?: string;
}

/** Split an AI response into text and fenced-code segments for rendering
 *  runnable command suggestions. */
export function parseAiResponse(response: string): AiResponseSegment[] {
  const segments: AiResponseSegment[] = [];
  const fence = /```([^\n`]*)\n([\s\S]*?)```/g;
  let last = 0;
  let m: RegExpExecArray | null;
  while ((m = fence.exec(response))) {
    if (m.index > last) {
      const text = response.slice(last, m.index).trim();
      if (text) segments.push({ type: "text", content: text });
    }
    const code = m[2].trim();
    if (code) {
      segments.push({ type: "code", content: code, language: m[1].trim() });
    }
    last = m.index + m[0].length;
  }
  const tail = response.slice(last).trim();
  if (tail) segments.push({ type: "text", content: tail });
  return segments;
}
