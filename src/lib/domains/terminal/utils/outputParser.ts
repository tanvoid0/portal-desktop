/**
 * Terminal Output Parser
 * Intelligently extracts actual command output from raw terminal stream
 * Works across different shells: zsh, bash, powershell, cmd
 */

/**
 * Common shell prompt patterns
 */
const PROMPT_PATTERNS = {
  // Linux/Unix shells
  bash: /^[^\s]+@[^\s]+:[^\s]+\$|^[^\s]+@[^\s]+:[^\s]+#/,
  zsh: /^[^\s]+@[^\s]+:[^\s]+%|^[^\s]+@[^\s]+:[^\s]+#|^[^\s]+%|^[^\s]+#/,
  fish: /^[^\s]+@[^\s]+[>]|^[^\s]+[>]/,
  sh: /^[^\s]+\$|^[^\s]+#/,

  // Windows shells
  powershell: /^PS [A-Z]:[\\][^>]*>|^[A-Z]:[\\][^>]*>/,
  cmd: /^[A-Z]:[\\][^>]*>|^[A-Z]:[\\][^>]*>/,

  // Generic patterns
  generic: /^[^\s]+[%$#>]|^[A-Z]:[\\][^>]*>/,
};

/**
 * Extract command output from raw terminal stream
 * This function identifies the actual output by:
 * 1. Finding where the command output starts (after command echo)
 * 2. Finding where it ends (before next prompt)
 * 3. Cleaning up terminal artifacts
 */
export function extractCommandOutput(
  rawOutput: string,
  command: string,
  shellType?: "bash" | "zsh" | "powershell" | "cmd" | "fish" | "sh",
): string {
  if (!rawOutput) return "";

  // First, clean ANSI codes and control sequences
  let cleaned = cleanRawOutput(rawOutput);

  // Try to detect shell type if not provided
  if (!shellType) {
    shellType = detectShellType(cleaned);
  }

  // Extract the actual output
  const output = extractOutputBetweenPrompts(cleaned, command, shellType);

  return output;
}

/**
 * Clean raw terminal output by removing ANSI codes and control sequences
 */
function cleanRawOutput(text: string): string {
  if (!text) return "";

  let cleaned = text;

  // Remove all ANSI escape sequences
  cleaned = cleaned.replace(
    /[\u001b\u009b][[()#;?]*(?:[0-9]{1,4}(?:;[0-9]{0,4})*)?[0-9A-PRZcf-nqry=><]/g,
    "",
  );
  cleaned = cleaned.replace(/\x1b\[[0-9;]*[A-Za-z]/g, "");
  cleaned = cleaned.replace(/\[[0-9;]*[A-Za-z]/g, "");

  // Remove OSC sequences (like ]2;command or ]1;command)
  cleaned = cleaned.replace(/\x1b\]\d+;[^\x07\x1b]*[\x07\x1b\\]/g, "");
  cleaned = cleaned.replace(/\]\d+;[^\x07\x1b]*[\x07\x1b\\]/g, "");
  // Also remove OSC sequences without proper termination (common in terminal output)
  cleaned = cleaned.replace(/\]\d+;[^\x07\x1b\n]*/g, "");
  cleaned = cleaned.replace(/\]\]\d+;/g, ""); // Handle double brackets like ]]2;

  // Remove backspace sequences (iteratively until no more matches)
  let prevLength = 0;
  while (cleaned.length !== prevLength) {
    prevLength = cleaned.length;
    cleaned = cleaned.replace(/.\x08/g, "");
  }

  // Remove control characters except newlines
  cleaned = cleaned.replace(/[\x00-\x08\x0B\x0C\x0E-\x1F\x7F-\x9F]/g, "");

  // Remove carriage returns
  cleaned = cleaned.replace(/\r/g, "");

  return cleaned;
}

/**
 * Detect shell type from output patterns
 */
function detectShellType(
  output: string,
): "bash" | "zsh" | "powershell" | "cmd" | "fish" | "sh" {
  // Check for PowerShell patterns
  if (PROMPT_PATTERNS.powershell.test(output)) {
    return "powershell";
  }

  // Check for CMD patterns
  if (PROMPT_PATTERNS.cmd.test(output)) {
    return "cmd";
  }

  // Check for zsh patterns (usually has % or special characters)
  if (PROMPT_PATTERNS.zsh.test(output) || output.includes("❯")) {
    return "zsh";
  }

  // Check for fish patterns
  if (PROMPT_PATTERNS.fish.test(output)) {
    return "fish";
  }

  // Default to bash
  return "bash";
}

