export function toGitHubUrl(remote?: string): string | null {
  if (!remote) return null;
  const trimmed = remote.trim().replace(/\.git$/, '');
  if (trimmed.startsWith('https://github.com/')) {
    return trimmed;
  }
  if (trimmed.startsWith('git@github.com:')) {
    return `https://github.com/${trimmed.slice('git@github.com:'.length)}`;
  }
  return null;
}

const RELATIVE_UNITS: [Intl.RelativeTimeFormatUnit, number][] = [
  ['year', 31536000],
  ['month', 2592000],
  ['day', 86400],
  ['hour', 3600],
  ['minute', 60],
];

/** "3 days ago" style text for the footer line. */
export function timeAgo(date: Date | string): string {
  const seconds = (new Date(date).getTime() - Date.now()) / 1000;
  const fmt = new Intl.RelativeTimeFormat('en', { numeric: 'auto' });
  for (const [unit, size] of RELATIVE_UNITS) {
    if (Math.abs(seconds) >= size) return fmt.format(Math.round(seconds / size), unit);
  }
  return 'just now';
}
