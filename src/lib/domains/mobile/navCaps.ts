import type { NavSection } from '$lib/components/shell/nav-types';
import { agentTabPath } from '$lib/config/agents-tabs';

/** Nav entry url -> daemon cap that serves it. Entries not listed have no remote backend. */
const NAV_CAP: Record<string, string> = {
  '/': 'dashboard',
  '/projects': 'projects',
  '/tasks': 'tasks',
  '/system': 'sysinfo',
  '/terminal': 'terminal',
  '/pipeline-runs': 'actions',
  '/github': 'github',
};

/** `/ai` hosts both the Portal workspace (cap `coder`) and agent backend tabs
 *  (cap `agents`) under the merged Code tab — keep the entry if either is
 *  served, landing on whichever side the device can reach. */
function aiEntryUrl(caps: string[]): string | null {
  if (caps.includes('coder')) return '/ai/coder';
  if (caps.includes('agents')) return agentTabPath();
  return null;
}

/** Keep only entries the device's caps can serve; submenus are dropped (not remote-capable). */
export function filterNavForCaps(sections: NavSection[], caps: string[]): NavSection[] {
  return sections
    .map((s) => ({
      ...s,
      items: s.items.flatMap((i) => {
        if (i.url === '/ai') {
          const url = aiEntryUrl(caps);
          return url ? [{ ...i, url, submenu: undefined }] : [];
        }
        if (!caps.includes(NAV_CAP[i.url] ?? '')) return [];
        return [{ ...i, submenu: undefined }];
      }),
    }))
    .filter((s) => s.items.length > 0);
}
