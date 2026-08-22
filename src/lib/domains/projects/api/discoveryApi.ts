import { invokeClient } from '$lib/utils/invokeClient';
import type { DiscoveryConfig } from '$lib/domains/projects/generated/DiscoveryConfig';
import type { DiscoveryReport } from '$lib/domains/projects/generated/DiscoveryReport';
import type { DiscoveryProgress } from '$lib/domains/projects/generated/DiscoveryProgress';
import type { DiscoveryImportResult } from '$lib/domains/projects/generated/DiscoveryImportResult';
import type { DiscoveryState } from '$lib/domains/projects/generated/DiscoveryState';
import type { DiscoveryChange } from '$lib/domains/projects/generated/DiscoveryChange';

export type TriageDecision = 'keep' | 'archive' | 'delete' | 'review';

export const DISCOVERY_PROGRESS_EVENT = 'discovery://progress';

/** Sensible starting point; the user edits roots in the Audit view. */
export function defaultDiscoveryConfig(roots: { name: string; path: string }[]): DiscoveryConfig {
  return {
    roots,
    projectDepth: 2,
    maxDepth: 7,
    activeDays: 120,
    recentDays: 365,
    dormantDays: 730,
    skipGit: false,
    measureSizes: false,
    markMissing: false,
  };
}

/** Roots, last report and change log — the view opens from this, not a scan. */
export async function getDiscoveryState(): Promise<DiscoveryState> {
  return invokeClient.post<DiscoveryState>('get_discovery_state');
}

/** Persists roots and scan options. The stored report is left alone. */
export async function saveDiscoveryConfig(config: DiscoveryConfig): Promise<DiscoveryState> {
  return invokeClient.post<DiscoveryState>('save_discovery_config', { config });
}

/** Writes `csv` to a path the user has already picked. Returns the path. */
export async function exportDiscoveryCsv(path: string, csv: string): Promise<string> {
  return invokeClient.post<string>('export_discovery_csv', { path, csv });
}

/** Walks the roots without writing anything — preview before import. */
export async function discoverProjects(config: DiscoveryConfig): Promise<DiscoveryReport> {
  return invokeClient.post<DiscoveryReport>('discover_projects', { config });
}

/** Walks and writes into the registry in one pass. */
export async function discoverAndImport(
  config: DiscoveryConfig,
  markMissing = false
): Promise<DiscoveryImportResult> {
  return invokeClient.post<DiscoveryImportResult>('discover_and_import_projects', {
    config,
    markMissing,
  });
}

/** Commits a report produced earlier by {@link discoverProjects}. */
export async function importDiscovered(
  report: DiscoveryReport,
  markMissing = false
): Promise<DiscoveryImportResult> {
  return invokeClient.post<DiscoveryImportResult>('import_discovered_projects', {
    report,
    markMissing,
  });
}

export async function cancelDiscovery(): Promise<void> {
  await invokeClient.post<void>('cancel_discovery');
}

export async function setProjectTriage(
  id: number,
  decision: TriageDecision | null,
  note?: string | null,
  successor?: string | null
): Promise<boolean> {
  return invokeClient.post<boolean>('set_project_triage', {
    id,
    decision,
    note: note ?? null,
    successor: successor ?? null,
  });
}

export async function setProjectTriageBulk(
  ids: number[],
  decision: TriageDecision | null
): Promise<number> {
  return invokeClient.post<number>('set_project_triage_bulk', { ids, decision });
}

export type {
  DiscoveryConfig,
  DiscoveryReport,
  DiscoveryProgress,
  DiscoveryImportResult,
  DiscoveryState,
  DiscoveryChange,
};
