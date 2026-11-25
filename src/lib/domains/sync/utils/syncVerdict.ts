import type { SyncStatus, SyncVerdict } from '../types';

/** Compare what this machine last synced against the cloud head and local edits since. */
export function syncVerdict(s: SyncStatus): SyncVerdict {
  if (!s.cloud) return s.cloudError ? 'unknown' : 'never';
  if (s.baseVersion === 0) return 'never';
  const localChanged =
    !s.lastSyncedAt ||
    (s.localDbModifiedAt !== null && Date.parse(s.localDbModifiedAt) > Date.parse(s.lastSyncedAt));
  const cloudAhead = s.cloud.version > s.baseVersion;
  if (cloudAhead) return localChanged ? 'diverged' : 'cloud-newer';
  return localChanged ? 'local-newer' : 'in-sync';
}

export function formatBytes(n: number): string {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
  return `${(n / 1024 / 1024).toFixed(1)} MB`;
}

export function formatAbsolute(iso: string | null): string {
  if (!iso || Number.isNaN(Date.parse(iso))) return 'unknown';
  return new Date(iso).toLocaleString();
}

export function formatRelative(iso: string | null, now = Date.now()): string {
  if (!iso || Number.isNaN(Date.parse(iso))) return '';
  const sec = Math.round((Date.parse(iso) - now) / 1000);
  const rtf = new Intl.RelativeTimeFormat(undefined, { numeric: 'auto' });
  for (const [unit, s] of [
    ['day', 86400],
    ['hour', 3600],
    ['minute', 60],
  ] as const) {
    if (Math.abs(sec) >= s) return rtf.format(Math.round(sec / s), unit);
  }
  return rtf.format(sec, 'second');
}
