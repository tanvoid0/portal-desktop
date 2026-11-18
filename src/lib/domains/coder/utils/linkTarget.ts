const BACKSLASH = /\\/g;

/** Workspace-relative path for a chat link (file://, :line suffix, abs-in-workspace), else null. */
export function workspacePathFromHref(href: string, workspaceRoot: string): string | null {
  if (!workspaceRoot || /^(mailto:|#)/i.test(href)) return null;
  let path = href.replace(/^file:\/+/i, '').replace(/[?#].*$/, '');
  try {
    path = decodeURIComponent(path);
  } catch {
    // keep raw
  }
  const norm = path.replace(BACKSLASH, '/').replace(/:\d+(:\d+)?$/, '');
  const root = workspaceRoot.replace(BACKSLASH, '/').replace(/\/+$/, '');
  if (norm.toLowerCase().startsWith(root.toLowerCase() + '/')) return norm.slice(root.length + 1);
  if (/^([a-z]:)?\//i.test(norm)) return null; // absolute, outside the workspace
  return norm.replace(/^\.\//, '') || null;
}
