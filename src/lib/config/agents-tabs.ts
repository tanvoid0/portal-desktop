export const AGENTS_RUN_PATH = '/agents/run';
export const AGENTS_SESSIONS_PATH = '/agents/sessions';

export const AGENTS_TABS = [
  { id: 'run', label: 'Run', icon: 'code', url: AGENTS_RUN_PATH },
  { id: 'sessions', label: 'Sessions', icon: 'history', url: AGENTS_SESSIONS_PATH },
] as const;

export type AgentsTabId = (typeof AGENTS_TABS)[number]['id'];

export function getActiveAgentsTab(pathname: string): AgentsTabId {
  if (pathname === AGENTS_SESSIONS_PATH || pathname.startsWith(`${AGENTS_SESSIONS_PATH}/`)) {
    return 'sessions';
  }
  return 'run';
}
