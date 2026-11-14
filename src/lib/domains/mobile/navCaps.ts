import type { NavSection } from '$lib/components/shell/nav-types';

/** Nav entry url -> daemon cap that serves it. Entries not listed have no remote backend. */
const NAV_CAP: Record<string, string> = {
  '/': 'dashboard',
  '/projects': 'projects',
  '/tasks': 'tasks',
  '/terminal': 'terminal',
  '/pipeline-runs': 'pipelines',
  '/github': 'github',
};

/** `/ai` hosts both the Code tab (cap `coder`) and the Agents tab (cap `agents`) —
 *  keep the entry if either is served, and land on whichever tab the device has. */
function aiEntryUrl(caps: string[]): string | null {
  if (caps.includes('coder')) return '/ai/coder';
  if (caps.includes('agents')) return '/ai/agents';
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
