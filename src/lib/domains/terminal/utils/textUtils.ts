/**
 * Utility functions for cleaning and formatting terminal text
 */

/**
 * Removes ANSI escape codes from text
 * ANSI escape codes are used for colors, cursor positioning, etc.
 */
export function stripAnsiCodes(text: string): string {
  // Regular expression to match ANSI escape sequences
  // This covers most common ANSI codes including:
  // - SGR (Select Graphic Rendition) codes for colors/formatting
  // - Cursor movement codes
  // - Erase in line/display codes
  // - Other control sequences
  const ansiRegex = /[\u001b\u009b][[()#;?]*(?:[0-9]{1,4}(?:;[0-9]{0,4})*)?[0-9A-PRZcf-nqry=><]/g;
  return text.replace(ansiRegex, '');
}

/**
 * Removes control characters that aren't useful for display
 * Keeps common whitespace like spaces, tabs, newlines, carriage returns
 */
export function stripControlCharacters(text: string): string {
  // Remove C0 and C1 control codes except for common whitespace
  // C0: \x00-\x1F (except \x09 tab, \x0A newline, \x0D carriage return)
  // C1: \x80-\x9F
  // Also remove specific problematic sequences like ]0; and J0;
  let cleaned = text.replace(/[\x00-\x08\x0B\x0C\x0E-\x1F\x7F-\x9F]/g, '');
  
  // Remove specific terminal control sequences that might leak through
  cleaned = cleaned.replace(/\]\d+;/g, ''); // Remove ]0;, ]1;, etc.
  cleaned = cleaned.replace(/J\d+;/g, '');  // Remove J0;, J1;, etc.
  
  return cleaned;
}

/**
 * Cleans terminal output by removing ANSI codes and control characters
 * This makes the output human-readable and suitable for display
 */
export function cleanTerminalOutput(text: string): string {
  if (!text) return '';
  
  let cleaned = stripAnsiCodes(text);
  cleaned = stripControlCharacters(cleaned);
  
  // Normalize whitespace - replace multiple spaces with single space
  // but preserve intentional formatting
  cleaned = cleaned.replace(/[ \t]+/g, ' ');
  
  // Remove leading/trailing whitespace from each line
  cleaned = cleaned.split('\n').map(line => line.trim()).join('\n');
  
  return cleaned;
}

/**
 * Truncates text to a specified length with ellipsis
 * Preserves line breaks and handles multi-line text
 */
export function truncateText(text: string, maxLength: number = 100): string {
  if (!text || text.length <= maxLength) return text;
  
  // If it's a single line, just truncate
  if (!text.includes('\n')) {
    return text.substring(0, maxLength - 3) + '...';
  }
  
  // For multi-line text, try to preserve some lines
  const lines = text.split('\n');
  let result = '';
  let currentLength = 0;
  
  for (const line of lines) {
    if (currentLength + line.length + 1 > maxLength - 3) {
      result += '...';
      break;
    }
    result += (result ? '\n' : '') + line;
    currentLength += line.length + 1;
  }
  
  return result;
}

/**
 * Formats command output for display in the history
 * Combines cleaning and truncation
 */
export function formatCommandOutput(text: string, maxLength: number = 100): string {
  const cleaned = cleanTerminalOutput(text);
  return truncateText(cleaned, maxLength);
}
