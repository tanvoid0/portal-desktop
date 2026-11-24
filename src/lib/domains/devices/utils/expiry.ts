import { formatRelativeTime } from '$lib/domains/shared/utils';

/** "in 3d" / "3d ago" — one relative form for both directions; pair with an
 * absolute `title` (`date.toLocaleString()`). */
export function relativeExpiry(date: Date): string {
  const ms = date.getTime() - Date.now();
  if (ms <= 0) return formatRelativeTime(date);
  const mins = Math.floor(ms / 60000);
  if (mins < 60) return `in ${Math.max(mins, 1)}m`;
  const hours = Math.floor(mins / 60);
  if (hours < 24) return `in ${hours}h`;
  return `in ${Math.floor(hours / 24)}d`;
}
