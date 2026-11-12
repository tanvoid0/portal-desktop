import type { NavSection } from '$lib/components/shell/nav-types';

/** Nav entry url -> daemon cap that serves it. Entries not listed have no remote backend. */
const NAV_CAP: Record<string, string> = {
  '/': 'dashboard',
  '/projects': 'projects',
  '/tasks': 'tasks',
  '/terminal': 'terminal',
  '/agents/run': 'agents',
  '/ai': 'coder',
};

/** Keep only entries the device's caps can serve; submenus are dropped (not remote-capable). */
export function filterNavForCaps(sections: NavSection[], caps: string[]): NavSection[] {
  return sections
    .map((s) => ({
      ...s,
      items: s.items
        .filter((i) => caps.includes(NAV_CAP[i.url] ?? ''))
        .map((i) => ({ ...i, url: i.url === '/ai' ? '/ai/coder' : i.url, submenu: undefined })),
    }))
    .filter((s) => s.items.length > 0);
}