/**
 * Extract output between command and next prompt
 */
function extractOutputBetweenPrompts(
  cleaned: string,
  command: string,
  shellType: "bash" | "zsh" | "powershell" | "cmd" | "fish" | "sh",
): string {
  if (!cleaned) return "";

  const lines = cleaned.split("\n");
  let outputLines: string[] = [];

  let foundCommandStart = false;
  let collectingOutput = false;
  const commandTrimmed = command.trim();
  const commandWords = commandTrimmed.split(/\s+/);
  const firstCommandWord = commandWords[0] || "";

  // Get prompt pattern for this shell
  const promptPattern = PROMPT_PATTERNS[shellType] || PROMPT_PATTERNS.generic;

  // Strategy: Find where command output actually starts (after command echo/typing)
  // and ends (before next prompt)

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const trimmed = line.trim();

    // Skip completely empty lines at start
    if (!foundCommandStart && !trimmed) continue;

    // Look for command being typed/echoed (may have artifacts from backspace)
    if (!foundCommandStart) {
      // Check if this line contains the command or command word
      const hasCommand =
        trimmed.includes(commandTrimmed) ||
        trimmed.includes(firstCommandWord) ||
        (firstCommandWord &&
          trimmed.toLowerCase().includes(firstCommandWord.toLowerCase()));

      if (hasCommand) {
        foundCommandStart = true;
        // Next non-empty line should be the start of output
        continue;
      }
    }

    // After finding command, start collecting output
    if (foundCommandStart) {
      // Stop if we hit a prompt (command finished)
      if (promptPattern.test(trimmed)) {
        break;
      }

      // Skip lines that are just the command itself
      if (trimmed === commandTrimmed || trimmed === command) {
        continue;
      }

      // Skip lines that look like command typing artifacts
      // (repeated characters, partial command words)
      if (firstCommandWord && trimmed.length > 0) {
        const words = trimmed.toLowerCase().split(/\s+/);
        const isCommandArtifact = words.every(
          (word) =>
            word.length > 0 &&
            (word.includes(firstCommandWord.toLowerCase()) ||
              firstCommandWord.toLowerCase().includes(word)),
        );
        if (isCommandArtifact && words.length <= commandWords.length) {
          continue;
        }
      }

      // Skip prompt-like lines (just symbols or full prompts)
      if (
        /^[%~$#❯>]+$/.test(trimmed) ||
        /^[^\s]+@[^\s]+:[^\s]+[%$#>❯]\s*$/.test(trimmed) ||
        /^PS [A-Z]:[\\][^>]*>\s*$/.test(trimmed) ||
        /^[A-Z]:[\\][^>]*>\s*$/.test(trimmed) ||
        /~\s*❯/.test(trimmed)
      ) {
        continue;
      }

      // Skip timestamp lines
      if (
        /\d{2}:\d{2}:\d{2}\s*(AM|PM)/.test(trimmed) ||
        /^[A-Z][a-z]{2}\s+\d{1,2}\s+\d{2}:\d{2}:\d{2}/.test(trimmed)
      ) {
        continue;
      }

      // Skip lines that are just control characters or escape sequences
      if (/^[\s\u0000-\u001F\u007F-\u009F]+$/.test(trimmed)) {
        continue;
      }

      // Skip lines that are just backslashes or special characters
      if (/^[\\\/\-\|]+$/.test(trimmed)) {
        continue;
      }

      // This looks like actual output
      collectingOutput = true;
      outputLines.push(line);
    }
  }

  // If we didn't find output between prompts, try a different strategy:
  // Look for the actual command output (usually appears after command, before prompt)
  if (outputLines.length === 0) {
    // Find last occurrence of command, then collect everything until next prompt
    let lastCommandIndex = -1;
    for (let i = lines.length - 1; i >= 0; i--) {
      const lineTrimmed = lines[i].trim();
      if (
        lineTrimmed.includes(commandTrimmed) ||
        lineTrimmed.includes(firstCommandWord) ||
        (firstCommandWord &&
          lineTrimmed.toLowerCase().includes(firstCommandWord.toLowerCase()))
      ) {
        lastCommandIndex = i;
        break;
      }
    }

    if (lastCommandIndex >= 0) {
      for (let i = lastCommandIndex + 1; i < lines.length; i++) {
        const trimmed = lines[i].trim();
        if (promptPattern.test(trimmed)) {
          break;
        }
        // Skip empty lines, prompts, timestamps, and command artifacts
        const isPrompt =
          /^[%~$#❯>]+$/.test(trimmed) ||
          /^[^\s]+@[^\s]+:[^\s]+[%$#>❯]\s*$/.test(trimmed) ||
          /~\s*❯/.test(trimmed) ||
          /\d{2}:\d{2}:\d{2}\s*(AM|PM)/.test(trimmed);

        if (
          trimmed &&
          !isPrompt &&
          trimmed !== commandTrimmed &&
          !(
            firstCommandWord &&
            trimmed.toLowerCase() === firstCommandWord.toLowerCase()
          ) &&
          !/^[\\\/\-\|]+$/.test(trimmed)
        ) {
          outputLines.push(lines[i]);
        }
      }
    }
  }

  // If still no output, try extracting just the unique content lines
  // (for commands like echo that output simple text)
  if (outputLines.length === 0) {
    const seenLines = new Set<string>();
    for (const line of lines) {
      const trimmed = line.trim();

      // Skip prompts, commands, timestamps, and artifacts
      const isPrompt =
        promptPattern.test(trimmed) ||
        /^[%~$#❯>]+$/.test(trimmed) ||
        /~\s*❯/.test(trimmed) ||
        /\d{2}:\d{2}:\d{2}\s*(AM|PM)/.test(trimmed);

      const isCommand =
        trimmed.includes(commandTrimmed) || trimmed.includes(firstCommandWord);

      if (
        trimmed &&
        !isPrompt &&
        !isCommand &&
        !/^[\\\/\-\|]+$/.test(trimmed) &&
        !seenLines.has(trimmed)
      ) {
        seenLines.add(trimmed);
        outputLines.push(line);
      }
    }
  }

  // Aggressive filtering: remove any lines that contain prompts or artifacts
  if (outputLines.length > 0) {
    const filtered: string[] = [];

    for (const line of outputLines) {
      const trimmed = line.trim();
      const originalLine = line;

      // Skip completely empty lines
      if (!trimmed) continue;

      // Skip lines that are just prompt symbols (with or without whitespace)
      if (/^[%~$#❯>]+$/.test(trimmed) || /^[%~$#❯>]+\s+$/.test(trimmed)) {
        continue;
      }

      // Skip lines that are mostly whitespace with just a prompt character
      // Pattern: % followed by lots of spaces (like "%                ")
      // Match: % followed by 5+ spaces, or any prompt char followed by mostly spaces
      if (
        /^%\s{5,}$/.test(trimmed) ||
        /^[%~$#❯>]\s{5,}$/.test(trimmed) ||
        /^%\s+$/.test(trimmed) ||
        /^[%~$#❯>]\s+$/.test(trimmed)
      ) {
        continue;
      }

      // Skip lines that are mostly whitespace (80%+ spaces)
      const spaceRatio = (trimmed.match(/\s/g) || []).length / trimmed.length;
      if (spaceRatio > 0.8 && trimmed.length > 10) {
        continue;
      }

      // Skip lines that are just backslashes or special characters
      // Match: 3+ backslashes, or lines that are 80%+ special chars
      if (
        /^[\\\/\-\|_\s]{3,}$/.test(trimmed) ||
        /^[\\\/\-\|_\s]+$/.test(trimmed)
      ) {
        continue;
      }

      // Skip lines that are mostly backslashes (like "\\\\\\")
      const backslashCount = (trimmed.match(/\\/g) || []).length;
      if (backslashCount >= 3 && backslashCount / trimmed.length > 0.5) {
        continue;
      }

      // Skip lines that are just whitespace
      if (/^\s+$/.test(trimmed)) {
        continue;
      }

      // Skip full prompts (user@host:path$)
      if (/^[^\s]+@[^\s]+:[^\s]+[%$#>❯]\s*$/.test(trimmed)) {
        continue;
      }

      // Skip PowerShell/CMD prompts
      if (
        /^PS [A-Z]:[\\][^>]*>\s*$/.test(trimmed) ||
        /^[A-Z]:[\\][^>]*>\s*$/.test(trimmed)
      ) {
        continue;
      }

      // Skip timestamp lines
      if (
        /\d{2}:\d{2}:\d{2}\s*(AM|PM)/.test(trimmed) ||
        /^[A-Z][a-z]{2}\s+\d{1,2}\s+\d{2}:\d{2}:\d{2}/.test(trimmed)
      ) {
        continue;
      }

      // If line starts with a prompt but has content, extract just the content
      if (/^[%~$#❯>]+\s+/.test(trimmed)) {
        const withoutPrompt = trimmed.replace(/^[%~$#❯>]+\s+/, "").trim();
        if (withoutPrompt && !/^[\\\/\-\|_\s]+$/.test(withoutPrompt)) {
          filtered.push(withoutPrompt);
        }
        continue;
      }

      // If line ends with a prompt, extract just the content before it
      if (/\s+[%~$#❯>]+$/.test(trimmed)) {
        const withoutPrompt = trimmed.replace(/\s+[%~$#❯>]+$/, "").trim();
        if (withoutPrompt && !/^[\\\/\-\|_\s]+$/.test(withoutPrompt)) {
          filtered.push(withoutPrompt);
        }
        continue;
      }

      // This looks like actual output
      filtered.push(trimmed);
    }

    outputLines = filtered;
  }

  let output = outputLines.join("\n");

  // Remove duplicate consecutive lines (terminal redraws)
  const deduplicated = removeDuplicateLines(output);

  // Final cleanup - normalize whitespace and remove trailing prompts
  let final = deduplicated.trim();

  // Split into lines for final filtering
  const finalLines = final.split("\n");
  const cleanLines: string[] = [];

  for (const line of finalLines) {
    const trimmed = line.trim();

    // Skip empty lines
    if (!trimmed) continue;

    // Skip lines that are just prompt symbols
    if (/^[%~$#❯>]+$/.test(trimmed)) {
      continue;
    }

    // Skip lines that are mostly whitespace with just a prompt (like "%                ")
    // Match: % or prompt char followed by 5+ spaces, or any amount of spaces
    if (
      /^[%~$#❯>]\s{5,}$/.test(trimmed) ||
      /^%\s+$/.test(trimmed) ||
      /^[%~$#❯>]\s+$/.test(trimmed)
    ) {
      continue;
    }

    // Skip lines that are mostly whitespace (80%+ spaces)
    const spaceRatio = (trimmed.match(/\s/g) || []).length / trimmed.length;
    if (spaceRatio > 0.8 && trimmed.length > 10) {
      continue;
    }

    // Skip lines that end with just a prompt and are short
    if (/%\s*$/.test(trimmed) && trimmed.length < 10) {
      continue;
    }

    // Skip zsh prompts
    if (/~\s*❯/.test(trimmed) && !trimmed.replace(/~\s*❯/, "").trim()) {
      continue;
    }

    // Skip lines that are just special characters (backslashes, etc.)
    // Match: 3+ backslashes, or lines that are mostly special chars
    if (
      /^[\\\/\-\|_\s]{3,}$/.test(trimmed) ||
      /^[\\\/\-\|_\s]+$/.test(trimmed)
    ) {
      continue;
    }

    // Skip lines that are mostly backslashes (like "\\\\\\")
    const backslashCount = (trimmed.match(/\\/g) || []).length;
    if (backslashCount >= 3 && backslashCount / trimmed.length > 0.5) {
      continue;
    }

    // Skip timestamp lines
    if (/\d{2}:\d{2}:\d{2}\s*(AM|PM)/.test(trimmed)) {
      continue;
    }

    // If line has a prompt at the start, extract content after it
    if (/^[%~$#❯>]+\s+/.test(trimmed)) {
      const extracted = trimmed.replace(/^[%~$#❯>]+\s+/, "").trim();
      if (extracted && !/^[\\\/\-\|_\s]+$/.test(extracted)) {
        cleanLines.push(extracted);
      }
      continue;
    }

    // If line has a prompt at the end, extract content before it
    if (/\s+[%~$#❯>]+$/.test(trimmed)) {
      const extracted = trimmed.replace(/\s+[%~$#❯>]+$/, "").trim();
      if (extracted && !/^[\\\/\-\|_\s]+$/.test(extracted)) {
        cleanLines.push(extracted);
      }
      continue;
    }

    // This is clean output
    cleanLines.push(trimmed);
  }

  // Remove duplicates one more time
  const uniqueLines: string[] = [];
  const seen = new Set<string>();
  for (const line of cleanLines) {
    if (!seen.has(line)) {
      seen.add(line);
      uniqueLines.push(line);
    }
  }

  return uniqueLines.join("\n").trim();
}

/**
 * Remove duplicate consecutive lines and filter out prompts/artifacts
 */
function removeDuplicateLines(text: string): string {
  if (!text) return "";

  const lines = text.split("\n");
  const result: string[] = [];
  const seenLines = new Set<string>();
  let lastLine = "";
  let lastLineCount = 0;

  // Patterns for prompts and terminal artifacts
  const promptPatterns = [
    /^[%~$#❯>]+$/, // Just prompt symbols
    /^[%~$#❯>]+\s*$/, // Prompt symbols with whitespace
    /^[^\s]+@[^\s]+:[^\s]+[%$#>❯]\s*$/, // Full prompt (user@host:path$)
    /^PS [A-Z]:[\\][^>]*>\s*$/, // PowerShell prompt
    /^[A-Z]:[\\][^>]*>\s*$/, // CMD prompt
    /~\s*❯/, // Zsh prompt with ~
    /\d{2}:\d{2}:\d{2}\s*(AM|PM)/, // Timestamps
    /^[A-Z][a-z]{2}\s+\d{1,2}\s+\d{2}:\d{2}:\d{2}/, // Date/time stamps
  ];

  // Pattern for OSC sequences that might appear in lines
  const oscPattern = /\]\]?\d+;[^\n]*/g;

  for (const line of lines) {
    let cleanedLine = line;

    // Remove OSC sequences from the line
    cleanedLine = cleanedLine.replace(oscPattern, "");

    const trimmed = cleanedLine.trim();

    // Skip empty lines
    if (!trimmed) continue;

    // Skip lines that are just prompts
    const isPrompt = promptPatterns.some((pattern) => pattern.test(trimmed));
    if (isPrompt) continue;

    // Skip lines that are mostly whitespace or control characters
    if (/^[\s\u0000-\u001F\u007F-\u009F]+$/.test(trimmed)) continue;

    // Skip lines that are just OSC artifacts
    if (/^\]\]?\d+;/.test(trimmed)) continue;

    // For password prompts, extract just the clean prompt text
    // Handle cases like "[sudo] password for tan:" repeated many times
    if (/\[sudo\]\s*password/i.test(trimmed)) {
      // Extract just one instance of the prompt
      const promptMatch = trimmed.match(/\[sudo\]\s*password[^\]]*:?/i);
      if (promptMatch) {
        const cleanPrompt = promptMatch[0].trim();
        // Only add if we haven't seen this exact prompt before
        if (!seenLines.has(cleanPrompt)) {
          seenLines.add(cleanPrompt);
          result.push(cleanPrompt);
        }
        continue;
      }
    }

    // Remove duplicate consecutive lines
    if (trimmed === lastLine) {
      lastLineCount++;
      // Keep only first occurrence of duplicates
      if (lastLineCount === 1) {
        result.push(cleanedLine);
      }
    } else {
      // Check if we've seen this exact line before (non-consecutive duplicate)
      if (!seenLines.has(trimmed)) {
        seenLines.add(trimmed);
        result.push(cleanedLine);
        lastLine = trimmed;
        lastLineCount = 1;
      }
    }
  }

  return result.join("\n");
}

/**
 * Parse output for a specific command execution
 * This is the main function to use - it extracts only the relevant output
 */
export function parseCommandOutput(
  rawOutput: string,
  command: string,
  shellType?: "bash" | "zsh" | "powershell" | "cmd" | "fish" | "sh",
): string {
  return extractCommandOutput(rawOutput, command, shellType);
}
