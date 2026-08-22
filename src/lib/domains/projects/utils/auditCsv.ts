/**
 * The triage queue as a spreadsheet, so a decision list can leave the app.
 *
 * Pure string work on purpose — no IPC — which is what makes the escaping
 * testable without a running backend.
 */
import type { DiscoveredProject } from '$lib/domains/projects/generated/DiscoveredProject';
import type { TriageDecision } from '$lib/domains/projects/api/discoveryApi';

const CSV_COLUMNS = [
  'path',
  'name',
  'abs',
  'status',
  'ageDays',
  'riskLevel',
  'risk',
  'decision',
  'modules',
  'langs',
  'repos',
  'branch',
  'remote',
  'unpushed',
  'stash',
  'dirty',
  'duplicateOf',
  'regenerable',
  'regenerableBytes',
] as const;

/** Escapes one CSV field: quote it, and double any quote inside it. */
function cell(value: unknown): string {
  const s = value === null || value === undefined ? '' : String(value);
  return `"${s.replace(/"/g, '""')}"`;
}

/**
 * The triage queue as a spreadsheet. Deliberately one row per project with the
 * decision inlined — the point of the export is to hand the list to something
 * that is not this app.
 */
export function toCsv(
  projects: DiscoveredProject[],
  decisions: Record<string, TriageDecision | null>
): string {
  const rows = projects.map((p) =>
    [
      p.path,
      p.name,
      p.abs,
      p.status,
      p.ageDays ?? '',
      p.riskLevel,
      p.risk.join(' | '),
      decisions[p.path] ?? '',
      p.moduleCount,
      p.langs.join(' | '),
      p.git.repos,
      p.git.branch ?? '',
      p.git.remote ?? '',
      p.git.ahead,
      p.git.stash,
      p.git.dirty,
      p.duplicateOf.join(' | '),
      p.regenerable.join(' | '),
      p.regenerableBytes < 0 ? '' : p.regenerableBytes,
    ]
      .map(cell)
      .join(',')
  );
  return [CSV_COLUMNS.map(cell).join(','), ...rows].join('\r\n');
}
