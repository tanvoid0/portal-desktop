/**
 * Utility functions for cleaning and formatting terminal text
 */

/**
 * Removes ANSI escape codes from text
 * ANSI escape codes are used for colors, cursor positioning, etc.
 * Enhanced to handle OSC (Operating System Command) sequences and more edge cases
 */
export function stripAnsiCodes(text: string): string {
  if (!text) return "";

  let cleaned = text;

  // Remove OSC (Operating System Command) sequences: ]0; ... \x07 or ]0; ... \x1b\\
  // These include window title changes, icon names, etc.
  // Match: ESC] followed by digits, semicolon, any chars until BEL (\x07) or ESC\ or end
  cleaned = cleaned.replace(/\x1b\]\d+;[^\x07\x1b]*[\x07\x1b\\]/g, "");
  cleaned = cleaned.replace(/\x1b\]\d+;[^\x07\x1b]*/g, "");
  // Also handle without ESC prefix (just ]digit;)
  cleaned = cleaned.replace(/\]\d+;[^\x07\x1b]*[\x07\x1b\\]/g, "");
  cleaned = cleaned.replace(/\]\d+;[^\x07\x1b]*/g, "");

  // Remove all ANSI escape sequences (CSI, SGR, etc.)
  // Match ESC[ or ESC( followed by parameters and final character
  cleaned = cleaned.replace(
    /[\u001b\u009b][[()#;?]*(?:[0-9]{1,4}(?:;[0-9]{0,4})*)?[0-9A-PRZcf-nqry=><]/g,
    "",
  );

  // Remove CSI sequences more aggressively
  cleaned = cleaned.replace(/\x1b\[[0-9;]*[A-Za-z]/g, "");
  cleaned = cleaned.replace(/\[[0-9;]*[A-Za-z]/g, ""); // Also match without ESC prefix

  // Remove specific ANSI codes
  cleaned = cleaned.replace(/\x1b\[[^m]*m/g, ""); // Color/format codes
  cleaned = cleaned.replace(/\x1b\[[0-9;]*[HJ]/g, ""); // Cursor position
  cleaned = cleaned.replace(/\x1b\[[0-9;]*[ABCD]/g, ""); // Cursor movement
  cleaned = cleaned.replace(/\x1b\[[0-9;]*[KL]/g, ""); // Erase operations
  cleaned = cleaned.replace(/\x1b\[[?0-9;]*[hl]/g, ""); // Mode changes
  cleaned = cleaned.replace(/\x1b\[[0-9;]*[JK]/g, ""); // Erase in display/line
  cleaned = cleaned.replace(/\x1b\[[0-9;]*[su]/g, ""); // Save/restore cursor

  // Remove any remaining escape characters
  cleaned = cleaned.replace(/\x1b/g, "");
  cleaned = cleaned.replace(/\u009b/g, "");

  return cleaned;
}

/**
 * Removes control characters that aren't useful for display
 * Keeps common whitespace like spaces, tabs, newlines, carriage returns
 * Enhanced to handle more terminal control sequences
 */
export function stripControlCharacters(text: string): string {
  if (!text) return "";

  let cleaned = text;

  // Remove backspace and overstrike sequences FIRST (before other cleaning)
  // This handles cases like "eeeeccho" where characters are overstruck
  // Pattern: any character followed by backspace, repeat until no more matches
  let prevLength = 0;
  while (cleaned.length !== prevLength) {
    prevLength = cleaned.length;
    cleaned = cleaned.replace(/.\x08/g, ""); // Remove backspace and preceding character
  }

  // Remove C0 and C1 control codes except for common whitespace
  // C0: \x00-\x1F (except \x09 tab, \x0A newline, \x0D carriage return)
  // C1: \x80-\x9F
  cleaned = cleaned.replace(/[\x00-\x08\x0B\x0C\x0E-\x1F\x7F-\x9F]/g, "");

  // Remove specific terminal control sequences
  cleaned = cleaned.replace(/\]\d+;/g, ""); // Remove ]0;, ]1;, ]2;, etc. (OSC sequences)
  cleaned = cleaned.replace(/J\d+;/g, ""); // Remove J0;, J1;, etc.

  // Remove carriage return sequences that cause overwriting
  cleaned = cleaned.replace(/\r[^\n]/g, ""); // Remove \r not followed by \n
  cleaned = cleaned.replace(/\r\n/g, "\n"); // Normalize CRLF to LF

  // Remove terminal mode sequences (even without ESC prefix)
  cleaned = cleaned.replace(/\[[?]?\d+[hl]/g, ""); // Mode changes like [?2004h, [?25l
  cleaned = cleaned.replace(/\[[?]?\d+[=<>]/g, ""); // Additional mode sequences

  // Remove BEL (bell) character
  cleaned = cleaned.replace(/\x07/g, "");

  return cleaned;
}

/**
 * Cleans terminal output by removing ANSI codes and control characters
 * This makes the output human-readable and suitable for display
 * Enhanced to preserve actual command output while removing terminal artifacts
 */
export function cleanTerminalOutput(text: string): string {
  if (!text) return "";

  // First pass: Remove ANSI codes
  let cleaned = stripAnsiCodes(text);

  // Second pass: Remove control characters (including backspaces)
  cleaned = stripControlCharacters(cleaned);

  // Remove lines that are just terminal prompts or artifacts
  // Common patterns: lines with just %, ~, $, #, or prompt-like characters
  const lines = cleaned.split("\n");
  const filteredLines: string[] = [];

  for (const line of lines) {
    const trimmed = line.trim();

    // Skip empty lines
    if (!trimmed) {
      continue;
    }

    // Skip lines that are just prompt characters or terminal artifacts
    if (/^[%~$#❯>]+$/.test(trimmed)) {
      continue;
    }

    // Skip lines that are mostly control characters or escape sequences
    if (/^[\s\u0000-\u001F\u007F-\u009F]+$/.test(trimmed)) {
      continue;
    }

    filteredLines.push(line);
  }

  cleaned = filteredLines.join("\n");

  // Remove duplicate consecutive lines (terminal redraws)
  const deduplicatedLines: string[] = [];
  let lastLine = "";
  let lastLineCount = 0;

  for (const line of filteredLines) {
    if (line === lastLine) {
      lastLineCount++;
      // Only keep if we haven't seen it more than 2 times in a row
      if (lastLineCount <= 2) {
        deduplicatedLines.push(line);
      }
    } else {
      lastLine = line;
      lastLineCount = 1;
      deduplicatedLines.push(line);
    }
  }

  cleaned = deduplicatedLines.join("\n");

  // Normalize excessive whitespace but preserve line structure
  // Replace multiple spaces/tabs with single space, but keep newlines
  cleaned = cleaned.replace(/[ \t]+/g, " ");

  // Remove trailing empty lines
  cleaned = cleaned.replace(/\n+$/, "");

  // Remove leading/trailing whitespace from each line
  cleaned = cleaned
    .split("\n")
    .map((line) => line.trimEnd())
    .join("\n");

  // Final cleanup: remove any remaining control characters
  cleaned = cleaned.replace(/[\x00-\x08\x0B\x0C\x0E-\x1F\x7F-\x9F]/g, "");

  return cleaned.trim();
}

/**
 * Truncates text to a specified length with ellipsis
 * Preserves line breaks and handles multi-line text
 */
export function truncateText(text: string, maxLength: number = 100): string {
  if (!text || text.length <= maxLength) return text;

  // If it's a single line, just truncate
  if (!text.includes("\n")) {
    return text.substring(0, maxLength - 3) + "...";
  }

  // For multi-line text, try to preserve some lines
  const lines = text.split("\n");
  let result = "";
  let currentLength = 0;

  for (const line of lines) {
    if (currentLength + line.length + 1 > maxLength - 3) {
      result += "...";
      break;
    }
    result += (result ? "\n" : "") + line;
    currentLength += line.length + 1;
  }

  return result;
}

/**
 * Formats command output for display in the history
 * Combines cleaning and truncation
 */
export function formatCommandOutput(
  text: string,
  maxLength: number = 100,
): string {
  const cleaned = cleanTerminalOutput(text);
  return truncateText(cleaned, maxLength);
}
