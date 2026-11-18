// portal:// deep links → in-app routes. Navigation only: any web page can fire
// a portal:// link, so a link never mutates anything.
//
//   portal://project/12      → /projects/12
//   portal://task/34         → /tasks/34
//   portal://document/5      → /documents/5
//   portal://pipeline-run/7  → /pipeline-runs/7
//   portal://script/9        → /scripts/9
//   portal://tasks | projects | documents | scripts | pipeline-runs → the list page

const ENTITY: Record<string, string> = {
  project: '/projects',
  task: '/tasks',
  document: '/documents',
  'pipeline-run': '/pipeline-runs',
  script: '/scripts',
};
const LISTS = new Set(['projects', 'tasks', 'documents', 'scripts', 'pipeline-runs']);

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
  if (!id) return LISTS.has(kind) ? `/${kind}` : null;
  const base = ENTITY[kind];
  return base && /^[\w-]+$/.test(id) ? `${base}/${id}` : null;
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
