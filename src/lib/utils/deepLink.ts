// portal:// deep links → in-app routes. Navigation only: any web page can fire
// a portal:// link, so a link never mutates anything.
//
//   portal://project/12[?tab=actions]   → /projects/12
//   portal://task/34                    → /tasks/34
//   portal://document/5                 → /documents/5
//   portal://run/7                      → /pipeline-runs/7
//   portal://script/9                   → /scripts/9
//   portal://agent/<session>            → agents workspace, session focused
//   portal://tasks[?project=3&view=list&task=34]
//   portal://runs[?status=failed&q=build&run=7]
//   portal://projects | documents | scripts   → the list page
//
// Filters ride along as query params, but only the ones each page actually
// reads (QUERY_ALLOW), with values restricted to a safe charset.

import { agentTabPath } from '$lib/config/agents-tabs';

const ENTITY: Record<string, string> = {
  project: '/projects',
  task: '/tasks',
  document: '/documents',
  run: '/pipeline-runs',
  'pipeline-run': '/pipeline-runs',
  script: '/scripts',
};
const LISTS: Record<string, string> = {
  projects: '/projects',
  tasks: '/tasks',
  documents: '/documents',
  scripts: '/scripts',
  runs: '/pipeline-runs',
  'pipeline-runs': '/pipeline-runs',
};
// Query params each destination reads (see TaskManager, pipeline-runs, project detail).
const QUERY_ALLOW: Record<string, readonly string[]> = {
  '/tasks': ['project', 'view', 'task'],
  '/pipeline-runs': ['q', 'status', 'run'],
  '/projects/*': ['tab'],
};
const SAFE_VALUE = /^[\w .:@/-]{1,200}$/;

function withQuery(path: string, from: URLSearchParams): string {
  const allowed =
    QUERY_ALLOW[path] ?? (/^\/projects\/[^/]+$/.test(path) ? QUERY_ALLOW['/projects/*'] : []);
  const out = new URLSearchParams();
  for (const key of allowed) {
    const v = from.get(key);
    if (v && SAFE_VALUE.test(v)) out.set(key, v);
  }
  const q = out.toString();
  return q ? `${path}?${q}` : path;
}

/** Route for a portal:// URL, or null if it is not one we understand. */
export function routeForDeepLink(raw: string): string | null {
  let url: URL;
  try {
    url = new URL(raw);
  } catch {
    return null;
  }
  if (url.protocol !== 'portal:') return null;
  // portal://project/12 parses as host=project, path=/12
  const kind = url.hostname;
  const id = url.pathname.replace(/^\/+|\/+$/g, '');
  const params = url.searchParams;
  if (!id) {
    const list = LISTS[kind];
    return list ? withQuery(list, params) : null;
  }
  if (!/^[\w-]+$/.test(id)) return null;
  if (kind === 'agent') return `${agentTabPath()}?session=${encodeURIComponent(id)}`;
  const base = ENTITY[kind];
  return base ? withQuery(`${base}/${id}`, params) : null;
}

/** Listen for deep links (cold start + running app) and navigate. No-op outside Tauri. */
export async function initDeepLinks(go: (route: string) => unknown): Promise<() => void> {
  if (!('__TAURI_INTERNALS__' in window)) return () => {};
  const { getCurrent, onOpenUrl } = await import('@tauri-apps/plugin-deep-link');
  const handle = (urls: string[] | null) => {
    for (const u of urls ?? []) {
      const route = routeForDeepLink(u);
      if (route) void go(route);
    }
  };
  handle(await getCurrent());
  return await onOpenUrl(handle);
}
