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

export interface AnsiSpan {
  text: string;
  /** Tailwind classes for the span's colour/weight; '' = inherit. */
  cls: string;
}

// Shades picked for a dark log panel (StepLogViewer is always zinc-950).
const FG = [
  'text-zinc-500',
  'text-red-400',
  'text-green-400',
  'text-yellow-300',
  'text-blue-400',
  'text-fuchsia-400',
  'text-cyan-400',
  'text-zinc-100',
];
const FG_BRIGHT = [
  'text-zinc-400',
  'text-red-300',
  'text-green-300',
  'text-yellow-200',
  'text-blue-300',
  'text-fuchsia-300',
  'text-cyan-300',
  'text-white',
];

/**
 * One output line as coloured spans: SGR foreground, bold and dim become classes,
 * every other escape is dropped, and a `\r` redraw keeps only the latest state.
 * ponytail: foreground only; backgrounds, underline and 256/truecolour (beyond the
 * 16 base indexes) render as plain text.
 */
export function ansiSpans(raw: string): AnsiSpan[] {
  const line = raw.replace(/\r+$/, '');
  const spans: AnsiSpan[] = [];
  let fg = '';
  let bold = false;
  let dim = false;
  const append = (text: string) => {
    if (!text) return;
    const cls = [fg, bold && 'font-bold', dim && 'opacity-60'].filter(Boolean).join(' ');
    const prev = spans.at(-1);
    if (prev && prev.cls === cls) prev.text += text;
    else spans.push({ text, cls });
  };
  // A `\r` mid-line is a redraw: drop what came before it, keep the colour state.
  const push = (text: string) =>
    text.split('\r').forEach((part, i) => {
      if (i) spans.length = 0;
      append(part);
    });

  let at = 0;
  for (const m of line.matchAll(ANSI)) {
    push(line.slice(at, m.index));
    at = m.index + m[0].length;
    const sgr = /^\x1b\[([\d;]*)m$/.exec(m[0]);
    if (!sgr) continue;
    const codes = sgr[1] ? sgr[1].split(';').map(Number) : [0];
    for (let i = 0; i < codes.length; i++) {
      const c = codes[i];
      if (c === 0) fg = '';
      if (c === 0 || c === 22) bold = dim = false;
      else if (c === 1) bold = true;
      else if (c === 2) dim = true;
      else if (c >= 30 && c <= 37) fg = FG[c - 30];
      else if (c >= 90 && c <= 97) fg = FG_BRIGHT[c - 90];
      else if (c === 39) fg = '';
      else if (c === 38 || c === 48) {
        // 38;5;n / 38;2;r;g;b — consume the arguments so they aren't read as codes.
        const n = codes[i + 2];
        if (c === 38 && codes[i + 1] === 5 && n < 16) fg = (n < 8 ? FG : FG_BRIGHT)[n % 8];
        i += codes[i + 1] === 5 ? 2 : codes[i + 1] === 2 ? 4 : 0;
      }
    }
  }
  push(line.slice(at));
  return spans;
}

/** Who asked for a run, for display — null when it was the user in this app. */
export function runRequester(triggeredBy: string | null | undefined): string | null {
  if (!triggeredBy || triggeredBy === 'user') return null;
  return triggeredBy.startsWith('mcp:') ? `MCP · ${triggeredBy.slice(4)}` : triggeredBy;
}
