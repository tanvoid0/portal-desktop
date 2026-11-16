import type { CapabilityInfo } from '../types';

/** Names "Select all" may bulk-toggle — every sensitive capability (RCE-equivalent
 * caps like terminal.exec/agents.exec/remote.input) stays opt-in per row, never
 * swept in by one click. */
export function bulkToggleable(capabilities: CapabilityInfo[]): string[] {
  return capabilities.filter((c) => !c.is_sensitive).map((c) => c.name);
}
