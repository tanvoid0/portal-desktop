/** Shared layout breakpoint constants (px). */
export const LAYOUT_BREAKPOINTS = {
  /** Compact tier: below this width uses drawer sidebars. */
  md: 768,
  /** Ultrawide tier: at/above this width uses wider gutters and sidebars. */
  ultrawide: 1920,
} as const;

export type ViewportTier = "compact" | "standard" | "ultrawide";

/** CSS custom property names used by layout components. */
export const LAYOUT_CSS_VARS = {
  headerHeight: "--header-height",
  sidebarWidth: "--sidebar-width",
  sidebarWidthIcon: "--sidebar-width-icon",
  contentMaxWidthReadable: "--content-max-width-readable",
  contentMaxWidthChat: "--content-max-width-chat",
  contentGutter: "--content-gutter",
} as const;

/** Routes whose pages manage their own full-viewport scroll regions. */
export const VIEWPORT_FILL_ROUTES = ["/coder", "/ai/chat"] as const;

/** Routes that use ShellSidebarLayout (domain sidebar + fixed height). */
export const SHELL_SIDEBAR_ROUTES = [
  "/ai",
  "/cloud",
  "/settings",
  "/sdk",
] as const;

export function isViewportFillRoute(pathname: string): boolean {
  return VIEWPORT_FILL_ROUTES.some(
    (route) => pathname === route || pathname.startsWith(`${route}/`),
  );
}

export function isShellSidebarRoute(pathname: string): boolean {
  return SHELL_SIDEBAR_ROUTES.some((route) => pathname.startsWith(route));
}

/** Main app sidebar is hidden only for routes that still use a standalone shell. */
export function isMainSidebarHidden(pathname: string): boolean {
  return false;
}
