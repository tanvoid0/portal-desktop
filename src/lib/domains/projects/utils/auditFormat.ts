import type { TriageDecision } from '$lib/domains/projects/api/discoveryApi';

export const DECISIONS: TriageDecision[] = ['keep', 'archive', 'delete', 'review'];

/**
 * `delete` sat in a row of four identical ghost buttons — the confirm dialog
 * only fires after the misclick, so the button has to read as dangerous first.
 */
export const DESTRUCTIVE_CLASS = 'text-destructive hover:text-destructive hover:bg-destructive/10';

export function ageLabel(days: number | null): string {
  if (days === null) return 'unknown';
  if (days < 1) return 'today';
  if (days < 30) return `${days}d`;
  if (days < 365) return `${Math.round(days / 30)}mo`;
  return `${(days / 365).toFixed(1)}y`;
}
