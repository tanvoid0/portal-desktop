import type { CapabilityInfo } from '../types';

/** Names "Select all" may bulk-toggle — every sensitive capability (RCE-equivalent
 * caps like terminal.exec/agents.exec) stays opt-in per row, never
 * swept in by one click. */
export function bulkToggleable(capabilities: CapabilityInfo[]): string[] {
  return capabilities.filter((c) => !c.is_sensitive).map((c) => c.name);
}

/** What simple mode grants in one click: the requested names the registry knows
 * to be non-sensitive. Sensitive caps are never granted without the advanced
 * checklist, and an unknown name (every name, while `capabilities` is still
 * empty because the query is loading or failed) is dropped rather than trusted,
 * so a one-click Allow can't hand out terminal.exec. */
export function withoutSensitive(names: string[], capabilities: CapabilityInfo[]): string[] {
  const safe = new Set(bulkToggleable(capabilities));
  return names.filter((name) => safe.has(name));
}
