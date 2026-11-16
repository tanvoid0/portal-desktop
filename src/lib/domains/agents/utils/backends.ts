import type { BackendInfo } from '../generated';

/** Whether there's a real choice to present. A single-backend install (today:
 *  Claude Code only) has nothing to pick, so the launcher's Agent selector
 *  stays hidden until a second backend is detected (#193 adds Codex). */
export function hasBackendChoice(backends: BackendInfo[]): boolean {
  return backends.length > 1;
}
