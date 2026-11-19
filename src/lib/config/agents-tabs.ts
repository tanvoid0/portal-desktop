export const DEFAULT_AGENT_BACKEND = 'claude-code';

const AGENT_TAB_BASE = '/ai/coder';

/** Route for a backend's engine tab in the merged Code workspace. */
export function agentTabPath(backendId: string = DEFAULT_AGENT_BACKEND): string {
  return `${AGENT_TAB_BASE}/${backendId}`;
}

/** Shareable deep link to one session of one provider; `portal` is Portal's own coder. */
export function sessionPath(backendId: string, sessionId: string): string {
  return `${AGENT_TAB_BASE}/${backendId}/${encodeURIComponent(sessionId)}`;
}

export interface EngineTab {
  id: string;
  label: string;
  url: string;
}

export const PORTAL_ENGINE_TAB: EngineTab = { id: 'portal', label: 'Portal', url: AGENT_TAB_BASE };

/** Cross-backend monitoring page; always the last tab. */
export const OVERVIEW_ENGINE_TAB: EngineTab = {
  id: 'overview',
  label: 'Overview',
  url: `${AGENT_TAB_BASE}/overview`,
};

/**
 * Engine tabs for the Code workspace. `caps` null = desktop (everything);
 * otherwise the mobile shell: Portal needs cap `coder`, backend tabs need `agents`,
 * Overview needs a backend plus `agents.history` (its session list is gated on it).
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
    if (backends.length && (!caps || caps.includes('agents.history')))
      tabs.push(OVERVIEW_ENGINE_TAB);
  }
  return tabs;
}

/** True for a backend's tab route (`/ai/coder/<backend>`) — not the bare Portal tab (`/ai/coder`). */
export function isAgentTabRoute(pathname: string): boolean {
  return pathname.startsWith(`${AGENT_TAB_BASE}/`);
}
