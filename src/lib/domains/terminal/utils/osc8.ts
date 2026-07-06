export interface Osc8InjectOptions {
  maxTailChars?: number;
  flush?: boolean;
}

const unixPathRe =
  /((?:\/[A-Za-z0-9_.-]+)+\/[A-Za-z0-9_.-]+?\.[A-Za-z0-9]{1,10})(?::(\d+)(?::(\d+))?)?/g;

// Matches e.g. C:\path\file.ts or C:\path\file.ts:12:3
const windowsPathRe =
  /([A-Za-z]:\\(?:[^\\s:<>"]+\\)*[^\\s:<>"]+\\.[A-Za-z0-9]{1,10})(?::(\d+)(?::(\d+))?)?/g;

function osc8Wrap(uri: string, visible: string): string {
  // OSC 8 hyperlink: ST = BEL (\\x07)
  const st = "\x07";
  return `\x1b]8;;${uri}${st}${visible}\x1b]8;;${st}`;
}

function linkify(content: string): string {
  if (!content) return content;

  // Replace Windows paths first to avoid conflicts with drive-letter `C:` patterns.
  let out = content.replace(windowsPathRe, (full, path) => {
    const uri = `file://${String(path)}`;
    return osc8Wrap(uri, String(full));
  });

  // Unix absolute paths
  out = out.replace(unixPathRe, (full, path) => {
    const uri = `file://${String(path)}`;
    return osc8Wrap(uri, String(full));
  });

  return out;
}

/**
 * Inject OSC 8 links into a streaming terminal output pipeline.
 *
 * `tail` is a small unprocessed suffix from the previous chunk. We keep it so file
 * paths split across chunks can still be linkified.
 */
export function injectOsc8Links(
  content: string,
  tail: string,
  options: Osc8InjectOptions = {},
): { transformed: string; newTail: string } {
  const maxTailChars = options.maxTailChars ?? 256;
  const flush = options.flush ?? false;

  const combined = `${tail}${content ?? ""}`;
  if (!combined) return { transformed: "", newTail: "" };

  if (flush) {
    return { transformed: linkify(combined), newTail: "" };
  }

  // Short streams: render immediately instead of holding everything in tail.
  if (combined.length <= maxTailChars) {
    return { transformed: linkify(combined), newTail: "" };
  }

  // Keep the last N chars unprocessed; linkification will happen when enough data arrives.
  const safeEnd = Math.max(0, combined.length - maxTailChars);
  const toProcess = combined.slice(0, safeEnd);
  const newTail = combined.slice(safeEnd);

  return {
    transformed: linkify(toProcess),
    newTail,
  };
}

