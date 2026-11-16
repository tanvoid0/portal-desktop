const ANSI = /\x1b\[[0-?]*[ -/]*[@-~]|\x1b\][^\x07\x1b]*(?:\x07|\x1b\\)|\x1b[@-Z\\-_]/g;

/**
 * Show raw process output the way a terminal would: escape codes removed, and a
 * `\r` redraw (progress bars, spinners) keeps only the line's latest state.
 */
export function toTerminalText(raw: string): string {
  return raw
    .replace(ANSI, '')
    .replace(/\r\n/g, '\n')
    .split('\n')
    .map((line) => {
      const trimmed = line.replace(/\r+$/, '');
      return trimmed.slice(trimmed.lastIndexOf('\r') + 1);
    })
    .join('\n');
}

/** Who asked for a run, for display — null when it was the user in this app. */
export function runRequester(triggeredBy: string | null | undefined): string | null {
  if (!triggeredBy || triggeredBy === 'user') return null;
  return triggeredBy.startsWith('mcp:') ? `MCP · ${triggeredBy.slice(4)}` : triggeredBy;
}
