/** Deepest folder containing every path; '' when they share none (different drives, UNC). */
export function commonDir(paths: string[]): string {
  // Splitting would collapse the leading "\\" of a UNC path into an invalid root.
  if (paths.some((p) => /^[\\/]{2}/.test(p))) return '';
  const [first = [], ...rest] = paths.map((p) => p.split(/[\\/]+/));
  // Only Windows drive paths are case-insensitive.
  const drive = /^[a-z]:/i.test(paths[0] ?? '');
  const fold = (s?: string) => (drive ? s?.toLowerCase() : s);
  let n = 0;
  while (n < first.length && rest.every((s) => fold(s[n]) === fold(first[n]))) n++;
  const sep = paths[0]?.includes('\\') ? '\\' : '/';
  // A bare "D:" is drive-relative on Windows, and "" is the unix root.
  const dir = first.slice(0, n).join(sep);
  return n === 1 ? dir + sep : dir;
}

/** A drive or filesystem root, the home folder or above: too wide for a coder workspace. */
export function isTooBroad(dir: string, home: string): boolean {
  const norm = (p: string) => {
    const s = p.replace(/\\/g, '/').replace(/\/+$/, '');
    return /^[a-z]:/i.test(s) ? s.toLowerCase() : s;
  };
  const d = norm(dir);
  return d === '' || /^[a-z]:$/.test(d) || `${norm(home)}/`.startsWith(`${d}/`);
}
