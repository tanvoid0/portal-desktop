/**
 * Best-effort parser for common shell / CLI error output.
 * Shells format errors differently — this extracts a human-friendly summary
 * when a known pattern matches, and falls back to the first meaningful line.
 */

export type ErrorCategory =
  | "command-not-found"
  | "permission-denied"
  | "file-not-found"
  | "syntax-error"
  | "network-error"
  | "package-error"
  | "git-error"
  | "unknown";

export interface ParsedTerminalError {
  category: ErrorCategory;
  title: string;
  message: string;
  hint?: string;
  details: string[];
  raw: string;
}

const DETAIL_SKIP =
  /^(?:\+|At line:|CategoryInfo\s*:|FullyQualifiedErrorId\s*:|~~+)/i;

export const ERROR_CATEGORY_LABELS: Record<ErrorCategory, string> = {
  "command-not-found": "Command not found",
  "permission-denied": "Permission denied",
  "file-not-found": "File not found",
  "syntax-error": "Syntax error",
  "network-error": "Network error",
  "package-error": "Package error",
  "git-error": "Git error",
  unknown: "Error",
};

function extractDetails(text: string): string[] {
  return text
    .split("\n")
    .map((l) => l.trim())
    .filter((l) => l && !DETAIL_SKIP.test(l))
    .slice(0, 6);
}

function firstMeaningfulLine(text: string): string {
  const line = text
    .split("\n")
    .map((l) => l.trim())
    .find((l) => l && !DETAIL_SKIP.test(l));
  return line ?? "Command failed";
}

/** Parse cleaned terminal output into a structured error, or null if empty. */
export function parseTerminalError(
  output: string,
  command?: string,
): ParsedTerminalError | null {
  const raw = output.trim();
  if (!raw) return null;

  // PowerShell: term not recognized
  let m = raw.match(
    /The term '([^']+)' is not recognized as the name of a cmdlet, function, script file, or operable program/i,
  );
  if (m) {
    const term = m[1];
    return {
      category: "command-not-found",
      title: `'${term}' is not a recognized command`,
      message: `PowerShell could not find a cmdlet, function, or script named '${term}'.`,
      hint: "Check the spelling, or install the tool if it is a third-party command.",
      details: extractDetails(raw),
      raw,
    };
  }

  // zsh: command not found: foo
  m = raw.match(/zsh:\s*command not found:\s*(.+)/i);
  if (m) {
    const term = m[1].trim();
    return {
      category: "command-not-found",
      title: `'${term}' is not a recognized command`,
      message: `zsh could not find '${term}' in your PATH.`,
      hint: "Install the package or check that the command name is spelled correctly.",
      details: extractDetails(raw),
      raw,
    };
  }

  // bash/fish: foo: command not found
  m = raw.match(/([^\s:]+):\s*command not found/i);
  if (m) {
    const term = m[1].trim();
    return {
      category: "command-not-found",
      title: `'${term}' is not a recognized command`,
      message: `The shell could not find '${term}' in your PATH.`,
      hint: "Install the package or verify the command name.",
      details: extractDetails(raw),
      raw,
    };
  }

  // generic: command not found
  if (/command not found/i.test(raw)) {
    const term = command?.split(/\s+/)[0] ?? "command";
    return {
      category: "command-not-found",
      title: `'${term}' is not a recognized command`,
      message: firstMeaningfulLine(raw),
      hint: "Install the required tool or check the spelling.",
      details: extractDetails(raw),
      raw,
    };
  }

  // PowerShell: Cannot find path
  m = raw.match(/Cannot find path '([^']+)'/i);
  if (m) {
    return {
      category: "file-not-found",
      title: `Path not found: ${m[1]}`,
      message: `PowerShell could not find '${m[1]}'.`,
      hint: "Check that the file or directory exists and the path is correct.",
      details: extractDetails(raw),
      raw,
    };
  }

  // Unix: No such file or directory
  m = raw.match(/([^:\n]+):\s*(.+?):\s*No such file or directory/i);
  if (m) {
    return {
      category: "file-not-found",
      title: "File or directory not found",
      message: m[0].trim(),
      hint: "Verify the path exists and you have the correct working directory.",
      details: extractDetails(raw),
      raw,
    };
  }
  if (/No such file or directory/i.test(raw)) {
    return {
      category: "file-not-found",
      title: "File or directory not found",
      message: firstMeaningfulLine(raw),
      hint: "Verify the path exists and you have the correct working directory.",
      details: extractDetails(raw),
      raw,
    };
  }

  // Permission denied
  if (/Permission denied|Access is denied/i.test(raw)) {
    return {
      category: "permission-denied",
      title: "Permission denied",
      message: firstMeaningfulLine(raw),
      hint: "You may need elevated privileges (sudo / Run as Administrator) or different file permissions.",
      details: extractDetails(raw),
      raw,
    };
  }

  // npm / yarn / pnpm
  m = raw.match(/npm ERR!\s*(.+)/i);
  if (m) {
    return {
      category: "package-error",
      title: "npm error",
      message: m[1].trim(),
      hint: "Check package.json, run npm install, or verify the script name.",
      details: extractDetails(raw),
      raw,
    };
  }
  if (/Cannot find module '([^']+)'/i.test(raw)) {
    m = raw.match(/Cannot find module '([^']+)'/i);
    return {
      category: "package-error",
      title: `Module not found: ${m?.[1] ?? "unknown"}`,
      message: firstMeaningfulLine(raw),
      hint: "Run npm install or add the missing dependency.",
      details: extractDetails(raw),
      raw,
    };
  }

  // git
  m = raw.match(/fatal:\s*(.+)/i);
  if (m) {
    return {
      category: "git-error",
      title: "Git error",
      message: m[1].trim(),
      hint: "Check your repository state, branch, and remote configuration.",
      details: extractDetails(raw),
      raw,
    };
  }

  // network
  if (
    /Could not resolve host|Connection refused|ECONNREFUSED|ETIMEDOUT|Network is unreachable/i.test(
      raw,
    )
  ) {
    return {
      category: "network-error",
      title: "Network error",
      message: firstMeaningfulLine(raw),
      hint: "Check your internet connection, proxy settings, or the target URL.",
      details: extractDetails(raw),
      raw,
    };
  }

  // syntax
  if (
    /ParserError|syntax error|Unexpected token|Parse error|invalid syntax/i.test(
      raw,
    )
  ) {
    return {
      category: "syntax-error",
      title: "Syntax error",
      message: firstMeaningfulLine(raw),
      hint: "Review the command for typos, missing quotes, or incorrect flags.",
      details: extractDetails(raw),
      raw,
    };
  }

  // Fallback — first meaningful line as title
  const details = extractDetails(raw);
  const title = firstMeaningfulLine(raw);
  return {
    category: "unknown",
    title: title.length > 100 ? `${title.slice(0, 97)}…` : title,
    message:
      details.length > 1
        ? details.slice(1, 4).join("\n")
        : title,
    details: details.slice(4),
    raw,
  };
}
