/** Shared layout breakpoint constants (px). */
export const LAYOUT_BREAKPOINTS = {
  /** Compact tier: below this width uses drawer sidebars. */
  md: 768,
  /** Ultrawide tier: at/above this width uses wider gutters and sidebars. */
  ultrawide: 1920,
} as const;

export type ViewportTier = 'compact' | 'standard' | 'ultrawide';

/** CSS custom property names used by layout components. */
export const LAYOUT_CSS_VARS = {
  headerHeight: '--header-height',
  sidebarWidth: '--sidebar-width',
  sidebarWidthIcon: '--sidebar-width-icon',
  contentMaxWidthReadable: '--content-max-width-readable',
  contentMaxWidthChat: '--content-max-width-chat',
  contentGutter: '--content-gutter',
} as const;

/** Routes whose pages manage their own full-viewport scroll regions. */
export const VIEWPORT_FILL_ROUTES = ['/ai/coder', '/ai/chat', '/terminal'] as const;

/** Routes that use ShellSidebarLayout (domain sidebar + fixed height). */
export const SHELL_SIDEBAR_ROUTES = ['/ai', '/cloud', '/settings', '/sdk', '/utilities'] as const;

/**
 * Routes whose own page — not a `+layout` — uses ShellSidebarLayout, so the
 * shell must bound their height, while their CHILD routes stay ordinary
 * scrolling pages. Matched exactly, never by prefix: `/tasks` is a full-height
 * board, but `/tasks/create` and `/tasks/[id]` are plain pages that would be
 * clipped by the domain branch's `overflow-hidden` (it provides no scroller).
 */
export const SHELL_SIDEBAR_EXACT_ROUTES = ['/tasks'] as const;

export function isViewportFillRoute(pathname: string): boolean {
  return VIEWPORT_FILL_ROUTES.some(
    (route) => pathname === route || pathname.startsWith(`${route}/`)
  );
}

export function isShellSidebarRoute(pathname: string): boolean {
  return (
    SHELL_SIDEBAR_ROUTES.some((route) => pathname.startsWith(route)) ||
    SHELL_SIDEBAR_EXACT_ROUTES.some((route) => pathname === route)
  );
}

/** Main app sidebar is hidden only for routes that still use a standalone shell. */
export function isMainSidebarHidden(pathname: string): boolean {
  return false;
}
