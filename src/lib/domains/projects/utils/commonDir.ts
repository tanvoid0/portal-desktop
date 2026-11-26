/** Deepest folder containing every path; '' when they share none (different drives). */
export function commonDir(paths: string[]): string {
  const [first = [], ...rest] = paths.map((p) => p.split(/[\\/]+/));
  let n = 0;
  while (n < first.length && rest.every((s) => s[n]?.toLowerCase() === first[n].toLowerCase())) n++;
  const sep = paths[0]?.includes('\\') ? '\\' : '/';
  // A bare "D:" is drive-relative on Windows, and "" is the unix root.
  const dir = first.slice(0, n).join(sep);
  return n === 1 ? dir + sep : dir;
}
