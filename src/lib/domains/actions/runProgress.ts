import type { RunSegment } from './types';

/** The one formula for a segment's fill between backend events. */
export function segmentFraction(s: RunSegment, now: number): number {
  if (s.status === 'success') return 1;
  if (s.status !== 'running') return s.fraction ?? 0;
  if (s.fraction != null) return s.fraction;
  if (!s.startedAt || !s.expectedMs) return 0;
  return Math.min((now - Date.parse(s.startedAt)) / s.expectedMs, 0.95);
}

/** Running segment has outlived its expected duration. */
export function isOverrun(s: RunSegment, now: number): boolean {
  return (
    s.status === 'running' &&
    !!s.startedAt &&
    !!s.expectedMs &&
    now - Date.parse(s.startedAt) > s.expectedMs
  );
}

export function etaText(etaAt: string | null | undefined, now: number): string {
  if (!etaAt) return 'No estimate yet';
  const sec = Math.round((Date.parse(etaAt) - now) / 1000);
  if (sec <= 0) return 'finishing up';
  if (sec < 60) return `about ${sec} sec left`;
  return `about ${Math.round(sec / 60)} min left`;
}

export function durationText(ms: number): string {
  const sec = Math.round(ms / 1000);
  return sec < 60 ? `${sec}s` : `${Math.floor(sec / 60)}m ${sec % 60}s`;
}
