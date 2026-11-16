export const DEFAULT_AGENT_BACKEND = 'claude-code';

const AGENT_TAB_BASE = '/ai/coder';

/** Route for a backend's engine tab in the merged Code workspace. */
export function agentTabPath(backendId: string = DEFAULT_AGENT_BACKEND): string {
  return `${AGENT_TAB_BASE}/${backendId}`;
}

export interface EngineTab {
  id: string;
  label: string;
  url: string;
}

export const PORTAL_ENGINE_TAB: EngineTab = { id: 'portal', label: 'Portal', url: AGENT_TAB_BASE };

/**
 * Engine tabs for the Code workspace. `caps` null = desktop (everything);
 * otherwise the mobile shell: Portal needs cap `coder`, backend tabs need `agents`.
 */
export function engineTabsFor(
  backends: ReadonlyArray<{ id: string; display_name: string }>,
  caps: readonly string[] | null
): EngineTab[] {
  const tabs: EngineTab[] = [];
  if (!caps || caps.includes('coder')) tabs.push(PORTAL_ENGINE_TAB);
  if (!caps || caps.includes('agents')) {
    for (const b of backends)
      tabs.push({ id: b.id, label: b.display_name, url: agentTabPath(b.id) });
  }
  return tabs;
}

/** True for a backend's tab route (`/ai/coder/<backend>`) — not the bare Portal tab (`/ai/coder`). */
export function isAgentTabRoute(pathname: string): boolean {
  return pathname.startsWith(`${AGENT_TAB_BASE}/`);
}
