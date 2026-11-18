// Projects-page view preferences. localStorage can throw or hold junk, in which
// case the defaults apply and the page still renders.

const OPEN_GROUPS_KEY = 'portal.projects.openGroups';
const VIEW_MODE_KEY = 'portal.projects.viewMode';

export type ViewMode = 'list' | 'grid';

export function readOpenGroups(): Record<string, boolean> {
  if (typeof localStorage === 'undefined') return {};
  try {
    const parsed: unknown = JSON.parse(localStorage.getItem(OPEN_GROUPS_KEY) ?? 'null');
    return parsed && typeof parsed === 'object' ? (parsed as Record<string, boolean>) : {};
  } catch {
    return {};
  }
}

export function saveOpenGroups(groups: Record<string, boolean>) {
  try {
    localStorage?.setItem(OPEN_GROUPS_KEY, JSON.stringify(groups));
  } catch {
    // Quota or private mode — the choice still holds for this session.
  }
}

export function readViewMode(): ViewMode {
  try {
    return localStorage?.getItem(VIEW_MODE_KEY) === 'grid' ? 'grid' : 'list';
  } catch {
    return 'list';
  }
}

export function saveViewMode(mode: ViewMode) {
  try {
    localStorage?.setItem(VIEW_MODE_KEY, mode);
  } catch {
    // Quota or private mode — the choice still holds for this session.
  }
}
